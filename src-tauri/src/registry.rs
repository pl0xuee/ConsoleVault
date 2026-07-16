//! Static definitions for every supported system and its emulator.
//!
//! One `EmulatorDef` per system describes how to detect, install, and launch the
//! emulator. Install sources follow the user's rule: prefer the official Arch/CachyOS
//! `pacman` package, else Flathub, with an opt-in build-from-source path using the
//! project's official upstream GitHub. The AUR is never used.

use serde::Serialize;

/// A shell recipe for building an emulator from its official upstream source.
#[derive(Debug, Clone, Serialize)]
pub struct SourceBuild {
    /// Official upstream git repository (used for display + `git clone`).
    pub repo_url: &'static str,
    /// Human-readable summary of what the build does (shown before running).
    pub summary: &'static str,
    /// Documented build steps, run in order from a fresh clone directory.
    pub steps: &'static [&'static str],
}

/// Everything the app needs to know about one system's emulator.
#[derive(Debug, Clone, Serialize)]
pub struct EmulatorDef {
    /// Stable system key, e.g. "snes", "n64", "ps1", "ps2", "ps3".
    pub system: &'static str,
    /// Display name of the system, e.g. "Super Nintendo".
    pub system_label: &'static str,
    /// Emulator name, e.g. "Snes9x".
    pub emulator: &'static str,
    /// libretro-thumbnails repo name for box art lookups.
    pub thumbnail_repo: &'static str,
    /// Official Arch/CachyOS repo package name, if the emulator is packaged there.
    pub pacman_pkg: Option<&'static str>,
    /// Flathub application id, if distributed on Flathub.
    pub flatpak_id: Option<&'static str>,
    /// Native binary names to probe on PATH (covers pacman/native/source installs).
    pub binaries: &'static [&'static str],
    /// Extra args inserted before the ROM path when launching a native binary.
    pub launch_args: &'static [&'static str],
    /// Extra args inserted before the ROM path when launching via `flatpak run`.
    pub flatpak_launch_args: &'static [&'static str],
    /// Accepted file extensions (lowercase, no dot). Empty => folder-based (PS3).
    pub extensions: &'static [&'static str],
    /// True when the system needs user-provided BIOS/firmware.
    pub bios_required: bool,
    /// Build-from-source recipe (opt-in install method).
    pub source_build: SourceBuild,
}

