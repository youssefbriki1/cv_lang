# cv_lang — TODO

Roadmap for turning `cv_lang` into the deterministic backend of an AI resume builder
(Chrome extension → agent → `.cv` → LaTeX/PDF). Ordered roughly by priority.

## Next up
- [x] Open the PR for `feat/agent-skill-docs-ci-docker` and confirm CI goes green.
- [x] Verify the Docker image end-to-end (build + `--pdf` round-trip).
- [ ] In repo Settings → Actions, confirm workflow permissions allow GHCR push
      (the `docker` job needs `packages: write`). *(manual — needs repo admin)*

## Language & compiler
- [x] True two-column sidebar layout (rendered when a `sidebar` is present).
- [x] `--check` mode: parse only, print warnings, write nothing. `--strict` makes
      warnings a non-zero exit.
- [x] Emit warnings/results as structured JSON (`--format json` / `--json`).
- [x] Richer contact/sidebar key handling (phone, website, x/twitter, gitlab,
      orcid, scholar).
- [x] Friendlier diagnostics: show the offending source line + a caret.
- [ ] Multiple resume templates beyond Jake (pluggable renderer / `--template`).

## Testing & quality
- [x] Golden/snapshot tests for rendered LaTeX (`tests/golden/`, stdlib only).
- [x] A test that actually runs `pdflatex` on each example (`tests/pdf.rs`, gated
      on TeX being present).
- [x] Lexer robustness tests (tabs, CRLF, comment-only, empty, `#` in strings).
- [x] Edge-case example (`examples/edge_cases.cv`: special chars, comments,
      unknown field, empty section).

## Packaging & infra
- [x] Pin the Rust toolchain (`rust-toolchain.toml`).
- [x] Trim the TeX Live package set / drop unused preamble packages
      (removed marvosym + latexsym, dropped `texlive-fonts-extra`).
- [x] Publish a tagged release (`v0.1.0`) and document the GHCR image tag scheme.
- [ ] Optional: build a static musl binary for a tiny no-PDF image variant.

## Docs
- [x] CONTRIBUTING.md (build, test, fmt, clippy, golden-bless workflow).
- [x] Keep README / CLAUDE.md / SKILL.md in sync (new flags, two-column, keys).
- [ ] Add a short GIF/screenshot of a rendered resume to the README.

## AI-agent integration (separate projects — out of scope for this repo)
- [ ] Thin HTTP service: `POST /compile` with `.cv` → `{ tex, pdf?, warnings }`.
- [ ] Backend glue: call Claude with `skills/cv-lang/SKILL.md` to author `.cv`.
- [ ] Chrome extension: scrape page → backend → preview/download the PDF.
- [ ] Prompt/skill hardening on real scraped inputs.
- [ ] Guardrails: cap input size, sanitize/validate, rate-limit.
