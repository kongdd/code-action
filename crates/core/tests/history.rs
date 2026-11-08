use super::*;

#[test]
fn file_uri_and_remote_paths() {
    let path = std::env::temp_dir().join("A B%20");
    let uri = Url::from_file_path(&path).unwrap().to_string();
    assert_eq!(normalize_path(&uri), Some(path.to_string_lossy().into_owned()));
    let remote = "vscode-remote://ssh-remote+host/home/A%20B";
    assert_eq!(normalize_path(remote), Some(remote.into()));
    assert_eq!(normalize_path(""), None);
    #[cfg(not(windows))]
    assert_eq!(normalize_path("file:///%ZZ"), Some("/%ZZ".into()));
}

#[test]
fn entry_shapes() {
    let path = std::env::temp_dir().join("test.code-workspace");
    let uri = Url::from_file_path(&path).unwrap().to_string();
    let entries = serde_json::json!([
        "vscode-remote://ssh-remote+host/a",
        {"folderUri": uri},
        {"fileUri": uri},
        {"workspace": {"configPath": uri}},
        {}
    ]);
    let paths: Vec<_> = entries.as_array().unwrap().iter().filter_map(entry_path).collect();
    assert_eq!(paths.len(), 4);
    assert_eq!(paths[3], path.to_string_lossy());
}

#[test]
fn filter_deduplicates_and_matches_every_keyword() {
    let a = "vscode-remote://host/Soil Rain";
    let b = "vscode-remote://host/Soil";
    let result = filter_paths([a.into(), a.into(), b.into(), "/nonexistent/file".into()], "soil RAIN");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].path, a);
}

#[test]
fn favorites_lead_the_result_and_are_not_duplicated() {
    let favorites = vec!["/a/soil".to_owned()];
    let found = vec![Item::new("/b/rain".into()), Item::new("/a/soil".into())];
    let merged = merge_favorites(found, &favorites, "");
    let paths: Vec<_> = merged.iter().map(|item| item.path.as_str()).collect();
    assert_eq!(paths, ["/a/soil", "/b/rain"]);
}

#[test]
fn favorites_obey_the_query() {
    let favorites = vec!["/a/soil".to_owned()];
    assert!(merge_favorites(Vec::new(), &favorites, "rain").is_empty());
    let merged = merge_favorites(Vec::new(), &favorites, "SOIL");
    assert_eq!(merged[0].path, "/a/soil");
}

fn create_history(path: &Path, key: &str, paths: &[&str]) -> Connection {
    let db = Connection::open(path).unwrap();
    db.execute_batch("CREATE TABLE IF NOT EXISTS ItemTable (key TEXT PRIMARY KEY, value TEXT);").unwrap();
    let value = serde_json::json!({"entries": paths.iter()
        .map(|path| serde_json::json!({"folderUri": path})).collect::<Vec<_>>(), "extra": 1}).to_string();
    db.execute("INSERT INTO ItemTable VALUES (?1, ?2)", [key, &value]).unwrap();
    db
}

#[test]
fn reads_current_history_key_without_legacy_key() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.vscdb");
    let remote = "vscode-remote://ssh-remote+host/today";
    let _db = create_history(&path, "recently.opened", &[remote]);
    let settings = Settings { database: path.to_string_lossy().into_owned(), ..Settings::default() };
    let items = recent(&settings, &[], "today").unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].path, remote);
    assert!(items[0].remote);
}

