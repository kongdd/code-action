#!/usr/bin/env bash
# Cargo 构建产物默认放在工程内的 src-tauri/target，可用 CARGO_TARGET_DIR 覆盖。
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$root/src-tauri/target}"
cd "$root"
exec npx tauri "$@"
