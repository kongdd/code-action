use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::PathBuf};

/// 应用标识，与 tauri.conf.json 的 identifier 保持一致，用于定位配置目录。
pub const APP_ID: &str = "io.github.kongdd.code-action";

/// 配置文件路径：桌面版与命令行版共用同一份配置。
pub fn config_path() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join(APP_ID).join("config.json"))
}

/// 全局快捷键绑定：accelerator 为 global-hotkey 可解析的组合键。
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Shortcut {
    pub id: String,
    pub accelerator: String,
    /// vscode / zed / fs / toggle / clipboard-vscode / clipboard-zed
    pub action: String,
    pub enabled: bool,
    pub auto_copy: bool,
}

impl Default for Shortcut {
    fn default() -> Self {
        Self {
            id: String::new(), accelerator: String::new(), action: "toggle".into(),
            enabled: true, auto_copy: false,
        }
    }
}

impl Shortcut {
    pub fn is_active(&self) -> bool {
        self.enabled && !self.accelerator.trim().is_empty()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DirectoryOpener {
    #[default]
    System,
    DirDeck,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub database: String,
    pub merge_history: bool,
    pub fs_binary: String,
    pub code: String,
    pub zed: String,
    pub terminal: String,
    pub directory_opener: DirectoryOpener,
    pub shortcuts: Vec<Shortcut>,
    #[serde(default)]
    pub clipboard_shortcuts_added: bool,
    /// 功能别名：模式 -> 显示名称
    pub aliases: BTreeMap<String, String>,
}

impl Default for Settings {
    fn default() -> Self {
        let binary = dirs::home_dir().unwrap_or_default().join(".cargo/bin")
            .join(if cfg!(windows) { "fs.exe" } else { "fs" });
        // 功能快捷键跟随平台：macOS 用 ⌘⌃，Windows/Linux 用 Ctrl+Alt。
        let (vscode, zed, fs) = if cfg!(target_os = "macos") {
            ("Command+Control+C", "Command+Control+Z", "Command+Control+E")
        } else {
            ("Control+Alt+C", "Control+Alt+Z", "Control+Alt+E")
        };
        Self {
            database: String::new(),
            merge_history: true,
            fs_binary: if binary.is_file() { binary.to_string_lossy().into() } else { "fs".into() },
            code: "code".into(),
            zed: "zed".into(),
            terminal: String::new(),
            directory_opener: DirectoryOpener::default(),
            shortcuts: vec![
                Shortcut { id: "toggle".into(), accelerator: "CommandOrControl+Shift+Space".into(), action: "toggle".into(), ..Shortcut::default() },
                Shortcut { id: "vscode".into(), accelerator: vscode.into(), action: "vscode".into(), ..Shortcut::default() },
                Shortcut { id: "zed".into(), accelerator: zed.into(), action: "zed".into(), ..Shortcut::default() },
                Shortcut { id: "fs".into(), accelerator: fs.into(), action: "fs".into(), ..Shortcut::default() },
                #[cfg(target_os = "macos")]
                Shortcut { id: "clipboard-vscode".into(), accelerator: "Control+Alt+C".into(), action: "clipboard-vscode".into(), ..Shortcut::default() },
                #[cfg(target_os = "macos")]
                Shortcut { id: "clipboard-zed".into(), accelerator: "Control+Alt+Z".into(), action: "clipboard-zed".into(), ..Shortcut::default() },
            ],
            clipboard_shortcuts_added: true,
            aliases: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub favorites: Vec<String>,
    pub queries: BTreeMap<String, String>,
    pub mode: String,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Store {
    pub settings: Settings,
    pub preferences: Preferences,
}

pub struct Storage {
    pub data: Store,
    path: PathBuf,
}

impl Storage {
    pub fn load(path: PathBuf) -> Result<Self, String> {
        let mut data: Store = match fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| format!("配置文件损坏：{e}"))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Store::default(),
            Err(e) => return Err(e.to_string()),
        };
        if !data.settings.clipboard_shortcuts_added {
            #[cfg(target_os = "macos")]
            for mut shortcut in Settings::default().shortcuts.into_iter().filter(|item| item.id.starts_with("clipboard-")) {
                if data.settings.shortcuts.iter().any(|item| item.id == shortcut.id) { continue; }
                if data.settings.shortcuts.iter().any(|item| item.is_active() && item.accelerator.eq_ignore_ascii_case(&shortcut.accelerator)) {
                    shortcut.enabled = false;
                }
                data.settings.shortcuts.push(shortcut);
            }
            data.settings.clipboard_shortcuts_added = true;
        }
        Ok(Self { data, path })
    }

    pub fn save(&mut self, data: Store) -> Result<(), String> {
        let parent = self.path.parent().ok_or("无效的配置目录")?;
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let temp = self.path.with_extension("tmp");
        let bytes = serde_json::to_vec_pretty(&data).map_err(|e| e.to_string())?;
        fs::write(&temp, bytes).map_err(|e| e.to_string())?;
        fs::rename(&temp, &self.path).map_err(|e| e.to_string())?;
        self.data = data;
        Ok(())
    }
}

#[cfg(test)]
#[path = "../tests/store.rs"]
mod tests;
