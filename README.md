## vscode 历史搜索

<https://github.com/mohuishou/utools>

<https://github.com/kongdd/utools-vscode>

utools 插件，搜索 vscode 历史项目，点击回车打开

- [**点击查看 chrome 历史搜索，vscode 历史项目搜索等更多插件**](https://github.com/mohuishou/utools)
- **你的 star 是我持续更新的动力**

### 使用说明

- 关键词: vsc 输入关键词可以搜索
- 关键词: vsc-setting 查询设置
- 将 `code` 加入 PATH，终端执行 `code .` 可打开。macOS 本地项目直接调用 VSCode 应用，无需配置 PATH；远程项目仍通过 `zsh -l -c` 调用 `code`。

### 构建与发布

```bash
npm install
npm run release
```

在 uTools 开发者工具中导入 **`release/plugin.json`** 后打包，不要直接打包仓库根目录或 `dist/`。`package.json` 中的 `exclude` 不能替代独立发布目录。

```text
release/
├── plugin.json
├── dist/preload.js
├── icon/
├── third_party/sqljs/
└── layui/                # 配置界面依赖
```

发布目录由脚本重新生成，保留现有相对路径，不包含 `.git`、源码或 `node_modules`；请勿手动修改其中的文件。

### fs 文件搜索

- 输入 `fs_action`，检索 fs 配置中的全部索引；最多显示 200 条，回车用默认应用打开，右键选择其他打开方式。
- 支持表达式、通配符与过滤，例如 `soil rain`、`*.pdf`、`Kong exts:docx,pdf`、`path:papers Kong`。
- 直接使用本机已安装的 [fs](https://github.com/kongdd/fs) 及现有查询配置，无需额外配置。
- 索引需预先建立；文件变化后在终端执行 `fs updatedb` 更新。

### 使用示例

![vsc.gif](https://i.loli.net/2019/12/03/35QbFJU67LoH1kP.gif)
