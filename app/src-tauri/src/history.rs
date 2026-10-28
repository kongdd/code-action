use crate::store::Settings;
use rusqlite::{Connection, OpenFlags};
use serde::Serialize;
use serde_json::Value;
use std::{collections::HashSet, path::{Path, PathBuf}, sync::Mutex, time::Duration};
use url::Url;

const HISTORY_KEY: &str = "history.recentlyOpenedPathsList";

#[derive(Debug, Clone, Serialize)]
pub struct Item {
    pub title: String,
    pub path: String,
    pub remote: bool,
}

impl Item {
    pub fn new(path: String) -> Self {
        let remote = path.starts_with("vscode-remote:");
        let title = path.trim_end_matches(['/', '\\'])
            .rsplit(['/', '\\']).next().unwrap_or(&path).to_owned();
        Self { title, path, remote }
    }
}

// Local file URIs are decoded exactly once; remote URIs must retain their scheme and encoding.
pub fn normalize_path(value: &str) -> Option<String> {
    if value.starts_with("file:") {
        return Url::parse(value).ok()?.to_file_path().ok()
            .map(|p| p.to_string_lossy().into_owned());
    }
    if value.is_empty() { None } else { Some(value.to_owned()) }
}

fn entry_path(entry: &Value) -> Option<String> {
    entry.as_str()
        .or_else(|| entry.get("fileUri")?.as_str())
        .or_else(|| entry.get("folderUri")?.as_str())
        .or_else(|| entry.get("workspace")?.get("configPath")?.as_str())
        .and_then(normalize_path)
}

pub fn database_candidates(settings: &Settings) -> Result<Vec<PathBuf>, String> {
    if !settings.database.trim().is_empty() {
        return Ok(vec![PathBuf::from(settings.database.trim())]);
    }
    let home = dirs::home_dir().ok_or("无法定位用户目录")?;
    let config = dirs::config_dir().ok_or("无法定位配置目录")?;
    Ok(vec![
        home.join(".vscode-shared/sharedStorage/state.vscdb"),
        config.join("Code/User/globalStorage/state.vscdb"),
    ])
}

/// 主库与 WAL 的时间戳、长度组合：任一变化即视为历史更新。
type Stamp = ((std::time::SystemTime, u64), Option<(std::time::SystemTime, u64)>);

/// 解析结果缓存。VSCode 每次打开项目都会重写数据库，按 Stamp 复用可省掉每次键入的读盘。
static CACHE: Mutex<Option<(PathBuf, Stamp, Vec<String>)>> = Mutex::new(None);

