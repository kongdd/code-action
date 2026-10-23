use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::PathBuf};

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
mod tests {
    use super::*;

    #[test]
    fn legacy_shortcuts_remain_enabled() {
        let shortcut: Shortcut = serde_json::from_value(serde_json::json!({
            "accelerator": "Command+Control+E", "action": "fs", "autoCopy": false,
        })).unwrap();
        assert!(shortcut.enabled);
        assert!(shortcut.is_active());
        assert!(Settings::default().shortcuts.iter().all(Shortcut::is_active));
    }

    #[test]
    fn disabled_or_empty_shortcuts_are_inactive() {
        let mut shortcut = Settings::default().shortcuts.remove(0);
        shortcut.enabled = false;
        assert!(!shortcut.is_active());
        let json = serde_json::to_string(&shortcut).unwrap();
        assert!(!serde_json::from_str::<Shortcut>(&json).unwrap().enabled);
        shortcut.enabled = true;
        shortcut.accelerator = "  ".into();
        assert!(!shortcut.is_active());
    }

    #[test]
    fn default_shortcuts_follow_the_platform() {
        let shortcuts = Settings::default().shortcuts;
        let prefix = if cfg!(target_os = "macos") {
            "Command+Control"
        } else {
            "Control+Alt"
        };
        for (action, key) in [("fs", "E"), ("zed", "Z"), ("vscode", "C")] {
            let shortcut = shortcuts.iter().find(|item| item.action == action).unwrap();
            assert_eq!(shortcut.accelerator, format!("{prefix}+{key}"));
        }
    }

    #[test]
    fn round_trip_and_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut storage = Storage::load(path.clone()).unwrap();
        let mut data = storage.data.clone();
        data.preferences.favorites.push("/tmp/中文 项目".into());
        data.preferences.queries.insert("fs".into(), "*.pdf".into());
        data.settings.shortcuts[0].auto_copy = true;
        data.settings.shortcuts[1].enabled = false;
        data.settings.aliases.insert("fs".into(), "文件".into());
        storage.save(data).unwrap();
        let loaded = Storage::load(path).unwrap();
        assert_eq!(loaded.data.preferences.favorites, ["/tmp/中文 项目"]);
        assert_eq!(loaded.data.preferences.queries["fs"], "*.pdf");
        assert_eq!(loaded.data.settings.code, "code");
        assert!(loaded.data.settings.shortcuts[0].auto_copy);
        assert!(!loaded.data.settings.shortcuts[1].enabled);
        assert_eq!(loaded.data.settings.aliases["fs"], "文件");
    }

    #[test]
    fn damaged_configuration_is_not_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, b"{broken").unwrap();
        assert!(Storage::load(path.clone()).is_err());
        assert_eq!(fs::read(path).unwrap(), b"{broken");
    }
}
