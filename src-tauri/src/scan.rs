//! Walk the user's ROM folders and classify entries by system.

use crate::registry;
use serde::Serialize;
use std::path::Path;
use walkdir::WalkDir;

/// One game found on disk.
#[derive(Debug, Clone, Serialize)]
pub struct Game {
    /// System key ("snes", "n64", "ps1", "ps2", "ps3").
    pub system: String,
    /// Cleaned title derived from the filename/folder.
    pub title: String,
    /// Absolute path passed to the emulator (file, or EBOOT.BIN for PS3).
    pub path: String,
    /// Stable id (the path) for the UI to key on.
    pub id: String,
}

/// Disc-image extensions shared by PS1 and PS2; disambiguated via path hints.
const AMBIGUOUS: &[&str] = &["bin", "cue", "chd", "iso", "img", "mdf"];

/// Scan every configured root folder and return a de-duplicated game list.
pub fn scan(folders: &[String]) -> Vec<Game> {
    let mut games: Vec<Game> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for folder in folders {
        for entry in WalkDir::new(folder)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path();

            // PS3 titles are folders containing PS3_GAME/USRDIR/EBOOT.BIN.
            if entry.file_type().is_dir() {
                if let Some(game) = detect_ps3_folder(path) {
                    if seen.insert(game.path.clone()) {
                        games.push(game);
                    }
                }
                continue;
            }

            if !entry.file_type().is_file() {
                continue;
            }

            // Skip files living inside a PS3 game folder (EBOOT.BIN and internal
            // data files). The folder is already handled as one PS3 entry above;
            // without this guard a game's EBOOT.BIN / *.bin / *.iso would be
            // misclassified as spurious PS1/PS2 titles.
            if path.components().any(|c| {
                c.as_os_str()
                    .to_str()
                    .is_some_and(|s| s.eq_ignore_ascii_case("PS3_GAME"))
            }) {
                continue;
            }

            let Some(ext) = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_lowercase())
            else {
                continue;
            };

            let Some(system) = classify(path, &ext) else {
                continue;
            };

            // A .bin next to a .cue is loaded via the .cue; skip lone .bin tracks.
            if ext == "bin" && has_sibling_cue(path) {
                continue;
            }

            let path_str = path.to_string_lossy().to_string();
            if seen.insert(path_str.clone()) {
                games.push(Game {
                    system: system.to_string(),
                    title: clean_title(path.file_stem().and_then(|s| s.to_str()).unwrap_or("")),
                    id: path_str.clone(),
                    path: path_str,
                });
            }
        }
    }

    games.sort_by(|a, b| (a.system.as_str(), a.title.as_str()).cmp(&(b.system.as_str(), b.title.as_str())));
    games
}

/// Classify a file to a system, using path hints for ambiguous disc formats.
fn classify(path: &Path, ext: &str) -> Option<&'static str> {
    if AMBIGUOUS.contains(&ext) {
        // Prefer a system named in the path (e.g. ".../PS2/game.iso").
        if let Some(sys) = system_from_path_hint(path) {
            // Only accept a hint whose emulator actually accepts this extension.
            if registry::find(sys).is_some_and(|d| d.extensions.contains(&ext)) {
                return Some(sys);
            }
        }
        // No usable hint: fall back to the registry's first accepting system.
        return registry::system_for_extension(ext);
    }
    registry::system_for_extension(ext)
}

/// Guess a system from keywords anywhere in the path (case-insensitive).
fn system_from_path_hint(path: &Path) -> Option<&'static str> {
    let hay = path.to_string_lossy().to_ascii_lowercase();
    // Order matters: check the most specific tokens first.
    const HINTS: &[(&str, &str)] = &[
        ("ps3", "ps3"),
        ("playstation 3", "ps3"),
        ("ps2", "ps2"),
        ("playstation 2", "ps2"),
        ("ps1", "ps1"),
        ("psx", "ps1"),
        ("playstation", "ps1"),
        ("n64", "n64"),
        ("nintendo 64", "n64"),
        ("snes", "snes"),
    ];
    HINTS
        .iter()
        .find(|(token, _)| hay.contains(token))
        .map(|(_, sys)| *sys)
}

