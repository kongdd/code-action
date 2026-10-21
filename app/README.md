# Code Action（独立版）

脱离 uTools 的本地启动器：搜索 VSCode/Zed 最近项目与全盘文件索引，回车直接打开。

## 功能

- VSCode / Zed 最近项目搜索（读取 VSCode 的 `state.vscdb`，含 WAL 实时数据）
- 全盘文件搜索（调用本机 `fs` 索引，支持 `*.pdf`、`soil rain`、`path:papers Kong` 等表达式）
- 托盘常驻，`Ctrl/⌘ Shift Space` 显示/隐藏窗口，失焦即隐藏
- macOS 功能快捷键：`⌘⌃E` → fs_action、`⌘⌃Z` → zed_action、`⌘⌃C` → vscode_action
- 右键菜单：VSCode / Insiders / Zed / 目录 / 终端 / 置顶 / 复制路径
- 设置与收藏仅保存在本机 `config.json`，无云同步

顶栏只显示当前功能，点击功能胶囊可在 `vsc_action` / `zed_action` / `fs_action` 间切换。

## 开发

```bash
npm install
npm run app:dev     # 开发运行
npm run app:build   # 打包（产物在 $CARGO_TARGET_DIR/release/bundle）
npm run test        # Rust 单元测试
```

### NFS 挂载注意事项

仓库位于 NFS 挂载盘时，macOS 会为每个文件生成 `._*` AppleDouble 旁路文件，
会破坏 Tauri 的权限扫描与 Cargo 构建。因此：

- `npm run build` 会先清理旁路文件（`scripts/clean-appledouble.sh`）；
- Cargo 产物默认输出到 `~/.cache/code-action-target`，避免在网络盘上编译。

### 界面预览

开发模式下访问 `http://127.0.0.1:1420/?preview` 可在浏览器中查看排版（使用模拟数据）。

## 常见问题

- **搜不到历史项目**：设置中检查“VSCode 数据库”。留空时自动依次尝试
  `~/.vscode-shared/sharedStorage/state.vscdb`（VSCode 1.118+）与
  `~/Library/Application Support/Code/User/globalStorage/state.vscdb`。
- **搜不到文件**：需安装 [fs](https://github.com/kongdd/fs) 并建立索引（`fs updatedb`）。
- **打开不了项目**：设置中填写 VSCode 可执行文件路径（macOS 会自动查找应用包内 CLI）。
