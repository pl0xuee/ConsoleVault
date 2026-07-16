//! Detect which emulators are installed and how they can be launched.

use crate::registry::{self, EmulatorDef};
use crate::syscmd;
use serde::Serialize;
use std::path::PathBuf;

/// How a detected emulator will be invoked.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Invocation {
    /// A native binary found on PATH (from pacman / native / source build).
    Native { program: String },
    /// A Flatpak app id run via `flatpak run <id>`.
    Flatpak { app_id: String },
}

/// Detection result for a single system, sent to the UI.
#[derive(Debug, Clone, Serialize)]
pub struct EmulatorStatus {
    pub system: String,
    pub system_label: String,
    pub emulator: String,
    pub installed: bool,
    /// How it will launch, when installed.
    pub invocation: Option<Invocation>,
    pub bios_required: bool,
    /// Whether an official pacman package exists for this emulator.
    pub has_pacman: bool,
    /// Whether a Flathub app id exists for this emulator.
    pub has_flatpak: bool,
}

/// Return the first existing path for `program` on PATH, if any.
fn which(program: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    std::env::split_paths(&path_var)
        .map(|dir| dir.join(program))
        .find(|candidate| candidate.is_file())
}

/// Whether a Flatpak app id is installed (`flatpak info <id>` succeeds).
pub fn flatpak_installed(app_id: &str) -> bool {
    flatpak_info(&["info", app_id])
}

/// Whether a Flatpak app id is installed in the per-user scope specifically —
/// which is the only scope we install into, and thus the only one we uninstall.
pub fn flatpak_installed_user(app_id: &str) -> bool {
    flatpak_info(&["info", "--user", app_id])
}

/// Run `flatpak <args>` quietly and report whether it succeeded.
fn flatpak_info(args: &[&str]) -> bool {
    syscmd::command("flatpak")
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Resolve how an emulator can be invoked right now, preferring a native binary.
pub fn resolve_invocation(def: &EmulatorDef) -> Option<Invocation> {
    for bin in def.binaries {
        if which(bin).is_some() {
            return Some(Invocation::Native {
                program: (*bin).to_string(),
            });
        }
    }
    if let Some(id) = def.flatpak_id {
        if flatpak_installed(id) {
            return Some(Invocation::Flatpak {
                app_id: id.to_string(),
            });
        }
    }
    None
}

/// Detection status for every supported system.
pub fn detect_all() -> Vec<EmulatorStatus> {
    registry::EMULATORS
        .iter()
        .map(|def| {
            let invocation = resolve_invocation(def);
            EmulatorStatus {
                system: def.system.to_string(),
                system_label: def.system_label.to_string(),
                emulator: def.emulator.to_string(),
                installed: invocation.is_some(),
                invocation,
                bios_required: def.bios_required,
                has_pacman: def.pacman_pkg.is_some(),
                has_flatpak: def.flatpak_id.is_some(),
            }
        })
        .collect()
}
