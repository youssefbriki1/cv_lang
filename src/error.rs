//! Diagnostics produced by the lexer, parser and renderer.
//!
//! The language is intentionally *forgiving*: unrecognised optional constructs
//! produce a [`Level::Warning`] rather than aborting compilation. Only genuinely
//! malformed input (e.g. an unterminated string) produces a [`Level::Error`].

use std::fmt;

/// Severity of a [`Diagnostic`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// Non-fatal: compilation continues, the message is surfaced to the user.
    Warning,
    /// Fatal: compilation cannot produce meaningful output.
    Error,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::Warning => write!(f, "warning"),
            Level::Error => write!(f, "error"),
        }
    }
}

/// A single message tied to a source line (1-based; `0` means "unknown line").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub level: Level,
    pub message: String,
    pub line: usize,
}

impl Diagnostic {
    pub fn error(line: usize, message: impl Into<String>) -> Self {
        Diagnostic {
            level: Level::Error,
            message: message.into(),
            line,
        }
    }

    pub fn warning(line: usize, message: impl Into<String>) -> Self {
        Diagnostic {
            level: Level::Warning,
            message: message.into(),
            line,
        }
    }

    pub fn is_error(&self) -> bool {
        self.level == Level::Error
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(f, "{}: {}", self.level, self.message)
        } else {
            write!(f, "{}: line {}: {}", self.level, self.line, self.message)
        }
    }
}
