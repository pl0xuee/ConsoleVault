mod config;
mod detect;
mod install;
mod launch;
mod metadata;
mod registry;
mod scan;
mod syscmd;

use config::AppConfig;
use detect::EmulatorStatus;
use install::{InstallMethod, InstallPlan, UninstallPlan};
use scan::Game;
use tauri::AppHandle;

/// Load persisted configuration (rom folders, overrides, setup flag).
#[tauri::command]
fn get_config(app: AppHandle) -> Result<AppConfig, String> {
    config::load(&app)
}

/// Persist configuration.
#[tauri::command]
fn save_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    config::save(&app, &config)
}

/// Detect which emulators are installed and how they'll launch.
#[tauri::command]
async fn detect_emulators() -> Result<Vec<EmulatorStatus>, String> {
    tauri::async_runtime::spawn_blocking(detect::detect_all)
        .await
        .map_err(|e| format!("detect task failed: {e}"))
}

/// Preview the install commands for a system (optionally for a chosen method).
#[tauri::command]
fn install_plan(system: String, method: Option<InstallMethod>) -> Result<InstallPlan, String> {
    install::plan(&system, method)
}

/// Install an emulator, streaming `install-log` events; resolves when finished.
#[tauri::command]
async fn install_emulator(
    app: AppHandle,
    system: String,
    method: InstallMethod,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || install::install(&app, &system, method))
        .await
        .map_err(|e| format!("install task failed: {e}"))?
}

/// Preview the uninstall commands for a system, based on what's installed now.
#[tauri::command]
fn uninstall_plan(app: AppHandle, system: String) -> Result<UninstallPlan, String> {
    install::uninstall_plan(&app, &system)
}

/// Uninstall an emulator, streaming `install-log` events; resolves when finished.
#[tauri::command]
async fn uninstall_emulator(app: AppHandle, system: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || install::uninstall(&app, &system))
        .await
        .map_err(|e| format!("uninstall task failed: {e}"))?
}

/// Scan all configured ROM folders and return the classified game list.
#[tauri::command]
async fn scan_library(app: AppHandle) -> Result<Vec<Game>, String> {
    let config = config::load(&app)?;
    tauri::async_runtime::spawn_blocking(move || scan::scan(&config.rom_folders))
        .await
        .map_err(|e| format!("scan task failed: {e}"))
}

/// Box art for a title as a data URL, or null when unavailable.
#[tauri::command]
async fn get_boxart(
    app: AppHandle,
    system: String,
    title: String,
) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || metadata::boxart(&app, &system, &title))
        .await
        .map_err(|e| format!("boxart task failed: {e}"))?
}

/// Launch a game with the appropriate emulator.
#[tauri::command]
fn launch_game(app: AppHandle, system: String, path: String) -> Result<(), String> {
    let config = config::load(&app)?;
    launch::launch(&config, &system, &path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            detect_emulators,
            install_plan,
            install_emulator,
            uninstall_plan,
            uninstall_emulator,
            scan_library,
            get_boxart,
            launch_game,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
