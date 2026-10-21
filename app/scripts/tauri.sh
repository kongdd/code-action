#!/usr/bin/env bash
# NFS 挂载盘上的构建产物会携带 AppleDouble 旁路文件并拖慢 IO，
# 因此默认将 Cargo 产物放到本机缓存目录。
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/.cache/code-action-target}"
cd "$root"
exec npx tauri "$@"
