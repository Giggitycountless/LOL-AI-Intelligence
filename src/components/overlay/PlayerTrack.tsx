import { memo, type MouseEvent } from "react";

import type { PlayerView, TeamTone } from "../../pages/selfHistoryOverlayUtils";
import {
  advisorTagClass,
  rankTierIconUrl,
  recentStatsStatusMessage,
  resultClass,
} from "../../pages/selfHistoryOverlayUtils";
import { initials, type T } from "../../utils/formatting";

import type { RecentMatchSummary } from "../../backend/types";

export const PlayerTrack = memo(function PlayerTrack({
  onChampionSelect,
  player,
  selectedChampionId,
  t,
  tone,
}: {
  onChampionSelect: (event: MouseEvent, championId: number | null | undefined) => void;
  player: PlayerView;
  selectedChampionId: number | null;
  t: T;
  tone: TeamTone;
}) {
  const isSelected = Boolean(player.championId && player.championId === selectedChampionId);

  return (
    <article
      className={[
        "flex min-w-0 flex-col rounded-md border bg-zinc-50 p-2",
        player.isEmpty ? "border-zinc-200 opacity-75" : tone === "ally" ? "border-emerald-200" : "border-rose-200",
      ].join(" ")}
      onClick={(event) => event.stopPropagation()}
    >
      <p
        className={[
          "mb-1.5 truncate text-[11px] font-semibold leading-tight",
          player.isEmpty ? "text-zinc-400" : "text-zinc-800",
        ].join(" ")}
        title={player.displayName}
      >
        {player.displayName}
      </p>
      <div className="grid h-16 grid-cols-[4rem_minmax(0,1fr)] gap-2">
        <ChampionPortrait
          championId={player.championId}
          displayName={player.displayName}
          isSelected={isSelected}
          masteryLevel={player.masteryLevel}
          onSelect={onChampionSelect}
          src={player.championUrl}
          summonerLevel={player.summonerLevel}
          t={t}
          tone={tone}
        />
        <div className="grid min-w-0 grid-rows-2 gap-1.5">
          <RankPill
            label="S"
            lp={player.soloRankLP}
            prominent
            tier={player.soloRankTier}
            title={t("overlay.rankUnavailable")}
            value={player.soloRank}
          />
          <RankPill
            label="F"
            lp={player.flexRankLP}
            tier={player.flexRankTier}
            title={t("overlay.rankUnavailable")}
            value={player.flexRank}
          />
        </div>
      </div>

      <SoloRankRecord losses={player.soloRankLosses} wins={player.soloRankWins} />

      <div className="relative mt-2.5 rounded-md border border-zinc-200 bg-white px-2 py-2">
        <div className="flex items-center justify-between gap-2">
          <span className="text-[11px] font-semibold uppercase tracking-wide text-zinc-500">{t("overlay.score")}</span>
          <span className="truncate text-base font-bold tabular-nums text-zinc-950">{player.score ?? "--"}</span>
        </div>
        <div className="mt-1.5 h-2.5 overflow-hidden rounded-full bg-zinc-100">
          <div
            className={["h-full rounded-full", tone === "ally" ? "bg-emerald-500" : "bg-rose-600"].join(" ")}
            style={{ width: `${player.scorePct}%` }}
          />
        </div>
        {player.badge && (
          <span
            className={[
              "absolute -right-px -top-2.5 flex h-5 min-w-7 items-center justify-center rounded-sm px-1 text-xs font-bold text-white shadow-sm",
              tone === "ally" ? "bg-emerald-700" : "bg-rose-700",
            ].join(" ")}
            title={`${player.winCount}/${player.gameCount}`}
          >
            {player.badge}
          </span>
        )}
      </div>

      {!player.isEmpty && (player.advisorTags.length > 0 || player.advisorSummary) && (
        <div className="mt-2 min-h-10 rounded-md border border-zinc-200 bg-white px-2 py-2">
          {player.advisorTags.length > 0 && (
            <div className="flex flex-wrap gap-1">
              {player.advisorTags.slice(0, 3).map((tag) => (
                <span
                  className={["rounded px-2 py-1 text-[11px] font-bold", advisorTagClass(tag.tone)].join(" ")}
                  key={`${player.id}-${tag.label}`}
                >
                  {tag.label}
                </span>
              ))}
            </div>
          )}
          {player.advisorSummary && (
            <p className="mt-1 line-clamp-2 text-[10px] font-medium leading-snug text-zinc-600" title={player.advisorSummary}>
              {player.advisorSummary}
            </p>
          )}
        </div>
      )}

      <div className="mt-3 flex items-center justify-between px-0.5 text-[11px] font-semibold tabular-nums text-zinc-500">
        {player.gameCount > 0 ? (
          <>
            <span className="uppercase tracking-wide">{player.winCount}W / {player.gameCount}G</span>
            {player.averageKda !== null && (
              <span>KDA {player.averageKda.toFixed(1)}</span>
            )}
          </>
        ) : (
          <span className="uppercase tracking-wide">{recentStatsStatusMessage(player.recentStatsStatus, t)}</span>
        )}
      </div>
      <div className="mt-1.5 grid gap-1.5">
        {player.rows.map((row) => (
          <MatchRow key={row.id} row={row} />
        ))}
      </div>
    </article>
  );
});

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
  tone,
}: {
  championId: number | null | undefined;
  displayName: string;
  isSelected: boolean;
  masteryLevel: number | null;
  onSelect: (event: MouseEvent, championId: number | null | undefined) => void;
  src: string | undefined;
  summonerLevel: number | null;
  t: T;
  tone: TeamTone;
}) {
  const baseClass = [
    "flex h-16 w-16 items-center justify-center rounded-md border object-cover shadow-sm transition",
    championId ? "cursor-pointer hover:scale-[1.02]" : "cursor-default",
    isSelected
      ? tone === "ally"
        ? "border-emerald-700 ring-2 ring-emerald-100"
        : "border-rose-700 ring-2 ring-rose-100"
      : "border-zinc-200",
  ].join(" ");

  return (
    <div className="relative h-16 w-16">
      <button
        aria-label={championId ? `${t("overlay.viewAbilities")} ${displayName}` : t("overlay.unselected")}
        className="h-16 w-16 rounded-md"
        disabled={!championId}
        onClick={(event) => onSelect(event, championId)}
        title={displayName}
        type="button"
      >
        {src ? (
          <img alt="" className={baseClass} src={src} />
        ) : (
          <span className={`${baseClass} bg-zinc-100 text-sm font-semibold text-zinc-500`}>{initials(displayName)}</span>
        )}
      </button>
      {masteryLevel !== null && masteryLevel >= 5 && (
        <span className="absolute -right-1 -top-1.5 flex h-4 min-w-4 items-center justify-center rounded bg-amber-500 px-0.5 text-[9px] font-bold leading-none text-white shadow-sm">
          M{masteryLevel}
        </span>
      )}
      {summonerLevel !== null && (
        <span className="absolute bottom-0 left-0 right-0 flex items-center justify-center rounded-b-md bg-black/50 py-0.5 text-[9px] font-semibold leading-none text-white">
          {summonerLevel}
        </span>
      )}
    </div>
  );
}

