# cv_lang

[![CI](https://github.com/youssefbriki1/cv_prog_language/actions/workflows/ci.yml/badge.svg)](https://github.com/youssefbriki1/cv_prog_language/actions/workflows/ci.yml)

A small, Rust-based declarative language for writing CVs/resumes. You write a
concise `.cv` file; `cv_lang` compiles it into a complete LaTeX document
(targeting [Jake Gutierrez's resume template](https://github.com/jakegut/resume))
and, optionally, a PDF.

```
name "Youssef Briki"
contact email "youssef@example.com", github "youssefbriki1", location "Montréal, QC"

section "Experience":
  entry role "AI Engineering Intern"
        org  "Desjardins"
        when "Summer 2025"
        bullets:
          - "Built domain-specific RAG on a knowledge graph"
          - "Reduced retrieval latency by 35% (P95)"

section "Skills":
  tags: "Python, Rust, Java, LangChain, FAISS, Docker"
```

## Why a DSL?

The `.cv` format is intentionally tiny, declarative, and forgiving. That makes it
a great target for an LLM to generate, while the compiler guarantees the output
is **deterministic**, **auto-escaped** (`&`, `%`, `_`, … can't break LaTeX), and
**template-correct**. The fuzzy work (facts → structure) stays in the model; the
brittle work (valid LaTeX) stays in the compiler.

## Pipeline

```
.cv source ─► Lexer ─► Parser ─► Renderer ─► .tex ─►(pdflatex)─► .pdf
            tokens     AST       LaTeX
```

- **Lexer** (`src/lexer.rs`) — indentation-aware tokenizer (emits Indent/Dedent/Newline).
- **Parser** (`src/parser.rs`) — recursive descent into an AST; unknown fields/constructs become warnings, not errors.
- **Renderer** (`src/renderer.rs`) — walks the AST and emits a standalone Jake-template document, escaping all user text.
- **CLI** (`src/main.rs`) — reads a `.cv`, writes `.tex`, optionally runs `pdflatex`.

The library entry point is `cv_lang::compile(source) -> Compiled { latex, warnings }`
(`src/lib.rs`).

## Install & build

Requires a Rust toolchain (edition 2024, Rust ≥ 1.85).

```bash
cargo build --release      # binary at target/release/cv_lang
```

PDF output additionally needs a TeX distribution (`pdflatex`). If you don't have
one, use the Docker image below.

## Usage

```bash
cargo run -- examples/core.cv              # writes examples/core.tex
cargo run -- examples/core.cv -o out.tex   # custom output path
cargo run -- examples/core.cv --pdf        # also run pdflatex (if installed)
cargo run -- examples/core.cv --check      # validate only; write nothing
cargo run -- examples/core.cv --strict     # exit non-zero if there are warnings
cargo run -- examples/core.cv --json       # machine-readable result + diagnostics
```

```
usage: cv_lang <input.cv> [-o <output.tex>] [--pdf] [--check] [--strict] [--format human|json]
```

Flags:

- `--pdf` — also run `pdflatex` (a missing binary is a warning, not a crash).
- `--check` — parse and validate only; do not write any files.
- `--strict` — treat warnings as failures (non-zero exit).
- `--format json` (or `--json`) — emit a single JSON object with the output
  path(s) and diagnostics, e.g.:
  ```json
  {"ok":true,"check":false,"output":"core.tex","pdf":null,"warnings":[]}
  ```

In human mode, warnings point at the offending source line:

```
warning: line 4: unknown entry field 'mood'; ignored
     4 |         mood "great"
       |         ^^^^^^^^^^^^
```

Warnings (unknown fields/constructs) never stop compilation; the only hard error
is malformed input such as an unterminated string.

## Language reference

Indentation defines nesting. **Every value is a double-quoted string.** `#`
starts a comment; blank lines are ignored.

### Core constructs

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

### Extended constructs

Per-entry optional fields:

- `location "..."` — entry location
- `link "https://..."` — makes the entry's role a hyperlink
- `stack: "tech, tech, ..."` — technology list (rendered as a leading italic bullet)

Top-level optional blocks:

- `summary:` — a list of `- "..."` bullets at the top of the resume
- `sidebar:` — `key "value"` lines (location, email, github, linkedin, languages, skills)

Linkified contact/sidebar keys: `email`, `github`, `gitlab`, `linkedin`,
`twitter`/`x`, `orcid`, `scholar`, `website`/`site`/`link`/`url`. Other keys
(e.g. `location`, `phone`, `languages`, `skills`) render as plain text.

```
summary:
  - "SWE + NLP, focused on RAG and knowledge graphs."

section "Experience":
  entry role "AI Engineering Intern"
        org  "Desjardins"
        when "Summer 2025"
        location "Montréal, QC"
        link "https://desjardins.com"
        stack: "Python, LangChain, FAISS, vLLM"
        bullets:
          - "Built domain RAG over the knowledge graph"

sidebar:
  location "Montréal, QC"
  email "youssef@example.com"
  github "github.com/youssefbriki1"
  linkedin "linkedin.com/in/youssefbriki"
  languages "English, French"
  skills "Python, Rust, RAG, Docker"
```

> Layout: with **no** `sidebar`, the document is the classic single-column Jake
> template. When a `sidebar` **is** present, `cv_lang` switches to a two-column
> layout — a left rail (name + contact + sidebar fields) beside the main column
> (summary + sections).

## Examples

Ready-to-compile samples live in [`examples/`](examples/):

- `core.cv` — core constructs only
- `extended.cv` — summary, per-entry location/link/stack, and a sidebar
- `summary_only.cv` — a mix used as a smoke test
- `edge_cases.cv` — special characters, comments, an unknown field, and an empty
  section (compiles with warnings — the language is forgiving)

```bash
cargo run -- examples/extended.cv --pdf
```

## Docker (PDF out of the box)

The image bundles `cv_lang` plus a minimal TeX Live, so `--pdf` works without
installing anything locally.

```bash
docker build -t cv_lang .

# Mount a directory containing your .cv file and compile it:
docker run --rm -v "$PWD/examples":/work cv_lang core.cv --pdf
# -> examples/core.tex and examples/core.pdf
```

Published images: `ghcr.io/youssefbriki1/cv_prog_language` (built and pushed by
CI). Tag scheme:

- `:latest` — the tip of `main`.
- `:vX.Y.Z` — a tagged release (e.g. `:v0.1.0`).
- `:sha-<commit>` — every pushed commit on `main`/tags.

## Using cv_lang from an AI agent

`cv_lang` is designed to be the deterministic backend of an AI resume builder —
for example a Chrome extension that scrapes a profile/job page and generates a
tailored CV:

```
 Chrome extension (scrape page + UI)
          │  POST scraped text
          ▼
 Backend service ──► LLM (with the cv-lang skill) ──► .cv source
          │
          ▼
 cv_lang (CLI / container) ──► .tex ──► pdflatex ──► .pdf  ──► back to the extension
```

The LLM only authors the `.cv` DSL; `cv_lang` produces the LaTeX/PDF. The browser
extension itself never runs the compiler — it calls the backend, which invokes
the `cv_lang` container.

A portable **Agent Skill** that teaches an LLM the full `.cv` syntax and how to
compile lives at [`skills/cv-lang/SKILL.md`](skills/cv-lang/SKILL.md). Copy it
into your agent's skill directory (or `.claude/skills/`).

## Development

```bash
cargo test                          # unit + integration + golden + pdf tests
cargo clippy --all-targets          # lints
cargo fmt --all                     # format
```

Tests include golden snapshots of the rendered LaTeX (`tests/golden/`) and a
`pdflatex` round-trip over every example (`tests/pdf.rs`, skipped when TeX is
absent). After an intentional renderer change, re-bless the snapshots with
`CV_LANG_BLESS=1 cargo test --test golden`. See [`CONTRIBUTING.md`](CONTRIBUTING.md).

The toolchain is pinned in `rust-toolchain.toml`. CI (`.github/workflows/ci.yml`)
runs `fmt --check`, `clippy -D warnings`, and the test suite, then builds the
Docker image (and pushes it to GHCR on `main`/tags).

## Roadmap

- An optional thin HTTP service (`POST .cv → .tex/.pdf`) so backends can call it
  directly instead of shelling out.
- More resume templates beyond Jake (pluggable renderer / `--template`).
