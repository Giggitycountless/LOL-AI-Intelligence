import { memo, type MouseEvent } from "react";

import type { PlayerView, TeamTone } from "../../pages/selfHistoryOverlayUtils";
import {
  advisorTagClass,
  recentStatsStatusMessage,
  resultClass,
  scoreWidth,
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
      <div className="grid h-16 grid-cols-[4rem_minmax(0,1fr)] gap-2">
        <ChampionPortrait
          championId={player.championId}
          displayName={player.displayName}
          isSelected={isSelected}
          onSelect={onChampionSelect}
          src={player.championUrl}
          t={t}
          tone={tone}
        />
        <div className="grid min-w-0 grid-rows-2 gap-1.5">
          <RankPill label="S" title={t("overlay.rankUnavailable")} value={player.soloRank} />
          <RankPill label="F" title={t("overlay.rankUnavailable")} value={player.flexRank} />
        </div>
      </div>

      <div className="relative mt-2.5 rounded-md border border-zinc-200 bg-white px-2 py-1.5">
        <div className="flex items-center justify-between gap-2">
          <span className="text-[11px] font-semibold uppercase tracking-wide text-zinc-500">{t("overlay.score")}</span>
          <span className="truncate text-sm font-semibold tabular-nums text-zinc-950">{player.score ?? "--"}</span>
        </div>
        <div className="mt-1 h-1.5 overflow-hidden rounded-full bg-zinc-100">
          <div
            className={["h-full rounded-full", tone === "ally" ? "bg-emerald-600" : "bg-rose-700"].join(" ")}
            style={{ width: `${scoreWidth(player.score)}%` }}
          />
        </div>
        {player.badge && (
          <span
            className={[
              "absolute -right-px -top-2 flex h-5 min-w-7 items-center justify-center rounded-sm px-1 text-xs font-semibold text-white shadow-sm",
              tone === "ally" ? "bg-emerald-700" : "bg-rose-700",
            ].join(" ")}
            title={`${player.winCount}/${player.gameCount}`}
          >
            {player.badge}
          </span>
        )}
      </div>

      {!player.isEmpty && (player.advisorTags.length > 0 || player.advisorSummary) && (
        <div className="mt-2 min-h-10 rounded-md border border-zinc-200 bg-white px-1.5 py-1.5">
          {player.advisorTags.length > 0 && (
            <div className="flex flex-wrap gap-1">
              {player.advisorTags.slice(0, 3).map((tag) => (
                <span
                  className={["rounded px-1.5 py-0.5 text-[10px] font-bold", advisorTagClass(tag.tone)].join(" ")}
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
  onSelect,
  src,
  t,
  tone,
}: {
  championId: number | null | undefined;
  displayName: string;
  isSelected: boolean;
  onSelect: (event: MouseEvent, championId: number | null | undefined) => void;
  src: string | undefined;
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

function RankPill({ label, title, value }: { label: string; title: string; value: string | null }) {
  return (
    <div
      className={[
        "flex min-w-0 items-center gap-1 rounded-md border px-1.5 text-[11px] font-semibold",
        value ? "border-zinc-200 bg-white text-zinc-800" : "border-zinc-200 bg-zinc-50 text-zinc-400",
      ].join(" ")}
      title={value ?? title}
    >
      <span className="shrink-0 text-zinc-500">{label}</span>
      <span className="truncate">{value ?? "--"}</span>
    </div>
  );
}

function MatchRow({ row }: { row: { id: string; imageUrl: string | undefined; match: RecentMatchSummary | null } }) {
  const match = row.match;

  return (
    <div
      className={[
        "grid h-9 grid-cols-[2rem_minmax(0,1fr)_2.4rem] items-center gap-1.5 rounded border px-1.5",
        match ? resultClass(match.result) : "border-zinc-200 bg-zinc-50 text-zinc-400",
      ].join(" ")}
    >
      <SmallChampionIcon championName={match?.championName ?? "?"} src={row.imageUrl} />
      <span className="truncate text-center text-xs font-semibold tabular-nums">
        {match ? `${match.kills}/${match.deaths}/${match.assists}` : "--"}
      </span>
      <span className="truncate text-right text-[11px] font-semibold tabular-nums">
        {match?.kda === null || match?.kda === undefined ? "--" : match.kda.toFixed(1)}
      </span>
    </div>
  );
}
