use crate::{history::Item, store::Settings};
use serde::Deserialize;
use std::{path::{Path, PathBuf}, process::Stdio, time::Duration};
use tokio::process::Command;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action { Default, Vscode, Zed, Directory, Terminal }

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode { Vscode, Zed, Fs }

fn executable(binary: &str) -> PathBuf {
    let path = PathBuf::from(binary);
    if path.is_absolute() || path.components().count() > 1 { return path; }

    // Finder-launched applications do not inherit a login shell's PATH.
    let mut paths: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect()).unwrap_or_default();
    if let Some(home) = dirs::home_dir() {
        paths.extend([home.join(".cargo/bin"), home.join(".local/bin")]);
        #[cfg(target_os = "macos")]
        paths.push(home.join("Applications/Visual Studio Code.app/Contents/Resources/app/bin"));
    }
    #[cfg(target_os = "macos")]
    paths.extend([
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/Applications/Visual Studio Code.app/Contents/Resources/app/bin"),
        PathBuf::from("/Applications/Zed.app/Contents/MacOS"),
    ]);
    paths.into_iter().map(|p| p.join(binary)).find(|p| p.is_file()).unwrap_or(path)
}

fn command(binary: &str) -> Command {
    #[allow(unused_mut)]
    let mut cmd = Command::new(executable(binary));
    #[cfg(windows)]
    {
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    cmd
}

fn locate_args(query: &str) -> Vec<String> {
    let (flags, words): (Vec<_>, Vec<_>) = query.split_whitespace()
        .partition(|word| word.starts_with('-'));
    if words.is_empty() { return Vec::new(); }
    ["locate", "--json", "-n", "200"].into_iter().chain(flags)
        .chain(["--"]).map(str::to_owned).chain([words.join(" ")]).collect()
}

/// 搜索结果上限，与 fs locate 的 -n 参数保持一致。
pub const LIMIT: usize = 200;

/// 可取消的检索：切换关键字时旧进程会被丢弃，不会一直跑满超时。
pub async fn locate(
    settings: &Settings,
    query: &str,
    cancel: Option<tokio::sync::oneshot::Receiver<()>>,
) -> Result<Vec<Item>, String> {
    match cancel {
        Some(cancel) => tokio::select! {
            _ = cancel => Err("搜索已取消".to_owned()),
            result = run(settings, query) => result,
        },
        None => run(settings, query).await,
    }
}

async fn run(settings: &Settings, query: &str) -> Result<Vec<Item>, String> {
    let args = locate_args(query);
    if args.is_empty() { return Ok(Vec::new()); }
    let output = tokio::time::timeout(
        Duration::from_secs(3),
        command(&settings.fs_binary).args(args).kill_on_drop(true).output(),
    ).await.map_err(|_| "fs 搜索超时（3 秒）".to_owned())?
        .map_err(|e| format!("无法运行 fs，请检查安装和可执行文件路径：{e}"))?;
    if !output.status.success() {
        return Err(format!("fs 搜索失败：{}", String::from_utf8_lossy(&output.stderr)));
    }
    #[derive(Deserialize)]
    struct Found { path: String }
    let files: Vec<Found> = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("fs 返回的数据格式错误：{e}"))?;
    Ok(files.into_iter().take(LIMIT).map(|f| Item::new(f.path)).collect())
}

fn editor(binary: &str, target: &str) -> Command {
    let remote = target.starts_with("vscode-remote:");
    if target.starts_with("ssh://") {
        let cli = Path::new("/Applications/Zed.app/Contents/MacOS/cli");
        let binary = if binary == "zed" && cfg!(target_os = "macos") && cli.is_file() {
            cli.to_str().unwrap()
        } else { binary };
        let mut cmd = command(binary);
        cmd.arg(target);
        return cmd;
    }
    #[cfg(target_os = "macos")]
    if !remote {
        let app = match binary {
            "code" => Some("Visual Studio Code"),
            "zed" => Some("Zed"),
            _ => None,
        };
        if let Some(app) = app {
            let mut cmd = command("/usr/bin/open");
            cmd.args(["-a", app, "--", target]);
            return cmd;
        }
    }
    let mut cmd = command(binary);
    if remote {
        cmd.arg(if target.ends_with(".code-workspace") { "--file-uri" } else { "--folder-uri" });
        cmd.arg(target);
    } else {
        cmd.arg("--").arg(target);
    }
    cmd
}

