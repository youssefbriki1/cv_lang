//! Command-line front-end for `cv_lang`.
//!
//! ```text
//! cv_lang <input.cv> [-o <output.tex>] [--pdf] [--check] [--strict] [--format human|json]
//! ```
//!
//! By default it writes a `.tex` file. `--pdf` also tries `pdflatex` (a missing
//! binary is a warning, not a crash). `--check` validates only (no files
//! written). `--strict` makes any warning a non-zero exit. `--format json`
//! prints a single machine-readable result object — handy when an agent drives
//! the compiler.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use cv_lang::error::{Diagnostic, Level};

/// Output format for diagnostics and the result summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
    Human,
    Json,
}

/// Parsed command-line arguments.
struct Args {
    input: PathBuf,
    output: Option<PathBuf>,
    pdf: bool,
    check: bool,
    strict: bool,
    format: Format,
}

const USAGE: &str = "usage: cv_lang <input.cv> [-o <output.tex>] [--pdf] [--check] [--strict] [--format human|json]";

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(Some(args)) => args,
        Ok(None) => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Err(message) => {
            eprintln!("error: {message}\n{USAGE}");
            return ExitCode::FAILURE;
        }
    };
    run(&args)
}

/// Returns `Ok(None)` when help was requested, `Ok(Some(args))` on a valid
/// invocation, and `Err` on a usage error.
fn parse_args(args: impl Iterator<Item = String>) -> Result<Option<Args>, String> {
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut pdf = false;
    let mut check = false;
    let mut strict = false;
    let mut format = Format::Human;

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(None),
            "--pdf" => pdf = true,
            "--check" => check = true,
            "--strict" => strict = true,
            "--json" => format = Format::Json,
            "--format" => {
                let value = args
                    .next()
                    .ok_or("`--format` requires a value (human|json)")?;
                format = match value.as_str() {
                    "human" => Format::Human,
                    "json" => Format::Json,
                    other => return Err(format!("unknown format '{other}' (expected human|json)")),
                };
            }
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
    Ok(Some(Args {
        input,
        output,
        pdf,
        check,
        strict,
        format,
    }))
}

