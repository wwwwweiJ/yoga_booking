# syntax=docker/dockerfile:1
#
# Multi-stage build for the Yuan Yoga app. One image contains both the SPA
# (served as static files) and the Rust API. Platform-agnostic — the same
# image runs on Koyeb, Render, Railway, Fly, or any Docker host.
#
# See DEPLOY.md for the Koyeb + Neon walkthrough and the env vars to set.

# ---------------------------------------------------------------------------
# Stage 1 — build the React SPA into static files (frontend/dist)
# ---------------------------------------------------------------------------
FROM node:22-slim AS frontend
WORKDIR /app/frontend

# Corepack ships with Node and pins pnpm to the version in package.json's
# "packageManager" field, so the build uses the exact pnpm the repo expects.
# The env var stops Corepack from prompting before it fetches that version.
ENV COREPACK_ENABLE_DOWNLOAD_PROMPT=0
RUN corepack enable

# Install deps first, in their own layer — cached unless the manifest or the
# lockfile changes, so source-only edits don't re-run install.
COPY frontend/package.json frontend/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

# Then build. The ts-rs bindings are committed under src/bindings, so the
# type-check has everything it needs without touching the Rust side.
COPY frontend/ ./
RUN pnpm build

# ---------------------------------------------------------------------------
# Stage 2 — build the Rust backend (release binary)
# ---------------------------------------------------------------------------
FROM rust:1-slim-bookworm AS backend
WORKDIR /app

# The whole stack (sea-orm + lettre) is rustls-based — no OpenSSL in the
# dependency tree — so the slim image's bundled C toolchain is all that
# `ring` needs to compile. No extra apt packages required.
COPY Cargo.toml Cargo.lock ./
COPY .cargo/ .cargo/
COPY migration/ migration/
COPY src/ src/
RUN cargo build --release --bin yoga_booking-cli

# ---------------------------------------------------------------------------
# Stage 3 — minimal runtime
# ---------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# ca-certificates: the DB driver speaks TLS to a managed Postgres (e.g. Neon),
# and needs the system trust roots to verify the server's certificate.
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# Everything the app reads at runtime, relative to this working directory:
#   ./yoga_booking-cli   the server binary
#   ./config/            production.yaml, selected by LOCO_ENV
#   ./frontend/dist/     the SPA — served as static files with an SPA fallback
COPY --from=backend /app/target/release/yoga_booking-cli ./yoga_booking-cli
COPY config/ ./config/
COPY --from=frontend /app/frontend/dist ./frontend/dist

# LOCO_ENV selects config/production.yaml. The server binds 0.0.0.0:$PORT (see
# config/production.yaml); PORT defaults to 5150 here — set your platform's
# service port to match, or override PORT to whatever it routes to.
ENV LOCO_ENV=production
ENV PORT=5150
EXPOSE 5150

CMD ["./yoga_booking-cli", "start"]
