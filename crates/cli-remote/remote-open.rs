use clap::Parser;
use std::{
    path::PathBuf,
    process::{Command, ExitCode},
};

#[derive(Parser)]
#[command(about = "在远程主机上用默认程序打开共享文件")]
struct Args {
    /// SSH 配置中的主机别名
    #[arg(long)]
    host: String,
    /// 共享文件路径，相对路径按本机当前目录解析
    file: PathBuf,
}

fn run(args: Args) -> Result<(), String> {
    if args.host.is_empty()
        || !args.host.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        || args.host.starts_with('-')
    {
        return Err("远程主机须为 SSH 别名".into());
    }
    let path = std::path::absolute(args.file).map_err(|e| format!("无效路径：{e}"))?;
    let path = path.to_str().ok_or("路径须为 UTF-8")?;
    let command = format!("open -- '{}'", path.replace('\'', "'\\''"));
    let status = Command::new("ssh")
        .args([&args.host, &command])
        .status()
        .map_err(|e| format!("无法启动 ssh：{e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("ssh 退出：{status}"))
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
