//! `cv_lang` — a small declarative DSL that compiles `.cv` resume files into
//! LaTeX targeting Jake's resume template.
//!
//! Pipeline: [`lexer::tokenize`] → [`parser::parse`] → [`renderer::render`].
//! The public entry point is [`compile`].

pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod renderer;

use error::Diagnostic;

/// The successful result of compiling a `.cv` source: the rendered LaTeX plus
/// any non-fatal warnings gathered along the way.
#[derive(Debug)]
pub struct Compiled {
    pub latex: String,
    pub warnings: Vec<Diagnostic>,
}

/// Compile `.cv` source text into a standalone LaTeX document.
///
/// Returns `Err` with the fatal [`Diagnostic`] on a hard error (currently only
/// an unterminated string from the lexer). Recoverable issues — unknown fields
/// or constructs — come back as `warnings` on the [`Compiled`] result.
pub fn compile(source: &str) -> Result<Compiled, Diagnostic> {
    let tokens = lexer::tokenize(source)?;
    let (document, warnings) = parser::parse(tokens);
    let latex = renderer::render(&document);
    Ok(Compiled { latex, warnings })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_end_to_end() {
        let src = "name \"Ada\"\nsection \"Skills\":\n  tags: \"Rust\"\n";
        let out = compile(src).unwrap();
        assert!(out.latex.contains("\\begin{document}"));
        assert!(out.latex.contains("Ada"));
        assert!(out.warnings.is_empty());
    }

    #[test]
    fn unterminated_string_is_fatal() {
        let err = compile("name \"oops\n").unwrap_err();
        assert!(err.is_error());
    }
}
