#!/usr/bin/env bash
#
# One-command dev launcher for Yoga Booking.
#
#   ./dev.sh
#
# Brings up the whole stack: Postgres (docker), the Loco API on :5150, and the
# Vite frontend on :5173. Ctrl-C stops the API and frontend; Postgres is left
# running (use `docker compose down` to stop it, `-v` to also wipe data).

set -uo pipefail
cd "$(dirname "$0")"

log() { printf '\033[1;32m▶ %s\033[0m\n' "$*"; }
die() { printf '\033[1;31m✗ %s\033[0m\n' "$*" >&2; exit 1; }

command -v docker >/dev/null || die "docker not found"
command -v cargo  >/dev/null || die "cargo not found"

# 1) Postgres — start and wait until the container reports healthy.
log "Starting Postgres…"
docker compose up -d || die "failed to start Postgres (is Docker running?)"
until [ "$(docker inspect -f '{{.State.Health.Status}}' yoga_booking_pg 2>/dev/null)" = "healthy" ]; do
  sleep 0.5
done
log "Postgres is healthy."

# 2) Frontend dependencies (first run only).
if [ ! -d frontend/node_modules ]; then
  log "Installing frontend dependencies…"
  ( cd frontend && corepack pnpm install ) || die "pnpm install failed"
fi

# 3) Build the backend up front, so a compile error stops us before we launch
#    anything (and migrations run automatically on boot via auto_migrate).
log "Building backend…"
cargo build --bin yoga_booking-cli || die "backend build failed"

# 4) Launch API + frontend, and make sure Ctrl-C takes both down.
pids=()
cleanup() {
  log "Stopping API & frontend (Postgres left running — 'docker compose down' to stop it)…"
  for pid in "${pids[@]:-}"; do
    pkill -P "$pid" 2>/dev/null || true   # child (the real server / vite)
    kill "$pid"     2>/dev/null || true   # the launcher itself
  done
}
trap cleanup EXIT INT TERM HUP

log "API      → http://localhost:5150"
target/debug/yoga_booking-cli start &
pids+=($!)

log "Frontend → http://localhost:5173"
( cd frontend && corepack pnpm dev ) &
pids+=($!)

log "Up. Press Ctrl-C to stop."
wait
