//! Golden (snapshot) tests: the rendered LaTeX for each example must match a
//! committed `tests/golden/<name>.tex` file. This catches unintended drift in
//! the renderer's output.
//!
//! To regenerate the golden files after an intentional change, run:
//! `CV_LANG_BLESS=1 cargo test --test golden`
//!
//! Uses only the standard library — no snapshot-testing dependency.

use std::path::PathBuf;

use cv_lang::compile;

fn manifest_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn check_golden(name: &str) {
    let source = std::fs::read_to_string(manifest_path(&format!("examples/{name}.cv")))
        .expect("read example");
    let got = compile(&source).expect("compile should succeed").latex;

    let golden = manifest_path(&format!("tests/golden/{name}.tex"));

    if std::env::var_os("CV_LANG_BLESS").is_some() {
        std::fs::create_dir_all(golden.parent().unwrap()).unwrap();
        std::fs::write(&golden, &got).expect("write golden");
        return;
    }

    let want = std::fs::read_to_string(&golden).unwrap_or_else(|_| {
        panic!(
            "missing golden file {}; run `CV_LANG_BLESS=1 cargo test --test golden` to create it",
            golden.display()
        )
    });
    assert_eq!(
        got,
        want,
        "rendered LaTeX for {name}.cv drifted from {}; \
         re-bless with CV_LANG_BLESS=1 if the change is intentional",
        golden.display()
    );
}

#[test]
fn core_matches_golden() {
    check_golden("core");
}

#[test]
fn extended_matches_golden() {
    check_golden("extended");
}

#[test]
fn summary_only_matches_golden() {
    check_golden("summary_only");
}
