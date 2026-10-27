#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod actions;
mod history;
mod store;

use actions::{Action, Mode};
use serde::Serialize;
use std::sync::Mutex;
use store::{Preferences, Settings, Shortcut, Storage, Store};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager, State,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut as Hotkey, ShortcutState};

type AppState = Mutex<Storage>;

/// 窗口是否曾获得过焦点：启动时可能立即收到失焦事件，此时不应隐藏。
static FOCUSED_ONCE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// 快捷键触发时通知前端的载荷。
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShortcutPayload {
    /// vscode / zed / fs，空表示仅切换窗口显隐
    mode: String,
    auto_copy: bool,
}

/// 隐藏 macOS 的红绿灯按钮，界面只保留自绘顶栏。
#[cfg(target_os = "macos")]
fn hide_traffic_lights(window: &tauri::WebviewWindow) {
    use objc2::{msg_send, runtime::AnyObject};
    let Ok(ptr) = window.ns_window() else { return };
    let ns_window = unsafe { &*(ptr as *const AnyObject) };
    // NSWindowButton：关闭 0、最小化 1、缩放 2。
    for button in [0usize, 1, 2] {
        let button: *mut AnyObject = unsafe { msg_send![ns_window, standardWindowButton: button] };
        if !button.is_null() {
            unsafe { let _: () = msg_send![button, setHidden: true]; }
        }
    }
}

/// 显示搜索窗口。
fn show(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// 隐藏搜索窗口。
fn hide(app: &tauri::AppHandle) {
    FOCUSED_ONCE.store(false, std::sync::atomic::Ordering::Relaxed);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

/// 切换窗口显隐。
fn toggle(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            hide(app);
        } else {
            show(app);
        }
    }
}

/// 校验并注册全部快捷键，先注销旧绑定以支持运行时修改。
fn apply_shortcuts(app: &tauri::AppHandle, shortcuts: &[Shortcut]) -> Result<(), String> {
    let manager = app.global_shortcut();
    manager.unregister_all().map_err(|e| e.to_string())?;
    for shortcut in shortcuts.iter().filter(|shortcut| shortcut.is_active()) {
        let accelerator = shortcut.accelerator.trim();
        // 提前解析，非法组合在此处报错而不是静默失效。
        accelerator.parse::<Hotkey>().map_err(|e| format!("快捷键 {accelerator} 无效：{e}"))?;
        let action = shortcut.action.clone();
        let auto_copy = shortcut.auto_copy;
        manager
            .on_shortcut(accelerator, move |app, _shortcut, event| {
                if event.state() != ShortcutState::Pressed {
                    return;
                }

                if action == "toggle" {
                    toggle(app);
                    return;
                }
                show(app);
                let _ = app.emit_to("main", "shortcut-triggered", ShortcutPayload {
                    mode: action.clone(),
                    auto_copy,
                });
            })
            .map_err(|e| format!("快捷键 {accelerator} 注册失败：{e}"))?;
    }
    Ok(())
}

#[tauri::command]
fn load_store(state: State<'_, AppState>) -> Result<Store, String> {
    Ok(state.lock().map_err(|e| e.to_string())?.data.clone())
}

#[tauri::command]
fn save_settings(settings: Settings, state: State<'_, AppState>) -> Result<(), String> {
    if [&settings.code, &settings.zed, &settings.fs_binary]
        .iter().any(|value| value.trim().is_empty()) {
        return Err("可执行文件不能为空".into());
    }
    let mut storage = state.lock().map_err(|e| e.to_string())?;
    let data = Store { settings, ..storage.data.clone() };
    storage.save(data)
}

#[tauri::command]
fn save_preferences(preferences: Preferences, state: State<'_, AppState>) -> Result<(), String> {
    let mut storage = state.lock().map_err(|e| e.to_string())?;
    let data = Store { preferences, ..storage.data.clone() };
    storage.save(data)
}

/// 保存并立即注册快捷键，失败时回滚到原有绑定。
#[tauri::command]
fn save_shortcuts(app: tauri::AppHandle, shortcuts: Vec<Shortcut>, state: State<'_, AppState>) -> Result<(), String> {
    let mut storage = state.lock().map_err(|e| e.to_string())?;
    let previous = storage.data.settings.shortcuts.clone();
    let mut settings = storage.data.settings.clone();
    settings.shortcuts = shortcuts;
    let data = Store { settings: settings.clone(), ..storage.data.clone() };
    // 注册与落盘均成功才提交；任一步失败都保留原配置及绑定。
    let result = apply_shortcuts(&app, &settings.shortcuts).and_then(|_| storage.save(data));
    if result.is_err() {
        let _ = apply_shortcuts(&app, &previous);
    }
    result
}

/// 取消上一次尚未结束的 fs 检索：新请求开始时丢弃旧进程。
#[derive(Default)]
struct FsCancel(Mutex<Option<tokio::sync::oneshot::Sender<()>>>);

