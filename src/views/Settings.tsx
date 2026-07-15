import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { getConfig, saveConfig, SYSTEM_LABELS, type AppConfig } from "../lib/ipc";

const SYSTEMS = ["snes", "n64", "ps1", "ps2", "ps3"];

export function Settings({ onSaved }: { onSaved?: (c: AppConfig) => void }) {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    getConfig().then(setConfig);
  }, []);

  if (!config) return <p className="muted">Loading…</p>;

  const persist = async (next: AppConfig) => {
    setConfig(next);
    await saveConfig(next);
    setSaved(true);
    onSaved?.(next);
    setTimeout(() => setSaved(false), 1500);
  };

  const addFolder = async () => {
    const picked = await open({ directory: true, multiple: false, title: "Select ROM folder" });
    if (typeof picked === "string" && !config.rom_folders.includes(picked)) {
      await persist({ ...config, rom_folders: [...config.rom_folders, picked] });
    }
  };

  const removeFolder = (folder: string) =>
    persist({ ...config, rom_folders: config.rom_folders.filter((f) => f !== folder) });

  const setOverride = (system: string, value: string) => {
    const overrides = { ...config.emulator_overrides };
    if (value.trim() === "") delete overrides[system];
    else overrides[system] = value;
    persist({ ...config, emulator_overrides: overrides });
  };

  return (
    <div>
      <h1>Settings {saved && <span className="pill ok">Saved</span>}</h1>
      <p className="subtitle">ROM folders are scanned locally. This app never downloads ROMs.</p>

      <div className="section">
        <h2>ROM folders</h2>
        {config.rom_folders.length === 0 && <p className="muted">No folders added yet.</p>}
        {config.rom_folders.map((f) => (
          <div className="folder-row" key={f}>
            <span className="path">{f}</span>
            <button onClick={() => removeFolder(f)}>Remove</button>
          </div>
        ))}
        <div style={{ marginTop: 12 }}>
          <button className="primary" onClick={addFolder}>
            Add folder…
          </button>
        </div>
      </div>

      <div className="section">
        <h2>Emulator command overrides</h2>
        <p className="muted" style={{ marginTop: 0 }}>
          Optional. Leave blank to use the auto-detected emulator. The ROM path is appended as the
          final argument (e.g. <code>flatpak run org.duckstation.DuckStation --</code>).
        </p>
        {SYSTEMS.map((s) => (
          <div className="override-row" key={s}>
            <label>{SYSTEM_LABELS[s]}</label>
            <input
              type="text"
              placeholder="auto-detect"
              value={config.emulator_overrides[s] ?? ""}
              onChange={(e) => setOverride(s, e.target.value)}
            />
          </div>
        ))}
      </div>
    </div>
  );
}
