#!/usr/bin/env bash
set -euo pipefail

PORT="${OMNIM_PORT:-3000}"

echo "[omnim] stopping server on port ${PORT}..."
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
if command -v ss >/dev/null 2>&1; then
  ss -tlnp | grep ":${PORT}" || echo "[omnim] port ${PORT} is free"
fi
