# syntax=docker/dockerfile:1

# ---- Build stage: compile the release binary ----
FROM rust:1-slim-bookworm AS builder
WORKDIR /app

# Cache dependencies: copy manifests, build a stub, then copy real sources.
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src \
    && echo 'fn main() {}' > src/main.rs \
    && echo '' > src/lib.rs \
    && cargo build --release \
    && rm -rf src

COPY src ./src
# Touch sources so cargo rebuilds them after the dependency-cache layer.
RUN touch src/main.rs src/lib.rs && cargo build --release

# ---- Runtime stage: binary + minimal TeX Live for --pdf ----
FROM debian:bookworm-slim AS runtime

# Packages covering the Jake-template preamble: titlesec, enumitem, hyperref,
# fancyhdr, tabularx, babel, color, fullpage. (No marvosym/latexsym, so
# texlive-fonts-extra is not needed — keeps the image smaller.)
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        texlive-latex-recommended \
        texlive-latex-extra \
        texlive-fonts-recommended \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/cv_lang /usr/local/bin/cv_lang

WORKDIR /work
ENTRYPOINT ["cv_lang"]
CMD ["--help"]
