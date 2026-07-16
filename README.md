# ConsoleVault

A desktop launcher for your **own physical** ROM collection across SNES, N64, PS1, PS2, and PS3.
It scans folders you point it at, shows a box-art game library, and launches the right native
emulator per system. **It never downloads ROMs or BIOS files** — only emulator software and
cosmetic box art.

Built with Tauri 2 (Rust backend) + React/TypeScript, styled in a dark gunmetal-grey theme.

## Emulators

| System | Emulator    | Install source                                             |
|--------|-------------|------------------------------------------------------------|
| SNES   | Snes9x      | pacman `snes9x-gtk` · Flathub `com.snes9x.Snes9x`          |
| N64    | gopher64    | Flathub `io.github.gopher64.gopher64` (pacman `mupen64plus` alt) |
| PS1    | DuckStation | Flathub `org.duckstation.DuckStation`                      |
| PS2    | PCSX2       | pacman `pcsx2` · Flathub `net.pcsx2.PCSX2`                 |
| PS3    | RPCS3       | Flathub `net.rpcs3.RPCS3`                                  |

Install sources, in preference order: **official Arch/CachyOS repos (`pacman`) → Flathub
(`flatpak`) → build from the project's official upstream GitHub. The AUR is never used.** PS1/PS2/
PS3 need BIOS/firmware you provide yourself; the app flags this but never fetches it.

## Development

```bash
npm install
npm run tauri dev      # launch the app
npm run build          # typecheck + build the frontend
cd src-tauri && cargo test --lib   # backend unit tests (scan/classify logic)
```

## Releasing

Distribution is a Linux **AppImage** (single portable file; not sandboxed, so it can drive the
host's package managers and launch emulators — which is why this app isn't itself a Flatpak).

Releases are built by GitHub Actions (`.github/workflows/release.yml`) whenever a version tag is
pushed:

```bash
# bump version in package.json + src-tauri/tauri.conf.json first, then:
git tag v0.1.0
git push origin v0.1.0
```

CI builds the AppImage and attaches it to a **draft** GitHub Release — review the assets and notes,
then hit Publish. Users download the `.AppImage`, `chmod +x`, and run it.

To build one locally instead, install `patchelf` (e.g. `sudo pacman -S patchelf`) then:

```bash
npm run tauri build    # → src-tauri/target/release/bundle/appimage/*.AppImage
```

## How it works

- `src-tauri/src/registry.rs` — per-system emulator definitions (packages, flatpak ids, source
  builds, launch templates, extensions, BIOS flags).
- `detect.rs` / `install.rs` — detect installed emulators (PATH + flatpak) and install missing
  ones, streaming output to the UI via `install-log` events.
- `scan.rs` — walk ROM folders, classify by extension (with PS1/PS2 path-hint disambiguation) and
  detect PS3 game folders (`PS3_GAME/USRDIR/EBOOT.BIN`).
- `metadata.rs` — fetch box art from libretro-thumbnails by title, cached to disk for offline use.
- `launch.rs` — spawn the emulator (native binary or `flatpak run`) with the ROM path.
- `src/views/` — Library, Emulators, and Settings screens; `src/theme.css` holds the gunmetal
  design tokens.

First launch shows a short welcome: add your ROM folders in **Settings**, install any missing
emulators in **Emulators**, then browse and play from **Library**.