function SmallChampionIcon({ championName, src }: { championName: string; src: string | undefined }) {
  if (src) {
    return <img alt="" className="h-6 w-6 rounded border border-zinc-200 object-cover shadow-sm" src={src} />;
  }

  return (
    <div className="flex h-6 w-6 items-center justify-center rounded border border-zinc-200 bg-zinc-100 text-[10px] font-semibold text-zinc-400">
      {initials(championName)}
    </div>
  );
}

function RankPill({
  label,
  lp,
  prominent,
  tier,
  title,
  value,
}: {
  label: string;
  lp: number | null;
  prominent?: boolean;
  tier: string | null;
  title: string;
  value: string | null;
}) {
  const iconUrl = rankTierIconUrl(tier);
  const fullTitle = value
    ? lp !== null ? `${value} · ${lp} LP` : value
    : title;

  return (
    <div
      className={[
        "flex min-w-0 items-center gap-1 rounded-md border px-1.5",
        prominent ? "text-sm font-bold" : "text-[11px] font-semibold",
        value ? "border-zinc-300 bg-white text-zinc-900" : "border-zinc-200 bg-zinc-50 text-zinc-400",
      ].join(" ")}
      title={fullTitle}
    >
      {iconUrl ? (
        <img
          alt=""
          className={["shrink-0 object-contain", prominent ? "h-5 w-5" : "h-4 w-4"].join(" ")}
          src={iconUrl}
        />
      ) : (
        <span className={["shrink-0", prominent ? "text-zinc-400" : "text-zinc-500"].join(" ")}>{label}</span>
      )}
      <span className="min-w-0 truncate">{value ?? "--"}</span>
      {value && lp !== null && (
        <span className="shrink-0 text-zinc-400">{lp}</span>
      )}
    </div>
  );
}

function SoloRankRecord({ wins, losses }: { wins: number; losses: number }) {
  const total = wins + losses;
  if (total === 0) return null;

  const winRate = Math.round((wins / total) * 100);

  return (
    <div className="mt-1.5 flex items-center gap-1.5 px-0.5 text-[10px] font-semibold tabular-nums text-zinc-500">
      <span className="text-emerald-700">{wins}W</span>
      <span className="text-rose-600">{losses}L</span>
      <span>·</span>
      <span>{winRate}%</span>
    </div>
  );
}

function MatchRow({ row }: { row: { id: string; imageUrl: string | undefined; match: RecentMatchSummary | null } }) {
  const match = row.match;

  return (
    <div
      className={[
        "flex items-center gap-1.5 rounded border px-1.5 py-1",
        match ? resultClass(match.result) : "border-zinc-200 bg-zinc-50 text-zinc-400",
      ].join(" ")}
    >
      <SmallChampionIcon championName={match?.championName ?? "?"} src={row.imageUrl} />
      <div className="min-w-0 flex-1">
        <div className="truncate text-xs font-semibold tabular-nums leading-tight">
          {match ? `${match.kills}/${match.deaths}/${match.assists}` : "--"}
        </div>
        {match && (
          <div className="text-[10px] font-semibold tabular-nums leading-tight opacity-70">
            {match.kda === null || match.kda === undefined ? "" : `KDA ${match.kda.toFixed(1)}`}
          </div>
        )}
      </div>
    </div>
  );
}
