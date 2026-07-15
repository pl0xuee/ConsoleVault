//! Fetch box art from the libretro-thumbnails project by title, cache it on disk,
//! and hand it to the UI as a data URL. No ROMs or BIOS are ever downloaded.

use crate::registry;
use base64_lite::to_base64;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Directory where cached thumbnails live: `<cache>/thumbnails/<system>/`.
fn cache_dir(app: &AppHandle, system: &str) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("no cache dir: {e}"))?
        .join("thumbnails")
        .join(system);
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir cache: {e}"))?;
    Ok(dir)
}

/// libretro-thumbnails replaces these characters with `_` in art filenames.
fn sanitize(title: &str) -> String {
    title
        .chars()
        .map(|c| match c {
            '&' | '*' | '/' | ':' | '`' | '<' | '>' | '?' | '\\' | '|' | '"' => '_',
            other => other,
        })
        .collect()
}

/// Return box art for `title` on `system` as a `data:image/png;base64,...` URL,
/// or `None` when no art is available. Results (hits and misses) are cached to
/// disk so repeat lookups work offline.
pub fn boxart(app: &AppHandle, system: &str, title: &str) -> Result<Option<String>, String> {
    let def = registry::find(system).ok_or_else(|| format!("unknown system: {system}"))?;
    let dir = cache_dir(app, system)?;
    let safe = sanitize(title);
    let png = dir.join(format!("{safe}.png"));
    let miss = dir.join(format!("{safe}.miss"));

    // Cached hit.
    if png.is_file() {
        let bytes = std::fs::read(&png).map_err(|e| format!("read cache: {e}"))?;
        return Ok(Some(to_data_url(&bytes)));
    }
    // Cached miss — don't re-hit the network.
    if miss.is_file() {
        return Ok(None);
    }

    // Fetch from libretro-thumbnails.
    let repo = def.thumbnail_repo.replace(' ', "_");
    let url = format!(
        "https://raw.githubusercontent.com/libretro-thumbnails/{}/master/Named_Boxarts/{}.png",
        urlencoding::encode(&repo),
        urlencoding::encode(&safe),
    );

    match fetch(&url)? {
        Some(bytes) => {
            std::fs::write(&png, &bytes).map_err(|e| format!("write cache: {e}"))?;
            Ok(Some(to_data_url(&bytes)))
        }
        None => {
            let _ = std::fs::write(&miss, b""); // remember the miss; ignore errors
            Ok(None)
        }
    }
}

/// GET `url`; `Ok(Some(bytes))` on 200, `Ok(None)` on 404, `Err` on other failures.
fn fetch(url: &str) -> Result<Option<Vec<u8>>, String> {
    let resp = reqwest::blocking::Client::builder()
        .user_agent("ConsoleVault/0.1")
        .build()
        .map_err(|e| format!("http client: {e}"))?
        .get(url)
        .send();

    match resp {
        Ok(r) if r.status().is_success() => {
            let bytes = r.bytes().map_err(|e| format!("read body: {e}"))?;
            Ok(Some(bytes.to_vec()))
        }
        Ok(r) if r.status().as_u16() == 404 => Ok(None),
        Ok(r) => Err(format!("box art request failed: HTTP {}", r.status())),
        // Offline / DNS failure: treat as "no art" rather than a hard error so the
        // library still renders.
        Err(_) => Ok(None),
    }
}

fn to_data_url(bytes: &[u8]) -> String {
    format!("data:image/png;base64,{}", to_base64(bytes))
}

/// Minimal standard-base64 encoder (no external crate needed).
mod base64_lite {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    pub fn to_base64(input: &[u8]) -> String {
        let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
        for chunk in input.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = *chunk.get(1).unwrap_or(&0) as u32;
            let b2 = *chunk.get(2).unwrap_or(&0) as u32;
            let n = (b0 << 16) | (b1 << 8) | b2;
            out.push(TABLE[(n >> 18 & 0x3f) as usize] as char);
            out.push(TABLE[(n >> 12 & 0x3f) as usize] as char);
            out.push(if chunk.len() > 1 { TABLE[(n >> 6 & 0x3f) as usize] as char } else { '=' });
            out.push(if chunk.len() > 2 { TABLE[(n & 0x3f) as usize] as char } else { '=' });
        }
        out
    }
}