impl FsCancel {
    fn begin(&self) -> tokio::sync::oneshot::Receiver<()> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        if let Ok(mut slot) = self.0.lock() {
            if let Some(previous) = slot.take() { let _ = previous.send(()); }
            *slot = Some(sender);
        }
        receiver
    }
}

/// fs 结果只来自索引，收藏项需要单独并入，否则置顶项在 fs 模式下永远不出现。
fn merge_favorites(found: Vec<history::Item>, favorites: &[String], query: &str) -> Vec<history::Item> {
    let keywords: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    let mut pinned: Vec<history::Item> = favorites.iter()
        .filter(|path| keywords.iter().all(|word| path.to_lowercase().contains(word)))
        .map(|path| history::Item::new(path.clone()))
        .collect();
    pinned.extend(found.into_iter().filter(|item| !favorites.contains(&item.path)));
    pinned
}

#[tauri::command]
async fn search(
    mode: Mode, query: String, state: State<'_, AppState>, cancel: State<'_, FsCancel>,
) -> Result<Vec<history::Item>, String> {
    let data = state.lock().map_err(|e| e.to_string())?.data.clone();
    match mode {
        Mode::Fs => {
            let receiver = cancel.begin();
            let found = actions::locate(&data.settings, &query, receiver).await?;
            Ok(merge_favorites(found, &data.preferences.favorites, &query))
        }
        _ => tauri::async_runtime::spawn_blocking(move || {
            history::recent(&data.settings, &data.preferences.favorites, &query)
        }).await.map_err(|e| e.to_string())?,
    }
}

#[tauri::command]
async fn open_item(action: Action, mode: Mode, path: String, state: State<'_, AppState>) -> Result<(), String> {
    let settings = state.lock().map_err(|e| e.to_string())?.data.settings.clone();
    actions::open(&settings, action, mode, &path).await
}

#[tauri::command]
fn copy_text(text: String) -> Result<(), String> {
    actions::copy(&text)
}

#[tauri::command]
fn hide_window(window: tauri::WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            // 二次启动时把窗口带回前台，并让前端重置查询与模式。
            show(app);
            let _ = app.emit_to("main", "launcher-show", ());
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            load_store, save_settings, save_preferences, save_shortcuts, search, open_item, copy_text,
            hide_window,
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let config = app.path().app_config_dir()?.join("config.json");
            let storage = Storage::load(config).map_err(std::io::Error::other)?;
            app.manage(Mutex::new(storage));
            app.manage(FsCancel::default());

            let show_item = MenuItem::with_id(app, "show", "显示 Code Action", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;
            let mut tray = TrayIconBuilder::new()
                .tooltip("Code Action")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show(app),
                    "quit" => app.exit(0),
                    _ => (),
                })
                .on_tray_icon_event(|tray, event| {
                    // 左键点击托盘图标切换窗口显隐。
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event {
                        toggle(tray.app_handle());
                    }
                });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.build(app)?;

            #[cfg(target_os = "macos")]
            if let Some(window) = app.get_webview_window("main") {
                hide_traffic_lights(&window);
            }
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::Focused(true) => {
                FOCUSED_ONCE.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            tauri::WindowEvent::Focused(false) => {
                if FOCUSED_ONCE.swap(false, std::sync::atomic::Ordering::Relaxed) {
                    hide(window.app_handle());
                }
            }
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                hide(window.app_handle());
            }
            _ => (),
        })
        .build(tauri::generate_context!())
        .expect("Code Action 启动失败")
        .run(|app, event| {
            // AppKit 在 Ready 之后才允许窗口成为主窗口，此处显示才能正常获得焦点。
            if let tauri::RunEvent::Ready = event {
                // 全局快捷键需在事件循环就绪后注册，否则系统不投递按键。
                let shortcuts = app
                    .state::<AppState>()
                    .lock()
                    .map(|state| state.data.settings.shortcuts.clone())
                    .unwrap_or_default();
                if let Err(error) = apply_shortcuts(app, &shortcuts) {
                    eprintln!("快捷键注册失败：{error}");
                } else {
                    let count = shortcuts.iter().filter(|shortcut| shortcut.is_active()).count();
                    eprintln!("已注册 {count} 组全局快捷键");
                }

                show(app);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(items: &[history::Item]) -> Vec<&str> {
        items.iter().map(|item| item.path.as_str()).collect()
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

    #[test]
    fn favorites_lead_the_result_and_are_not_duplicated() {
        let favorites = vec!["/a/soil".to_owned()];
        let found = vec![history::Item::new("/b/rain".into()), history::Item::new("/a/soil".into())];
        let merged = merge_favorites(found, &favorites, "");
        assert_eq!(paths(&merged), ["/a/soil", "/b/rain"]);
    }

    #[test]
    fn favorites_obey_the_query() {
        let favorites = vec!["/a/soil".to_owned()];
        assert!(merge_favorites(Vec::new(), &favorites, "rain").is_empty());
        assert_eq!(paths(&merge_favorites(Vec::new(), &favorites, "SOIL")), ["/a/soil"]);
    }
}
