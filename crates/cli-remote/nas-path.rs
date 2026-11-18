use clap::Parser;
use cli_remote::nas_path;
use std::{path::PathBuf, process::ExitCode};

#[derive(Parser)]
#[command(about = "将 /mnt 挂载路径转换为 NAS 路径")]
struct Args {
    /// 要转换的路径，相对路径按当前目录解析
    path: PathBuf,
}

fn convert(path: PathBuf) -> Result<String, String> {
    let absolute = std::path::absolute(path).map_err(|e| format!("无效路径：{e}"))?;
    nas_path(absolute.to_str().ok_or("路径须为 UTF-8")?).map_err(str::to_owned)
}

fn main() -> ExitCode {
    match convert(Args::parse().path) {
        Ok(path) => {
            println!("{path}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("nas-path: {err}");
            ExitCode::FAILURE
        }
    }
}
