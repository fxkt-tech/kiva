#!/bin/sh
# Usage: tools/wasteland/run_cape_v5.sh <script.py> [args...]   (run from the repository root)
set -eu
blender_bin=${BLENDER_BIN:-'/mnt/c/Program Files/Blender Foundation/Blender 5.3/blender.exe'}
root_win=$(wslpath -w "$PWD")
script=$1; shift
"$blender_bin" --background --factory-startup --python-exit-code 1 \
  --python "$root_win\\tools\\wasteland\\$script" -- \
  --sheet "$root_win\\art\\wasteland\\concept\\cape-original-three-views-v03.png" \
  --output "$root_win\\art\\wasteland\\cape-v5" "$@" 2>&1 \
  | grep -v -E "^Fra:|DeprecationWarning|use_nodes = True|HIPEW|cannot be made relative|^ *$"
