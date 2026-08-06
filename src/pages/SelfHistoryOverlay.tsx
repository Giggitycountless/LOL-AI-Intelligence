import { memo, useCallback, useEffect, useMemo, useRef, useState, type MouseEvent, type ReactNode } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

import type {
  ChampSelectPlayer,
  ChampSelectRecentStatsStatus,
  LiveOverlaySnapshot,
} from "../backend/types";
import type { EffectiveLanguage } from "../i18n";
import type { LeagueChampionAbilityView, LeagueChampionDetailsView } from "../state/AppStateProvider";
import { useAdvisor, useAppCore, useChampSelect, useLeagueAssets } from "../state/AppStateProvider";
import { canOpenSelfHistoryOverlayWindow, destroySelfHistoryOverlayWindow } from "../windows/selfHistoryOverlayWindow";
import { openParticipantProfileWindow } from "../windows/participantProfileWindow";
import { type T } from "../utils/formatting";
import { HideIcon, RefreshIcon } from "../components/overlay/Icons";
import { PlayerTrack } from "../components/overlay/PlayerTrack";
import { ChampionDetailsPanel } from "../components/overlay/ChampionDetailsPanel";
import { MatchDetailPanel } from "../components/overlay/MatchDetailPanel";
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
  premadeGroupStyle,
  TEAM_SIZE,
} from "./selfHistoryOverlayUtils";

const HISTORY_LOAD_TIMEOUT_MS = 8000;

function isDevModeOverlay() {
  // Hash format: #/self-history-overlay?devMode=1
  const query = window.location.hash.split("?")[1] ?? "";
  return new URLSearchParams(query).get("devMode") === "1";
}

