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

`app/` 是 Cargo 工作区：`crates/core` 存放共享逻辑，`crates/cli` 是命令行版（产物 `ca`），`src-tauri` 只负责界面与系统集成。
Cargo 产物目录由 Cargo 配置决定，可用 `CARGO_TARGET_DIR` 覆盖；CI 中位于 `app/target`。

### 命令行版

```bash
cargo build -p code-action-cli --release
./target/release/ca list          # 列出 VSCode 最近打开的项目
./target/release/ca find '*.pdf'  # 用 fs 检索本地文件
```

命令行版与桌面版共用 `crates/core` 与同一份 `config.json`；配置不存在时使用默认值，读取失败或损坏时中止并报错。

### 持续集成

[Desktop build](../.github/workflows/build-desktop.yml) 在 Windows、Linux、macOS 上运行 Rust 测试，编译 CLI、前端与 Tauri 应用，不打包或发布。
修改 `app/` 或工作流后推送、提交 PR 自动触发，也可在 GitHub Actions 页面手动运行。
编译检查不代表已验证各平台的桌面交互。

### 发布

[Release](../.github/workflows/release.yml) 在推送 `v*` 标签、发布 GitHub Release 或手动触发时，
并行构建三个平台的安装包，并上传到对应 Release：

| 平台 | 运行器 | 产物 |
| --- | --- | --- |
| macOS（仅 Apple Silicon） | `macos-15` | `.dmg` |
| Windows | `windows-latest` | `.exe` / `.msi` |
| Linux | `ubuntu-24.04` | `.deb` |

构建前会把 `Cargo.toml` 与 `tauri.conf.json` 的版本号对齐为标签版本，产物附 `checksums.txt`。
尚未配置 Apple 公证与 Windows 代码签名，macOS 首次打开需在“系统设置 → 隐私与安全性”中放行。

### NFS 挂载

macOS 在 NFS 挂载盘上生成的 `._*` AppleDouble 旁路文件可能破坏 Tauri 权限扫描。
`npm run build` 会先调用 `scripts/clean-appledouble.sh` 清理旁路文件；Cargo 产物默认放在本机缓存目录，避免网络盘编译。

### 界面预览

运行开发服务后，在浏览器访问 `http://127.0.0.1:1420/?preview`，使用模拟数据预览界面。

### 历史项目与文件搜索

- 默认合并 VSCode 与 Zed 的项目历史，可在基础设置中关闭合并、分别检索。VSCode 读取 `state.vscdb` 的 `recently.opened` 与 `history.recentlyOpenedPathsList`，支持 WAL 实时数据；Zed 读取自身工作区历史，支持本地与 SSH 项目。合并模式暂不支持从 App 删除历史。
- “VSCode 数据库”留空时，自动合并 `~/.vscode-shared/sharedStorage/state.vscdb`（VSCode 1.118+）与平台默认位置的记录并去重；macOS 的传统位置为 `~/Library/Application Support/Code/User/globalStorage/state.vscdb`。指定路径时只读取该数据库。
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
