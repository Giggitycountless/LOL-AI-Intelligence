import { memo, type MouseEvent } from "react";

import type { PlayerView, TeamTone } from "../../pages/selfHistoryOverlayUtils";
import {
  advisorTagLabel,
  formatMatchDate,
  kdaToneClass,
  matchResultLabel,
  premadeGroupStyle,
  recentStatsStatusMessage,
  scoreBarClass,
  scoreToneClass,
  winRateToneClass,
} from "../../pages/selfHistoryOverlayUtils";
import { initials, type T } from "../../utils/formatting";

import type { RecentMatchSummary } from "../../backend/types";

function advisorTagClassDark(tone: import("../../backend/types").AdvisorPlayerTag["tone"]) {
  switch (tone) {
    case "good": return "bg-emerald-900/50 text-emerald-400";
    case "warn": return "bg-amber-900/50 text-amber-400";
    case "info": return "bg-zinc-700 text-zinc-400";
  }
}

/**
 * Match-row theme: wins are emerald, losses rose (PRODUCT.md — emerald is
 * always ally/win/positive, rose is always enemy/loss/alert), remakes/unknown
 * neutral gray — tinted backgrounds instead of borders.
 */
function matchRowTheme(result: RecentMatchSummary["result"] | null) {
  if (result === "win") {
    return {
      row: "bg-[rgba(16,185,129,0.25)] text-white/80",
      result: "text-emerald-300",
    };
  }
  if (result === "loss") {
    return {
      row: "bg-[rgba(244,63,94,0.25)] text-white/80",
      result: "text-rose-300",
    };
  }
  if (result !== null) {
    return {
      row: "bg-white/15 text-white/80",
      result: "text-white/80",
    };
  }

  return { row: "bg-white/5 text-zinc-600", result: "text-zinc-600" };
}

