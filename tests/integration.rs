//! End-to-end tests: each `examples/*.cv` should compile to a standalone LaTeX
//! document containing the expected structural markers and content.

use cv_lang::compile;

fn compile_example(name: &str) -> String {
    let path = format!("{}/examples/{name}", env!("CARGO_MANIFEST_DIR"));
    let source = std::fs::read_to_string(&path).expect("read example");
    let out = compile(&source).expect("compile should succeed");
    assert!(out.warnings.is_empty(), "{name} produced warnings: {:?}", out.warnings);
    out.latex
}

fn assert_standalone(tex: &str) {
    assert!(tex.contains("\\documentclass[letterpaper,11pt]{article}"));
    assert!(tex.contains("\\begin{document}"));
    assert!(tex.contains("\\end{document}"));
}

#[test]
fn core_example_compiles() {
    let tex = compile_example("core.cv");
    assert_standalone(&tex);
    assert!(tex.contains("Youssef Briki"));
    assert!(tex.contains("\\resumeSubheading"));
    // The `&` and `%` in the source must be escaped.
    assert!(tex.contains("Lab Q\\&A Agent"));
    assert!(tex.contains("35\\% (P95)"));
    // Skills section rendered as a tag list, not entries.
    assert!(tex.contains("\\section{Skills}"));
    assert!(tex.contains("Python, Rust, Java"));
}

#[test]
fn extended_example_compiles() {
    let tex = compile_example("extended.cv");
    assert_standalone(&tex);
    // Summary block.
    assert!(tex.contains("\\section{Summary}"));
    // Per-entry stack rendered as an item.
    assert!(tex.contains("\\textit{Stack:}"));
    // `link` turns the role into a hyperlink.
    assert!(tex.contains("\\href{https://desjardins.com}"));
    // Sidebar fields folded into the header.
    assert!(tex.contains("English, French"));
}

#[test]
fn summary_only_example_compiles() {
    let tex = compile_example("summary_only.cv");
    assert_standalone(&tex);
    assert!(tex.contains("\\section{Summary}"));
    assert!(tex.contains("\\section{Experience}"));
    assert!(tex.contains("\\section{Education}"));
}
