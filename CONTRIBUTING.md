# Contributing to cv_lang

Thanks for your interest! This is a small, dependency-free Rust project, so the
workflow is simple.

## Prerequisites

- Rust (the toolchain is pinned in `rust-toolchain.toml`; rustup will install it
  automatically). Edition 2024 requires Rust ≥ 1.85.
- Optional: a TeX distribution (`pdflatex`) to test PDF output, or use Docker.

## Development workflow

```bash
cargo build                  # compile
cargo test                   # run all tests
cargo clippy --all-targets   # lint (CI runs with -D warnings)
cargo fmt --all              # format (CI checks with --check)
```

Before opening a PR, make sure these all pass — they are exactly what CI gates on:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all
```

## Project layout

The pipeline is **Lexer → Parser → Renderer → CLI** (see `CLAUDE.md` for detail):

- `src/lexer.rs` — indentation-aware tokenizer.
- `src/ast.rs` — `Document` / `Section` / `Entry` types.
- `src/parser.rs` — recursive descent; unknown fields become *warnings*.
- `src/renderer.rs` — AST → standalone LaTeX (Jake template; two-column when a
  `sidebar` is present).
- `src/main.rs` — CLI; `src/lib.rs` — the `compile()` entry point.

## Tests

- **Unit tests** live in each module under `#[cfg(test)]`.
- **`tests/integration.rs`** — compiles the core example fixtures end to end.
- **`tests/golden.rs`** — snapshot tests comparing rendered LaTeX to
  `tests/golden/*.tex`. After an intentional renderer change, regenerate them:
  ```bash
  CV_LANG_BLESS=1 cargo test --test golden
  ```
  Review the diff before committing the updated golden files.
- **`tests/pdf.rs`** — compiles every `examples/*.cv` to a real PDF. Skipped
  automatically when `pdflatex` is not installed.

## Adding an example

Drop a `.cv` file in `examples/`. The `tests/pdf.rs` test will pick it up and
require it to compile to a PDF, so keep it valid (warnings are fine). If you want
it covered by golden tests too, add its stem to the lists in `tests/golden.rs`
and bless it.

## Language reference

The full `.cv` syntax is documented in `README.md` and, for LLM/agent use, in
`skills/cv-lang/SKILL.md`. Keep both in sync when you add or change a construct.
