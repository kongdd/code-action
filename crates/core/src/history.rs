use crate::store::Settings;
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use serde::Serialize;
use serde_json::Value;
use std::{collections::{BTreeMap, HashSet}, path::{Path, PathBuf}, sync::Mutex, time::Duration};
use url::Url;

const HISTORY_KEY: &str = "history.recentlyOpenedPathsList";
const HISTORY_KEYS: [&str; 2] = ["recently.opened", HISTORY_KEY];

#[derive(Debug, Clone, Serialize)]
pub struct Item {
    pub title: String,
    pub path: String,
    pub remote: bool,
}

impl Item {
    pub fn new(path: String) -> Self {
        let remote = path.starts_with("vscode-remote:") || path.starts_with("ssh://");
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
static CACHE: Mutex<BTreeMap<PathBuf, (Stamp, Vec<String>)>> = Mutex::new(BTreeMap::new());

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
        if let Some((known, entries)) = cache.get(path) {
            if *known == current { return Ok(entries.clone()); }
        }
    }
    let entries = query_history(path)?;
    if let Ok(mut cache) = CACHE.lock() {
        cache.insert(path.to_path_buf(), (current, entries.clone()));
    }
    Ok(entries)
}

fn query_history(path: &Path) -> Result<Vec<String>, String> {
    // Native SQLite includes committed WAL data without copying or modifying VSCode's database.
    let db = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| e.to_string())?;
    db.busy_timeout(Duration::from_millis(500)).map_err(|e| e.to_string())?;
    let mut paths = Vec::new();
    for (_, data) in history_data(&db)? {
        paths.extend(data["entries"].as_array().unwrap().iter().filter_map(entry_path));
    }
    Ok(paths)
}

fn history_data(db: &Connection) -> Result<Vec<(&'static str, Value)>, String> {
    let mut histories = Vec::new();
    // 不同版本/作用域使用不同键，不能因旧键存在就跳过当前记录。
    for key in HISTORY_KEYS {
        let value: Option<String> = db.query_row(
            "SELECT value FROM ItemTable WHERE key = ?1", [key], |row| row.get(0),
        ).optional().map_err(|e| e.to_string())?;
        let Some(value) = value else { continue; };
        let data: Value = serde_json::from_str(&value).map_err(|e| e.to_string())?;
        if !data.get("entries").is_some_and(Value::is_array) {
            return Err("历史记录格式不正确".into());
        }
        histories.push((key, data));
    }
    if histories.is_empty() { return Err("未找到历史记录数据".into()); }
    Ok(histories)
}

fn read_histories(candidates: &[PathBuf]) -> Result<Vec<(PathBuf, Vec<String>)>, String> {
    let mut histories = Vec::new();
    let mut errors = Vec::new();
    for path in candidates {
        match read_database(path) {
            Ok(entries) => histories.push((path.clone(), entries)),
            Err(error) => errors.push(format!("{}: {error}", path.display())),
        }
    }
    if histories.is_empty() {
        return Err(format!("无法读取 VSCode 历史，请在设置中检查数据库路径。\n{}", errors.join("\n")));
    }
    Ok(histories)
}

pub fn remove(settings: &Settings, target: &str) -> Result<(), String> {
    remove_from_databases(&database_candidates(settings)?, target)
}

fn remove_from_databases(candidates: &[PathBuf], target: &str) -> Result<(), String> {
    // 合并展示后，删除也必须覆盖所有来源，避免记录从另一数据库重新出现。
    for (path, entries) in read_histories(candidates)? {
        if entries.iter().any(|entry| entry == target) {
            remove_from_database(&path, target)?;
        }
    }
    Ok(())
}

fn remove_from_database(path: &Path, target: &str) -> Result<(), String> {
    let mut db = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE)
        .map_err(|e| e.to_string())?;
    db.busy_timeout(Duration::from_millis(3000)).map_err(|e| e.to_string())?;
    let tx = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .map_err(|e| e.to_string())?;
    for (key, mut data) in history_data(&tx)? {
        let entries = data["entries"].as_array_mut().unwrap();
        let count = entries.len();
        entries.retain(|entry| entry_path(entry).as_deref() != Some(target));
        if entries.len() != count {
            tx.execute("UPDATE ItemTable SET value = ?1 WHERE key = ?2", [&data.to_string(), key])
                .map_err(|e| e.to_string())?;
        }
    }
    tx.commit().map_err(|e| e.to_string())?;
    if let Ok(mut cache) = CACHE.lock() { cache.remove(path); }
    Ok(())
}

pub fn recent(settings: &Settings, favorites: &[String], query: &str) -> Result<Vec<Item>, String> {
    // shared storage 与旧库可能各保留部分记录，合并后统一过滤、去重。
    let histories = read_histories(&database_candidates(settings)?)?;
    let paths = histories.into_iter().flat_map(|(_, entries)| entries);
    Ok(filter_paths(favorites.iter().filter(|p| !p.starts_with("ssh://")).cloned().chain(paths), query))
}

/// fs 结果只来自索引，收藏项需要单独并入，否则置顶项在 fs 模式下永远不出现。
pub fn merge_favorites(found: Vec<Item>, favorites: &[String], query: &str) -> Vec<Item> {
    let keywords: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    let mut pinned: Vec<Item> = favorites.iter()
        .filter(|path| keywords.iter().all(|word| path.to_lowercase().contains(word)))
        .map(|path| Item::new(path.clone()))
        .collect();
    pinned.extend(found.into_iter().filter(|item| !favorites.contains(&item.path)));
    pinned
}

pub(crate) fn filter_paths(paths: impl IntoIterator<Item = String>, query: &str) -> Vec<Item> {
    let keywords: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    let mut seen = HashSet::new();
    paths.into_iter()
        .filter(|path| path.starts_with("vscode-remote:") || path.starts_with("ssh://") || Path::new(path).exists())
        .filter(|path| {
            let lower = path.to_lowercase();
            keywords.iter().all(|word| lower.contains(word))
        })
        .filter(|path| seen.insert(path.clone()))
        .map(Item::new)
        .collect()
}

#[cfg(test)]
#[path = "../tests/history.rs"]
mod tests;