fn open_path(target: &str) -> Command {
    #[cfg(target_os = "macos")]
    { let mut cmd = command("/usr/bin/open"); cmd.args(["--", target]); cmd }
    #[cfg(target_os = "windows")]
    { let mut cmd = command("explorer.exe"); cmd.arg(target); cmd }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    { let mut cmd = command("xdg-open"); cmd.arg(target); cmd }
}

fn terminal(settings: &Settings, dir: &str) -> Command {
    if !settings.terminal.trim().is_empty() {
        let mut cmd = command(settings.terminal.trim());
        cmd.current_dir(dir);
        return cmd;
    }
    #[cfg(target_os = "macos")]
    {
        // LaunchServices opens Terminal.app in the given folder; no shell interpolation.
        let mut cmd = command("/usr/bin/open");
        cmd.args(["-a", "Terminal"]).arg(dir);
        cmd
    }
    #[cfg(target_os = "windows")]
    { let mut cmd = command("wt.exe"); cmd.args(["-d", dir]); cmd }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    { let mut cmd = command("gnome-terminal"); cmd.args(["--working-directory", dir]); cmd }
}

/// 复制文本到系统剪贴板。Tauri 自定义协议属于非安全上下文，navigator.clipboard 不可用。
pub fn copy(text: &str) -> Result<(), String> {
    let mut errors = Vec::new();
    for binary in CLIPBOARD {
        match pipe(binary, text) {
            Ok(()) => return Ok(()),
            Err(error) => errors.push(format!("{binary}: {error}")),
        }
    }
    Err(format!("复制到剪贴板失败（{}）", errors.join("；")))
}

#[cfg(target_os = "macos")]
const CLIPBOARD: &[&str] = &["pbcopy"];
#[cfg(target_os = "windows")]
const CLIPBOARD: &[&str] = &["clip"];
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const CLIPBOARD: &[&str] = &["wl-copy", "xclip", "xsel"];

fn clipboard_args(binary: &str) -> &'static [&'static str] {
    match binary {
        "xclip" => &["-selection", "clipboard"],
        "xsel" => &["--clipboard", "--input"],
        _ => &[],
    }
}

fn pipe(binary: &str, text: &str) -> Result<(), String> {
    use std::io::Write;
    use std::process::{Command as Std, Stdio};
    #[allow(unused_mut)]
    let mut command = Std::new(binary);
    command.args(clipboard_args(binary)).stdin(Stdio::piped())
        .stdout(Stdio::null()).stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    child.stdin.take().ok_or("标准输入不可用")?
        .write_all(text.as_bytes()).map_err(|e| e.to_string())?;
    let status = child.wait().map_err(|e| e.to_string())?;
    if status.success() { Ok(()) } else { Err(format!("退出码 {:?}", status.code())) }
}

