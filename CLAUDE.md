# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust-based domain-specific language (DSL) that compiles CV/resume source files (`.cv`)
into LaTeX, targeting Jake Gutierrez's resume template. The language syntax is
Rust-inspired but declarative. `cv_lang` is meant to be the deterministic backend of an
AI resume builder: an LLM authors the `.cv` DSL (the fuzzy part) and `cv_lang` produces
correct, escaped LaTeX/PDF (the brittle part).

## Language Syntax

Indentation defines nesting. Every value is a double-quoted string. `#` starts a comment;
blank lines are ignored.

**Core constructs:**
```
name "Full Name"
contact email "...", github "...", location "..."

section "Title":
  entry role "..."
        org  "..."
        when "..."
        bullets:
          - "..."

section "Skills":
  tags: "comma-separated list"
```

**Extended constructs** (additional entry fields):
- `location "..."` — per-entry location
- `link "https://..."` — URL for the entry (renders the role as a hyperlink)
- `stack: "tech, tech, ..."` — technology tags per entry (leading italic bullet)
- `summary:` block — top-level summary bullets
- `sidebar:` block — personal data (location, email, github, linkedin, languages, skills)

## Architecture (implemented)

Pipeline: **Lexer → Parser → Renderer → CLI**.

- `src/lexer.rs` — indentation-aware tokenizer. Emits `Indent`/`Dedent`/`Newline` tokens
  and is deliberately keyword-agnostic: every bare word is an `Ident`, so the parser owns
  all keyword meaning (this is what keeps the language forgiving).
- `src/ast.rs` — `Document`, `Section`, `SectionBody` (`Entries`/`Tags`), `Entry`, `Field`.
- `src/parser.rs` — recursive descent. Dispatches by `match`ing the leading identifier.
  Unknown fields/constructs become **warnings**, not errors, and the line is skipped.
- `src/renderer.rs` — walks the AST and emits a **standalone** Jake-template document
  (`\documentclass … \end{document}`). All user text passes through `latex_escape`. The
  Jake preamble + helper macros live in a `PREAMBLE` constant.
- `src/lib.rs` — public entry point `compile(source) -> Result<Compiled, Diagnostic>`
  where `Compiled { latex, warnings }`.
- `src/main.rs` — CLI: `cv_lang <input.cv> [-o <output.tex>] [--pdf]`. Always writes the
  `.tex`; `--pdf` runs `pdflatex` best-effort (a missing binary is a warning, not a crash).
- `src/error.rs` — `Diagnostic { level, message, line }`, `Level::{Warning, Error}`.

Tests: unit tests live in each module (`#[cfg(test)]`); end-to-end tests in
`tests/integration.rs` compile every `examples/*.cv`.

Design note: the **sidebar is currently folded into the header** (classic Jake is
single-column). A true two-column sidebar is a roadmap item.

Other paths: `examples/*.cv` (fixtures), `skills/cv-lang/SKILL.md` (portable Agent Skill
teaching an LLM the syntax), `Dockerfile` (PDF-capable image), `.github/workflows/ci.yml`.

## Build & Run

```bash
cargo build                 # compile
cargo run -- <file.cv>      # compile a .cv (writes <file>.tex)
cargo run -- <file.cv> --pdf# also run pdflatex (needs a TeX install)
cargo test                  # run all tests
cargo test <name>           # run a single test by name
cargo clippy --all-targets  # lint
cargo fmt --all             # format

# Docker (bundles TeX Live, so --pdf works without a local TeX install):
docker build -t cv_lang .
docker run --rm -v "$PWD/examples":/work cv_lang core.cv --pdf
```

CI (`.github/workflows/ci.yml`) gates on `fmt --check`, `clippy -D warnings`, and `cargo
test`, then builds the Docker image (pushing to GHCR on `main`/tags).

## Notes

- Output LaTeX targets Jake's resume template; the renderer emits a complete, compilable
  document.
- The language is forgiving by design: unrecognised optional fields/constructs produce a
  warning (collected on `Compiled.warnings` and printed to stderr), not a hard error. The
  only fatal lexer condition is an unterminated string literal.
- No external crate dependencies — standard library only.
