import { useEffect, useState, type FormEvent } from "react";

import { useAppCore, useLeagueAssets } from "../state/AppStateProvider";
import { ProfileOverview, PlaystyleSection, StatePanel } from "../components/profile/ProfileOverview";
import { searchPlayerProfile } from "../backend/leagueClient";
import { initials } from "../utils/formatting";
import type { PlayerProfileSnapshot } from "../backend/types";

// Wider window so the searched player's per-champion W/L records and playstyle
// fingerprint are meaningful (matches the self Profile's record window).
const SEARCH_MATCH_WINDOW = 50;

const RECENT_KEY = "lol.playerSearch.recent";
const RECENT_MAX = 8;

type RecentSearch = { query: string; displayName: string; profileIconId: number | null };

function loadRecentSearches(): RecentSearch[] {
  try {
    const raw = localStorage.getItem(RECENT_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (entry): entry is RecentSearch =>
        Boolean(entry) && typeof entry.query === "string" && typeof entry.displayName === "string",
    );
  } catch {
    return [];
  }
}

export function PlayerSearch() {
  const { effectiveLanguage, t } = useAppCore();
  const { leagueImages, loadLeagueChampionIcon, loadLeagueProfileIcon, loadLeagueRankTierIcon } = useLeagueAssets();

  const [query, setQuery] = useState("");
  const [submittedQuery, setSubmittedQuery] = useState("");
  const [result, setResult] = useState<PlayerProfileSnapshot | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(false);
  const [recent, setRecent] = useState<RecentSearch[]>(() => loadRecentSearches());
  const [showRecent, setShowRecent] = useState(false);

  // Preload avatars for the recent list so the dropdown shows icons immediately.
  useEffect(() => {
    for (const entry of recent) {
      if (entry.profileIconId) {
        void loadLeagueProfileIcon(entry.profileIconId);
      }
    }
  }, [loadLeagueProfileIcon, recent]);

  const runSearch = async (raw: string) => {
    const trimmed = raw.trim();
    if (!trimmed || loading) {
      return;
    }
    setQuery(trimmed);
    setShowRecent(false);
    setLoading(true);
    setError(false);
    setSubmittedQuery(trimmed);
    try {
      const snapshot = await searchPlayerProfile({ query: trimmed, matchLimit: SEARCH_MATCH_WINDOW });
      setResult(snapshot);
      if (snapshot.found && snapshot.summoner) {
        const entry: RecentSearch = {
          query: trimmed,
          displayName: snapshot.summoner.displayName,
          profileIconId: snapshot.summoner.profileIconId,
        };
        setRecent((prev) => {
          const next = [
            entry,
            ...prev.filter((e) => e.displayName.toLowerCase() !== entry.displayName.toLowerCase()),
          ].slice(0, RECENT_MAX);
          try {
            localStorage.setItem(RECENT_KEY, JSON.stringify(next));
          } catch {
            // Persistence is best-effort; the in-memory list still works.
          }
          return next;
        });
      }
    } catch {
      setResult(null);
      setError(true);
    } finally {
      setLoading(false);
    }
  };

  const onSubmit = (event: FormEvent) => {
    event.preventDefault();
    void runSearch(query);
  };

  const summoner = result?.found ? result.summoner : null;

  return (
    <main className="min-h-0 flex-1 overflow-auto px-8 py-7">
      <div className="mx-auto flex w-full max-w-6xl flex-col gap-7">
        <header className="flex flex-col gap-4">
          <div>
            <p className="text-sm font-medium uppercase tracking-wide text-rose-700">{t("search.eyebrow")}</p>
            <h1 className="mt-2 text-3xl font-semibold text-zinc-950 dark:text-zinc-50">{t("search.title")}</h1>
          </div>
          <div className="relative w-full max-w-xl">
            <form className="flex items-center gap-2" onSubmit={onSubmit}>
              <input
                aria-label={t("search.title")}
                className="h-10 flex-1 rounded-md border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-950 px-3 text-sm text-zinc-900 dark:text-zinc-100 outline-none transition focus:border-rose-500"
                onBlur={() => window.setTimeout(() => setShowRecent(false), 150)}
                onChange={(event) => setQuery(event.target.value)}
                onFocus={() => setShowRecent(true)}
                placeholder={t("search.placeholder")}
                type="search"
                value={query}
              />
              <button
                className="inline-flex h-10 items-center rounded-md bg-rose-700 px-4 text-sm font-medium text-white transition hover:bg-rose-800 disabled:cursor-not-allowed disabled:opacity-60"
                disabled={loading || query.trim().length === 0}
                type="submit"
              >
                {loading ? t("search.searching") : t("search.button")}
              </button>
            </form>

            {showRecent && recent.length > 0 && (
              <div className="absolute z-20 mt-1 w-full overflow-hidden rounded-md border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 shadow-lg">
                <p className="px-3 pt-2 pb-1 text-xs font-medium uppercase tracking-wide text-zinc-400 dark:text-zinc-500">
                  {t("search.recent")}
                </p>
                <ul>
                  {recent.map((entry) => (
                    <li key={entry.displayName}>
                      <button
                        className="flex w-full items-center gap-3 px-3 py-2 text-left transition hover:bg-zinc-100 dark:hover:bg-zinc-800"
                        // Prevent the input blur from firing before the click registers.
                        onMouseDown={(event) => event.preventDefault()}
                        onClick={() => void runSearch(entry.query)}
                        type="button"
                      >
                        <RecentAvatar
                          name={entry.displayName}
                          src={entry.profileIconId ? leagueImages.profileIcons[entry.profileIconId] : undefined}
                        />
                        <span className="truncate text-sm text-zinc-800 dark:text-zinc-200">{entry.displayName}</span>
                      </button>
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        </header>

        {loading && <StatePanel title={t("search.searching")} body={t("search.placeholder")} />}

        {!loading && error && <StatePanel title={t("search.error")} body={t("search.errorBody")} />}

        {!loading && !error && result && !result.found && (
          <StatePanel
            title={t("search.notFound")}
            body={t("search.notFoundBody").replace("{query}", submittedQuery)}
          />
        )}

        {!loading && !error && !result && <StatePanel title={t("search.prompt")} body={t("search.promptBody")} />}

        {!loading && !error && result?.found && summoner && (
          <ProfileOverview
            championRecords={result.championRecords}
            effectiveLanguage={effectiveLanguage}
            leagueImages={leagueImages}
            loadLeagueChampionIcon={loadLeagueChampionIcon}
            loadLeagueProfileIcon={loadLeagueProfileIcon}
            loadLeagueRankTierIcon={loadLeagueRankTierIcon}
            playstyleSlot={<PlaystyleSection profile={result.playstyle} t={t} />}
            recentMatches={result.recentMatches}
            rankedQueues={result.rankedQueues}
            recentPerformance={result.recentPerformance}
            refreshedAt={result.refreshedAt}
            statusPhase={result.status.phase}
            summoner={summoner}
            t={t}
          />
        )}
      </div>
    </main>
  );
}

function RecentAvatar({ name, src }: { name: string; src: string | undefined }) {
  if (src) {
    return <img alt="" className="h-7 w-7 shrink-0 rounded-full border border-zinc-200 dark:border-zinc-700 object-cover" src={src} />;
  }
  return (
    <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full border border-zinc-200 dark:border-zinc-700 bg-zinc-100 dark:bg-zinc-800 text-xs font-semibold text-zinc-500 dark:text-zinc-400">
      {initials(name)}
    </div>
  );
}
