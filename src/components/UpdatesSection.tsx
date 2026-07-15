import { useEffect, useState } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";

const RELEASES_URL = "https://github.com/pl0xuee/ConsoleVault/releases/latest";

// Update flow states. Like StreamHub, checking is manual and nothing is
// downloaded until the user clicks again once an update is found.
type Status =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "available"; update: Update }
  | { kind: "uptodate" }
  | { kind: "downloading"; percent: number | null }
  | { kind: "error"; message: string };

export function UpdatesSection() {
  const [version, setVersion] = useState("");
  const [status, setStatus] = useState<Status>({ kind: "idle" });

  useEffect(() => {
    getVersion().then(setVersion).catch(() => {});
  }, []);

  // Check the release endpoint. If the updater can't run (dev build, or the
  // endpoint is unreachable), fall back to opening the download page.
  const runCheck = async () => {
    setStatus({ kind: "checking" });
    try {
      const update = await check();
      setStatus(update ? { kind: "available", update } : { kind: "uptodate" });
    } catch {
      setStatus({
        kind: "error",
        message: "Couldn't check automatically — opened the releases page instead.",
      });
      await openUrl(RELEASES_URL).catch(() => {});
    }
  };

  // Download the new AppImage in place (reporting progress), then relaunch.
  const install = async (update: Update) => {
    let total = 0;
    let done = 0;
    setStatus({ kind: "downloading", percent: null });
    try {
      await update.downloadAndInstall((ev) => {
        if (ev.event === "Started") {
          total = ev.data.contentLength ?? 0;
        } else if (ev.event === "Progress") {
          done += ev.data.chunkLength;
          setStatus({
            kind: "downloading",
            percent: total ? Math.round((done / total) * 100) : null,
          });
        }
      });
      await relaunch();
    } catch (e) {
      setStatus({ kind: "error", message: String(e) });
    }
  };

  const available = status.kind === "available";
  const busy = status.kind === "checking" || status.kind === "downloading";

  const buttonLabel = () => {
    switch (status.kind) {
      case "checking":
        return "Checking…";
      case "available":
        return `Update to v${status.update.version}`;
      case "downloading":
        return status.percent === null ? "Downloading…" : `Downloading ${status.percent}%`;
      default:
        return "Check for updates";
    }
  };

  const subText = () => {
    switch (status.kind) {
      case "checking":
        return "Checking…";
      case "available":
        return `v${status.update.version} is available. Installing restarts ConsoleVault.`;
      case "uptodate":
        return `ConsoleVault v${version} is the latest version.`;
      case "downloading":
        return "Downloading the update — ConsoleVault will restart when it's done.";
      case "error":
        return status.message;
      default:
        return "Checking is manual — nothing is downloaded until you say so.";
    }
  };

  return (
    <div className="section">
      <h2>Updates</h2>
      <div className="update-row">
        <div className="update-text">
          <div className="update-title">
            ConsoleVault{version ? ` v${version}` : ""}
          </div>
          <div className="muted update-sub">{subText()}</div>
        </div>
        <button
          className={available ? "primary has-update" : ""}
          disabled={busy}
          onClick={() => (available ? install(status.update) : runCheck())}
        >
          {buttonLabel()}
        </button>
      </div>
    </div>
  );
}
