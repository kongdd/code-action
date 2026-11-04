#!/usr/bin/env bash
# NFS 挂载不支持扩展属性，macOS 会为每个文件生成 ._* 旁路文件（AppleDouble）。
# Tauri 的权限扫描器会把 capabilities/._default.json 当作配置读取并失败，故构建前清理。
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
find . \( -path './node_modules' -o -path './target' -o -path './src-tauri/target' \) -prune -o -name '._*' -exec rm -f {} +
# 不递归清理 node_modules：部分网络挂载通过扩展属性保存符号链接。
find src-tauri/capabilities -type f -exec xattr -c {} + 2>/dev/null || true
