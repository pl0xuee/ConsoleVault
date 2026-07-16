//! Build `std::process::Command`s for external system tools with a host-clean
//! environment.
//!
//! ConsoleVault ships as an AppImage. Its `AppRun` injects variables such as
//! `LD_LIBRARY_PATH` and `GTK_PATH` that point *inside* the mounted image so the
//! bundled Tauri/GTK libraries are found. If those leak into the system tools we
//! spawn — `flatpak`, `pacman`, `cmake`, the emulators themselves — the tool
//! loads the AppImage's bundled libraries instead of the host's and breaks. In
//! practice `flatpak install`/`flatpak info` fail this way, which is why a
//! freshly-installed emulator never registers as "Installed".
//!
//! `command()` restores each injected variable to the value `AppRun` saved (the
//! `*_ORIG` backups it may write) or unsets it, giving children a host-like
//! environment. It is a no-op when we are not running from an AppImage, so
//! `tauri dev` and native builds are unaffected.

use std::process::Command;

/// Variables an AppImage's `AppRun` rewrites to point inside the mounted image.
const APPIMAGE_INJECTED: &[&str] = &[
    "LD_LIBRARY_PATH",
    "LD_PRELOAD",
    "GTK_PATH",
    "GTK_EXE_PREFIX",
    "GTK_DATA_PREFIX",
    "GDK_PIXBUF_MODULE_FILE",
    "GDK_PIXBUF_MODULEDIR",
    "GIO_MODULE_DIR",
    "GSETTINGS_SCHEMA_DIR",
    "GST_PLUGIN_SYSTEM_PATH",
    "GST_PLUGIN_SYSTEM_PATH_1_0",
    "FONTCONFIG_FILE",
    "FONTCONFIG_PATH",
    "QT_PLUGIN_PATH",
    "PYTHONHOME",
    "PYTHONPATH",
    "PERLLIB",
];

/// True when the current process is running from an AppImage.
fn in_appimage() -> bool {
    std::env::var_os("APPDIR").is_some() || std::env::var_os("APPIMAGE").is_some()
}

/// Build a `Command` for an external system tool with the AppImage's injected
/// environment removed. Outside an AppImage the host environment is inherited
/// unchanged.
pub fn command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    if !in_appimage() {
        return cmd;
    }
    for var in APPIMAGE_INJECTED {
        match std::env::var_os(format!("{var}_ORIG")) {
            // AppRun saved the host's original value — restore it.
            Some(orig) if !orig.is_empty() => {
                cmd.env(var, orig);
            }
            // No usable backup — drop the AppImage-local value entirely.
            _ => {
                cmd.env_remove(var);
            }
        }
    }
    cmd
}
