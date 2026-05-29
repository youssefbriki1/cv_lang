---
name: cv-lang
description: >-
  Authoring and compiling .cv resume files with the cv_lang compiler. Use when
  turning a person's experience (for example scraped LinkedIn, portfolio, or
  GitHub text) into a resume: write a .cv DSL source file, then compile it to
  LaTeX/PDF. Covers the full .cv syntax plus the CLI and Docker invocation.
---

# cv-lang

`cv_lang` is a small Rust compiler that turns a declarative `.cv` file into a
complete LaTeX document (Jake Gutierrez's resume template) and, optionally, a
PDF.

## When and why to use this

When you have unstructured information about a person (scraped from a web page,
pasted from a profile, etc.) and you need a polished resume, **write a `.cv`
file — do not write LaTeX directly.**

The `.cv` DSL is tiny and forgiving, so it is easy to produce correctly. The
compiler then guarantees the output is:

- **deterministic** — same input, same LaTeX;
- **auto-escaped** — `&`, `%`, `$`, `_`, `{`, `}`, etc. are escaped for you, so
  content like "R&D" or "cut cost 35%" never breaks the build;
- **template-correct** — always a valid, compilable Jake-template document.

So your job is only the fuzzy part: map the source facts into `.cv` structure.

## The `.cv` language

Indentation defines nesting (like YAML/Python). **Every value is a
double-quoted string.** `#` starts a comment. Blank lines are ignored.

### Top-level constructs

```
name "Full Name"

contact email "you@example.com", github "you", location "City, Country"

summary:
  - "One- or two-line positioning statement."
  - "Optional second bullet."

section "Experience":
  entry role "Job Title"
        org  "Company"
        when "2023–2025"
        location "City"            # optional
        link "https://company.com" # optional; makes the role a hyperlink
        stack: "Python, Rust, K8s" # optional; tech list for this entry
        bullets:
          - "Impactful, quantified achievement."
          - "Another achievement."

section "Skills":
  tags: "Comma, Separated, List, Of, Skills"

sidebar:
  location "City, Country"
  email "you@example.com"
  github "github.com/you"
  linkedin "linkedin.com/in/you"
  languages "English, French"
  skills "Python, Rust, Docker"
```

### Field reference

| Construct | Form | Notes |
|-----------|------|-------|
| `name` | `name "..."` | The person's full name (header). |
| `contact` | `contact key "val", key "val", ...` | Inline, comma-separated. `email` → mailto link; `github`/`linkedin` → hyperlinks; other keys (`location`, `phone`, …) render as plain text. |
| `summary` | `summary:` then `- "..."` lines | Optional top-of-resume bullets. |
| `section` | `section "Title":` then `entry` blocks **or** a single `tags:` line | A `tags:` body renders as a flat skills list; otherwise it holds entries. |
| `entry` | `entry role "..."` + indented fields | Fields: `role`, `org`, `when`, `location`, `link`, `stack`, and a `bullets:` block. |
| `sidebar` | `sidebar:` then `key "val"` lines | Personal/contact data. |

### Authoring rules (important)

- Use **only known field names** where possible. Unknown fields are *warnings*,
  not errors — the file still compiles — but they are dropped from the output.
- **Do not invent facts.** Only include information present in the source text.
- Keep bullets concise and impactful; quantify when the source provides numbers.
- Indent consistently. Entry fields sit deeper than the `entry` keyword; bullet
  `-` items sit deeper than `bullets:`.

## Compiling a `.cv` file

The compiler always writes a `.tex`. `--pdf` additionally runs `pdflatex`
(requires a TeX installation; the Docker image below bundles one).

```bash
# Local (Rust toolchain installed):
cargo run -- resume.cv                 # writes resume.tex
cv_lang resume.cv -o out.tex --pdf     # custom output + PDF

# Docker (PDF works out of the box — TeX Live is included):
docker run --rm -v "$PWD":/work \
  ghcr.io/youssefbriki1/cv_prog_language resume.cv --pdf
# -> resume.tex and resume.pdf in the mounted directory
```

Warnings (unknown fields/constructs) are printed to stderr; a non-zero exit only
happens on a hard error such as an unterminated string.

## Worked example

Source facts (e.g. scraped):

> Ada Lovelace — Software Engineer at Acme (2023 to now), based in London.
> Built a billing service handling 2M req/day, cut p99 latency 40%. Email
> ada@example.com, GitHub adalove. Skills: Rust, Go, PostgreSQL.

Authored `.cv`:

```
name "Ada Lovelace"
contact email "ada@example.com", github "adalove", location "London, UK"

section "Experience":
  entry role "Software Engineer"
        org  "Acme"
        when "2023–Present"
        location "London, UK"
        stack: "Rust, Go, PostgreSQL"
        bullets:
          - "Built a billing service handling 2M requests/day."
          - "Cut p99 latency by 40%."

section "Skills":
  tags: "Rust, Go, PostgreSQL"
```

Compiling produces a standalone `\documentclass … \end{document}` Jake-template
document with the `&`/`%`-safe content already escaped.

## Gotchas

- **Sidebar** is currently folded into the header contact line (classic Jake is
  single-column). A true left-column sidebar is on the roadmap.
- **`link`** on an entry turns the role into a clickable hyperlink.
- **`stack`** renders as the first, italicised "Stack:" bullet of the entry.
