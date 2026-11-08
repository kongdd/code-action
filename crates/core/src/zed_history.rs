use crate::history::{filter_paths, Item};
use rusqlite::{Connection, OpenFlags};
use std::{path::{Path, PathBuf}, time::Duration};
use url::Url;

fn database() -> Result<PathBuf, String> {
    let root = if cfg!(target_os = "macos") {
        dirs::data_dir()
    } else {
        dirs::data_local_dir()
    }.ok_or("无法定位 Zed 数据目录")?;
    let name = if cfg!(any(target_os = "macos", windows)) { "Zed" } else { "zed" };
    Ok(root.join(name).join("db/0-stable/db.sqlite"))
}

fn ssh_uri(host: &str, user: Option<&str>, port: Option<u16>, path: &str) -> Option<String> {
    let mut uri = Url::parse("ssh://localhost/").ok()?;
    uri.set_host(Some(host)).ok()?;
    if let Some(user) = user { uri.set_username(user).ok()?; }
    uri.set_port(port).ok()?;
    uri.set_path(path);
    Some(uri.into())
}

fn read_database(path: &Path) -> Result<Vec<String>, String> {
    let db = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| e.to_string())?;
    db.busy_timeout(Duration::from_millis(500)).map_err(|e| e.to_string())?;
    let mut statement = db.prepare(
        "SELECT w.paths, w.paths_order, w.remote_connection_id, r.kind, r.host, r.user, r.port \
         FROM workspaces w LEFT JOIN remote_connections r ON r.id = w.remote_connection_id \
         ORDER BY w.timestamp DESC, w.workspace_id DESC"
    ).map_err(|e| e.to_string())?;
    let mut rows = statement.query([]).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let paths: Option<String> = row.get(0).map_err(|e| e.to_string())?;
        let order: Option<String> = row.get(1).map_err(|e| e.to_string())?;
        let remote: Option<i64> = row.get(2).map_err(|e| e.to_string())?;
        let kind: Option<String> = row.get(3).map_err(|e| e.to_string())?;
        let host: Option<String> = row.get(4).map_err(|e| e.to_string())?;
        let user: Option<String> = row.get(5).map_err(|e| e.to_string())?;
        let port: Option<u16> = row.get(6).map_err(|e| e.to_string())?;
        let paths: Vec<_> = paths.as_deref().unwrap_or("").split('\n').filter(|p| !p.is_empty()).collect();
        let indices: Vec<usize> = order.as_deref().unwrap_or("").split(',').filter_map(|s| s.parse().ok()).collect();
        let indices = if indices.len() == paths.len() && indices.iter().all(|&i| i < paths.len()) {
            indices
        } else { (0..paths.len()).collect() };
        for i in indices {
            let target = if remote.is_none() {
                Some(paths[i].to_owned())
            } else if kind.as_deref() == Some("ssh") {
                host.as_deref().and_then(|h| ssh_uri(h, user.as_deref(), port, paths[i]))
            } else {
                None
            };
            if let Some(target) = target { result.push(target); }
        }
    }
    Ok(result)
}

pub fn recent(favorites: &[String], query: &str) -> Result<Vec<Item>, String> {
    let path = database()?;
    let paths = read_database(&path).map_err(|e| format!("无法读取 Zed 历史 {}：{e}", path.display()))?;
    Ok(filter_paths(favorites.iter().filter(|p| !p.starts_with("vscode-remote:")).cloned().chain(paths), query))
}

#[cfg(test)]
#[path = "../tests/zed_history.rs"]
mod tests;
