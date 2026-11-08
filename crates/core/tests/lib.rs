use super::*;

#[test]
fn combined_history_deduplicates_and_preserves_favorites() {
    let item = |path: &str| Item::new(path.into());
    let favorites = vec!["/pinned".into()];
    let result = merge_histories(
        Ok(vec![item("/a"), item("/shared")]),
        Ok(vec![item("/pinned"), item("/shared")]), &favorites,
    ).unwrap();
    assert_eq!(result.iter().map(|i| i.path.as_str()).collect::<Vec<_>>(), ["/pinned", "/a", "/shared"]);
    assert_eq!(merge_histories(Err("missing".into()), Ok(vec![item("/a")]), &[]).unwrap().len(), 1);
    assert!(merge_histories(Err("one".into()), Err("two".into()), &[]).is_err());
}

#[test]
fn old_settings_enable_merged_history() {
    let settings: store::Settings = serde_json::from_str("{}").unwrap();
    assert!(settings.merge_history);
    let settings: store::Settings = serde_json::from_str(r#"{"mergeHistory":false}"#).unwrap();
    assert!(!settings.merge_history);
}

#[tokio::test]
async fn vscode_history_search_and_favorites() {
    let dir = tempfile::tempdir().unwrap();
    let database = dir.path().join("state.vscdb");
    let db = rusqlite::Connection::open(&database).unwrap();
    db.execute_batch("CREATE TABLE ItemTable (key TEXT PRIMARY KEY, value TEXT)").unwrap();
    db.execute("INSERT INTO ItemTable VALUES (?1, ?2)", [
        "history.recentlyOpenedPathsList",
        r#"{"entries":[{"folderUri":"vscode-remote://host/soil"}]}"#,
    ]).unwrap();
    let mut store = Store::default();
    store.settings.database = database.to_string_lossy().into_owned();
    store.settings.merge_history = false;
    store.preferences.favorites = vec!["vscode-remote://host/soil-favorite".into()];
    let items = search(&store, Mode::Vscode, "soil", None).await.unwrap();
    assert_eq!(items.iter().map(|item| item.path.as_str()).collect::<Vec<_>>(),
        ["vscode-remote://host/soil-favorite", "vscode-remote://host/soil"]);
}
