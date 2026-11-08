//! 桌面版与命令行版共用的核心逻辑：历史记录读取、文件检索与打开动作。

pub mod actions;
pub mod history;
pub mod store;
pub mod zed_history;

use actions::Mode;
use history::Item;
use store::Store;
use std::collections::HashSet;

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
                let vscode = || history::recent(&store.settings, &store.preferences.favorites, &query);
                let zed = || zed_history::recent(&store.preferences.favorites, &query);
                if !store.settings.merge_history {
                    return match mode {
                        Mode::Vscode => vscode(),
                        Mode::Zed => zed(),
                        Mode::Fs => unreachable!(),
                    };
                }
                let (primary, secondary) = match mode {
                    Mode::Vscode => (vscode(), zed()),
                    Mode::Zed => (zed(), vscode()),
                    Mode::Fs => unreachable!(),
                };
                merge_histories(primary, secondary, &store.preferences.favorites)
            }).await.map_err(|e| e.to_string())?
        }
    }
}

fn merge_histories(
    primary: Result<Vec<Item>, String>, secondary: Result<Vec<Item>, String>, favorites: &[String],
) -> Result<Vec<Item>, String> {
    let (mut items, extra) = match (primary, secondary) {
        (Ok(a), Ok(b)) => (a, b),
        (Ok(a), Err(_)) | (Err(_), Ok(a)) => (a, Vec::new()),
        (Err(a), Err(b)) => return Err(format!("{a}\n{b}")),
    };
    let mut seen: HashSet<String> = items.iter().map(|item| item.path.clone()).collect();
    items.extend(extra.into_iter().filter(|item| seen.insert(item.path.clone())));
    items.sort_by_key(|item| !favorites.contains(&item.path));
    Ok(items)
}

#[cfg(test)]
#[path = "../tests/lib.rs"]
mod tests;
