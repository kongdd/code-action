## vscode 历史搜索

<https://github.com/mohuishou/utools>

<https://github.com/kongdd/utools-vscode>

utools 插件，搜索 vscode 历史项目，点击回车打开

- [**点击查看 chrome 历史搜索，vscode 历史项目搜索等更多插件**](https://github.com/mohuishou/utools)
- **你的 star 是我持续更新的动力**

### 使用说明

- 关键词: vsc 输入关键词可以搜索
- 关键词: vsc-setting 查询设置
- 将 `code` 加入 PATH，终端执行 `code .` 可打开。macOS 默认用 `zsh -l -c` 加载登录环境

### fs 文件搜索

- 输入 `fs_action`，检索 fs 配置中的全部索引；最多显示 200 条，回车用默认应用打开，右键选择其他打开方式。
- 支持表达式、通配符与过滤，例如 `soil rain`、`*.pdf`、`Kong exts:docx,pdf`、`path:papers Kong`。
- 直接使用本机已安装的 [fs](https://github.com/kongdd/fs) 及现有查询配置，无需额外配置。
- 索引需预先建立；文件变化后在终端执行 `fs updatedb` 更新。

### 使用示例

![vsc.gif](https://i.loli.net/2019/12/03/35QbFJU67LoH1kP.gif)

### 更新

#### v3.1.0

- 支持 vscode 1.55.0
- 全新配置页面
- 支持多设备多操作系统使用


#### v2.0.4

fix(vsc): 修复 code 命令无法打开项目 #15 #16

#### v2.0.3

- fix: 插件被主动退出导致 vscode 进程被杀掉的问题

#### v2.0.2

- fix: 无法设置可执行文件的问题

#### v2.0.1

- fix: windows 无法匹配可执行文件的问题

#### v2.0.0

- 适配 utools v1.0
- 新增打开 wsl 等远程开发文件夹
