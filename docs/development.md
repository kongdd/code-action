# 开发与构建

[项目首页](../README.md) · [独立版使用介绍](../app/README.md)

## 独立桌面版

在仓库的 `app/` 目录运行，需安装 Node.js、Rust 及当前平台的 [Tauri 开发依赖](https://v2.tauri.app/start/prerequisites/)。

```bash
cd app
npm ci
npm run app:dev     # 开发运行
npm run app:build   # 打包
npm run test        # Rust 单元测试
```

`app/scripts/tauri.sh` 默认将 Cargo 产物放在 `~/.cache/code-action-target`，可通过 `CARGO_TARGET_DIR` 覆盖。
macOS 应用位于 `~/.cache/code-action-target/release/bundle/macos/Code Action.app`。

### 持续集成

[Desktop build](../.github/workflows/build-desktop.yml) 在 Windows、Linux、macOS 上编译前端与 Tauri 应用，不打包或发布。
修改 `app/` 或工作流后推送、提交 PR 自动触发，也可在 GitHub Actions 页面手动运行。
编译检查不代表已验证各平台的桌面交互。

### NFS 挂载

macOS 在 NFS 挂载盘上生成的 `._*` AppleDouble 旁路文件可能破坏 Tauri 权限扫描。
`npm run build` 会先调用 `scripts/clean-appledouble.sh` 清理旁路文件；Cargo 产物默认放在本机缓存目录，避免网络盘编译。

### 界面预览

运行开发服务后，在浏览器访问 `http://127.0.0.1:1420/?preview`，使用模拟数据预览界面。

### 历史项目与文件搜索

- 最近项目读取 VSCode 的 `state.vscdb`，支持 WAL 实时数据；Zed 模式使用同一项目列表，只改变打开方式。
- “VSCode 数据库”留空时，自动依次尝试 `~/.vscode-shared/sharedStorage/state.vscdb`（VSCode 1.118+）与平台默认位置；macOS 的传统位置为 `~/Library/Application Support/Code/User/globalStorage/state.vscdb`。
- 文件搜索调用本机 [fs](https://github.com/kongdd/fs) 索引。需先安装并建立索引，文件变化后运行 `fs updatedb`。
- 搜索表达式支持 `soil rain`、`*.pdf`、`Kong exts:docx,pdf`、`path:papers Kong`，最多显示 200 条结果。
- 打不开项目时，在设置中填写编辑器可执行文件路径；macOS 会自动查找应用包内 CLI。
- 设置与收藏仅保存在应用配置目录的 `config.json`，无云同步。

## uTools 插件

在仓库根目录运行：

```bash
npm install
npm run release
```

在 uTools 开发者工具中导入 `release/plugin.json` 后打包，不要直接打包仓库根目录或 `dist/`。
`package.json` 中的 `exclude` 不能替代独立发布目录。

```text
release/
├── plugin.json
├── dist/preload.js
├── icon/
├── third_party/sqljs/
└── layui/                # 配置界面依赖
```

发布目录由脚本重新生成，保留现有相对路径，不包含 `.git`、源码或 `node_modules`；不要手动修改生成文件。

### 使用与排障

- 输入 `vsc` 搜索历史项目，输入 `vsc-setting` 打开设置。
- 将 `code` 加入 PATH，确保终端中的 `code .` 可用。macOS 本地项目直接调用 VSCode 应用，无需配置 PATH；远程项目通过 `zsh -l -c` 调用 `code`。
- 输入 `fs_action` 检索 fs 配置中的全部索引，回车用默认应用打开，右键可选择其他打开方式。
- 插件复用本机已安装的 fs 及查询配置，无需额外配置索引路径。

[旧版操作演示](https://i.loli.net/2019/12/03/35QbFJU67LoH1kP.gif) · [原始插件项目](https://github.com/mohuishou/utools)
