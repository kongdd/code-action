#!/usr/bin/env bash
# 构建桌面版安装包，产物在 app/src-tauri/target/release/bundle/
set -euo pipefail
cd "$(dirname "$0")/app"
npm run app:build