fn run(args: &Args) -> ExitCode {
    let source = match std::fs::read_to_string(&args.input) {
        Ok(s) => s,
        Err(e) => {
            report_fatal(
                args.format,
                &format!("could not read {}: {e}", args.input.display()),
            );
            return ExitCode::FAILURE;
        }
    };

    let compiled = match cv_lang::compile(&source) {
        Ok(c) => c,
        Err(diag) => {
            report_compile_error(args.format, &source, &diag);
            return ExitCode::FAILURE;
        }
    };

    // Write outputs unless we're only checking.
    let mut tex_path: Option<PathBuf> = None;
    let mut pdf_path: Option<PathBuf> = None;
    let mut pdf_error: Option<String> = None;

    if !args.check {
        let out_path = args
            .output
            .clone()
            .unwrap_or_else(|| args.input.with_extension("tex"));
        if let Err(e) = std::fs::write(&out_path, &compiled.latex) {
            report_fatal(
                args.format,
                &format!("could not write {}: {e}", out_path.display()),
            );
            return ExitCode::FAILURE;
        }
        tex_path = Some(out_path.clone());

        if args.pdf {
            match try_pdflatex(&out_path) {
                PdfOutcome::Built(p) => pdf_path = Some(p),
                PdfOutcome::Failed(m) | PdfOutcome::NotAvailable(m) => pdf_error = Some(m),
            }
        }
    }

    report_success(
        args,
        &source,
        &compiled.warnings,
        &tex_path,
        &pdf_path,
        &pdf_error,
    );

    if args.strict && !compiled.warnings.is_empty() {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

// --- reporting ---------------------------------------------------------------

fn report_success(
    args: &Args,
    source: &str,
    warnings: &[Diagnostic],
    tex: &Option<PathBuf>,
    pdf: &Option<PathBuf>,
    pdf_error: &Option<String>,
) {
    match args.format {
        Format::Human => {
            if let Some(p) = tex {
                println!("wrote {}", p.display());
            }
            if let Some(p) = pdf {
                println!("wrote {}", p.display());
            }
            if let Some(e) = pdf_error {
                eprintln!("warning: {e}");
            }
            for w in warnings {
                print_diagnostic_human(source, w);
            }
            if args.check {
                let n = warnings.len();
                println!("check: ok ({n} warning{})", if n == 1 { "" } else { "s" });
            }
        }
        Format::Json => print_result_json(args.check, warnings, tex, pdf, pdf_error),
    }
}

fn report_compile_error(format: Format, source: &str, diag: &Diagnostic) {
    match format {
        Format::Human => print_diagnostic_human(source, diag),
        Format::Json => {
            println!("{{\"ok\":false,\"error\":{}}}", diag_to_json(diag));
        }
    }
}

fn report_fatal(format: Format, message: &str) {
    match format {
        Format::Human => eprintln!("error: {message}"),
        Format::Json => println!(
            "{{\"ok\":false,\"error\":{{\"level\":\"error\",\"line\":0,\"message\":{}}}}}",
            json_string(message)
        ),
    }
}

/// Human-friendly diagnostic: the message, then the offending source line with
/// a caret under its content.
fn print_diagnostic_human(source: &str, diag: &Diagnostic) {
    eprintln!("{diag}");
    if diag.line == 0 {
        return;
    }
    if let Some(text) = source.lines().nth(diag.line - 1) {
        let gutter = format!("  {:>4} | ", diag.line);
        let blank = format!("  {:>4} | ", "");
        let leading = text.chars().take_while(|c| c.is_whitespace()).count();
        let span = text.trim().chars().count().max(1);
        eprintln!("{gutter}{text}");
        eprintln!("{blank}{}{}", " ".repeat(leading), "^".repeat(span));
    }
}

fn print_result_json(
    check: bool,
    warnings: &[Diagnostic],
    tex: &Option<PathBuf>,
    pdf: &Option<PathBuf>,
    pdf_error: &Option<String>,
) {
    let warnings_json: Vec<String> = warnings.iter().map(diag_to_json).collect();
    let mut obj = String::new();
    obj.push_str("{\"ok\":true");
    obj.push_str(&format!(",\"check\":{check}"));
    obj.push_str(&format!(",\"output\":{}", opt_path_json(tex)));
    obj.push_str(&format!(",\"pdf\":{}", opt_path_json(pdf)));
    if let Some(e) = pdf_error {
        obj.push_str(&format!(",\"pdf_error\":{}", json_string(e)));
    }
    obj.push_str(&format!(",\"warnings\":[{}]", warnings_json.join(",")));
    obj.push('}');
    println!("{obj}");
}

fn diag_to_json(d: &Diagnostic) -> String {
    let level = match d.level {
        Level::Warning => "warning",
        Level::Error => "error",
    };
    format!(
        "{{\"level\":\"{level}\",\"line\":{},\"message\":{}}}",
        d.line,
        json_string(&d.message)
    )
}

fn opt_path_json(p: &Option<PathBuf>) -> String {
    match p {
        Some(p) => json_string(&p.display().to_string()),
        None => "null".to_string(),
    }
}

/// JSON-encode a string (quotes + escapes).
fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// --- pdflatex ----------------------------------------------------------------

enum PdfOutcome {
    Built(PathBuf),
    Failed(String),
    NotAvailable(String),
}

/// Best-effort PDF generation. A missing `pdflatex` is reported, not fatal.
fn try_pdflatex(tex_path: &Path) -> PdfOutcome {
    let dir = tex_path.parent().filter(|p| !p.as_os_str().is_empty());
    let mut cmd = Command::new("pdflatex");
    cmd.arg("-interaction=nonstopmode");
    if let Some(dir) = dir {
        cmd.arg(format!("-output-directory={}", dir.display()));
    }
    cmd.arg(tex_path);

    match cmd.status() {
        Ok(status) if status.success() => PdfOutcome::Built(tex_path.with_extension("pdf")),
        Ok(status) => {
            PdfOutcome::Failed(format!("pdflatex exited with {status}; see the .log file"))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => PdfOutcome::NotAvailable(
            "pdflatex not found; wrote .tex only. Install a TeX distribution to enable --pdf."
                .to_string(),
        ),
        Err(e) => PdfOutcome::Failed(format!("could not run pdflatex: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Option<Args>, String> {
        parse_args(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn parses_input_only() {
        let args = parse(&["resume.cv"]).unwrap().unwrap();
        assert_eq!(args.input, PathBuf::from("resume.cv"));
        assert!(args.output.is_none());
        assert!(!args.pdf && !args.check && !args.strict);
        assert_eq!(args.format, Format::Human);
    }

    #[test]
    fn parses_all_flags() {
        let args = parse(&[
            "r.cv", "-o", "out.tex", "--pdf", "--check", "--strict", "--format", "json",
        ])
        .unwrap()
        .unwrap();
        assert_eq!(args.output, Some(PathBuf::from("out.tex")));
        assert!(args.pdf && args.check && args.strict);
        assert_eq!(args.format, Format::Json);
    }

    #[test]
    fn json_shortcut_flag() {
        let args = parse(&["r.cv", "--json"]).unwrap().unwrap();
        assert_eq!(args.format, Format::Json);
    }

    #[test]
    fn help_returns_none() {
        assert!(parse(&["--help"]).unwrap().is_none());
    }

    #[test]
    fn rejects_missing_input() {
        assert!(parse(&["--pdf"]).is_err());
    }

    #[test]
    fn rejects_unknown_flag() {
        assert!(parse(&["resume.cv", "--nope"]).is_err());
    }

    #[test]
    fn rejects_bad_format() {
        assert!(parse(&["resume.cv", "--format", "yaml"]).is_err());
    }

    #[test]
    fn json_string_escapes() {
        assert_eq!(json_string("a\"b\\c"), "\"a\\\"b\\\\c\"");
        assert_eq!(json_string("line\nbreak"), "\"line\\nbreak\"");
    }

    #[test]
    fn diag_json_shape() {
        let d = Diagnostic::warning(3, "unknown field 'mood'");
        let j = diag_to_json(&d);
        assert!(j.contains("\"level\":\"warning\""));
        assert!(j.contains("\"line\":3"));
        assert!(j.contains("unknown field 'mood'"));
    }
}
