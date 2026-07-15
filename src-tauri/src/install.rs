//! Install emulators from allowed sources: official pacman repos, Flathub, or a
//! build from the project's official upstream GitHub. The AUR is never used.

use crate::registry::{self, EmulatorDef};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use tauri::{AppHandle, Emitter, Manager};

/// Which install source to use for an emulator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstallMethod {
    /// Official Arch/CachyOS repo via pacman.
    Pacman,
    /// Flathub via flatpak (per-user install).
    Flatpak,
    /// Build from the official upstream GitHub source.
    Source,
}

/// A previewable install plan: the method and the exact shell commands.
#[derive(Debug, Clone, Serialize)]
pub struct InstallPlan {
    pub system: String,
    pub emulator: String,
    pub method: InstallMethod,
    /// Human-readable commands, shown to the user before they confirm.
    pub commands: Vec<String>,
    /// Available alternate methods the user could pick instead.
    pub available_methods: Vec<InstallMethod>,
}

/// Methods available for an emulator, in preference order.
fn available_methods(def: &EmulatorDef) -> Vec<InstallMethod> {
    let mut methods = Vec::new();
    if def.pacman_pkg.is_some() {
        methods.push(InstallMethod::Pacman);
    }
    if def.flatpak_id.is_some() {
        methods.push(InstallMethod::Flatpak);
    }
    methods.push(InstallMethod::Source); // always possible
    methods
}

/// The default (preferred) method for an emulator.
fn preferred_method(def: &EmulatorDef) -> InstallMethod {
    available_methods(def)[0]
}

/// Build a preview plan for `system`, defaulting to its preferred method (or the
/// caller's chosen `method` when provided).
pub fn plan(system: &str, method: Option<InstallMethod>) -> Result<InstallPlan, String> {
    let def = registry::find(system).ok_or_else(|| format!("unknown system: {system}"))?;
    let available = available_methods(def);
    let method = method.unwrap_or_else(|| preferred_method(def));
    if !available.contains(&method) {
        return Err(format!("{} is unavailable for {}", label(method), def.emulator));
    }

    let commands = match method {
        InstallMethod::Pacman => vec![format!(
            "pkexec pacman -S --needed --noconfirm {}",
            def.pacman_pkg.unwrap()
        )],
        InstallMethod::Flatpak => vec![
            "flatpak remote-add --if-not-exists --user flathub \
             https://flathub.org/repo/flathub.flatpakrepo"
                .to_string(),
            format!("flatpak install --user -y flathub {}", def.flatpak_id.unwrap()),
        ],
        InstallMethod::Source => def.source_build.steps.iter().map(|s| s.to_string()).collect(),
    };

    Ok(InstallPlan {
        system: def.system.to_string(),
        emulator: def.emulator.to_string(),
        method,
        commands,
        available_methods: available,
    })
}

/// Run the install for `system` using `method`, streaming output as `install-log`
/// events. Returns Ok when all steps exit 0.
pub fn install(app: &AppHandle, system: &str, method: InstallMethod) -> Result<(), String> {
    let def = registry::find(system).ok_or_else(|| format!("unknown system: {system}"))?;
    emit(app, system, &format!("== Installing {} via {} ==", def.emulator, label(method)));

    match method {
        InstallMethod::Pacman => {
            let pkg = def.pacman_pkg.ok_or("no pacman package for this emulator")?;
            run(app, system, "pkexec", &["pacman", "-S", "--needed", "--noconfirm", pkg], None)
        }
        InstallMethod::Flatpak => {
            ensure_flatpak(app, system)?;
            let id = def.flatpak_id.ok_or("no flatpak id for this emulator")?;
            run(
                app,
                system,
                "flatpak",
                &["remote-add", "--if-not-exists", "--user", "flathub",
                  "https://flathub.org/repo/flathub.flatpakrepo"],
                None,
            )?;
            run(app, system, "flatpak", &["install", "--user", "-y", "flathub", id], None)
        }
        InstallMethod::Source => run_source_build(app, def),
    }
}

/// Fail early with a clear message if flatpak itself is missing.
fn ensure_flatpak(app: &AppHandle, system: &str) -> Result<(), String> {
    if which("flatpak") {
        return Ok(());
    }
    let msg = "flatpak is not installed. Install it once with: sudo pacman -S flatpak";
    emit(app, system, msg);
    Err(msg.to_string())
}

/// Clone and build an emulator from its official source in a per-system cache dir.
fn run_source_build(app: &AppHandle, def: &EmulatorDef) -> Result<(), String> {
    let cache = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("no cache dir: {e}"))?
        .join("build")
        .join(def.system);
    // Start from a clean directory so a re-run's `git clone .` doesn't fail on an
    // existing (possibly half-built) checkout.
    let _ = std::fs::remove_dir_all(&cache);
    std::fs::create_dir_all(&cache).map_err(|e| format!("mkdir build dir: {e}"))?;
    emit(app, def.system, &format!("Build dir (clean): {}", cache.display()));
    emit(app, def.system, def.source_build.summary);

    for step in def.source_build.steps {
        emit(app, def.system, &format!("$ {step}"));
        // Steps are documented `program arg arg ...` lines; run via the shell so
        // that `cmake --build build` etc. behave exactly as documented upstream.
        let status = stream_command(
            app,
            def.system,
            Command::new("sh").arg("-c").arg(step).current_dir(&cache),
        )?;
        if !status {
            return Err(format!("source build step failed: {step}"));
        }
    }
    Ok(())
}

/// Run `program args...` streaming output; error if it exits non-zero.
fn run(
    app: &AppHandle,
    system: &str,
    program: &str,
    args: &[&str],
    cwd: Option<&std::path::Path>,
) -> Result<(), String> {
    emit(app, system, &format!("$ {program} {}", args.join(" ")));
    let mut cmd = Command::new(program);
    cmd.args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    if stream_command(app, system, &mut cmd)? {
        Ok(())
    } else {
        Err(format!("`{program}` exited with a non-zero status"))
    }
}

/// Spawn `cmd`, forward stdout+stderr line-by-line as events, return success.
fn stream_command(app: &AppHandle, system: &str, cmd: &mut Command) -> Result<bool, String> {
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to start process: {e}"))?;

    // Read stderr on a background thread so a full pipe can't deadlock stdout.
    let stderr_handle = child.stderr.take().map(|stderr| {
        let app = app.clone();
        let system = system.to_string();
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                emit(&app, &system, &line);
            }
        })
    });

    if let Some(stdout) = child.stdout.take() {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            emit(app, system, &line);
        }
    }
    if let Some(h) = stderr_handle {
        let _ = h.join();
    }

    let status = child.wait().map_err(|e| format!("process wait failed: {e}"))?;
    Ok(status.success())
}

/// Emit one log line to the UI, tagged with its system.
fn emit(app: &AppHandle, system: &str, line: &str) {
    let _ = app.emit("install-log", InstallLog { system: system.to_string(), line: line.to_string() });
}

/// Payload for the `install-log` event.
#[derive(Clone, Serialize)]
struct InstallLog {
    system: String,
    line: String,
}

/// Is `program` on PATH?
fn which(program: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|d| d.join(program).is_file()))
        .unwrap_or(false)
}

/// Display label for a method.
fn label(method: InstallMethod) -> &'static str {
    match method {
        InstallMethod::Pacman => "official repo (pacman)",
        InstallMethod::Flatpak => "Flathub (flatpak)",
        InstallMethod::Source => "source build",
    }
}
