#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
if [[ -f "$HOME/.cargo/env" ]]; then
  source "$HOME/.cargo/env"
fi
for command in cargo trunk python3; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "Missing $command. See README.md for WSL setup." >&2
    exit 1
  fi
done
python3 scripts/web_server.py --port 8001 &
proxy_pid=$!
trap 'kill "$proxy_pid" 2>/dev/null || true' EXIT
# Wait until our process is ready; refuse to reuse an unrelated listener.
for attempt in {1..50}; do
  kill -0 "$proxy_pid" 2>/dev/null || { echo 'LLM proxy failed to start.' >&2; exit 1; }
  if python3 -c 'import urllib.request; urllib.request.urlopen("http://127.0.0.1:8001/api/config", timeout=0.2)' 2>/dev/null; then
    break
  fi
  sleep 0.1
done
echo 'Open http://localhost:8080 in your Windows browser.'
trunk serve "$@"
