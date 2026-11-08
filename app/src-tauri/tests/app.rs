use super::*;

#[test]
fn config_path_matches_tauri_identifier() {
    let config: serde_json::Value = serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
    assert_eq!(config["identifier"], code_action_core::store::APP_ID);
    assert!(config["app"].get("appDirectoriesOverride").is_none());
    let mut context = tauri::test::mock_context(tauri::test::noop_assets());
    context.config_mut().identifier = config["identifier"].as_str().unwrap().into();
    let app = tauri::test::mock_builder().build(context).unwrap();
    assert_eq!(config_path(), Some(app.path().app_config_dir().unwrap().join("config.json")));
}

#[test]
fn shortcut_payload_matches_frontend() {
    let payload = ShortcutPayload { mode: "fs".into(), auto_copy: true };
    assert_eq!(serde_json::to_value(payload).unwrap(), serde_json::json!({
        "mode": "fs", "autoCopy": true,
    }));
}

#[test]
fn default_shortcuts_are_valid_and_unique() {
    let mut ids = std::collections::HashSet::new();
    for shortcut in Settings::default().shortcuts {
        assert!(ids.insert(shortcut.accelerator.parse::<Hotkey>().unwrap().id()));
    }
}
