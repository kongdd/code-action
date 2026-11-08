#!/usr/bin/env bash
# 启动 tauri CLI；Cargo 产物目录由 Cargo 配置决定，可用 CARGO_TARGET_DIR 覆盖。
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
exec node node_modules/@tauri-apps/cli/tauri.js "$@"
