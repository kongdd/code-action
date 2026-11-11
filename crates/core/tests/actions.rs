use super::*;

#[cfg(target_os = "macos")]
#[test]
fn native_file_clipboard_preserves_files_and_folders() {
    use objc2_app_kit::{NSPasteboard, NSPasteboardTypeFileURL};
    use objc2_foundation::NSString;

    objc2::rc::autoreleasepool(|_| {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("中文 file #%.txt");
        std::fs::write(&file, "clipboard test").unwrap();
        let board = NSPasteboard::pasteboardWithUniqueName();
        for path in [file.as_path(), dir.path()] {
            write_file_to_pasteboard(&board, path).unwrap();
            let value = board.stringForType(unsafe { NSPasteboardTypeFileURL }).unwrap();
            let url = url::Url::parse(&value.to_string()).unwrap();
            assert_eq!(url.to_file_path().unwrap(), path);
            assert!(board
                .propertyListForType(&NSString::from_str("NSFilenamesPboardType"))
                .is_some());
        }
        board.clearContents();
    });
}

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
fn vscode_ssh_targets_convert_to_zed_uris() {
    assert_eq!(zed_ssh_target("vscode-remote://ssh-remote+my-host/home/A%20B").unwrap(),
        "ssh://my-host/home/A%20B");
    assert_eq!(zed_ssh_target("vscode-remote://ssh-remote+7b22686f73744e616d65223a22616c696365406578616d706c652e636f6d227d/home/a").unwrap(),
        "ssh://alice@example.com/home/a");
    assert!(zed_ssh_target("vscode-remote://wsl+Ubuntu/home/a").is_err());
    assert!(zed_ssh_target("vscode-remote://ssh-remote+/home/a").is_err());
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
