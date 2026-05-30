//! pdflatex round-trip test: every `examples/*.cv` must compile to a real PDF.
//!
//! This is gated on `pdflatex` being installed — if it isn't, the test prints a
//! note and passes (so CI without TeX, and the Docker job with TeX, both work).

use std::path::{Path, PathBuf};
use std::process::Command;

use cv_lang::compile;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn pdflatex_available() -> bool {
    Command::new("pdflatex")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn example_files() -> Vec<PathBuf> {
    let dir = manifest_dir().join("examples");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("read examples dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "cv"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no example .cv files found");
    files
}

fn compile_pdf(cv: &Path, work: &Path) -> Result<PathBuf, String> {
    let source = std::fs::read_to_string(cv).map_err(|e| e.to_string())?;
    let latex = compile(&source).map_err(|d| d.to_string())?.latex;

    let stem = cv.file_stem().unwrap().to_string_lossy().to_string();
    let tex = work.join(format!("{stem}.tex"));
    std::fs::write(&tex, latex).map_err(|e| e.to_string())?;

    let status = Command::new("pdflatex")
        .arg("-interaction=nonstopmode")
        .arg("-halt-on-error")
        .arg(format!("-output-directory={}", work.display()))
        .arg(&tex)
        .status()
        .map_err(|e| e.to_string())?;

    let pdf = work.join(format!("{stem}.pdf"));
    if status.success() && pdf.exists() {
        Ok(pdf)
    } else {
        Err(format!(
            "pdflatex failed for {}; see {}",
            cv.display(),
            work.join(format!("{stem}.log")).display()
        ))
    }
}

#[test]
fn examples_compile_to_pdf() {
    if !pdflatex_available() {
        eprintln!("skipping: pdflatex not installed");
        return;
    }

    let work = std::env::temp_dir().join(format!("cv_lang_pdf_{}", std::process::id()));
    std::fs::create_dir_all(&work).expect("create work dir");

    for cv in example_files() {
        match compile_pdf(&cv, &work) {
            Ok(pdf) => assert!(pdf.exists(), "{} produced no PDF", cv.display()),
            Err(e) => panic!("{e}"),
        }
    }

    let _ = std::fs::remove_dir_all(&work);
}
