#!/usr/bin/env bash
# NFS 挂载不支持扩展属性，macOS 会为每个文件生成 ._* 旁路文件（AppleDouble）。
# Tauri 的权限扫描器会把 capabilities/._default.json 当作配置读取并失败，故构建前清理。
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
find . -name '._*' -not -path './node_modules/*' -not -path './src-tauri/target/*' -delete
xattr -cr . 2>/dev/null || true
