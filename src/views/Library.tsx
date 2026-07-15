import { useEffect, useMemo, useState } from "react";
import { getBoxart, launchGame, scanLibrary, SYSTEM_LABELS, type Game } from "../lib/ipc";

const SYSTEMS = ["snes", "n64", "ps1", "ps2", "ps3"];

export function Library() {
  const [games, setGames] = useState<Game[]>([]);
  const [filter, setFilter] = useState<string>("all");
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const rescan = async () => {
    setLoading(true);
    setError(null);
    try {
      setGames(await scanLibrary());
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    rescan();
  }, []);

  const visible = useMemo(() => {
    const q = query.trim().toLowerCase();
    return games.filter(
      (g) =>
        (filter === "all" || g.system === filter) &&
        (q === "" || g.title.toLowerCase().includes(q)),
    );
  }, [games, filter, query]);

  const play = async (g: Game) => {
    try {
      await launchGame(g.system, g.path);
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div>
      <h1>Library</h1>
      <p className="subtitle">{games.length} games from your ROM folders.</p>

      <div className="toolbar">
        <div className="tabs">
          <button className={`tab ${filter === "all" ? "active" : ""}`} onClick={() => setFilter("all")}>
            All
          </button>
          {SYSTEMS.map((s) => (
            <button
              key={s}
              className={`tab ${filter === s ? "active" : ""}`}
              onClick={() => setFilter(s)}
            >
              {SYSTEM_LABELS[s]}
            </button>
          ))}
        </div>
        <div className="spacer" />
        <input
          type="search"
          placeholder="Search…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <button onClick={rescan} disabled={loading}>
          {loading ? "Scanning…" : "Rescan"}
        </button>
      </div>

      {error && <p style={{ color: "var(--danger)" }}>{error}</p>}

      {!loading && visible.length === 0 ? (
        <div className="empty">
          No games found. Add your ROM folders in <strong>Settings</strong>, then Rescan.
        </div>
      ) : (
        <div className="grid">
          {visible.map((g) => (
            <GameCard key={g.id} game={g} onPlay={() => play(g)} />
          ))}
        </div>
      )}
    </div>
  );
}

function GameCard({ game, onPlay }: { game: Game; onPlay: () => void }) {
  const [art, setArt] = useState<string | null>(null);
  const [tried, setTried] = useState(false);

  useEffect(() => {
    let alive = true;
    getBoxart(game.system, game.title)
      .then((url) => alive && setArt(url))
      .catch(() => {})
      .finally(() => alive && setTried(true));
    return () => {
      alive = false;
    };
  }, [game.system, game.title]);

  return (
    <div className="card" role="button" tabIndex={0} onClick={onPlay}
      onKeyDown={(e) => (e.key === "Enter" || e.key === " ") && onPlay()}>
      {art ? (
        <img className="art" src={art} alt={game.title} loading="lazy" />
      ) : (
        <div className="art placeholder">{tried ? game.title : "…"}</div>
      )}
      <div className="label">
        <div className="sys-badge">{SYSTEM_LABELS[game.system] ?? game.system}</div>
        {game.title}
      </div>
    </div>
  );
}