/// The full set of supported emulators, in display order.
pub const EMULATORS: &[EmulatorDef] = &[
    EmulatorDef {
        system: "snes",
        system_label: "Super Nintendo",
        emulator: "Snes9x",
        thumbnail_repo: "Nintendo - Super Nintendo Entertainment System",
        pacman_pkg: Some("snes9x-gtk"),
        flatpak_id: Some("com.snes9x.Snes9x"),
        binaries: &["snes9x-gtk", "snes9x"],
        launch_args: &[],
        flatpak_launch_args: &[],
        extensions: &["sfc", "smc", "swc", "fig"],
        bios_required: false,
        source_build: SourceBuild {
            repo_url: "https://github.com/snes9xgit/snes9x",
            summary: "Build the GTK front-end of Snes9x with CMake.",
            steps: &[
                "git clone --depth 1 https://github.com/snes9xgit/snes9x .",
                "cmake -S gtk -B build -DCMAKE_BUILD_TYPE=Release",
                "cmake --build build --parallel",
            ],
        },
    },
    EmulatorDef {
        system: "n64",
        system_label: "Nintendo 64",
        // simple64 was renamed to gopher64 (its Flathub app is EOL-rebased to it),
        // so that's what we install and detect. simple64-gui/mupen64plus stay as
        // fallback binaries for machines that still have the older tools on PATH.
        emulator: "gopher64",
        thumbnail_repo: "Nintendo - Nintendo 64",
        // gopher64 is not in the official repos; mupen64plus is the packaged alternative.
        pacman_pkg: None,
        flatpak_id: Some("io.github.gopher64.gopher64"),
        binaries: &["gopher64", "simple64-gui", "mupen64plus"],
        launch_args: &[],
        flatpak_launch_args: &[],
        extensions: &["z64", "n64", "v64"],
        bios_required: false,
        source_build: SourceBuild {
            repo_url: "https://github.com/gopher64/gopher64",
            summary: "Build gopher64 (Rust) with Cargo. Needs the Rust toolchain and SDL3.",
            steps: &[
                "git clone --depth 1 --recursive https://github.com/gopher64/gopher64.git .",
                "cargo build --release",
            ],
        },
    },
    EmulatorDef {
        system: "ps1",
        system_label: "PlayStation",
        emulator: "DuckStation",
        thumbnail_repo: "Sony - PlayStation",
        pacman_pkg: None,
        flatpak_id: Some("org.duckstation.DuckStation"),
        binaries: &["duckstation-qt-x11", "duckstation-qt", "duckstation"],
        // DuckStation takes the ROM as a positional arg after `--`.
        launch_args: &["--"],
        flatpak_launch_args: &["--"],
        // pbp/m3u/ecm are PS1-only; bin/cue/chd overlap with PS2 (disambiguated in scan.rs).
        extensions: &["cue", "bin", "chd", "pbp", "m3u", "ecm"],
        bios_required: true,
        source_build: SourceBuild {
            repo_url: "https://github.com/stenzek/duckstation",
            summary: "Build DuckStation with CMake (Qt front-end).",
            steps: &[
                "git clone --depth 1 https://github.com/stenzek/duckstation .",
                "cmake -S . -B build -DCMAKE_BUILD_TYPE=Release",
                "cmake --build build --parallel",
            ],
        },
    },
    EmulatorDef {
        system: "ps2",
        system_label: "PlayStation 2",
        emulator: "PCSX2",
        thumbnail_repo: "Sony - PlayStation 2",
        pacman_pkg: Some("pcsx2"),
        flatpak_id: Some("net.pcsx2.PCSX2"),
        binaries: &["pcsx2-qt", "pcsx2"],
        launch_args: &[],
        flatpak_launch_args: &[],
        extensions: &["iso", "chd", "bin", "cue", "mdf", "gz"],
        bios_required: true,
        source_build: SourceBuild {
            repo_url: "https://github.com/PCSX2/pcsx2",
            summary: "Build PCSX2 with CMake (heavy; many multimedia deps required).",
            steps: &[
                "git clone --depth 1 https://github.com/PCSX2/pcsx2 .",
                "cmake -S . -B build -DCMAKE_BUILD_TYPE=Release -DPACKAGE_MODE=OFF",
                "cmake --build build --parallel",
            ],
        },
    },
    EmulatorDef {
        system: "ps3",
        system_label: "PlayStation 3",
        emulator: "RPCS3",
        thumbnail_repo: "Sony - PlayStation 3",
        pacman_pkg: None,
        flatpak_id: Some("net.rpcs3.RPCS3"),
        binaries: &["rpcs3"],
        launch_args: &[],
        flatpak_launch_args: &[],
        // PS3 titles are folders (see scan.rs); no file extensions.
        extensions: &[],
        bios_required: true,
        source_build: SourceBuild {
            repo_url: "https://github.com/RPCS3/rpcs3",
            summary: "Build RPCS3 with CMake (very heavy; long compile, many deps).",
            steps: &[
                "git clone --depth 1 --recursive https://github.com/RPCS3/rpcs3 .",
                "cmake -S . -B build -DCMAKE_BUILD_TYPE=Release",
                "cmake --build build --parallel",
            ],
        },
    },
];

/// Look up an emulator definition by system key.
pub fn find(system: &str) -> Option<&'static EmulatorDef> {
    EMULATORS.iter().find(|e| e.system == system)
}

/// Classify a file extension (lowercase, no dot) to a system key, if any.
pub fn system_for_extension(ext: &str) -> Option<&'static str> {
    EMULATORS
        .iter()
        .find(|e| e.extensions.contains(&ext))
        .map(|e| e.system)
}
