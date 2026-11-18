use clap::Parser;
use cli_remote::nas_path;
use std::{
    borrow::Cow,
    fmt::Write,
    process::{Command, ExitCode},
};

#[derive(Parser)]
#[command(
    about = "用 VSCode 或 Zed 打开本地或 SSH 远程路径",
    after_help = "示例：\n  remote-editor .\n  remote-editor ~/project --zed\n  remote-editor /home/user/project --host my-server\n  remote-editor /mnt/x/project --host nas\n\n远程相对路径按本机当前目录解析。使用 --host nas 时，按 nas-path 规则转换路径。"
)]
struct Args {
    /// 项目路径，默认为当前目录
    #[arg(default_value = ".")]
    path: String,
    /// 使用 VSCode（默认）
    #[arg(long, conflicts_with = "zed")]
    code: bool,
    /// 使用 Zed
    #[arg(long)]
    zed: bool,
    /// 使用 SSH 配置中的主机别名连接远程主机
    #[arg(long, value_name = "SSH别名")]
    host: Option<String>,
}

fn remote_uri(host: &str, path: &str, zed: bool) -> Result<String, &'static str> {
    let path = if host == "nas" { Cow::Owned(nas_path(path)?) } else { Cow::Borrowed(path) };
    let mut uri =
        if zed { format!("ssh://{host}") } else { format!("vscode-remote://ssh-remote+{host}") };
    for b in path.bytes() {
        if b.is_ascii_alphanumeric() || b"/._~-".contains(&b) {
            uri.push(b as char);
        } else {
            write!(uri, "%{b:02X}").unwrap();
        }
    }
    Ok(uri)
}

fn run(args: Args) -> Result<(), String> {
    let editor = if args.zed { "zed" } else { "code" };
    let binary = if cfg!(windows) && !args.zed { "code.cmd" } else { editor };
    let mut cmd = Command::new(binary);
    if let Some(host) = args.host {
        if host.is_empty()
            || !host.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        {
            return Err("远程主机须为 SSH 别名".into());
        }
        let absolute = if args.path.starts_with('/') {
            args.path.into()
        } else {
            std::path::absolute(&args.path).map_err(|e| format!("无效路径：{e}"))?
        };
        let remote_path =
            absolute.to_str().filter(|p| p.starts_with('/')).ok_or("远程路径须为 Unix 绝对路径")?;
        let uri = remote_uri(&host, remote_path, args.zed)?;
        if !args.zed {
            cmd.arg("--folder-uri");
        }
        cmd.arg(uri);
    } else {
        let path = std::path::absolute(&args.path).map_err(|e| format!("无效路径：{e}"))?;
        cmd.arg("--").arg(path);
    }
    let status = cmd.status().map_err(|e| format!("无法启动 {editor}：{e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{editor} 退出：{status}"))
    }
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::remote_uri;

    #[test]
    fn converts_nas_paths_before_encoding() {
        let path = "/mnt/x/my project";
        assert_eq!(
            remote_uri("nas", path, false).unwrap(),
            "vscode-remote://ssh-remote+nas/volume2/GitHub/my%20project"
        );
        assert_eq!(remote_uri("nas", path, true).unwrap(), "ssh://nas/volume2/GitHub/my%20project");
    }

    #[test]
    fn requires_mapped_paths_for_nas_only() {
        assert!(remote_uri("nas", "/home/user/project", false).is_err());
        assert_eq!(
            remote_uri("my-server", "/mnt/x/project", true).unwrap(),
            "ssh://my-server/mnt/x/project"
        );
    }
}
