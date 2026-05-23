import { memo, useCallback, useEffect, useMemo, useState, type MouseEvent, type ReactNode } from "react";

import type {
  ChampSelectPlayer,
  ChampSelectRecentStatsStatus,
  LiveOverlaySnapshot,
} from "../backend/types";
import type { EffectiveLanguage } from "../i18n";
import type { LeagueChampionAbilityView, LeagueChampionDetailsView } from "../state/AppStateProvider";
import { useAdvisor, useAppCore, useChampSelect, useLeagueAssets } from "../state/AppStateProvider";
import { canOpenSelfHistoryOverlayWindow, destroySelfHistoryOverlayWindow } from "../windows/selfHistoryOverlayWindow";
import { type T } from "../utils/formatting";
import { RefreshIcon } from "../components/overlay/Icons";
import { PlayerTrack } from "../components/overlay/PlayerTrack";
import { ChampionDetailsPanel } from "../components/overlay/ChampionDetailsPanel";
import {
  createOverlayModel,
  type InitialSnapshotStatus,
  type OverlayModel,
  type PlayerView,
  type TeamTone,
  formatGameTime,
  formatNumber,
  formatSignedNumber,
  eventSummary,
  initialSnapshotMessage,
  TEAM_SIZE,
} from "./selfHistoryOverlayUtils";

const HISTORY_LOAD_TIMEOUT_MS = 8000;