export const PlayerTrack = memo(function PlayerTrack({
  onChampionSelect,
  onMatchSelect,
  player,
  rankTierIcons,
  selectedChampionId,
  t,
  tone: _tone,
}: {
  onChampionSelect: (event: MouseEvent, championId: number | null | undefined) => void;
  onMatchSelect: (event: MouseEvent, gameId: number) => void;
  player: PlayerView;
  rankTierIcons: Record<string, string>;
  selectedChampionId: number | null;
  t: T;
  tone: TeamTone;
}) {
  const isSelected = Boolean(player.championId && player.championId === selectedChampionId);
  const premadeStyle = player.premadeGroup !== null ? premadeGroupStyle(player.premadeGroup) : null;
  const hasTags =
    !player.isEmpty &&
    (premadeStyle !== null ||
      player.winningStreak >= 3 ||
      player.losingStreak >= 3 ||
      player.advisorTags.length > 0);

  return (
    // League Akari card frame: neutral surface, team color lives on the team
    // header only; a premade group tints the border and the corner deco.
    <article
      className={[
        "relative flex min-w-0 flex-col overflow-hidden rounded border bg-zinc-900/90 p-2",
        premadeStyle ? "" : "border-white/10",
        player.isEmpty ? "opacity-60" : "",
      ].join(" ")}
      onClick={(event) => event.stopPropagation()}
      style={premadeStyle ? { borderColor: `${premadeStyle.background}d0` } : undefined}
    >
      {premadeStyle && (
        <div
          className="absolute right-0 top-0 z-0 h-4 w-4 -translate-y-1/2 translate-x-1/2 rotate-45"
          style={{ backgroundColor: premadeStyle.background }}
          title={t("overlay.premadeGroupHint")}
        />
      )}

      {/* ── Header: round portrait + name / ranks (Akari PlayerInfoCardHeader) ── */}
      <div className="mb-1 flex">
        <ChampionPortrait
          championId={player.championId}
          displayName={player.displayName}
          isSelected={isSelected}
          masteryLevel={player.masteryLevel}
          onSelect={onChampionSelect}
          src={player.championUrl}
          summonerLevel={player.summonerLevel}
          t={t}
        />
        <div className="flex w-0 flex-1 flex-col justify-center gap-1">
          <p
            className={[
              "truncate text-[13px] font-bold leading-tight",
              player.isEmpty ? "text-zinc-600" : "text-white/80",
            ].join(" ")}
            style={premadeStyle ? { color: premadeStyle.background } : undefined}
            title={player.displayName}
          >
            {player.displayName}
          </p>
          <div className="flex gap-1">
            <RankEntry
              iconUrl={player.soloRankTier ? (rankTierIcons[player.soloRankTier] ?? null) : null}
              lp={player.soloRankLP}
              title={rankTitle(player.soloRank, player.soloRankLP, player.soloRankWins, player.soloRankLosses, t)}
              unrankedLabel={t("overlay.unranked")}
              value={player.soloRank}
            />
            <RankEntry
              iconUrl={player.flexRankTier ? (rankTierIcons[player.flexRankTier] ?? null) : null}
              lp={player.flexRankLP}
              title={rankTitle(player.flexRank, player.flexRankLP, 0, 0, t)}
              unrankedLabel={t("overlay.unranked")}
              value={player.flexRank}
            />
          </div>
        </div>
      </div>

      {/* ── Score: primary anchor alongside rank (PRODUCT.md design principle 1).
          Its own row, visibly labeled, with an intensity bar driven by
          scorePct (relative to this match's top score) — previously computed
          but never rendered, leaving the anchor metric the same weight as
          the secondary stats below it. ── */}
      <div className="mb-1" title={t("overlay.scoreHint")}>
        <div className="flex items-baseline justify-between">
          <span className="text-[9px] font-semibold uppercase tracking-wide text-white/60">
            {t("overlay.score")}
          </span>
          <span className={["text-[17px] font-extrabold leading-none tabular-nums", scoreToneClass(player.scorePct)].join(" ")}>
            {player.score === null ? "—" : player.score}
          </span>
        </div>
        <div className="mt-1 h-1.5 overflow-hidden rounded-full bg-white/10">
          <div
            className={["h-full rounded-full", scoreBarClass(player.scorePct)].join(" ")}
            style={{ width: `${player.scorePct}%` }}
          />
        </div>
      </div>

      {/* ── Secondary stats: win rate / KDA (Akari PlayerInfoCardStats) ── */}
      <div className="mb-1 flex items-center">
        <div
          className={["flex-1 text-center text-[12px] font-semibold tabular-nums", winRateToneClass(player.winRate)].join(" ")}
          title={t("overlay.winRate")}
        >
          {player.winRate === null ? "— %" : `${player.winRate}%`}
          {player.gameCount > 0 && (
            <span className="text-[9px] font-normal text-white/60"> ({player.gameCount})</span>
          )}
        </div>
        <div
          className={["flex-1 text-center text-[12px] font-semibold tabular-nums", kdaToneClass(player.averageKda)].join(" ")}
          title="KDA"
        >
          {player.averageKda === null ? "—" : player.averageKda.toFixed(2)}
        </div>
      </div>

      {/* ── Tag chips (Akari PlayerCardTagsArea) ── */}
      {hasTags && (
        <div className="mb-1 flex flex-wrap gap-1">
          {premadeStyle && (
            <span
              className="rounded-sm px-1 py-0.5 text-[11px] font-bold leading-[11px]"
              style={{ backgroundColor: premadeStyle.background, color: premadeStyle.color }}
              title={t("overlay.premadeGroupHint")}
            >
              {t("overlay.premadeGroup").replace("{letter}", premadeStyle.letter)}
            </span>
          )}
          {player.winningStreak >= 3 && (
            <span className="rounded-sm bg-[#18571c] px-1 py-0.5 text-[11px] leading-[11px] text-white">
              {t("overlay.winningStreak").replace("{n}", String(player.winningStreak))}
            </span>
          )}
          {player.losingStreak >= 3 && (
            <span className="rounded-sm bg-[#893b3b] px-1 py-0.5 text-[11px] leading-[11px] text-white">
              {t("overlay.losingStreak").replace("{n}", String(player.losingStreak))}
            </span>
          )}
          {player.advisorTags.slice(0, 3).map((tag) => (
            <span
              className={["rounded-sm px-1 py-0.5 text-[11px] leading-[11px]", advisorTagClassDark(tag.tone)].join(" ")}
              key={`${player.id}-${tag.kind}`}
            >
              {advisorTagLabel(tag, t)}
            </span>
          ))}
        </div>
      )}
      {!player.isEmpty && player.advisorSummary && (
        <p className="mb-1 truncate text-[10px] leading-snug text-white/50" title={player.advisorSummary}>
          {player.advisorSummary}
        </p>
      )}

      {/* ── Match history list (Akari PlayerInfoCardMatchHistory) ── */}
      <div className="mt-1 flex min-h-0 flex-1 flex-col gap-0.5">
        {player.gameCount === 0 && (
          <div className="flex flex-1 items-center justify-center rounded bg-white/5 text-xs text-white/60">
            {recentStatsStatusMessage(player.recentStatsStatus, t)}
          </div>
        )}
        {player.gameCount > 0 &&
          player.rows.map((row) => (
            <MatchRow key={row.id} onSelect={onMatchSelect} row={row} t={t} />
          ))}
      </div>
    </article>
  );
});

function rankTitle(value: string | null, lp: number | null, wins: number, losses: number, t: T) {
  if (!value) {
    return t("overlay.rankUnavailable");
  }

  const parts = [lp !== null ? `${value} · ${lp} LP` : value];
  if (wins + losses > 0) {
    parts.push(`${wins}W ${losses}L`);
  }
  return parts.join(" · ");
}

// ─── Internal sub-components ───

