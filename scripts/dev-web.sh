#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../web"

export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
if [ -s "$NVM_DIR/nvm.sh" ]; then
  # shellcheck disable=SC1090
  . "$NVM_DIR/nvm.sh"
  nvm use 24 >/dev/null 2>&1 || nvm use node >/dev/null 2>&1 || true
fi

if ! command -v node >/dev/null 2>&1; then
  echo "[omnim] Linux node is not installed in this shell."
  echo "[omnim] Install with nvm or apt before running React/Vite:"
  echo "        curl -fsSL https://raw.githubusercontent.com/nvm-sh/nvm/master/install.sh | bash"
  echo "        nvm install --lts"
  exit 1
fi

if [ ! -d node_modules ]; then
  echo "[omnim] installing web dependencies..."
  npm install
fi

# Rust dev-server.sh clears its port first; mirror that here so a stale Vite
# instance never pushes us onto 5174 (reserved for Playwright E2E).
DEV_PORT=5173
echo "[omnim] stopping existing dev server on port ${DEV_PORT}..."
if command -v fuser >/dev/null 2>&1; then
  fuser -k "${DEV_PORT}/tcp" >/dev/null 2>&1 || true
fi
if command -v lsof >/dev/null 2>&1; then
  pids="$(lsof -ti tcp:"${DEV_PORT}" || true)"
  if [ -n "${pids}" ]; then
    echo "${pids}" | xargs -r kill -9 || true
  fi
fi
sleep 1

echo "[omnim] starting React dev server..."
echo "[omnim] node: $(node -v)"
echo "[omnim] npm:  $(npm -v)"
echo "[omnim] local: http://localhost:5173/"
# Default bind (localhost): WSL's localhostForwarding serves this to the
# Windows host, and an explicit 0.0.0.0 bind was observed to break it.
npm run dev