export function SelfHistoryOverlay() {
  const { effectiveLanguage, t } = useAppCore();
  const { champSelectSnapshot, refreshChampSelectSnapshot } = useChampSelect();
  const {
    champSelectAdvisorSnapshot,
    liveOverlaySnapshot,
    refreshChampSelectAdvisorSnapshot,
    refreshLiveOverlaySnapshot,
  } = useAdvisor();
  const { championDetailsById, leagueImages, loadLeagueChampionDetails } = useLeagueAssets();
  const [selectedChampionId, setSelectedChampionId] = useState<number | null>(null);
  const [isChampionDetailsLoading, setIsChampionDetailsLoading] = useState(false);
  const [championDetailsError, setChampionDetailsError] = useState(false);
  const [isRefreshingChampSelect, setIsRefreshingChampSelect] = useState(false);
  const [refreshFailed, setRefreshFailed] = useState(false);
  const [isOverlayAllowed, setIsOverlayAllowed] = useState(false);
  const [initialSnapshotStatus, setInitialSnapshotStatus] = useState<InitialSnapshotStatus>("loading");
  const players = champSelectSnapshot?.players ?? [];
  const hasPlayers = players.length > 0;
  const hasRecentStats = players.some((player) => player.recentStats !== null);
  const isHistoryLoading = hasPlayers && !hasRecentStats && initialSnapshotStatus === "loading";
  const isHistoryUnavailable = hasPlayers && !hasRecentStats && initialSnapshotStatus === "error";
  const selectedChampionDetails = selectedChampionId ? championDetailsById[selectedChampionId] : undefined;
  const model = useMemo(
    () =>
      createOverlayModel(
        players,
        champSelectAdvisorSnapshot?.players ?? [],
        leagueImages.championIcons,
        effectiveLanguage,
        t,
      ),
    [champSelectAdvisorSnapshot?.players, effectiveLanguage, leagueImages.championIcons, players, t],
  );

  useEffect(() => {
    let wasCancelled = false;

    void canOpenSelfHistoryOverlayWindow().then(async (canOpen) => {
      if (wasCancelled) {
        return;
      }

      if (!canOpen) {
        await destroySelfHistoryOverlayWindow();
        if (wasCancelled) {
          return;
        }
        return;
      }

      setIsOverlayAllowed(true);
    });

    return () => {
      wasCancelled = true;
    };
  }, []);

  useEffect(() => {
    if (!refreshFailed) {
      return;
    }

    const timer = window.setTimeout(() => setRefreshFailed(false), 2500);
    return () => window.clearTimeout(timer);
  }, [refreshFailed]);

  useEffect(() => {
    if (!isOverlayAllowed) {
      return;
    }

    let wasCancelled = false;
    setInitialSnapshotStatus("loading");
    void refreshChampSelectSnapshot().then((didRefresh) => {
      if (!wasCancelled) {
        setInitialSnapshotStatus(didRefresh ? "ready" : "error");
      }
    });

    return () => {
      wasCancelled = true;
    };
  }, [isOverlayAllowed, refreshChampSelectSnapshot]);

  useEffect(() => {
    if (hasRecentStats) {
      setInitialSnapshotStatus("ready");
    }
  }, [hasRecentStats]);

  useEffect(() => {
    if (!isHistoryLoading) {
      return;
    }

    const timer = window.setTimeout(() => {
      setInitialSnapshotStatus((currentStatus) => (currentStatus === "loading" ? "error" : currentStatus));
    }, HISTORY_LOAD_TIMEOUT_MS);
    return () => window.clearTimeout(timer);
  }, [isHistoryLoading]);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setSelectedChampionId(null);
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  const closeChampionDetails = useCallback(() => {
    setSelectedChampionId(null);
  }, []);

  const handleChampionSelect = useCallback(
    async (event: MouseEvent, championId: number | null | undefined) => {
      event.stopPropagation();
      if (!championId) {
        return;
      }

      setSelectedChampionId(championId);
      setChampionDetailsError(false);
      if (championDetailsById[championId]) {
        return;
      }

      setIsChampionDetailsLoading(true);
      const didLoad = await loadLeagueChampionDetails(championId);
      setIsChampionDetailsLoading(false);
      setChampionDetailsError(!didLoad);
    },
    [championDetailsById, loadLeagueChampionDetails],
  );

  const handleRefreshChampSelect = useCallback(
    async (event: MouseEvent<HTMLButtonElement>) => {
      event.stopPropagation();
      if (isRefreshingChampSelect) {
        return;
      }

      setRefreshFailed(false);
      setInitialSnapshotStatus("loading");
      setIsRefreshingChampSelect(true);

      // Run all three in parallel; treat the refresh as successful if either the champ-select
      // snapshot OR the advisor snapshot came back with data. (During In-Progress phase the
      // champ-select session is gone but advisor still serves cached recent stats — that is
      // still a useful refresh from the user's perspective.)
      const results = await Promise.allSettled([
        refreshChampSelectSnapshot(),
        refreshChampSelectAdvisorSnapshot(),
        refreshLiveOverlaySnapshot(),
      ]);
      const champSelectOk = results[0].status === "fulfilled" && results[0].value === true;
      const advisorOk = results[1].status === "fulfilled" && results[1].value === true;
      const anyOk = champSelectOk || advisorOk;

      setIsRefreshingChampSelect(false);
      setRefreshFailed(!anyOk);
      setInitialSnapshotStatus(anyOk ? "ready" : "error");
    },
    [isRefreshingChampSelect, refreshChampSelectAdvisorSnapshot, refreshChampSelectSnapshot, refreshLiveOverlaySnapshot],
  );

  if (!isOverlayAllowed) {
    return (
      <main className="flex h-screen items-center justify-center bg-zinc-100 text-sm font-semibold text-zinc-500">
        {t("common.pending")}
      </main>
    );
  }

  return (
    <main
      className="relative flex h-screen flex-col overflow-hidden bg-zinc-100 p-2 text-zinc-700"
      onClick={closeChampionDetails}
    >
      <header
        className="mb-2 flex h-10 shrink-0 items-center justify-between rounded-lg border border-zinc-200 bg-white px-3 shadow-sm"
      >
        <div className="flex min-w-0 items-center gap-2" data-tauri-drag-region>
          <span className="h-2.5 w-2.5 rounded-full bg-rose-700" />
          <p className="truncate text-xs font-semibold uppercase tracking-wide text-zinc-700" data-tauri-drag-region>
            {t("overlay.windowTitle")}
          </p>
          <span className="hidden text-[11px] font-medium text-zinc-500 lg:inline" data-tauri-drag-region>
            {t("overlay.dragHint")}
          </span>
        </div>
        <div className="flex items-center gap-2" onClick={(event) => event.stopPropagation()}>
          {refreshFailed && (
            <span className="rounded-md border border-rose-200 bg-rose-50 px-2.5 py-1 text-xs font-semibold text-rose-700">
              {t("overlay.refreshFailed")}
            </span>
          )}
          <IconButton
            ariaLabel={t("overlay.refresh")}
            disabled={isRefreshingChampSelect}
            onClick={handleRefreshChampSelect}
            title={t("overlay.refresh")}
          >
            <RefreshIcon isSpinning={isRefreshingChampSelect} />
          </IconButton>
        </div>
      </header>

      <div className="grid min-h-0 flex-1 grid-cols-2 gap-2">
        <TeamBoard
          onChampionSelect={handleChampionSelect}
          players={model.enemies}
          selectedChampionId={selectedChampionId}
          t={t}
          tone="enemy"
        />
        <TeamBoard
          onChampionSelect={handleChampionSelect}
          players={model.allies}
          selectedChampionId={selectedChampionId}
          t={t}
          tone="ally"
        />
      </div>

      {(players.length === 0 || isHistoryLoading || isHistoryUnavailable) && (
        <div className="pointer-events-none absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 rounded-lg border border-zinc-200 bg-white px-5 py-3 text-center text-sm font-semibold text-zinc-500 shadow-sm">
          {initialSnapshotMessage(initialSnapshotStatus, t)}
        </div>
      )}

      {selectedChampionId && (
        <ChampionDetailsPanel
          details={selectedChampionDetails}
          hasError={championDetailsError}
          isLoading={isChampionDetailsLoading && !selectedChampionDetails}
          onClose={closeChampionDetails}
          t={t}
        />
      )}

      <LiveOverlayBar snapshot={liveOverlaySnapshot} t={t} />
      <SummaryBar summary={model.summary} t={t} />
    </main>
  );
}

