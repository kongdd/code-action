//! 桌面版与命令行版共用的核心逻辑：历史记录读取、文件检索与打开动作。

pub mod actions;
pub mod history;
pub mod store;

use actions::Mode;
use history::Item;
use store::Store;

/// 按模式检索；历史数据库读取在线程池中执行，避免阻塞异步运行时。
pub async fn search(
    store: &Store,
    mode: Mode,
    query: &str,
    cancel: Option<tokio::sync::oneshot::Receiver<()>>,
) -> Result<Vec<Item>, String> {
    match mode {
        Mode::Fs => {
            let found = actions::locate(&store.settings, query, cancel).await?;
            Ok(history::merge_favorites(found, &store.preferences.favorites, query))
        }
        Mode::Vscode | Mode::Zed => {
            let store = store.clone();
            let query = query.to_owned();
            tokio::task::spawn_blocking(move || {
                history::recent(&store.settings, &store.preferences.favorites, &query)
            }).await.map_err(|e| e.to_string())?
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn history_modes_share_search_and_favorites() {
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
        store.preferences.favorites = vec!["vscode-remote://host/soil-favorite".into()];
        for mode in [Mode::Vscode, Mode::Zed] {
            let items = search(&store, mode, "soil", None).await.unwrap();
            assert_eq!(items.iter().map(|item| item.path.as_str()).collect::<Vec<_>>(),
                ["vscode-remote://host/soil-favorite", "vscode-remote://host/soil"]);
        }
    }
}
