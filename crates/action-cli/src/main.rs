use clap::Parser;
use std::{fmt::Write, process::{Command, ExitCode}};

#[derive(Parser)]
#[command(
    about = "用 VSCode 或 Zed 打开本地或 SSH 远程路径",
    after_help = "示例：\n  action .\n  action ~/project --zed\n  action /home/user/project --host my-server\n  action /home/user/project --zed --host my-server\n\n远程相对路径按本机当前目录解析，远程主机上须存在相同路径。"
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

fn run(args: Args) -> Result<(), String> {
    let editor = if args.zed { "zed" } else { "code" };
    let binary = if cfg!(windows) && !args.zed { "code.cmd" } else { editor };
    let mut cmd = Command::new(binary);
    if let Some(host) = args.host {
        if host.is_empty() || !host.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b)) {
            return Err("远程主机须为 SSH 别名".into());
        }
        let absolute = if args.path.starts_with('/') { args.path.into() } else {
            std::path::absolute(&args.path).map_err(|e| format!("无效路径：{e}"))?
        };
        let remote_path = absolute.to_str().filter(|p| p.starts_with('/')).ok_or("远程路径须为 Unix 绝对路径")?;
        let mut path = String::new();
        for b in remote_path.bytes() {
            if b.is_ascii_alphanumeric() || b"/._~-".contains(&b) { path.push(b as char); }
            else { write!(path, "%{b:02X}").unwrap(); }
        }
        if args.zed { cmd.arg(format!("ssh://{host}{path}")); }
        else { cmd.args(["--folder-uri", &format!("vscode-remote://ssh-remote+{host}{path}")]); }
    } else {
        let path = std::path::absolute(&args.path).map_err(|e| format!("无效路径：{e}"))?;
        cmd.arg("--").arg(path);
    }
    let status = cmd.status().map_err(|e| format!("无法启动 {editor}：{e}"))?;
    if status.success() { Ok(()) } else { Err(format!("{editor} 退出：{status}")) }
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => { eprintln!("{error}"); ExitCode::FAILURE }
    }
}
