//! Launch a game by spawning the appropriate emulator as a detached process.

use crate::config::AppConfig;
use crate::detect::{resolve_invocation, Invocation};
use crate::registry;
use std::process::Command;

/// Launch `rom_path` on `system`.
///
/// Resolution order:
/// 1. A user override command from config (ROM appended as the final arg).
/// 2. A detected native binary on PATH.
/// 3. The Flatpak app id via `flatpak run`.
///
/// Spawns and returns immediately; the emulator runs independently of the app.
pub fn launch(config: &AppConfig, system: &str, rom_path: &str) -> Result<(), String> {
    let def = registry::find(system).ok_or_else(|| format!("unknown system: {system}"))?;

    // 1. User override: split into program + args, then append the ROM path.
    if let Some(override_cmd) = config.emulator_overrides.get(system) {
        let mut parts = override_cmd.split_whitespace();
        let program = parts
            .next()
            .ok_or_else(|| "empty emulator override command".to_string())?;
        let mut cmd = Command::new(program);
        cmd.args(parts);
        cmd.arg(rom_path);
        return spawn(cmd, &def.emulator);
    }

    // 2 & 3. Detected native binary or flatpak.
    let invocation = resolve_invocation(def).ok_or_else(|| {
        format!(
            "{} is not installed. Install it from the Emulators screen first.",
            def.emulator
        )
    })?;

    let cmd = match invocation {
        Invocation::Native { program } => {
            let mut cmd = Command::new(program);
            cmd.args(def.launch_args);
            cmd.arg(rom_path);
            cmd
        }
        Invocation::Flatpak { app_id } => {
            let mut cmd = Command::new("flatpak");
            cmd.arg("run").arg(app_id);
            cmd.args(def.flatpak_launch_args);
            cmd.arg(rom_path);
            cmd
        }
    };

    spawn(cmd, &def.emulator)
}

/// Spawn a prepared command, mapping failures to a readable message.
///
/// The emulator runs independently; we reap it on a detached thread so that when
/// the user closes it, the process doesn't linger as a zombie.
fn spawn(mut cmd: Command, emulator: &str) -> Result<(), String> {
    match cmd.spawn() {
        Ok(mut child) => {
            std::thread::spawn(move || {
                let _ = child.wait();
            });
            Ok(())
        }
        Err(e) => Err(format!("failed to launch {emulator}: {e}")),
    }
}
