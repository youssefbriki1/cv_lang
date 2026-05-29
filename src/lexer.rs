//! Indentation-aware lexer.
//!
//! The DSL is line- and indentation-oriented (entries nest under sections,
//! bullets under `bullets:`), so the token stream carries explicit [`Indent`] /
//! [`Dedent`] / [`Newline`] markers — much like Python's tokenizer.
//!
//! The lexer is deliberately "dumb": it never decides whether a bare word is a
//! keyword. Every unquoted word becomes [`TokenKind::Ident`] and the parser
//! attaches meaning. That is what lets the language stay *forgiving* — an
//! unknown field is just an `Ident` the parser can warn about rather than a lex
//! error.
//!
//! [`Indent`]: TokenKind::Indent
//! [`Dedent`]: TokenKind::Dedent
//! [`Newline`]: TokenKind::Newline

use crate::error::Diagnostic;

/// The kinds of token the lexer can emit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    /// Any bare word: `name`, `section`, `entry`, `role`, `email`, `foo`, ...
    Ident(String),
    /// A `"quoted literal"` (with `\"` and `\\` unescaped).
    Str(String),
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// `-` — the bullet / list-item marker.
    Dash,
    /// End of a logical (non-blank) line.
    Newline,
    /// Indentation increased relative to the enclosing line.
    Indent,
    /// Indentation decreased back to an enclosing level.
    Dedent,
    /// End of input.
    Eof,
}

/// A token plus the 1-based source line it came from (for diagnostics).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
}

impl Token {
    fn new(kind: TokenKind, line: usize) -> Self {
        Token { kind, line }
    }
}

/// Tokenize a `.cv` source string.
///
/// Returns a flat token vector terminated by [`TokenKind::Eof`]. The only fatal
/// lexer condition is an unterminated string literal, reported as an error
/// [`Diagnostic`].
pub fn tokenize(src: &str) -> Result<Vec<Token>, Diagnostic> {
    let mut tokens = Vec::new();
    // Stack of active indentation widths; always non-empty (base level 0).
    let mut indent_stack = vec![0usize];
    let mut last_line = 1usize;

    for (idx, raw_line) in src.lines().enumerate() {
        let line_no = idx + 1;
        last_line = line_no;

        // Leading whitespace is ASCII (space/tab), so the char count equals the
        // byte offset of the line's content.
        let indent = raw_line
            .chars()
            .take_while(|c| *c == ' ' || *c == '\t')
            .count();
        let content = &raw_line[indent..];

        // Blank or comment-only lines never affect indentation or emit tokens.
        if content.is_empty() || content.starts_with('#') {
            continue;
        }

        emit_indentation(indent, &mut indent_stack, line_no, &mut tokens);
        lex_line(content, line_no, &mut tokens)?;
        tokens.push(Token::new(TokenKind::Newline, line_no));
    }

    // Close any open indentation blocks, then signal end of input.
    while indent_stack.len() > 1 {
        indent_stack.pop();
        tokens.push(Token::new(TokenKind::Dedent, last_line));
    }
    tokens.push(Token::new(TokenKind::Eof, last_line));
    Ok(tokens)
}

/// Compare `indent` against the indent stack, emitting Indent / Dedent tokens.
///
/// Dedents that don't land exactly on a previous level are tolerated (we pop to
/// the nearest enclosing level) — the language favours leniency over strictness.
fn emit_indentation(
    indent: usize,
    indent_stack: &mut Vec<usize>,
    line_no: usize,
    tokens: &mut Vec<Token>,
) {
    let current = *indent_stack.last().unwrap();
    if indent > current {
        indent_stack.push(indent);
        tokens.push(Token::new(TokenKind::Indent, line_no));
    } else if indent < current {
        while indent_stack.len() > 1 && *indent_stack.last().unwrap() > indent {
            indent_stack.pop();
            tokens.push(Token::new(TokenKind::Dedent, line_no));
        }
    }
}

