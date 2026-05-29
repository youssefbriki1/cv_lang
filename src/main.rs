//! Command-line front-end for `cv_lang`.
//!
//! ```text
//! cv_lang <input.cv> [-o <output.tex>] [--pdf]
//! ```
//!
//! Always writes a `.tex` file. With `--pdf`, it additionally tries to invoke
//! `pdflatex`; if that binary is missing, it prints a warning and exits cleanly
//! with the `.tex` still on disk.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// Parsed command-line arguments.
struct Args {
    input: PathBuf,
    output: Option<PathBuf>,
    pdf: bool,
}

const USAGE: &str = "usage: cv_lang <input.cv> [-o <output.tex>] [--pdf]";

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("error: {message}\n{USAGE}");
            return ExitCode::FAILURE;
        }
    };

    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut pdf = false;

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Err("showing usage".into()),
            "--pdf" => pdf = true,
            "-o" | "--output" => {
                let value = args
                    .next()
                    .ok_or_else(|| format!("`{arg}` requires a file path"))?;
                output = Some(PathBuf::from(value));
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown flag '{other}'"));
            }
            _ => {
                if input.is_some() {
                    return Err("more than one input file given".into());
                }
                input = Some(PathBuf::from(arg));
            }
        }
    }

    let input = input.ok_or("no input file given")?;
    Ok(Args { input, output, pdf })
}

fn run(args: &Args) -> Result<(), String> {
    let source = std::fs::read_to_string(&args.input)
        .map_err(|e| format!("could not read {}: {e}", args.input.display()))?;

    let compiled = cv_lang::compile(&source).map_err(|d| d.to_string())?;

    // Surface warnings, but they never stop compilation.
    for warning in &compiled.warnings {
        eprintln!("{warning}");
    }

    let out_path = args
        .output
        .clone()
        .unwrap_or_else(|| args.input.with_extension("tex"));
    std::fs::write(&out_path, &compiled.latex)
        .map_err(|e| format!("could not write {}: {e}", out_path.display()))?;
    println!("wrote {}", out_path.display());

    if args.pdf {
        run_pdflatex(&out_path);
    }
    Ok(())
}

/// Best-effort PDF generation. A missing `pdflatex` is a warning, not an error.
fn run_pdflatex(tex_path: &Path) {
    let dir = tex_path.parent().filter(|p| !p.as_os_str().is_empty());
    let mut cmd = Command::new("pdflatex");
    cmd.arg("-interaction=nonstopmode");
    if let Some(dir) = dir {
        cmd.arg(format!("-output-directory={}", dir.display()));
    }
    cmd.arg(tex_path);

    match cmd.status() {
        Ok(status) if status.success() => {
            println!("wrote {}", tex_path.with_extension("pdf").display());
        }
        Ok(status) => {
            eprintln!("warning: pdflatex exited with {status}; see the .log file");
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!(
                "warning: pdflatex not found; wrote .tex only. \
                 Install a TeX distribution (e.g. TeX Live) to enable --pdf."
            );
        }
        Err(e) => eprintln!("warning: could not run pdflatex: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Args, String> {
        parse_args(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn parses_input_only() {
        let args = parse(&["resume.cv"]).unwrap();
        assert_eq!(args.input, PathBuf::from("resume.cv"));
        assert!(args.output.is_none());
        assert!(!args.pdf);
    }

    #[test]
    fn parses_output_and_pdf_flag() {
        let args = parse(&["resume.cv", "-o", "out.tex", "--pdf"]).unwrap();
        assert_eq!(args.output, Some(PathBuf::from("out.tex")));
        assert!(args.pdf);
    }

    #[test]
    fn rejects_missing_input() {
        assert!(parse(&["--pdf"]).is_err());
    }

    #[test]
    fn rejects_unknown_flag() {
        assert!(parse(&["resume.cv", "--nope"]).is_err());
    }
}
