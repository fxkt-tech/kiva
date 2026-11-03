#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
if [[ -f "$HOME/.cargo/env" ]]; then source "$HOME/.cargo/env"; fi
test_dir=$(mktemp -d -t kiva-combat.XXXXXX)
trap 'rm -rf "$test_dir"' EXIT
rustc --edition 2024 --test src/game/combat/core.rs -o "$test_dir/combat-tests"
"$test_dir/combat-tests"