pub async fn open(settings: &Settings, action: Action, target: &str) -> Result<(), String> {
    let remote = target.starts_with("vscode-remote:") || target.starts_with("ssh://");
    if target.starts_with("ssh://") && !matches!(action, Action::Zed) {
        return Err("Zed 远程项目只能通过 Zed 打开".into());
    }
    if !remote && (!Path::new(target).is_absolute() || !Path::new(target).exists()) {
        return Err("目标路径不存在或不是绝对路径".into());
    }
    if remote && matches!(action, Action::Default | Action::Terminal | Action::Directory) {
        return Err("远程项目只能通过编辑器打开".into());
    }
    let mut cmd = match action {
        Action::Vscode => editor(&settings.code, target),
        Action::Zed => editor(&settings.zed, target),
        Action::Default => open_path(target),
        Action::Directory | Action::Terminal => {
            let path = Path::new(target);
            let dir = if path.is_dir() { path } else { path.parent().ok_or("无法定位目录")? };
            let dir = dir.to_str().ok_or("目录编码无效")?;
            if matches!(action, Action::Directory) { open_path(dir) } else { terminal(settings, dir) }
        }
    };
    // No shell evaluation; all paths are passed as individual arguments.
    cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    let mut child = cmd.spawn().map_err(|e| format!("无法启动打开命令：{e}"))?;
    // 回收启动器进程，但不阻塞调用方，也不拖长编辑器的生命周期。
    tokio::spawn(async move { let _ = child.wait().await; });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_flags_precede_expression_and_preserve_quotes() {
        assert_eq!(locate_args(r#"soil -i path:"A B" rain"#),
            ["locate", "--json", "-n", "200", "-i", "--", r#"soil path:"A B" rain"#]);
        assert!(locate_args("  -i  ").is_empty());
        assert!(locate_args("").is_empty());
    }

    #[test]
    fn locates_files_with_the_local_index() {
        // 依赖本机环境：存在 fs 索引时校验解析结果。
        let mut settings = Settings::default();
        let cargo = dirs::home_dir().unwrap_or_default().join(".cargo/bin")
            .join(if cfg!(windows) { "fs.exe" } else { "fs" });
        if cargo.is_file() {
            settings.fs_binary = cargo.to_string_lossy().into_owned();
        }
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let (_sender, receiver) = tokio::sync::oneshot::channel();
        let result = runtime.block_on(locate(&settings, "*.pdf", Some(receiver)));
        if let Ok(files) = result {
            assert!(files.iter().all(|item| item.path.ends_with(".pdf")));
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn cancelled_and_timed_out_searches_terminate_the_process() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("fs");
        let pidfile = dir.path().join("pid");
        std::fs::write(&script, format!("#!/bin/sh\necho $$ > '{}'; exec sleep 30\n", pidfile.display())).unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        let settings = Settings { fs_binary: script.to_string_lossy().into_owned(), ..Settings::default() };

        for cancel_search in [true, false] {
            let _ = std::fs::remove_file(&pidfile);
            let (sender, receiver) = tokio::sync::oneshot::channel();
            let task = tokio::spawn({
                let settings = settings.clone();
                async move { locate(&settings, "query", Some(receiver)).await }
            });
            let pid = tokio::time::timeout(Duration::from_secs(2), async {
                loop {
                    if let Ok(pid) = std::fs::read_to_string(&pidfile) {
                        if let Ok(pid) = pid.trim().parse::<u32>() { break pid; }
                    }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }).await.unwrap();
            let sender = if cancel_search {
                sender.send(()).unwrap();
                None
            } else {
                Some(sender)
            };
            let error = task.await.unwrap().unwrap_err();
            drop(sender);
            assert!(error.contains(if cancel_search { "取消" } else { "超时" }), "{error}");
            tokio::time::timeout(Duration::from_secs(2), async {
                while std::process::Command::new("kill").args(["-0", &pid.to_string()])
                    .stderr(Stdio::null()).status().unwrap().success() {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }).await.expect("搜索进程未退出");
        }
    }

    #[test]
    fn remote_uri_is_a_single_argument() {
        let target = "vscode-remote://ssh-remote+host/home/A%20B";
        let cmd = editor("code", target);
        let args: Vec<_> = cmd.as_std().get_args().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(args, ["--folder-uri", target]);
    }

    #[test]
    fn zed_ssh_uri_is_passed_without_vscode_flags() {
        let target = "ssh://alice@example.com:2222/home/A%20B";
        let cmd = editor("zed", target);
        let args: Vec<_> = cmd.as_std().get_args().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(args, [target]);
    }

    #[test]
    fn workspace_uses_file_uri_flag() {
        let cmd = editor("code", "vscode-remote://host/a.code-workspace");
        let args: Vec<_> = cmd.as_std().get_args().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(args, ["--file-uri", "vscode-remote://host/a.code-workspace"]);
    }
}
