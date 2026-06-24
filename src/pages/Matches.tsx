import { useEffect } from "react";

import { ChampionImage, StatePanel, ResultBadge, RefreshIcon } from "../components/common";
import { useAppCore, useLeagueAssets } from "../state/AppStateProvider";
import { openMatchRecapWindow } from "../windows/matchRecapWindow";
import type { RecentMatchSummary } from "../backend/types";
import { formatTimestamp, type T } from "../utils/formatting";

export function Matches() {
  const {
    leagueSelfSnapshot,
    isLeagueClientLoading,
    refreshLeagueClient,
    t,
  } = useAppCore();
  const { leagueImages, loadLeagueChampionIcon } = useLeagueAssets();
  const matches = leagueSelfSnapshot?.recentMatches ?? [];

  useEffect(() => {
    const championIds = new Set<number>();
    for (const match of matches) {
      if (match.championId) {
        championIds.add(match.championId);
      }
    }
    for (const championId of championIds) {
      void loadLeagueChampionIcon(championId);
    }
  }, [loadLeagueChampionIcon, matches]);

  return (
    <main className="min-h-0 flex-1 overflow-auto px-8 py-7">
      <div className="mx-auto flex w-full max-w-6xl flex-col gap-7">
          <header className="flex flex-wrap items-end justify-between gap-4">
            <div>
              <p className="text-sm font-medium uppercase tracking-wide text-rose-700">{t("matches.eyebrow")}</p>
              <h1 className="mt-2 text-3xl font-semibold text-zinc-950 dark:text-zinc-50">{t("matches.title")}</h1>
            </div>
            <button
              className="inline-flex h-10 items-center gap-2 rounded-md border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-900 px-3 text-sm font-medium text-zinc-800 dark:text-zinc-200 transition hover:border-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800 disabled:cursor-not-allowed disabled:opacity-60"
              disabled={isLeagueClientLoading}
              onClick={() => refreshLeagueClient({ matchLimit: 12 })}
              type="button"
            >
              <RefreshIcon />
              {isLeagueClientLoading ? t("common.refreshing") : t("common.refresh")}
            </button>
          </header>

          <section className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 p-5 shadow-sm">
            <div>
              <h2 className="text-base font-semibold text-zinc-950 dark:text-zinc-50">{t("matches.completed")}</h2>
              <p className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">{matchCountLabel(matches.length, isLeagueClientLoading, t)}</p>
            </div>

            <div className="mt-5 grid gap-3">
              {!leagueSelfSnapshot && isLeagueClientLoading && <StatePanel title={t("matches.loading")} body={t("matches.readingClient")} />}
              {leagueSelfSnapshot && matches.length === 0 && (
                <StatePanel title={t("matches.none")} body={emptyMatchesBody(leagueSelfSnapshot.status.phase, t)} />
              )}
              {matches.map((match) => (
                <MatchCard
                  imageUrl={match.championId ? leagueImages.championIcons[match.championId] : undefined}
                  key={match.gameId}
                  match={match}
                  onOpen={() => void openMatchRecapWindow({ gameId: match.gameId }, t("recap.title"))}
                  t={t}
                />
              ))}
            </div>
          </section>
      </div>
    </main>
  );
}

function MatchCard({
  imageUrl,
  match,
  onOpen,
  t,
}: {
  imageUrl: string | undefined;
  match: RecentMatchSummary;
  onOpen: () => void;
  t: T;
}) {
  return (
    <button
      className="grid w-full gap-3 rounded-md border border-zinc-200 dark:border-zinc-700 bg-zinc-50 dark:bg-zinc-800 p-3 text-left transition hover:border-rose-300 hover:bg-white dark:hover:bg-zinc-900 sm:grid-cols-[1fr_auto]"
      onClick={onOpen}
      type="button"
    >
      <div className="flex min-w-0 items-center gap-3">
        <ChampionImage championName={match.championName} imageUrl={imageUrl} />
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <p className="truncate text-sm font-semibold text-zinc-950 dark:text-zinc-50">{match.championName}</p>
            <ResultBadge result={match.result} />
          </div>
          <p className="mt-1 truncate text-xs text-zinc-500 dark:text-zinc-400">
            {match.queueName ?? t("common.unknown")} - {formatTimestamp(match.playedAt, t)}
          </p>
        </div>
      </div>
      <div className="flex items-center justify-between gap-5 sm:justify-end">
        <div className="text-left sm:text-right">
          <p className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
            {match.kills}/{match.deaths}/{match.assists}
          </p>
          <p className="mt-1 text-xs text-zinc-500 dark:text-zinc-400">KDA {match.kda === null ? "n/a" : match.kda.toFixed(1)}</p>
        </div>
        <span
          className={[
            "text-xs font-medium",
            match.result === "win"
              ? "text-emerald-600 dark:text-emerald-400"
              : match.result === "loss"
                ? "text-rose-600 dark:text-rose-400"
                : "text-zinc-500 dark:text-zinc-400",
          ].join(" ")}
        >
          {t("matches.openRecap")} →
        </span>
      </div>
    </button>
  );
}


function matchCountLabel(count: number, isLoading: boolean, t: T) {
  if (isLoading && count === 0) {
    return t("matches.loading");
  }

  return `${count} ${t("participant.recentMatches")}`;
}

function emptyMatchesBody(phase: string, t: T) {
  if (phase === "notLoggedIn") {
    return t("matches.loginHint");
  }

  if (phase === "notRunning") {
    return t("matches.startHint");
  }

  return t("matches.unavailableHint");
}


