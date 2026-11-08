# editor

跨平台打开本地目录或 SSH 远程目录。需安装 VSCode 或 Zed 的命令行工具，并将其加入 PATH。

```bash
cargo install --path crates/cli-editor
editor ~/project
editor ~/project --zed
editor /home/user/project --host my-server
editor /home/user/project --zed --host my-server
```

默认使用 VSCode 和当前目录；相对路径按本机当前目录转换为绝对路径，远程机器上须有对应路径。远程路径须为 Unix 路径，主机使用 SSH 配置中的别名，Windows 使用 `code.cmd`。

## remote-open

通过 SSH 在远程 macOS 上用默认程序打开共享文件。两台机器须将 NAS 挂载到相同路径；相对路径按本机当前目录转换为绝对路径。

```bash
remote-open --host my-server ./file.pdf
```

## remote-run

通过 SSH 执行远程命令，本机及远程主机均支持 Linux、macOS、Windows，需安装 OpenSSH。

```bash
cargo install --path crates/cli-remote --bin remote-run
remote-run --host my-server 'python script.py --flag'
remote-run --host my-server 'cd /shared/project && Rscript analysis.R'
remote-run --host windows-server 'cmd /c ver'
remote-run --host windows-server 'powershell -NoProfile -Command "Get-Date"'
```

命令须整体作为一个参数传入；示例适用于本机 Bash、Zsh、PowerShell，本机 CMD 须使用双引号。命令内容按**远程默认 Shell** 的语法解析，不会自动转换路径或切换到本机当前目录；默认工作目录为远程登录目录。主机使用 SSH 配置中的别名。

标准输入、输出、错误直接透传；保留 SSH 退出码（远程命令退出码，连接失败通常为 `255`）。如需管道、重定向或变量展开，应将其放入命令字符串，由远程 Shell 执行；仅对可信命令使用本工具。
