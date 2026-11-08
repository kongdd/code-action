use clap::Parser;
use std::process::{Command, ExitCode};

#[derive(Parser)]
#[command(
    about = "通过 SSH 执行远程命令（Linux、macOS、Windows）",
    after_help = "示例：remote-run --host my-server 'python script.py --flag'\n命令须作为一个参数传入，语法取决于远程默认 Shell；工作目录为远程登录目录。"
)]
struct Args {
    /// SSH 配置中的主机别名
    #[arg(long)]
    host: String,
    /// 由远程默认 Shell 解释的完整命令
    #[arg(value_name = "COMMAND")]
    command: String,
}

fn ssh_command(args: &Args) -> Result<Command, String> {
    if args.host.is_empty()
        || args.host.starts_with('-')
        || !args
            .host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    {
        return Err("远程主机须为 SSH 别名".into());
    }
    if args.command.trim().is_empty() {
        return Err("远程命令不能为空".into());
    }
    // 不套用本机 Shell 的转义规则，由远程默认 Shell 解析。
    let mut command = Command::new("ssh");
    command.args(["--", &args.host, &args.command]);
    Ok(command)
}

fn run(args: Args) -> Result<ExitCode, String> {
    let status = ssh_command(&args)?
        .status()
        .map_err(|e| format!("无法启动 ssh：{e}"))?;
    Ok(ExitCode::from(status.code().unwrap_or(1) as u8))
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn preserves_remote_shell_commands() {
        for script in [
            "printf '%s\\n' 'a b' | grep b",
            "open '/Volumes/共享文件/a.pdf'",
            r#"cmd /c "echo hello & ver""#,
            r#"powershell -NoProfile -Command "Write-Output '你好'""#,
        ] {
            let args = Args::try_parse_from(["remote-run", "--host", "my-server", script]).unwrap();
            let command = ssh_command(&args).unwrap();
            assert_eq!(command.get_program(), "ssh");
            assert_eq!(
                command.get_args().collect::<Vec<_>>(),
                ["--", "my-server", script].map(OsStr::new)
            );
        }
    }

    #[test]
    fn rejects_invalid_hosts() {
        for host in ["", "-server", "user@server", "server;id", "server\n", "a b"] {
            let args = Args {
                host: host.into(),
                command: "echo hello".into(),
            };
            assert!(ssh_command(&args).is_err(), "{host:?}");
        }
    }

    #[test]
    fn rejects_empty_commands() {
        for script in ["", " \n\t"] {
            let args = Args {
                host: "my-server".into(),
                command: script.into(),
            };
            assert!(ssh_command(&args).is_err());
        }
    }

    #[test]
    fn requires_host_and_one_command() {
        assert!(Args::try_parse_from(["remote-run", "echo hello"]).is_err());
        assert!(Args::try_parse_from(["remote-run", "--host", "my-server"]).is_err());
        assert!(
            Args::try_parse_from(["remote-run", "--host", "my-server", "echo", "hello"]).is_err()
        );
    }
}