/// Tokenize the content of a single (already indent-stripped) line.
fn lex_line(content: &str, line_no: usize, tokens: &mut Vec<Token>) -> Result<(), Diagnostic> {
    let mut chars = content.char_indices().peekable();
    while let Some(&(_, c)) = chars.peek() {
        match c {
            ' ' | '\t' => {
                chars.next();
            }
            '#' => break, // trailing comment runs to end of line
            ':' => {
                chars.next();
                tokens.push(Token::new(TokenKind::Colon, line_no));
            }
            ',' => {
                chars.next();
                tokens.push(Token::new(TokenKind::Comma, line_no));
            }
            '-' => {
                chars.next();
                tokens.push(Token::new(TokenKind::Dash, line_no));
            }
            '"' => {
                chars.next(); // opening quote
                let mut value = String::new();
                let mut closed = false;
                while let Some((_, ch)) = chars.next() {
                    match ch {
                        '\\' => match chars.next() {
                            Some((_, '"')) => value.push('"'),
                            Some((_, '\\')) => value.push('\\'),
                            Some((_, 'n')) => value.push('\n'),
                            Some((_, 't')) => value.push('\t'),
                            // Unknown escape: keep it verbatim.
                            Some((_, other)) => {
                                value.push('\\');
                                value.push(other);
                            }
                            None => {}
                        },
                        '"' => {
                            closed = true;
                            break;
                        }
                        _ => value.push(ch),
                    }
                }
                if !closed {
                    return Err(Diagnostic::error(line_no, "unterminated string literal"));
                }
                tokens.push(Token::new(TokenKind::Str(value), line_no));
            }
            _ => {
                // A bare word runs until whitespace or a punctuation/quote/comment.
                let mut word = String::new();
                while let Some(&(_, ch)) = chars.peek() {
                    if ch.is_whitespace() || matches!(ch, ':' | ',' | '"' | '#') {
                        break;
                    }
                    word.push(ch);
                    chars.next();
                }
                tokens.push(Token::new(TokenKind::Ident(word), line_no));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<TokenKind> {
        tokenize(src).unwrap().into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn lexes_simple_header() {
        let got = kinds("name \"Ada Lovelace\"\n");
        assert_eq!(
            got,
            vec![
                TokenKind::Ident("name".into()),
                TokenKind::Str("Ada Lovelace".into()),
                TokenKind::Newline,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn emits_indent_and_dedent() {
        let src = "section \"Skills\":\n  tags: \"Rust\"\n";
        let got = kinds(src);
        assert_eq!(
            got,
            vec![
                TokenKind::Ident("section".into()),
                TokenKind::Str("Skills".into()),
                TokenKind::Colon,
                TokenKind::Newline,
                TokenKind::Indent,
                TokenKind::Ident("tags".into()),
                TokenKind::Colon,
                TokenKind::Str("Rust".into()),
                TokenKind::Newline,
                TokenKind::Dedent,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn skips_blank_and_comment_lines() {
        let src = "name \"X\"\n\n  # a comment\ncontact email \"a@b.c\"\n";
        let got = kinds(src);
        // No Indent should appear from the comment line's leading spaces.
        assert!(!got.contains(&TokenKind::Indent));
        assert_eq!(got.iter().filter(|k| **k == TokenKind::Newline).count(), 2);
    }

    #[test]
    fn handles_escapes_in_strings() {
        let got = kinds("name \"a\\\"b\\\\c\"\n");
        assert_eq!(got[1], TokenKind::Str("a\"b\\c".into()));
    }

    #[test]
    fn unterminated_string_is_an_error() {
        let err = tokenize("name \"oops\n").unwrap_err();
        assert!(err.is_error());
        assert_eq!(err.line, 1);
    }

    #[test]
    fn dash_marks_bullets() {
        let src = "summary:\n  - \"hello\"\n";
        let got = kinds(src);
        assert!(got.contains(&TokenKind::Dash));
    }
}
