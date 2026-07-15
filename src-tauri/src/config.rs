//! Persistent user configuration, stored as JSON in the app config dir.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// User-editable settings persisted across runs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// Root folders to scan for ROMs.
    pub rom_folders: Vec<String>,
    /// Per-system manual override of the emulator invocation.
    ///
    /// Key = system key ("snes", ...). Value = a full command; the ROM path is
    /// appended as the final argument. Overrides auto-detection when set.
    pub emulator_overrides: HashMap<String, String>,
    /// Whether the first-run setup wizard has been completed.
    pub setup_complete: bool,
}

/// Absolute path to the config file (`<config_dir>/config.json`).
fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("no config dir: {e}"))?;
    Ok(dir.join("config.json"))
}

/// Load config from disk, returning defaults if it does not exist yet.
pub fn load(app: &AppHandle) -> Result<AppConfig, String> {
    let path = config_path(app)?;
    match std::fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).map_err(|e| format!("corrupt config: {e}")),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(AppConfig::default()),
        Err(e) => Err(format!("read config: {e}")),
    }
}

/// Persist config to disk, creating the config directory if needed.
pub fn save(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = config_path(app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir config: {e}"))?;
    }
    let text = serde_json::to_string_pretty(config).map_err(|e| format!("serialize: {e}"))?;
    std::fs::write(&path, text).map_err(|e| format!("write config: {e}"))
}
