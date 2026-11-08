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
