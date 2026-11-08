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
    /// vscode / zed / fs / toggle
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub database: String,
    pub merge_history: bool,
    pub fs_binary: String,
    pub code: String,
    pub zed: String,
    pub terminal: String,
    pub shortcuts: Vec<Shortcut>,
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
            shortcuts: vec![
                Shortcut { id: "toggle".into(), accelerator: "CommandOrControl+Shift+Space".into(), action: "toggle".into(), ..Shortcut::default() },
                Shortcut { id: "vscode".into(), accelerator: vscode.into(), action: "vscode".into(), ..Shortcut::default() },
                Shortcut { id: "zed".into(), accelerator: zed.into(), action: "zed".into(), ..Shortcut::default() },
                Shortcut { id: "fs".into(), accelerator: fs.into(), action: "fs".into(), ..Shortcut::default() },
            ],
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
        let data = match fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| format!("配置文件损坏：{e}"))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Store::default(),
            Err(e) => return Err(e.to_string()),
        };
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
