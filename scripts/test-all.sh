#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "[omnim] running Rust tests..."
cargo test

if command -v node >/dev/null 2>&1; then
  echo "[omnim] running web build..."
  (cd web && npm install && npm run build)
else
  echo "[omnim] skipping web build: Linux node is not installed"
fi