fn file_stamp(path: &Path) -> Result<Option<(std::time::SystemTime, u64)>, String> {
    match std::fs::metadata(path) {
        Ok(meta) => Ok(Some((meta.modified().map_err(|e| e.to_string())?, meta.len()))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

fn stamp(path: &Path) -> Result<Stamp, String> {
    let db = file_stamp(path)?.ok_or_else(|| "数据库不存在".to_owned())?;
    // WAL 模式下新记录只写进 -wal，必须一并纳入判断。
    let mut wal = path.as_os_str().to_owned();
    wal.push("-wal");
    Ok((db, file_stamp(Path::new(&wal))?))
}

fn read_database(path: &Path) -> Result<Vec<String>, String> {
    let current = stamp(path)?;
    if let Ok(cache) = CACHE.lock() {
        if let Some((cached, known, entries)) = cache.as_ref() {
            if cached == path && *known == current { return Ok(entries.clone()); }
        }
    }
    let entries = query_history(path)?;
    if let Ok(mut cache) = CACHE.lock() {
        *cache = Some((path.to_path_buf(), current, entries.clone()));
    }
    Ok(entries)
}

fn query_history(path: &Path) -> Result<Vec<String>, String> {
    // Native SQLite includes committed WAL data without copying or modifying VSCode's database.
    let db = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| e.to_string())?;
    db.busy_timeout(Duration::from_millis(500)).map_err(|e| e.to_string())?;
    let value: String = db.query_row(
        "SELECT value FROM ItemTable WHERE key = ?1",
        [HISTORY_KEY], |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    let data: Value = serde_json::from_str(&value).map_err(|e| e.to_string())?;
    let entries = data.get("entries").and_then(Value::as_array).ok_or("历史记录格式不正确")?;
    Ok(entries.iter().filter_map(entry_path).collect())
}

pub fn remove(settings: &Settings, target: &str) -> Result<(), String> {
    let path = database_candidates(settings)?.into_iter()
        .find(|path| read_database(path).is_ok())
        .ok_or("无法读取 VSCode 历史数据库")?;
    let mut db = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_WRITE)
        .map_err(|e| e.to_string())?;
    db.busy_timeout(Duration::from_millis(3000)).map_err(|e| e.to_string())?;
    let tx = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .map_err(|e| e.to_string())?;
    let value: String = tx.query_row(
        "SELECT value FROM ItemTable WHERE key = ?1", [HISTORY_KEY], |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    let mut data: Value = serde_json::from_str(&value).map_err(|e| e.to_string())?;
    let entries = data.get_mut("entries").and_then(Value::as_array_mut).ok_or("历史记录格式不正确")?;
    entries.retain(|entry| entry_path(entry).as_deref() != Some(target));
    tx.execute("UPDATE ItemTable SET value = ?1 WHERE key = ?2", [&data.to_string(), HISTORY_KEY])
        .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    if let Ok(mut cache) = CACHE.lock() { *cache = None; }
    Ok(())
}

pub fn recent(settings: &Settings, favorites: &[String], query: &str) -> Result<Vec<Item>, String> {
    let mut errors = Vec::new();
    let mut paths = None;
    for candidate in database_candidates(settings)? {
        match read_database(&candidate) {
            Ok(entries) => { paths = Some(entries); break; }
            Err(error) => errors.push(format!("{}: {error}", candidate.display())),
        }
    }
    let paths = paths.ok_or_else(|| format!("无法读取 VSCode 历史，请在设置中检查数据库路径。\n{}", errors.join("\n")))?;
    Ok(filter_paths(favorites.iter().cloned().chain(paths), query))
}

fn filter_paths(paths: impl IntoIterator<Item = String>, query: &str) -> Vec<Item> {
    let keywords: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    let mut seen = HashSet::new();
    paths.into_iter()
        .filter(|path| path.starts_with("vscode-remote:") || Path::new(path).exists())
        .filter(|path| {
            let lower = path.to_lowercase();
            keywords.iter().all(|word| lower.contains(word))
        })
        .filter(|path| seen.insert(path.clone()))
        .map(Item::new)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_uri_and_remote_paths() {
        assert_eq!(normalize_path("file:///tmp/A%20B%2520"), Some("/tmp/A B%20".into()));
        let remote = "vscode-remote://ssh-remote+host/home/A%20B";
        assert_eq!(normalize_path(remote), Some(remote.into()));
        assert_eq!(normalize_path(""), None);
        assert_eq!(normalize_path("file:///%ZZ"), Some("/%ZZ".into()));
    }

    #[test]
    fn entry_shapes() {
        let entries = serde_json::json!([
            "vscode-remote://ssh-remote+host/a",
            {"folderUri": "file:///tmp/project"},
            {"fileUri": "file:///tmp/file"},
            {"workspace": {"configPath": "file:///tmp/test.code-workspace"}},
            {}
        ]);
        let paths: Vec<_> = entries.as_array().unwrap().iter().filter_map(entry_path).collect();
        assert_eq!(paths.len(), 4);
        assert_eq!(paths[3], "/tmp/test.code-workspace");
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
        let value = r#"{"entries":[{"fileUri":"file:///tmp/a%20b"},{"folderUri":"vscode-remote://host/keep"}],"extra":1}"#;
        db.execute("INSERT INTO ItemTable VALUES (?1, ?2)", [HISTORY_KEY, value]).unwrap();
        assert_eq!(read_database(&path).unwrap().len(), 2);
        let settings = Settings { database: path.to_string_lossy().into_owned(), ..Settings::default() };
        remove(&settings, "/tmp/a b").unwrap();
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
        let entries = read_database(&path).unwrap();
        assert!(!entries.is_empty(), "历史记录不应为空");
    }

    #[test]
    fn explicit_database_does_not_use_default() {
        let settings = Settings { database: "/custom/state.vscdb".into(), ..Settings::default() };
        assert_eq!(database_candidates(&settings).unwrap(), [PathBuf::from("/custom/state.vscdb")]);
    }
}
