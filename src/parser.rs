//! Recursive-descent parser: a `Vec<Token>` becomes a [`Document`].
//!
//! Structure is dispatched almost entirely by `match`ing on the leading
//! identifier of each construct. Unknown constructs and fields never abort the
//! parse — they record a [`Level::Warning`](crate::error::Level::Warning) and
//! the offending line is skipped, honouring the language's forgiving contract.

use crate::ast::{Document, Entry, Field, Section, SectionBody};
use crate::error::Diagnostic;
use crate::lexer::{Token, TokenKind};

/// Parse a token stream into a [`Document`] plus any non-fatal warnings.
pub fn parse(tokens: Vec<Token>) -> (Document, Vec<Diagnostic>) {
    let mut parser = Parser::new(tokens);
    let doc = parser.parse_document();
    (doc, parser.warnings)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    warnings: Vec<Diagnostic>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            pos: 0,
            warnings: Vec::new(),
        }
    }

    // --- cursor primitives ---------------------------------------------------

    fn peek(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    fn peek_line(&self) -> usize {
        self.tokens[self.pos].line
    }

    fn advance(&mut self) -> TokenKind {
        let kind = self.tokens[self.pos].kind.clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        kind
    }

    fn is(&self, kind: &TokenKind) -> bool {
        self.peek() == kind
    }

    /// Consume `kind` if it is next; report whether it was.
    fn eat(&mut self, kind: &TokenKind) -> bool {
        if self.is(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn skip_newlines(&mut self) {
        while self.eat(&TokenKind::Newline) {}
    }

    /// Take the next token if it is a string literal.
    fn take_str(&mut self) -> Option<String> {
        if let TokenKind::Str(_) = self.peek() {
            match self.advance() {
                TokenKind::Str(s) => Some(s),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    /// Take the next token if it is an identifier.
    fn take_ident(&mut self) -> Option<String> {
        if let TokenKind::Ident(_) = self.peek() {
            match self.advance() {
                TokenKind::Ident(s) => Some(s),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    fn warn(&mut self, line: usize, message: impl Into<String>) {
        self.warnings.push(Diagnostic::warning(line, message));
    }

    /// Discard tokens up to and including the next [`Newline`](TokenKind::Newline)
    /// (or end of input) — used for error recovery.
    fn skip_line(&mut self) {
        while !matches!(self.peek(), TokenKind::Newline | TokenKind::Eof) {
            self.advance();
        }
        self.eat(&TokenKind::Newline);
    }

    /// Skip a block (`Newline Indent ... Dedent`) attached to an unrecognised
    /// field, balancing nested Indent/Dedent pairs.
    fn skip_optional_block(&mut self) {
        self.eat(&TokenKind::Newline);
        if !self.eat(&TokenKind::Indent) {
            return;
        }
        let mut depth = 1usize;
        while depth > 0 && !matches!(self.peek(), TokenKind::Eof) {
            match self.peek() {
                TokenKind::Indent => depth += 1,
                TokenKind::Dedent => depth -= 1,
                _ => {}
            }
            self.advance();
        }
    }

    // --- grammar -------------------------------------------------------------

    fn parse_document(&mut self) -> Document {
        let mut doc = Document::default();
        loop {
            // Tolerate stray structural tokens between top-level constructs.
            match self.peek() {
                TokenKind::Eof => break,
                TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent => {
                    self.advance();
                    continue;
                }
                _ => {}
            }

            let line = self.peek_line();
            match self.take_ident() {
                Some(kw) => match kw.as_str() {
                    "name" => self.parse_name(&mut doc),
                    "contact" => self.parse_pairs_inline(line, &mut doc.contact),
                    "summary" => doc.summary = self.parse_dash_block(),
                    "sidebar" => self.parse_pairs_block(&mut doc.sidebar),
                    "section" => {
                        if let Some(section) = self.parse_section(line) {
                            doc.sections.push(section);
                        }
                    }
                    other => {
                        self.warn(
                            line,
                            format!("unknown top-level construct '{other}'; ignored"),
                        );
                        self.skip_line();
                    }
                },
                None => {
                    self.warn(line, "expected a construct keyword; line ignored");
                    self.skip_line();
                }
            }
        }
        doc
    }

    fn parse_name(&mut self, doc: &mut Document) {
        let line = self.peek_line();
        match self.take_str() {
            Some(name) => doc.name = Some(name),
            None => self.warn(line, "`name` expects a quoted string"),
        }
        self.skip_line();
    }

    /// `key "value", key "value", ...` on a single line (used by `contact`).
    fn parse_pairs_inline(&mut self, line: usize, out: &mut Vec<Field>) {
        loop {
            let Some(key) = self.take_ident() else {
                self.warn(line, "expected a field name (e.g. email \"...\")");
                break;
            };
            self.eat(&TokenKind::Colon); // optional
            match self.take_str() {
                Some(value) => out.push(Field { key, value }),
                None => self.warn(line, format!("`{key}` expects a quoted string")),
            }
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }
        self.skip_line();
    }

    /// One `key "value"` per indented line (used by `sidebar`).
    fn parse_pairs_block(&mut self, out: &mut Vec<Field>) {
        self.eat(&TokenKind::Colon);
        self.eat(&TokenKind::Newline);
        if !self.eat(&TokenKind::Indent) {
            return;
        }
        loop {
            self.skip_newlines();
            let line = self.peek_line();
            let Some(key) = self.take_ident() else { break };
            self.eat(&TokenKind::Colon);
            match self.take_str() {
                Some(value) => out.push(Field { key, value }),
                None => self.warn(line, format!("`{key}` expects a quoted string")),
            }
            self.eat(&TokenKind::Newline);
        }
        self.eat(&TokenKind::Dedent);
    }

    /// A `:`-introduced block of `- "item"` lines (used by `summary` and `bullets`).
    fn parse_dash_block(&mut self) -> Vec<String> {
        let mut items = Vec::new();
        self.eat(&TokenKind::Colon);
        self.eat(&TokenKind::Newline);
        if !self.eat(&TokenKind::Indent) {
            return items;
        }
        loop {
            self.skip_newlines();
            if !self.eat(&TokenKind::Dash) {
                break;
            }
            let line = self.peek_line();
            match self.take_str() {
                Some(item) => items.push(item),
                None => self.warn(line, "bullet `-` expects a quoted string"),
            }
            self.eat(&TokenKind::Newline);
        }
        self.eat(&TokenKind::Dedent);
        items
    }

    fn parse_section(&mut self, line: usize) -> Option<Section> {
        let Some(title) = self.take_str() else {
            self.warn(line, "`section` expects a quoted title");
            self.skip_line();
            return None;
        };
        self.eat(&TokenKind::Colon);
        self.eat(&TokenKind::Newline);

        if !self.eat(&TokenKind::Indent) {
            self.warn(line, format!("section '{title}' is empty"));
            return Some(Section {
                title,
                body: SectionBody::Entries(Vec::new()),
            });
        }

        self.skip_newlines();
        // A `tags:` line means a flat skills section; otherwise it's entries.
        let body = if matches!(self.peek(), TokenKind::Ident(k) if k == "tags") {
            self.advance(); // `tags`
            self.eat(&TokenKind::Colon);
            let tags = self.take_str().unwrap_or_default();
            self.skip_line();
            SectionBody::Tags(tags)
        } else {
            SectionBody::Entries(self.parse_entries())
        };

        // Consume to the end of this section's indented block.
        while !matches!(self.peek(), TokenKind::Dedent | TokenKind::Eof) {
            self.advance();
        }
        self.eat(&TokenKind::Dedent);
        Some(Section { title, body })
    }

    fn parse_entries(&mut self) -> Vec<Entry> {
        let mut entries = Vec::new();
        loop {
            self.skip_newlines();
            match self.peek() {
                TokenKind::Ident(k) if k == "entry" => {
                    self.advance();
                    entries.push(self.parse_entry());
                }
                _ => break,
            }
        }
        entries
    }

    fn parse_entry(&mut self) -> Entry {
        let mut entry = Entry::default();

        // Inline fields on the `entry ...` line (typically `role "..."`).
        while matches!(self.peek(), TokenKind::Ident(_)) {
            self.parse_entry_field(&mut entry);
        }
        self.eat(&TokenKind::Newline);

        // Indented block of further fields (org, when, stack, bullets, ...).
        if self.eat(&TokenKind::Indent) {
            loop {
                self.skip_newlines();
                if !matches!(self.peek(), TokenKind::Ident(_)) {
                    break;
                }
                self.parse_entry_field(&mut entry);
                self.eat(&TokenKind::Newline);
            }
            self.eat(&TokenKind::Dedent);
        }
        entry
    }

    /// Parse one entry field. Inline fields carry a string value; `bullets`
    /// introduces a dash block. The field name drives assignment via `match`.
    fn parse_entry_field(&mut self, entry: &mut Entry) {
        let line = self.peek_line();
        let Some(name) = self.take_ident() else {
            return;
        };
        self.eat(&TokenKind::Colon); // optional

        match self.peek() {
            TokenKind::Str(_) => {
                let value = self.take_str().unwrap();
                match name.as_str() {
                    "role" => entry.role = Some(value),
                    "org" => entry.org = Some(value),
                    "when" => entry.when = Some(value),
                    "location" => entry.location = Some(value),
                    "link" => entry.link = Some(value),
                    "stack" => entry.stack = Some(value),
                    other => self.warn(line, format!("unknown entry field '{other}'; ignored")),
                }
            }
            TokenKind::Newline => {
                if name == "bullets" {
                    entry.bullets = self.parse_dash_block();
                } else {
                    self.warn(line, format!("unknown entry block '{name}'; ignored"));
                    self.skip_optional_block();
                }
            }
            _ => self.warn(line, format!("field '{name}' has no value; ignored")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    fn parse_src(src: &str) -> (Document, Vec<Diagnostic>) {
        parse(tokenize(src).unwrap())
    }

    #[test]
    fn parses_core_example() {
        let src = "\
name \"Youssef Briki\"
contact email \"y@example.com\", github \"youssefbriki1\"

section \"Experience\":
  entry role \"AI Engineering Intern\"
        org  \"Desjardins\"
        when \"Summer 2025\"
        bullets:
          - \"Built RAG\"
          - \"Cut latency 35%\"

section \"Skills\":
  tags: \"Python, Rust\"
";
        let (doc, warnings) = parse_src(src);
        assert_eq!(doc.name.as_deref(), Some("Youssef Briki"));
        assert_eq!(doc.contact.len(), 2);
        assert_eq!(
            doc.contact[0],
            Field {
                key: "email".into(),
                value: "y@example.com".into()
            }
        );
        assert_eq!(doc.sections.len(), 2);

        match &doc.sections[0].body {
            SectionBody::Entries(es) => {
                assert_eq!(es.len(), 1);
                let e = &es[0];
                assert_eq!(e.role.as_deref(), Some("AI Engineering Intern"));
                assert_eq!(e.org.as_deref(), Some("Desjardins"));
                assert_eq!(e.when.as_deref(), Some("Summer 2025"));
                assert_eq!(e.bullets, vec!["Built RAG", "Cut latency 35%"]);
            }
            other => panic!("expected entries, got {other:?}"),
        }
        match &doc.sections[1].body {
            SectionBody::Tags(t) => assert_eq!(t, "Python, Rust"),
            other => panic!("expected tags, got {other:?}"),
        }
        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
    }

    #[test]
    fn parses_extended_entry_fields() {
        let src = "\
section \"Projects\":
  entry role \"LabMate\"
        org  \"Personal\"
        when \"2024\"
        location \"Montreal\"
        link \"https://example.com\"
        stack: \"Python, vLLM\"
        bullets:
          - \"Multi-hop retrieval\"
";
        let (doc, warnings) = parse_src(src);
        let SectionBody::Entries(es) = &doc.sections[0].body else {
            panic!()
        };
        let e = &es[0];
        assert_eq!(e.location.as_deref(), Some("Montreal"));
        assert_eq!(e.link.as_deref(), Some("https://example.com"));
        assert_eq!(e.stack.as_deref(), Some("Python, vLLM"));
        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
    }

    #[test]
    fn parses_summary_and_sidebar() {
        let src = "\
summary:
  - \"SWE + NLP\"

sidebar:
  location \"Montreal\"
  email \"y@example.com\"
";
        let (doc, _) = parse_src(src);
        assert_eq!(doc.summary, vec!["SWE + NLP"]);
        assert_eq!(doc.sidebar.len(), 2);
        assert_eq!(doc.sidebar[0].key, "location");
    }

    #[test]
    fn unknown_field_is_a_warning_not_an_error() {
        let src = "\
section \"Experience\":
  entry role \"Dev\"
        mood \"great\"
";
        let (doc, warnings) = parse_src(src);
        let SectionBody::Entries(es) = &doc.sections[0].body else {
            panic!()
        };
        assert_eq!(es[0].role.as_deref(), Some("Dev"));
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("mood"));
        assert!(!warnings[0].is_error());
    }

    #[test]
    fn unknown_top_level_construct_warns() {
        let (_doc, warnings) = parse_src("wibble \"x\"\nname \"A\"\n");
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("wibble"));
    }
}
