#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "[omnim] checking documentation..."
python3 scripts/check_docs.py

echo "[omnim] checking the single write path..."
python3 scripts/check_write_paths.py

echo "[omnim] running Rust tests..."
cargo test

echo "[omnim] running web unit tests..."
(cd web && npm test)


if command -v node >/dev/null 2>&1; then
  if [ ! -d web/node_modules ]; then
    echo "[omnim] installing web dependencies..."
    (cd web && npm ci)
  fi
  echo "[omnim] running web unit tests + build..."
  (cd web && npm test && npm run build)
else
  echo "[omnim] skipping web tests/build: node is not installed"
fi
