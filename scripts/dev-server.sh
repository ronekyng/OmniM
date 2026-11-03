#!/usr/bin/env bash
set -euo pipefail

PORT="${OMNIM_PORT:-3000}"
HOST="${OMNIM_HOST:-0.0.0.0}"
PROJECT="${OMNIM_PROJECT:-project.omnim}"

# OMNIM_PORT=3100 (and the frontend 5174) are reserved for Playwright E2E;
# an exported leftover would silently move this dev server off 3000 — warn so
# the port can only move loudly, never quietly.
if [ -n "${OMNIM_PORT:-}" ] && [ "${OMNIM_PORT}" != "3000" ]; then
  echo "[omnim] WARNING: OMNIM_PORT='${OMNIM_PORT}' is exported in this shell." >&2
  echo "[omnim] WARNING: dev normally runs on 3000; 3100 is reserved for E2E." >&2
fi

cd "$(dirname "$0")/.."

echo "[omnim] stopping existing server on port ${PORT}..."
if command -v fuser >/dev/null 2>&1; then
  fuser -k "${PORT}/tcp" >/dev/null 2>&1 || true
fi
if command -v lsof >/dev/null 2>&1; then
  pids="$(lsof -ti tcp:"${PORT}" || true)"
  if [ -n "${pids}" ]; then
    echo "${pids}" | xargs -r kill -9 || true
  fi
fi
sleep 1

echo "[omnim] building..."
cargo build

echo "[omnim] starting server..."
echo "[omnim] project: ${PROJECT}"
echo "[omnim] local:   http://localhost:${PORT}/"
if command -v ip >/dev/null 2>&1; then
  WSL_IP="$(ip addr show eth0 2>/dev/null | grep -oP '(?<=inet\s)\d+(\.\d+){3}' | head -1 || true)"
  if [ -n "${WSL_IP}" ]; then
    echo "[omnim] wsl:     http://${WSL_IP}:${PORT}/"
  fi
fi

env OMNIM_PROJECT="${PROJECT}" ./target/debug/omnim
