use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub hotkey: String,
    pub language: String,
    pub model: String,
    pub overlay_position: String,
    pub activation_mode: String,
    pub autostart: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey: "ctrl+shift+space".to_string(),
            language: "de".to_string(),
            model: "base".to_string(),
            overlay_position: "bottom-right".to_string(),
            activation_mode: "push-to-talk".to_string(),
            autostart: false,
        }
    }
}

/// Get the portable base path (next to the exe)
/// Portabler Basispfad (neben der exe)
pub fn get_base_path() -> PathBuf {
    std::env::current_exe()
        .expect("failed to get exe path")
        .parent()
        .expect("failed to get exe directory")
        .to_path_buf()
}

pub fn get_config_path() -> PathBuf {
    get_base_path().join("config").join("settings.json")
}

pub fn get_models_path() -> PathBuf {
    get_base_path().join("models")
}

pub fn load_settings() -> Settings {
    let path = get_config_path();
    if path.exists() {
        let data = std::fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        let settings = Settings::default();
        save_settings(&settings);
        settings
    }
}

pub fn save_settings(settings: &Settings) {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let data = serde_json::to_string_pretty(settings).expect("failed to serialize settings");
    std::fs::write(&path, data).expect("failed to write settings");
}
