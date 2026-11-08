#!/usr/bin/env bash
# 构建桌面版安装包，产物在 target/release/bundle/
set -euo pipefail
cd "$(dirname "$0")/app"
npm run app:build
