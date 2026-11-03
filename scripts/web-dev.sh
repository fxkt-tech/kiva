#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

if [[ -f "$HOME/.cargo/env" ]]; then
  source "$HOME/.cargo/env"
fi

for tool in cargo rustup trunk; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "Missing $tool. Follow the setup instructions in README.md." >&2
    exit 1
  fi
done

if ! rustup target list --installed | grep -qx wasm32-unknown-unknown; then
  echo 'Install the browser target: rustup target add wasm32-unknown-unknown' >&2
  exit 1
fi

echo 'Open http://localhost:8080 in a WebGPU-capable browser after the build succeeds.'
exec trunk serve "$@"
