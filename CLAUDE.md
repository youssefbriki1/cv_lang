# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust-based domain-specific language (DSL) that compiles CV/resume source files into LaTeX, initially targeting Jake's resume template. The language syntax is Rust-inspired but declarative.

## Language Syntax

The DSL has two layers of constructs — core (implemented first) and extended (richer optional fields):

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
- `link "https://..."` — URL for the entry
- `stack: "tech, tech, ..."` — technology tags per entry
- `summary:` block — top-level summary bullets
- `sidebar:` block — left-column data (location, email, github, linkedin, languages, skills)

## Expected Architecture

When implemented, the pipeline will be:
1. **Lexer** — tokenize `.cv` source files
2. **Parser** — produce an AST from tokens (sections, entries, bullets, tags, sidebar, summary)
3. **Renderer** — walk the AST and emit LaTeX targeting Jake's resume template
4. **CLI** — accept a `.cv` input file path and emit `.tex` (and optionally invoke `pdflatex`)

## Build & Run (Rust project conventions)

```bash
cargo build          # compile
cargo run -- <file>  # compile a .cv file
cargo test           # run all tests
cargo test <name>    # run a single test by name
cargo clippy         # lint
```

## Notes

- The repository is in early stage — only README.md exists; source code has not been committed yet.
- Output LaTeX must be compatible with Jake's resume template (two-column layout with optional sidebar).
- The language should be forgiving: unrecognised optional fields should produce a warning, not a hard error.
