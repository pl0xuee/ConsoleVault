import { useEffect, useState } from "react";
import { getConfig, saveConfig, type AppConfig } from "./lib/ipc";
import { Library } from "./views/Library";
import { EmulatorSetup } from "./views/EmulatorSetup";
import { Settings } from "./views/Settings";

type Tab = "library" | "emulators" | "settings";

const NAV: { id: Tab; label: string }[] = [
  { id: "library", label: "Library" },
  { id: "emulators", label: "Emulators" },
  { id: "settings", label: "Settings" },
];

function App() {
  const [tab, setTab] = useState<Tab>("library");
  const [config, setConfig] = useState<AppConfig | null>(null);

  useEffect(() => {
    getConfig().then(setConfig);
  }, []);

  const showWelcome = config !== null && !config.setup_complete;

  const dismissWelcome = async () => {
    if (!config) return;
    const next = { ...config, setup_complete: true };
    setConfig(next);
    await saveConfig(next);
  };

  return (
    <div className="app">
      <nav className="sidebar">
        <div className="brand">🎮 ConsoleVault</div>
        {NAV.map((n) => (
          <button
            key={n.id}
            className={`nav-item ${tab === n.id ? "active" : ""}`}
            onClick={() => setTab(n.id)}
          >
            {n.label}
          </button>
        ))}
      </nav>

      <main className="content">
        {tab === "library" && <Library />}
        {tab === "emulators" && <EmulatorSetup />}
        {tab === "settings" && <Settings onSaved={setConfig} />}
      </main>

      {showWelcome && (
        <div className="modal-backdrop">
          <div className="modal">
            <h2>Welcome to ConsoleVault</h2>
            <p className="muted">
              A launcher for your own physical ROM collection across SNES, N64, PS1, PS2, and PS3.
              It never downloads ROMs or BIOS — only emulators and box art. Two quick steps:
            </p>
            <ol className="muted">
              <li>
                <strong>Settings</strong> → add the folder(s) where your ROMs live.
              </li>
              <li>
                <strong>Emulators</strong> → install any that are missing.
              </li>
            </ol>
            <div className="row end" style={{ marginTop: 16 }}>
              <button
                className="primary"
                onClick={async () => {
                  await dismissWelcome();
                  setTab("settings");
                }}
              >
                Get started
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
