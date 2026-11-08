use super::*;

#[test]
fn legacy_shortcuts_remain_enabled() {
    let shortcut: Shortcut = serde_json::from_value(serde_json::json!({
        "accelerator": "Command+Control+E", "action": "fs", "autoCopy": false,
    })).unwrap();
    assert!(shortcut.enabled);
    assert!(shortcut.is_active());
    assert!(Settings::default().shortcuts.iter().all(Shortcut::is_active));
}

#[test]
fn disabled_or_empty_shortcuts_are_inactive() {
    let mut shortcut = Settings::default().shortcuts.remove(0);
    shortcut.enabled = false;
    assert!(!shortcut.is_active());
    let json = serde_json::to_string(&shortcut).unwrap();
    assert!(!serde_json::from_str::<Shortcut>(&json).unwrap().enabled);
    shortcut.enabled = true;
    shortcut.accelerator = "  ".into();
    assert!(!shortcut.is_active());
}

#[test]
fn default_shortcuts_follow_the_platform() {
    let shortcuts = Settings::default().shortcuts;
    let prefix = if cfg!(target_os = "macos") {
        "Command+Control"
    } else {
        "Control+Alt"
    };
    for (action, key) in [("fs", "E"), ("zed", "Z"), ("vscode", "C")] {
        let shortcut = shortcuts.iter().find(|item| item.action == action).unwrap();
        assert_eq!(shortcut.accelerator, format!("{prefix}+{key}"));
    }
}

#[test]
fn round_trip_and_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let mut storage = Storage::load(path.clone()).unwrap();
    let mut data = storage.data.clone();
    data.preferences.favorites.push("/tmp/中文 项目".into());
    data.preferences.queries.insert("fs".into(), "*.pdf".into());
    data.settings.shortcuts[0].auto_copy = true;
    data.settings.shortcuts[1].enabled = false;
    data.settings.aliases.insert("fs".into(), "文件".into());
    storage.save(data).unwrap();
    let loaded = Storage::load(path).unwrap();
    assert_eq!(loaded.data.preferences.favorites, ["/tmp/中文 项目"]);
    assert_eq!(loaded.data.preferences.queries["fs"], "*.pdf");
    assert_eq!(loaded.data.settings.code, "code");
    assert!(loaded.data.settings.shortcuts[0].auto_copy);
    assert!(!loaded.data.settings.shortcuts[1].enabled);
    assert_eq!(loaded.data.settings.aliases["fs"], "文件");
}

#[test]
fn damaged_configuration_is_not_overwritten() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    fs::write(&path, b"{broken").unwrap();
    assert!(Storage::load(path.clone()).is_err());
    assert_eq!(fs::read(path).unwrap(), b"{broken");
}
