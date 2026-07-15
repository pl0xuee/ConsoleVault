import { useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  detectEmulators,
  installEmulator,
  installPlan,
  type EmulatorStatus,
  type InstallMethod,
  type InstallPlan,
} from "../lib/ipc";

const METHOD_LABEL: Record<InstallMethod, string> = {
  pacman: "Official repo (pacman)",
  flatpak: "Flathub (flatpak)",
  source: "Build from source",
};

export function EmulatorSetup() {
  const [statuses, setStatuses] = useState<EmulatorStatus[]>([]);
  const [loading, setLoading] = useState(true);
  const [target, setTarget] = useState<EmulatorStatus | null>(null);

  const refresh = async () => {
    setLoading(true);
    try {
      setStatuses(await detectEmulators());
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    refresh();
  }, []);

  return (
    <div>
      <h1>Emulators</h1>
      <p className="subtitle">
        Installed from official Arch/CachyOS repos, Flathub, or built from the project's official
        source — never the AUR.
      </p>
      <div className="toolbar">
        <button onClick={refresh} disabled={loading}>
          {loading ? "Checking…" : "Re-check"}
        </button>
      </div>

      {statuses.map((s) => (
        <div className="emu-row" key={s.system}>
          <div className="meta">
            <div className="name">
              {s.emulator} <span className="muted">· {s.system_label}</span>
            </div>
            <div className="sub">
              {s.installed
                ? describeInvocation(s)
                : "Not installed"}
            </div>
          </div>
          {s.bios_required && <span className="pill bios">BIOS required</span>}
          {s.installed ? (
            <span className="pill ok">Installed</span>
          ) : (
            <>
              <span className="pill missing">Missing</span>
              <button className="primary" onClick={() => setTarget(s)}>
                Install
              </button>
            </>
          )}
        </div>
      ))}

      {target && (
        <InstallModal
          status={target}
          onClose={() => setTarget(null)}
          onDone={async () => {
            setTarget(null);
            await refresh();
          }}
        />
      )}
    </div>
  );
}

function describeInvocation(s: EmulatorStatus): string {
  if (!s.invocation) return "Installed";
  return s.invocation.kind === "native"
    ? `Native binary: ${s.invocation.program}`
    : `Flatpak: ${s.invocation.app_id}`;
}

function InstallModal({
  status,
  onClose,
  onDone,
}: {
  status: EmulatorStatus;
  onClose: () => void;
  onDone: () => void;
}) {
  const [plan, setPlan] = useState<InstallPlan | null>(null);
  const [method, setMethod] = useState<InstallMethod | null>(null);
  const [running, setRunning] = useState(false);
  const [log, setLog] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  const consoleRef = useRef<HTMLDivElement>(null);

  // Load the preview plan for the selected (or default) method.
  useEffect(() => {
    let alive = true;
    installPlan(status.system, method ?? undefined)
      .then((p) => {
        if (!alive) return;
        setPlan(p);
        if (method === null) setMethod(p.method);
      })
      .catch((e) => alive && setError(String(e)));
    return () => {
      alive = false;
    };
  }, [status.system, method]);

  // Stream install output for this system.
  useEffect(() => {
    const un = listen<{ system: string; line: string }>("install-log", (ev) => {
      if (ev.payload.system === status.system) {
        setLog((prev) => [...prev, ev.payload.line]);
      }
    });
    return () => {
      un.then((f) => f());
    };
  }, [status.system]);

  useEffect(() => {
    consoleRef.current?.scrollTo(0, consoleRef.current.scrollHeight);
  }, [log]);

  const run = async () => {
    if (!method) return;
    setRunning(true);
    setError(null);
    setLog([]);
    try {
      await installEmulator(status.system, method);
      onDone();
    } catch (e) {
      setError(String(e));
    } finally {
      setRunning(false);
    }
  };

  const methods = useMemo(() => plan?.available_methods ?? [], [plan]);

  return (
    <div className="modal-backdrop" onClick={running ? undefined : onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <h2>Install {status.emulator}</h2>
        <p className="muted">Choose a source, then review the exact commands before running.</p>

        <div className="tabs" style={{ margin: "12px 0" }}>
          {methods.map((m) => (
            <button
              key={m}
              className={`tab ${method === m ? "active" : ""}`}
              disabled={running}
              onClick={() => setMethod(m)}
            >
              {METHOD_LABEL[m]}
            </button>
          ))}
        </div>

        {plan && <div className="cmds">{plan.commands.join("\n")}</div>}

        {status.bios_required && (
          <p className="muted">
            Note: {status.emulator} needs BIOS/firmware you provide yourself — set it up in the
            emulator on first launch. This app never downloads BIOS.
          </p>
        )}

        {log.length > 0 && (
          <div className="console" ref={consoleRef}>
            {log.join("\n")}
          </div>
        )}
        {error && <p style={{ color: "var(--danger)" }}>{error}</p>}

        <div className="row end" style={{ marginTop: 16 }}>
          <button onClick={onClose} disabled={running}>
            Close
          </button>
          <button className="primary" onClick={run} disabled={running || !method}>
            {running ? "Installing…" : "Run install"}
          </button>
        </div>
      </div>
    </div>
  );
}