/// If `dir` is a PS3 game folder, return its Game (launched via EBOOT.BIN).
fn detect_ps3_folder(dir: &Path) -> Option<Game> {
    let eboot = dir.join("PS3_GAME").join("USRDIR").join("EBOOT.BIN");
    if !eboot.is_file() {
        return None;
    }
    let title = dir
        .file_name()
        .and_then(|s| s.to_str())
        .map(clean_title)
        .unwrap_or_else(|| "Unknown PS3 Game".to_string());
    let path_str = eboot.to_string_lossy().to_string();
    Some(Game {
        system: "ps3".to_string(),
        title,
        id: path_str.clone(),
        path: path_str,
    })
}

/// True if a `.cue` with the same stem sits beside this file.
fn has_sibling_cue(path: &Path) -> bool {
    path.with_extension("cue").is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn cleans_titles() {
        assert_eq!(clean_title("Super Mario 64 (USA)"), "Super Mario 64");
        assert_eq!(clean_title("Chrono_Trigger_[!]"), "Chrono Trigger");
        assert_eq!(clean_title("Final Fantasy VII (USA) (Disc 1)"), "Final Fantasy VII");
    }

    #[test]
    fn classifies_unambiguous_extensions() {
        assert_eq!(classify(Path::new("/roms/Mario.sfc"), "sfc"), Some("snes"));
        assert_eq!(classify(Path::new("/roms/Zelda.z64"), "z64"), Some("n64"));
        assert_eq!(classify(Path::new("/roms/Crash.pbp"), "pbp"), Some("ps1"));
    }

    #[test]
    fn path_hint_disambiguates_disc_images() {
        // A bare .iso defaults to PS2 (registry order), but a PS1 path wins the hint.
        assert_eq!(classify(Path::new("/games/PS2/GT4.iso"), "iso"), Some("ps2"));
        assert_eq!(classify(Path::new("/games/PSX/FF7.cue"), "cue"), Some("ps1"));
    }

    #[test]
    fn scans_folder_with_mixed_systems_and_ps3() {
        let dir = std::env::temp_dir().join("emuapp_scan_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("Super Mario World (USA).sfc"), b"x").unwrap();
        fs::write(dir.join("GoldenEye 007 (USA).z64"), b"x").unwrap();
        // PS3 game folder with EBOOT.BIN plus an internal data .bin file.
        let ps3 = dir.join("MyPS3Game").join("PS3_GAME").join("USRDIR");
        fs::create_dir_all(&ps3).unwrap();
        fs::write(ps3.join("EBOOT.BIN"), b"x").unwrap();
        fs::write(ps3.join("game_data.bin"), b"x").unwrap();

        let games = scan(&[dir.to_string_lossy().to_string()]);
        let by_sys: std::collections::HashMap<_, _> =
            games.iter().map(|g| (g.system.as_str(), g.title.as_str())).collect();

        assert_eq!(by_sys.get("snes"), Some(&"Super Mario World"));
        assert_eq!(by_sys.get("n64"), Some(&"GoldenEye 007"));
        assert_eq!(by_sys.get("ps3"), Some(&"MyPS3Game"));
        let ps3_game = games.iter().find(|g| g.system == "ps3").unwrap();
        assert!(ps3_game.path.ends_with("EBOOT.BIN"));
        // Internal PS3 files must NOT surface as PS1/PS2 titles.
        assert_eq!(games.iter().filter(|g| g.system == "ps1").count(), 0);
        assert_eq!(games.iter().filter(|g| g.system == "ps2").count(), 0);
        assert_eq!(games.iter().filter(|g| g.system == "ps3").count(), 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn skips_bin_track_next_to_cue() {
        let dir = std::env::temp_dir().join("emuapp_cue_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("PSX Game.cue"), b"x").unwrap();
        fs::write(dir.join("PSX Game.bin"), b"x").unwrap();

        let games = scan(&[dir.to_string_lossy().to_string()]);
        // Only the .cue should surface, not the paired .bin track.
        assert_eq!(games.len(), 1);
        assert!(games[0].path.ends_with(".cue"));

        let _ = fs::remove_dir_all(&dir);
    }
}

/// Turn a raw filename stem into a display title: strip region/dump tags,
/// replace separators with spaces, collapse whitespace.
fn clean_title(stem: &str) -> String {
    let mut out = String::with_capacity(stem.len());
    let mut depth: i32 = 0;
    for ch in stem.chars() {
        match ch {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = (depth - 1).max(0),
            '_' | '.' if depth == 0 => out.push(' '),
            c if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}