export function SelfHistoryOverlay() {
  const { effectiveLanguage, loadPostMatchDetail, postMatchDetails, t } = useAppCore();
  const { champSelectSnapshot, refreshChampSelectSnapshot } = useChampSelect();
  const {
    champSelectAdvisorSnapshot,
    liveOverlaySnapshot,
    refreshChampSelectAdvisorSnapshot,
    refreshLiveOverlaySnapshot,
  } = useAdvisor();
  const {
    championDetailsById,
    leagueImages,
    loadLeagueChampionDetails,
    loadLeagueChampionIcon,
    loadLeagueGameAsset,
    loadLeagueRankTierIcon,
  } = useLeagueAssets();
  const [selectedChampionId, setSelectedChampionId] = useState<number | null>(null);
  const [isChampionDetailsLoading, setIsChampionDetailsLoading] = useState(false);
  const [championDetailsError, setChampionDetailsError] = useState(false);
  const [selectedMatchGameId, setSelectedMatchGameId] = useState<number | null>(null);
  const [isMatchDetailLoading, setIsMatchDetailLoading] = useState(false);
  const [matchDetailError, setMatchDetailError] = useState(false);
  const matchRequestIdRef = useRef(0);
  const championRequestIdRef = useRef(0);
  const [isRefreshingChampSelect, setIsRefreshingChampSelect] = useState(false);
  const [refreshFailed, setRefreshFailed] = useState(false);
  const devMode = isDevModeOverlay();
  const [isOverlayAllowed, setIsOverlayAllowed] = useState(devMode);
  const [initialSnapshotStatus, setInitialSnapshotStatus] = useState<InitialSnapshotStatus>("loading");
  const players = champSelectSnapshot?.players ?? [];
  const hasPlayers = players.length > 0;
  const hasRecentStats = players.some((player) => player.recentStats !== null);
  const isHistoryLoading = hasPlayers && !hasRecentStats && initialSnapshotStatus === "loading";
  const isHistoryUnavailable = hasPlayers && !hasRecentStats && initialSnapshotStatus === "error";
  const selectedChampionDetails = selectedChampionId ? championDetailsById[selectedChampionId] : undefined;
  const selectedMatchDetail = selectedMatchGameId ? postMatchDetails[selectedMatchGameId] : undefined;
  const premadeGroups = champSelectSnapshot?.premadeGroups;
  const model = useMemo(
    () =>
      createOverlayModel(
        players,
        champSelectAdvisorSnapshot?.players ?? [],
        leagueImages.championIcons,
        effectiveLanguage,
        t,
        premadeGroups ?? [],
      ),
    [champSelectAdvisorSnapshot?.players, effectiveLanguage, leagueImages.championIcons, players, premadeGroups, t],
  );

  useEffect(() => {
    if (devMode) {
      return;
    }

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
  }, [devMode]);

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
      if (!wasCancelled && !didRefresh) {
        // Fetch failed entirely — surface an error immediately.
        // If didRefresh is true but stats are absent, keep "loading" so the
        // hasRecentStats effect can set "ready" once history arrives, and the
        // 8-second timeout can surface "error" if it never does.
        setInitialSnapshotStatus("error");
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
    const tiers = new Set<string>();
    for (const player of players) {
      for (const queue of player.rankedQueues) {
        if (queue.isRanked && queue.tier) {
          tiers.add(queue.tier.toLowerCase());
        }
      }
    }
    for (const tier of tiers) {
      void loadLeagueRankTierIcon(tier);
    }
  }, [players, loadLeagueRankTierIcon]);

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
        setSelectedMatchGameId(null);
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  // Load champion icons + item/rune/spell assets for whichever match detail is
  // currently open, mirroring the same pattern used by MatchRecap and
  // ParticipantProfilePanel.
  useEffect(() => {
    if (!selectedMatchDetail) {
      return;
    }

    const championIds = new Set<number>();
    const itemIds = new Set<number>();
    const runeIds = new Set<number>();
    const spellIds = new Set<number>();
    for (const team of selectedMatchDetail.teams) {
      for (const participant of team.participants) {
        if (participant.championId) championIds.add(participant.championId);
        participant.items.forEach((id) => itemIds.add(id));
        participant.runes.forEach((id) => runeIds.add(id));
        participant.spells.forEach((id) => spellIds.add(id));
      }
    }
    championIds.forEach((id) => void loadLeagueChampionIcon(id));
    itemIds.forEach((id) => void loadLeagueGameAsset("item", id));
    runeIds.forEach((id) => void loadLeagueGameAsset("rune", id));
    spellIds.forEach((id) => void loadLeagueGameAsset("spell", id));
  }, [selectedMatchDetail, loadLeagueChampionIcon, loadLeagueGameAsset]);

  const closeChampionDetails = useCallback(() => {
    setSelectedChampionId(null);
  }, []);

  const closeMatchDetails = useCallback(() => {
    setSelectedMatchGameId(null);
  }, []);

  const handleMatchSelect = useCallback(
    async (event: MouseEvent, gameId: number) => {
      event.stopPropagation();
      setSelectedChampionId(null);
      setSelectedMatchGameId(gameId);
      setMatchDetailError(false);
      // Track which match this request is for — if the user clicks another
      // match before this fetch resolves, a stale response must not clobber
      // the loading/error state of the newer selection (it would otherwise
      // leave the panel blank: not loading, not errored, no detail yet).
      const requestId = ++matchRequestIdRef.current;
      if (postMatchDetails[gameId]) {
        return;
      }

      setIsMatchDetailLoading(true);
      const didLoad = await loadPostMatchDetail(gameId);
      if (matchRequestIdRef.current !== requestId) {
        return;
      }
      setIsMatchDetailLoading(false);
      setMatchDetailError(!didLoad);
    },
    [loadPostMatchDetail, postMatchDetails],
  );

  const handleParticipantSelect = useCallback((participantId: number) => {
    if (!selectedMatchGameId) {
      return;
    }
    void openParticipantProfileWindow({ gameId: selectedMatchGameId, participantId });
  }, [selectedMatchGameId]);

  const handleChampionSelect = useCallback(
    async (event: MouseEvent, championId: number | null | undefined) => {
      event.stopPropagation();
      if (!championId) {
        return;
      }

      setSelectedMatchGameId(null);
      setSelectedChampionId(championId);
      setChampionDetailsError(false);
      // Same stale-response guard as handleMatchSelect: ignore this fetch's
      // result if a newer champion click has started since.
      const requestId = ++championRequestIdRef.current;
      if (championDetailsById[championId]) {
        return;
      }

      setIsChampionDetailsLoading(true);
      const didLoad = await loadLeagueChampionDetails(championId);
      if (championRequestIdRef.current !== requestId) {
        return;
      }
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
      <main className="flex h-screen items-center justify-center bg-zinc-950 text-sm font-semibold text-zinc-500">
        {t("common.pending")}
      </main>
    );
  }

  return (
    <main
      className="relative flex h-screen flex-col overflow-hidden bg-zinc-950 p-2 text-zinc-100"
      onClick={() => {
        closeChampionDetails();
        closeMatchDetails();
      }}
    >
      <header
        className="mb-2 flex h-10 shrink-0 items-center justify-between rounded-lg border border-zinc-700 bg-zinc-900 px-3 shadow-sm"
      >
        <div className="flex min-w-0 items-center gap-2" data-tauri-drag-region>
          <span className="h-2.5 w-2.5 rounded-full bg-rose-700" />
          <p className="truncate text-xs font-semibold uppercase tracking-wide text-zinc-100" data-tauri-drag-region>
            {t("overlay.windowTitle")}
          </p>
          {devMode && (
            <span className="rounded-full border border-amber-600 bg-amber-900/60 px-1.5 py-0.5 text-[10px] font-bold uppercase tracking-wide text-amber-400" data-tauri-drag-region>
              DEV
            </span>
          )}
          <span className="hidden text-[11px] font-medium text-zinc-500 lg:inline" data-tauri-drag-region>
            {t("overlay.dragHint")}
          </span>
        </div>
        <div className="flex items-center gap-2" onClick={(event) => event.stopPropagation()}>
          {refreshFailed && (
            <span className="rounded-md border border-red-700 bg-red-950 px-2.5 py-1 text-xs font-semibold text-red-400">
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
          <IconButton
            ariaLabel={t("overlay.hide")}
            onClick={() => void getCurrentWindow().hide()}
            title={t("overlay.hide")}
          >
            <HideIcon />
          </IconButton>
        </div>
      </header>

      {/* League Akari's stacked layout: ally team (blue) on top, enemy team
          (red) below, each row of five cards spanning the full width.
          Scrolls vertically so the boards are never cut off unreachably. */}
      <div className="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto">
        <TeamBoard
          onChampionSelect={handleChampionSelect}
          onMatchSelect={handleMatchSelect}
          players={model.allies}
          rankTierIcons={leagueImages.rankTierIcons}
          selectedChampionId={selectedChampionId}
          t={t}
          team={model.allyTeam}
          tone="ally"
        />
        <TeamBoard
          onChampionSelect={handleChampionSelect}
          onMatchSelect={handleMatchSelect}
          players={model.enemies}
          rankTierIcons={leagueImages.rankTierIcons}
          selectedChampionId={selectedChampionId}
          t={t}
          team={model.enemyTeam}
          tone="enemy"
        />
      </div>

      {(players.length === 0 || isHistoryLoading || isHistoryUnavailable) && (
        <div className="pointer-events-none absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 rounded-lg border border-zinc-700 bg-zinc-900 px-5 py-3 text-center text-sm font-semibold text-zinc-400 shadow-sm">
          {/* With an empty roster the snapshot fetch has finished but champ
              select simply has no players yet — that is "waiting", not
              "loading" (which would otherwise show forever). */}
          {players.length === 0 && initialSnapshotStatus === "loading"
            ? t("overlay.empty")
            : initialSnapshotMessage(initialSnapshotStatus, t)}
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

      {selectedMatchGameId && (
        <MatchDetailPanel
          detail={selectedMatchDetail}
          gameAssets={leagueImages.gameAssets}
          hasError={matchDetailError}
          isLoading={isMatchDetailLoading && !selectedMatchDetail}
          onClose={closeMatchDetails}
          onParticipantSelect={handleParticipantSelect}
          participantImages={leagueImages.championIcons}
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
  onMatchSelect,
  players,
  rankTierIcons,
  selectedChampionId,
  t,
  team,
  tone,
}: {
  onChampionSelect: (event: MouseEvent, championId: number | null | undefined) => void;
  onMatchSelect: (event: MouseEvent, gameId: number) => void;
  players: PlayerView[];
  rankTierIcons: Record<string, string>;
  selectedChampionId: number | null;
  t: T;
  team: OverlayModel["allyTeam"];
  tone: TeamTone;
}) {
  const premadeSizes = premadeGroupSizes(players);

  // League Akari's team block: no framed box — a header line (team dot,
  // name, win rate | KDA, premade chips) above the card grid. Sized by its
  // content so the two boards stack without forcing a scroll each.
  return (
    <section className="flex shrink-0 flex-col gap-2">
      <div className="flex items-end px-0.5">
        <span
          className={[
            "mr-2 h-[10px] w-[10px] self-center rounded-full border border-white/20",
            tone === "ally" ? "bg-emerald-500" : "bg-red-500",
          ].join(" ")}
        />
        <span className="mr-3 text-base font-bold leading-tight text-white/90">
          {tone === "ally" ? t("overlay.allyTeam") : t("overlay.enemyTeam")}
        </span>
        {team.winRate !== null && (
          <>
            <span
              className={[
                "self-center text-sm font-bold tabular-nums",
                team.winRate >= 50 ? "text-emerald-400" : "text-red-400",
              ].join(" ")}
              title={`${t("overlay.winRate")} ${team.wins}/${team.games}`}
            >
              {team.winRate}%
            </span>
            {team.kda !== null && (
              <>
                <span className="mx-2 h-[0.9em] w-px self-center bg-white/15" />
                <span className="self-center text-sm tabular-nums text-white/80" title={t("overlay.teamKda")}>
                  {team.kda.toFixed(2)}
                </span>
              </>
            )}
          </>
        )}
        {premadeSizes.length > 0 && (
          <div className="ml-2 flex gap-2 self-center">
            {premadeSizes.map(({ groupIndex, size }) => {
              const style = premadeGroupStyle(groupIndex);
              return (
                <span
                  className="rounded-sm px-1 py-0.5 text-xs leading-3"
                  key={groupIndex}
                  style={{ backgroundColor: style.background, color: style.color }}
                  title={t("overlay.premadeGroupHint")}
                >
                  {t("overlay.premadeSize").replace("{n}", String(size))}
                </span>
              );
            })}
          </div>
        )}
      </div>
      <div className="grid grid-cols-5 gap-1">
        {players.map((player) => (
          <PlayerTrack
            key={player.id}
            onChampionSelect={onChampionSelect}
            onMatchSelect={onMatchSelect}
            player={player}
            rankTierIcons={rankTierIcons}
            selectedChampionId={selectedChampionId}
            t={t}
            tone={tone}
          />
        ))}
      </div>
    </section>
  );
});

/** Premade groups present on this board with their member counts. */
function premadeGroupSizes(players: PlayerView[]): { groupIndex: number; size: number }[] {
  const sizes = new Map<number, number>();
  for (const player of players) {
    if (player.premadeGroup !== null) {
      sizes.set(player.premadeGroup, (sizes.get(player.premadeGroup) ?? 0) + 1);
    }
  }

  return [...sizes.entries()]
    .map(([groupIndex, size]) => ({ groupIndex, size }))
    .sort((a, b) => a.groupIndex - b.groupIndex);
}

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
    <div className="rounded-lg border border-zinc-700 bg-zinc-900 px-3 py-1.5 shadow-sm">
      <p className="text-xs font-medium text-zinc-500">{label}</p>
      <p className={["mt-0.5 text-base font-semibold tabular-nums", tone === "ally" ? "text-emerald-400" : "text-red-400"].join(" ")}>
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
      <LiveInfoCard label={t("live.event")} value={lastEvent ? eventSummary(lastEvent, t) : "-"} />
    </div>
  );
}

function LiveInfoCard({ label, tone, value }: { label: string; tone?: TeamTone; value: string }) {
  return (
    <div className="min-w-0 rounded-lg border border-zinc-700 bg-zinc-900 px-3 py-1.5 shadow-sm">
      <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-500">{label}</p>
      <p
        className={[
          "mt-0.5 truncate text-sm font-semibold tabular-nums",
          tone === "ally" ? "text-emerald-400" : tone === "enemy" ? "text-red-400" : "text-zinc-100",
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
      className="flex h-8 w-8 items-center justify-center rounded-md border border-zinc-700 bg-zinc-800 text-zinc-400 transition hover:border-zinc-600 hover:bg-zinc-700 hover:text-red-400 disabled:cursor-wait disabled:opacity-70"
      disabled={disabled}
      onClick={onClick}
      title={title}
      type="button"
    >
      {children}
    </button>
  );
}
