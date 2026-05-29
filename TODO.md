# cv_lang — TODO

Roadmap for turning `cv_lang` into the deterministic backend of an AI resume builder
(Chrome extension → agent → `.cv` → LaTeX/PDF). Ordered roughly by priority.

## Next up
- [ ] Open the PR for `feat/agent-skill-docs-ci-docker` and confirm CI goes green.
- [ ] Verify the Docker image end-to-end (couldn't run locally — no `pdflatex`):
      `docker build -t cv_lang .` then
      `docker run --rm -v "$PWD/examples":/work cv_lang core.cv --pdf` → expect a PDF.
- [ ] In repo Settings → Actions, confirm workflow permissions allow GHCR push
      (the `docker` job needs `packages: write`).

## Language & compiler
- [ ] True two-column sidebar layout (currently folded into the header).
- [ ] Add a `--check` / lint mode: parse only, print warnings, non-zero exit on warnings
      (useful for the agent to validate its own `.cv` output).
- [ ] Emit warnings as structured JSON (`--format json`) so the agent can consume them.
- [ ] Support multiple resume templates beyond Jake (pluggable renderer / `--template`).
- [ ] Richer contact/sidebar key handling (phone, website, scholar, ORCID).
- [ ] Friendlier diagnostics: show the offending source line + a caret.

## Testing & quality
- [ ] Snapshot tests for rendered LaTeX (e.g. `insta`) to catch unintended output drift.
- [ ] A test that actually runs `pdflatex` on each example (gated on TeX being present,
      e.g. only in the Docker/CI job).
- [ ] Fuzz / property tests for the lexer (unterminated strings, weird indentation, tabs).
- [ ] Add `examples/` covering edge cases: empty sections, unknown fields, comments,
      special chars (`& % _ # $`).

## Packaging & infra
- [ ] Pin the Rust toolchain (`rust-toolchain.toml`) so local + CI match.
- [ ] Cache `pdflatex` runs / trim the TeX Live package set if the image is too big
      (dropping unused `marvosym`/`latexsym` lets you skip `texlive-fonts-extra`).
- [ ] Publish a tagged release (`vX.Y.Z`) and document the GHCR image tag scheme.
- [ ] Optional: build a static musl binary for a tiny no-PDF image variant.

## AI-agent integration
- [ ] Thin HTTP service: `POST /compile` with `.cv` body → `{ tex, pdf?, warnings }`.
      (Keeps the backend from shelling out; small web dep like `axum`.)
- [ ] Backend glue: call Claude with `skills/cv-lang/SKILL.md` to author `.cv` from
      scraped text, then compile.
- [ ] Chrome extension: scrape page → send to backend → preview/download the PDF.
- [ ] Prompt/skill hardening: test the SKILL on real scraped inputs; iterate on the
      authoring rules so the model reliably produces warning-free `.cv`.
- [ ] Guardrails: cap input size, sanitize/validate before compiling, rate-limit.

## Docs
- [ ] Add a short GIF/screenshot of a rendered resume to the README.
- [ ] Document the HTTP API once it exists.
- [ ] Write a CONTRIBUTING.md (build, test, fmt, clippy expectations).
