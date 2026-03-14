use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;

// --- Default value functions ---
fn default_mode() -> String {
    "proxy".to_string()
}
fn default_font_size() -> u32 {
    14
}
fn default_theme() -> String {
    "dark".to_string()
}
fn default_log_format() -> String {
    "txt".to_string()
}

// --- Settings structs ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiSettings {
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub model: String,
}

impl Default for GeminiSettings {
    fn default() -> Self {
        Self {
            mode: default_mode(),
            base_url: String::new(),
            api_key: String::new(),
            model: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiSettings {
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    #[serde(default = "default_theme")]
    pub theme: String,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            font_size: default_font_size(),
            theme: default_theme(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct OutputSettings {
    /// Empty = output to same directory as video file
    #[serde(default)]
    pub default_dir: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogSettings {
    #[serde(default)]
    pub auto_export: bool,
    #[serde(default = "default_log_format")]
    pub export_format: String,
    /// Empty = same as output directory
    #[serde(default)]
    pub export_dir: String,
}

impl Default for LogSettings {
    fn default() -> Self {
        Self {
            auto_export: false,
            export_format: default_log_format(),
            export_dir: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Settings {
    #[serde(default)]
    pub gemini: GeminiSettings,
    #[serde(default)]
    pub ui: UiSettings,
    #[serde(default)]
    pub output: OutputSettings,
    #[serde(default)]
    pub log: LogSettings,
}

// --- Path resolution ---

pub fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("無法取得設定目錄: {}", e))?;
    Ok(config_dir.join("settings.json"))
}

// --- Load / Save ---

pub fn load_settings(path: &Path) -> Result<Settings, String> {
    match fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).map_err(|e| format!("設定檔格式錯誤: {}", e)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(e) => Err(format!("無法讀取設定檔: {}", e)),
    }
}

pub fn save_settings_to_file(path: &Path, settings: &serde_json::Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("無法建立設定目錄: {}", e))?;
    }
    let content =
        serde_json::to_string_pretty(settings).map_err(|e| format!("序列化設定失敗: {}", e))?;
    fs::write(path, content).map_err(|e| format!("寫入設定檔失敗: {}", e))
}