function ChampionPortrait({
  championId,
  displayName,
  isSelected,
  masteryLevel,
  onSelect,
  src,
  summonerLevel,
  t,
}: {
  championId: number | null | undefined;
  displayName: string;
  isSelected: boolean;
  masteryLevel: number | null;
  onSelect: (event: MouseEvent, championId: number | null | undefined) => void;
  src: string | undefined;
  summonerLevel: number | null;
  t: T;
}) {
  // Akari's header portrait: round icon with a white/30 ring and the summoner
  // level pill overflowing the bottom-right edge.
  const iconClass = [
    "h-full w-full rounded-full border border-white/30 object-cover",
    isSelected ? "ring-2 ring-white/40" : "",
  ].join(" ");

  return (
    <div className="relative mr-2 h-[42px] w-[42px] shrink-0">
      <button
        aria-label={championId ? `${t("overlay.viewAbilities")} ${displayName}` : t("overlay.unselected")}
        className={[
          "h-full w-full rounded-full transition-[filter]",
          championId ? "cursor-pointer hover:brightness-110" : "cursor-default",
        ].join(" ")}
        disabled={!championId}
        onClick={(event) => onSelect(event, championId)}
        title={displayName}
        type="button"
      >
        {src ? (
          <img alt="" className={iconClass} src={src} />
        ) : (
          <span className={`${iconClass} flex items-center justify-center bg-zinc-800 text-xs font-semibold text-zinc-500`}>
            {initials(displayName)}
          </span>
        )}
      </button>
      {summonerLevel !== null && (
        <span className="absolute bottom-0 right-0 translate-x-[35%] rounded bg-black/50 px-1 text-[10px] leading-normal text-white">
          {summonerLevel}
        </span>
      )}
      {masteryLevel !== null && masteryLevel >= 5 && (
        <span
          className="absolute -top-1 right-0 flex h-4 min-w-4 items-center justify-center rounded bg-amber-500 px-0.5 text-[9px] font-bold leading-none text-white shadow-sm"
          title={`${t("profile.mastery")} ${masteryLevel}`}
        >
          M{masteryLevel}
        </span>
      )}
    </div>
  );
}

function RankEntry({
  iconUrl,
  lp,
  title,
  unrankedLabel,
  value,
}: {
  iconUrl: string | null;
  lp: number | null;
  title: string;
  unrankedLabel: string;
  value: string | null;
}) {
  if (!value) {
    return (
      <div className="flex w-0 flex-1 items-center justify-center" title={title}>
        <span className="text-[12px] text-white/60">{unrankedLabel}</span>
      </div>
    );
  }

  return (
    <div className="flex w-0 flex-1 items-center justify-start" title={title}>
      {iconUrl && <img alt="" className="mr-1 h-[18px] w-[18px] shrink-0 object-contain" src={iconUrl} />}
      <span className="overflow-hidden text-ellipsis whitespace-nowrap text-[12px] font-semibold text-white/90">
        {lp !== null ? `${value} ${lp}` : value}
      </span>
    </div>
  );
}

function SmallChampionIcon({ championName, src }: { championName: string; src: string | undefined }) {
  if (src) {
    return <img alt="" className="h-6 w-6 shrink-0 rounded bg-[#4b5b7d] object-cover" src={src} />;
  }

  return (
    <div className="flex h-6 w-6 shrink-0 items-center justify-center rounded bg-[#4b5b7d] text-[10px] font-semibold text-white/70">
      {initials(championName)}
    </div>
  );
}

function MatchRow({
  onSelect,
  row,
  t,
}: {
  onSelect: (event: MouseEvent, gameId: number) => void;
  row: { id: string; imageUrl: string | undefined; match: RecentMatchSummary | null };
  t: T;
}) {
  const match = row.match;
  const playedAt = match ? formatMatchDate(match.playedAt) : null;
  const theme = matchRowTheme(match ? match.result : null);

  // Akari's row: tinted background, 24px champion icon, queue name over
  // date+result, K/D/A trailing. Clickable when a match is present, to open
  // the full post-match detail panel for that game.
  return (
    <div
      className={[
        "flex h-[34px] shrink-0 items-center rounded px-2 py-0.5",
        match ? "cursor-pointer transition hover:brightness-125" : "",
        theme.row,
      ].join(" ")}
      onClick={match ? (event) => onSelect(event, match.gameId) : undefined}
      role={match ? "button" : undefined}
      tabIndex={match ? 0 : undefined}
      title={match ? t("overlay.viewMatchDetails") : undefined}
    >
      <SmallChampionIcon championName={match?.championName ?? "?"} src={row.imageUrl} />
      <div className="ml-1 mr-1 w-0 flex-1">
        <div className="overflow-hidden text-ellipsis whitespace-nowrap text-xs leading-tight">
          {match?.queueName ?? (match ? match.championName : "--")}
        </div>
        <div className="text-[10px] leading-tight">
          {playedAt}
          {match && (
            <span className={["ml-1 font-bold", theme.result].join(" ")}>
              {matchResultLabel(match.result, t)}
            </span>
          )}
        </div>
      </div>
      <div className="shrink-0 text-xs tabular-nums leading-tight">
        {match ? `${match.kills} / ${match.deaths} / ${match.assists}` : ""}
      </div>
    </div>
  );
}