#[test]
fn merges_databases_and_keys_and_removes_from_every_source() {
    let dir = tempfile::tempdir().unwrap();
    let shared = dir.path().join("shared.vscdb");
    let local = dir.path().join("local.vscdb");
    let duplicate = "vscode-remote://host/duplicate";
    let today = "vscode-remote://host/today";
    let shared_db = create_history(&shared, HISTORY_KEY, &[duplicate]);
    let local_db = create_history(&local, "recently.opened", &[today, duplicate]);
    create_history(&local, HISTORY_KEY, &[duplicate]);
    let candidates = vec![dir.path().join("missing.vscdb"), shared, local];
    let paths = read_histories(&candidates).unwrap().into_iter().flat_map(|(_, entries)| entries);
    let items = filter_paths(paths, "");
    assert_eq!(items.iter().map(|item| item.path.as_str()).collect::<Vec<_>>(), [duplicate, today]);
    remove_from_databases(&candidates, duplicate).unwrap();
    assert!(read_database(&candidates[1]).unwrap().is_empty());
    assert_eq!(read_database(&candidates[2]).unwrap(), [today]);
    for db in [shared_db, local_db] {
        for (_, data) in history_data(&db).unwrap() { assert_eq!(data["extra"], 1); }
    }
}

#[test]
fn empty_shared_history_does_not_hide_local_remote_history() {
    let dir = tempfile::tempdir().unwrap();
    let shared = dir.path().join("shared.vscdb");
    let local = dir.path().join("local.vscdb");
    let remote = "vscode-remote://host/today";
    create_history(&shared, HISTORY_KEY, &[]);
    create_history(&local, "recently.opened", &[remote]);
    let paths: Vec<_> = read_histories(&[shared, local]).unwrap().into_iter()
        .flat_map(|(_, entries)| entries).collect();
    assert_eq!(paths, [remote]);
    assert!(read_histories(&[dir.path().join("missing.vscdb")]).is_err());
}

#[test]
fn reads_live_wal_database() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.vscdb");
    let writer = Connection::open(&path).unwrap();
    writer.execute_batch("PRAGMA journal_mode=WAL; PRAGMA wal_autocheckpoint=0;
        CREATE TABLE ItemTable (key TEXT PRIMARY KEY, value TEXT);").unwrap();
    let value = r#"{"entries":[{"folderUri":"vscode-remote://host/new-project"}]}"#;
    writer.execute("INSERT INTO ItemTable VALUES (?1, ?2)", [HISTORY_KEY, value]).unwrap();
    assert!(path.with_extension("vscdb-wal").exists());
    assert_eq!(read_database(&path).unwrap(), ["vscode-remote://host/new-project"]);
}

#[test]
fn removes_history_from_live_wal_and_invalidates_cache() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.vscdb");
    let db = Connection::open(&path).unwrap();
    db.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE ItemTable (key TEXT PRIMARY KEY, value TEXT);").unwrap();
    let target = dir.path().join("a b");
    let value = serde_json::json!({
        "entries": [{"fileUri": Url::from_file_path(&target).unwrap().to_string()},
            {"folderUri": "vscode-remote://host/keep"}],
        "extra": 1,
    }).to_string();
    db.execute("INSERT INTO ItemTable VALUES (?1, ?2)", [HISTORY_KEY, &value]).unwrap();
    assert_eq!(read_database(&path).unwrap().len(), 2);
    let settings = Settings { database: path.to_string_lossy().into_owned(), ..Settings::default() };
    remove(&settings, &target.to_string_lossy()).unwrap();
    assert_eq!(read_database(&path).unwrap(), ["vscode-remote://host/keep"]);
    let value: String = db.query_row("SELECT value FROM ItemTable WHERE key = ?1", [HISTORY_KEY], |row| row.get(0)).unwrap();
    assert_eq!(serde_json::from_str::<Value>(&value).unwrap()["extra"], 1);
}

#[test]
fn reads_the_local_vscode_history_when_present() {
    // 依赖本机环境：存在真实数据库时校验解析结果，不存在则跳过。
    let candidates = database_candidates(&Settings::default()).unwrap();
    let Some(path) = candidates.into_iter().find(|path| path.exists()) else {
        return;
    };
    // 用户可能清空过历史，不应要求数据库非空。
    read_database(&path).unwrap();
}

#[test]
fn explicit_database_does_not_use_default() {
    let settings = Settings { database: "/custom/state.vscdb".into(), ..Settings::default() };
    assert_eq!(database_candidates(&settings).unwrap(), [PathBuf::from("/custom/state.vscdb")]);
}