const TeamBoard = memo(function TeamBoard({
  onChampionSelect,
  players,
  selectedChampionId,
  t,
  tone,
}: {
  onChampionSelect: (event: MouseEvent, championId: number | null | undefined) => void;
  players: PlayerView[];
  selectedChampionId: number | null;
  t: T;
  tone: TeamTone;
}) {
  return (
    <section
      className={[
        "grid h-full min-h-0 grid-cols-5 gap-2 rounded-lg border bg-white p-2 shadow-sm",
        tone === "ally" ? "border-emerald-200" : "border-rose-200",
      ].join(" ")}
    >
      {players.map((player) => (
        <PlayerTrack
          key={player.id}
          onChampionSelect={onChampionSelect}
          player={player}
          selectedChampionId={selectedChampionId}
          t={t}
          tone={tone}
        />
      ))}
    </section>
  );
});

function SummaryBar({
  summary,
  t,
}: {
  summary: OverlayModel["summary"];
  t: T;
}) {
  return (
    <div className="mt-2 flex shrink-0 gap-3">
      <SummaryCard label={t("overlay.allyWins")} tone="ally" value={`${summary.allyWins}/${summary.allyGames}`} />
      <SummaryCard label={t("overlay.enemyWins")} tone="enemy" value={`${summary.enemyWins}/${summary.enemyGames}`} />
    </div>
  );
}

function SummaryCard({ label, tone, value }: { label: string; tone: TeamTone; value: string }) {
  return (
    <div className="rounded-lg border border-zinc-200 bg-white px-3 py-1.5 shadow-sm">
      <p className="text-xs font-medium text-zinc-500">{label}</p>
      <p className={["mt-0.5 text-base font-semibold tabular-nums", tone === "ally" ? "text-emerald-700" : "text-rose-700"].join(" ")}>
        {tone === "ally" ? "+" : "-"} {value}
      </p>
    </div>
  );
}

function LiveOverlayBar({ snapshot, t }: { snapshot: LiveOverlaySnapshot | null; t: T }) {
  if (!snapshot) {
    return null;
  }

  const lastEvent = snapshot.events.at(-1);
  const activeGold = snapshot.activePlayer?.currentGold;
  const diff = snapshot.gold.itemValueDiff;

  return (
    <div className="mt-2 grid shrink-0 grid-cols-4 gap-2">
      <LiveInfoCard label={t("live.game")} value={snapshot.gameTimeSeconds === null ? "-" : formatGameTime(snapshot.gameTimeSeconds)} />
      <LiveInfoCard label={t("live.items")} value={formatSignedNumber(diff)} tone={diff >= 0 ? "ally" : "enemy"} />
      <LiveInfoCard label={t("live.gold")} value={activeGold === null || activeGold === undefined ? "-" : formatNumber(Math.round(activeGold))} />
      <LiveInfoCard label={t("live.event")} value={lastEvent ? eventSummary(lastEvent) : "-"} />
    </div>
  );
}

function LiveInfoCard({ label, tone, value }: { label: string; tone?: TeamTone; value: string }) {
  return (
    <div className="min-w-0 rounded-lg border border-zinc-200 bg-white px-3 py-1.5 shadow-sm">
      <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-500">{label}</p>
      <p
        className={[
          "mt-0.5 truncate text-sm font-semibold tabular-nums",
          tone === "ally" ? "text-emerald-700" : tone === "enemy" ? "text-rose-700" : "text-zinc-800",
        ].join(" ")}
        title={value}
      >
        {value}
      </p>
    </div>
  );
}

function IconButton({
  ariaLabel,
  children,
  disabled,
  onClick,
  title,
}: {
  ariaLabel: string;
  children: ReactNode;
  disabled?: boolean;
  onClick: (event: MouseEvent<HTMLButtonElement>) => void;
  title: string;
}) {
  return (
    <button
      aria-label={ariaLabel}
      className="flex h-8 w-8 items-center justify-center rounded-md border border-zinc-300 bg-white text-zinc-600 transition hover:border-zinc-400 hover:bg-zinc-50 hover:text-rose-700 disabled:cursor-wait disabled:opacity-70"
      disabled={disabled}
      onClick={onClick}
      title={title}
      type="button"
    >
      {children}
    </button>
  );
}
