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

echo "[omnim] starting React dev server..."
echo "[omnim] node: $(node -v)"
echo "[omnim] npm:  $(npm -v)"
echo "[omnim] local: http://localhost:5173/"
npm run dev
