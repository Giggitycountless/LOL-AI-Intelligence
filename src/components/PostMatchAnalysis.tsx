import { useState, type ReactNode } from "react";

import { ChampionImage, ResultBadge } from "./common";
import { leagueGameAssetKey, useAppCore, type LeagueGameAssetView } from "../state/AppStateProvider";
import type {
  LeagueGameAssetKind,
  ParticipantMetricLeader,
  PostMatchDetail,
  PostMatchParticipant,
  PostMatchTeam,
} from "../backend/types";
import { formatDuration, formatResult, type T } from "../utils/formatting";

export function PostMatchAnalysis({
  detail,
  gameAssets,
  onParticipantSelect,
  participantImages,
  expandable = true,
  // A team table's natural minimum is ~41rem/656px (see COLS + gaps + padding).
  // Two-up needs 2×41rem + gap + outer padding ≈ 1372px of viewport, so the
  // panels only pair up from 1380px; below that they stack full-width and the
  // inner overflow-x-auto never has to truncate columns.
  teamsLayoutClassName = "min-[1380px]:grid-cols-2",
}: {
  detail: PostMatchDetail;
  gameAssets: Record<string, LeagueGameAssetView>;
  onParticipantSelect: (participantId: number) => void;
  participantImages: Record<number, string>;
  expandable?: boolean;
  teamsLayoutClassName?: string;
}) {
  const { t } = useAppCore();
  const [expandedIds, setExpandedIds] = useState<Set<number>>(() => new Set());

  const toggleExpanded = (participantId: number) => {
    setExpandedIds((prev) => {
      const next = new Set(prev);
      if (next.has(participantId)) {
        next.delete(participantId);
      } else {
        next.add(participantId);
      }
      return next;
    });
  };

  const maxDamage = Math.max(
    1,
    ...detail.teams.flatMap((team) => team.participants.map((p) => p.damageToChampions)),
  );

  // Lets the comparison strip resolve each leader's champion icon from its
  // participantId (the leader payload only carries id + name + value).
  const participantById = new Map<number, PostMatchParticipant>();
  for (const team of detail.teams) {
    for (const participant of team.participants) {
      participantById.set(participant.participantId, participant);
    }
  }

  return (
    <div className="grid gap-4">
      <ComparisonStrip
        comparison={detail.comparison}
        participantById={participantById}
        participantImages={participantImages}
        t={t}
      />

      <div className={`grid gap-3 ${teamsLayoutClassName}`}>
        {detail.teams.map((team) => (
          <TeamBlock
            expandable={expandable}
            expandedIds={expandedIds}
            gameAssets={gameAssets}
            key={team.teamId}
            maxDamage={maxDamage}
            onParticipantSelect={onParticipantSelect}
            onToggleExpanded={toggleExpanded}
            participantImages={participantImages}
            selfParticipantId={detail.selfParticipantId}
            team={team}
            t={t}
          />
        ))}
      </div>

      {detail.warnings.length > 0 && (
        <div className="rounded-md border border-amber-200 bg-amber-50 p-3 text-sm text-amber-800 dark:border-amber-800 dark:bg-amber-950 dark:text-amber-300">
          {detail.warnings.map((warning) => (
            <p key={`${warning.section}-${warning.message}`}>{warning.message}</p>
          ))}
        </div>
      )}
    </div>
  );
}

// ── Team block ────────────────────────────────────────────────────────────────

// Shared column template: Player | Score | KDA | Damage | CS | Gold | Build | ⌄
const COLS =
  "grid-cols-[minmax(10rem,1.5fr)_3.5rem_4rem_minmax(5.5rem,0.6fr)_2.5rem_3rem_minmax(8rem,0.9fr)_2rem]";
const MIN_W = "min-w-[43rem]";

const TEAM_TONE = {
  win: {
    border: "border-emerald-200 dark:border-emerald-800",
    header: "bg-emerald-50 border-emerald-200 dark:bg-emerald-950 dark:border-emerald-800",
    accent: "border-l-4 border-l-emerald-400 dark:border-l-emerald-500",
    title: "text-emerald-800 dark:text-emerald-300",
  },
  loss: {
    border: "border-rose-200 dark:border-rose-800",
    header: "bg-rose-50 border-rose-200 dark:bg-rose-950 dark:border-rose-800",
    accent: "border-l-4 border-l-rose-400 dark:border-l-rose-500",
    title: "text-rose-700 dark:text-rose-300",
  },
  unknown: {
    border: "border-zinc-200 dark:border-zinc-700",
    header: "bg-zinc-50 dark:bg-zinc-800 border-zinc-200 dark:border-zinc-700",
    accent: "",
    title: "text-zinc-950 dark:text-zinc-50",
  },
} as const;

function TeamBlock({
  expandable,
  expandedIds,
  gameAssets,
  maxDamage,
  onParticipantSelect,
  onToggleExpanded,
  participantImages,
  selfParticipantId,
  team,
  t,
}: {
  expandable: boolean;
  expandedIds: Set<number>;
  gameAssets: Record<string, LeagueGameAssetView>;
  maxDamage: number;
  onParticipantSelect: (participantId: number) => void;
  onToggleExpanded: (participantId: number) => void;
  participantImages: Record<number, string>;
  selfParticipantId: number | null;
  team: PostMatchTeam;
  t: T;
}) {
  const tone = TEAM_TONE[team.result] ?? TEAM_TONE.unknown;

  return (
    <div className={`overflow-visible rounded-md border bg-white dark:bg-zinc-900 ${tone.border} ${tone.accent}`}>
      {/* Header */}
      <div className={`flex items-center justify-between gap-3 border-b px-3 py-2 ${tone.header}`}>
        <div>
          <p className={`text-sm font-semibold ${tone.title}`}>{formatResult(team.result, t)}</p>
          <p className="mt-1 text-xs text-zinc-500 dark:text-zinc-400">
            {team.totals.kills}/{team.totals.deaths}/{team.totals.assists} · {formatCompact(team.totals.goldEarned)} {t("analysis.gold")}
          </p>
        </div>
        <ResultBadge result={team.result} />
      </div>

      {/* Column headers */}
      <div className={`overflow-x-auto`}>
        <div className={`grid ${COLS} ${MIN_W} gap-2 border-b border-zinc-200 dark:border-zinc-700 bg-zinc-100 dark:bg-zinc-900 px-3 py-2 text-[11px] font-semibold uppercase tracking-wide text-zinc-500 dark:text-zinc-400`}>
          <span>{t("analysis.player")}</span>
          <span>{t("analysis.score")}</span>
          <span>KDA</span>
          <span>{t("analysis.damage")}</span>
          <span>CS</span>
          <span>{t("analysis.gold")}</span>
          <span>{t("analysis.build")}</span>
          <span className="sr-only">{t("analysis.expand")}</span>
        </div>

        <div>
          {team.participants.map((participant) => (
            <ParticipantRow
              expandable={expandable}
              gameAssets={gameAssets}
              imageUrl={participant.championId ? participantImages[participant.championId] : undefined}
              isExpanded={expandedIds.has(participant.participantId)}
              isSelf={selfParticipantId === participant.participantId}
              key={participant.participantId}
              maxDamage={maxDamage}
              onSelect={() => onParticipantSelect(participant.participantId)}
              onToggleExpanded={() => onToggleExpanded(participant.participantId)}
              participant={participant}
              t={t}
            />
          ))}
        </div>
      </div>
    </div>
  );
}

function ParticipantRow({
  expandable,
  gameAssets,
  imageUrl,
  isExpanded,
  isSelf,
  maxDamage,
  onSelect,
  onToggleExpanded,
  participant,
  t,
}: {
  expandable: boolean;
  gameAssets: Record<string, LeagueGameAssetView>;
  imageUrl: string | undefined;
  isExpanded: boolean;
  isSelf: boolean;
  maxDamage: number;
  onSelect: () => void;
  onToggleExpanded: () => void;
  participant: PostMatchParticipant;
  t: T;
}) {
  const selfClass = isSelf
    ? "bg-sky-50 ring-1 ring-inset ring-sky-300 dark:bg-sky-950/40 dark:ring-sky-700"
    : "";

  return (
    <div className={`${MIN_W} border-b border-zinc-100 dark:border-zinc-700 last:border-b-0 ${selfClass}`}>
      <div className={`grid ${COLS} items-center gap-2 px-3 py-2`}>
        {/* Player — opens profile */}
        <button
          className="-mx-1 flex min-w-0 items-center gap-2 rounded px-1 py-0.5 text-left transition hover:bg-zinc-100 dark:hover:bg-zinc-800"
          onClick={onSelect}
          type="button"
        >
          <ChampionImage championName={participant.championName} imageUrl={imageUrl} size="xs" />
          <div className="min-w-0">
            <p className="truncate text-sm font-semibold text-zinc-950 dark:text-zinc-50">{participant.displayName}</p>
            <p className="mt-0.5 truncate text-xs text-zinc-500 dark:text-zinc-400">{participant.championName}</p>
          </div>
        </button>

        {/* Score */}
        <ScoreBadge score={participant.performanceScore} />

        {/* KDA */}
        <KdaCell participant={participant} />

        {/* Damage */}
        <DamageCell damage={participant.damageToChampions} maxDamage={maxDamage} />

        {/* CS */}
        <span className="text-sm font-semibold text-zinc-700 dark:text-zinc-300">{participant.cs}</span>

        {/* Gold */}
        <span className="text-sm font-semibold text-zinc-700 dark:text-zinc-300">{formatCompact(participant.goldEarned)}</span>

        {/* Build */}
        <BuildCell assets={gameAssets} participant={participant} t={t} />

        {/* Expand toggle */}
        {expandable ? (
          <button
            aria-expanded={isExpanded}
            aria-label={isExpanded ? t("analysis.collapse") : t("analysis.expand")}
            className="flex h-6 w-6 items-center justify-center rounded text-zinc-400 transition hover:bg-zinc-100 hover:text-zinc-700 dark:text-zinc-500 dark:hover:bg-zinc-800 dark:hover:text-zinc-200"
            onClick={onToggleExpanded}
            title={isExpanded ? t("analysis.collapse") : t("analysis.expand")}
            type="button"
          >
            <Chevron expanded={isExpanded} />
          </button>
        ) : (
          <span />
        )}
      </div>

      {expandable && isExpanded && <ParticipantDeepPanel participant={participant} t={t} />}
    </div>
  );
}

function Chevron({ expanded }: { expanded: boolean }) {
  return (
    <svg
      aria-hidden="true"
      className={`h-4 w-4 transition-transform ${expanded ? "rotate-180" : ""}`}
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      viewBox="0 0 24 24"
    >
      <path d="M6 9l6 6 6-6" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

// ── Deep panel ──────────────────────────────────────────────────────────────

function ParticipantDeepPanel({ participant, t }: { participant: PostMatchParticipant; t: T }) {
  return (
    <div className="grid gap-3 border-t border-zinc-100 bg-zinc-50 px-3 py-3 dark:border-zinc-800 dark:bg-zinc-950/40 sm:grid-cols-2 xl:grid-cols-4">
      {/* Damage breakdown */}
      <DeepCard title={t("analysis.damageBreakdown")}>
        <DamageBreakdownBar participant={participant} t={t} />
        <div className="mt-2 grid gap-1">
          <DeepStat label={t("analysis.damageToTurrets")} value={formatCompact(participant.damageToTurrets)} />
          <DeepStat label={t("analysis.damageToObjectives")} value={formatCompact(participant.damageToObjectives)} />
          <DeepStat label={t("analysis.damageTaken")} value={formatCompact(participant.damageTaken)} />
        </div>
      </DeepCard>

      {/* Vision */}
      <DeepCard title={t("analysis.vision")}>
        <div className="grid gap-1">
          <DeepStat label={t("analysis.vision")} value={String(participant.visionScore)} />
          <DeepStat label={t("analysis.wardsPlaced")} value={String(participant.wardsPlaced)} />
          <DeepStat label={t("analysis.wardsKilled")} value={String(participant.wardsKilled)} />
          <DeepStat label={t("analysis.controlWards")} value={String(participant.controlWardsBought)} />
        </div>
      </DeepCard>

      {/* Combat */}
      <DeepCard title={t("analysis.combat")}>
        <div className="grid gap-1">
          <DeepStat label={t("analysis.killingSpree")} value={String(participant.largestKillingSpree)} />
          <DeepStat label={t("analysis.multiKill")} value={String(participant.largestMultiKill)} />
          <MultiKillRow participant={participant} t={t} />
          <div className="mt-1 flex flex-wrap gap-1">
            {participant.firstBlood && <FirstBadge label={t("analysis.firstBlood")} />}
            {participant.firstTower && <FirstBadge label={t("analysis.firstTower")} />}
          </div>
        </div>
      </DeepCard>

      {/* Other */}
      <DeepCard title={t("analysis.other")}>
        <div className="grid gap-1">
          <DeepStat label={t("analysis.timeSpentDead")} value={formatDuration(participant.timeSpentDeadSeconds, t)} />
        </div>
      </DeepCard>
    </div>
  );
}

function DeepCard({ title, children }: { title: string; children: ReactNode }) {
  return (
    <div className="rounded-md border border-zinc-200 bg-white px-3 py-2 dark:border-zinc-700 dark:bg-zinc-900">
      <p className="mb-1.5 text-[11px] font-semibold uppercase tracking-wide text-zinc-500 dark:text-zinc-400">{title}</p>
      {children}
    </div>
  );
}

function DeepStat({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-baseline justify-between gap-2 text-xs">
      <span className="text-zinc-500 dark:text-zinc-400">{label}</span>
      <span className="font-semibold text-zinc-900 dark:text-zinc-100">{value}</span>
    </div>
  );
}

function DamageBreakdownBar({ participant, t }: { participant: PostMatchParticipant; t: T }) {
  const physical = Math.max(0, participant.physicalDamageToChampions);
  const magic = Math.max(0, participant.magicDamageToChampions);
  const trueDmg = Math.max(0, participant.trueDamageToChampions);
  const total = physical + magic + trueDmg;
  const pct = (value: number) => (total > 0 ? `${(value / total) * 100}%` : "0%");

  return (
    <div>
      <div className="flex h-2 w-full overflow-hidden rounded-full bg-zinc-200 dark:bg-zinc-700">
        <div className="h-full bg-orange-500" style={{ width: pct(physical) }} />
        <div className="h-full bg-sky-500" style={{ width: pct(magic) }} />
        <div className="h-full bg-zinc-400 dark:bg-zinc-300" style={{ width: pct(trueDmg) }} />
      </div>
      <div className="mt-2 grid gap-1">
        <DeepStat label={t("analysis.physicalDamage")} value={formatCompact(physical)} />
        <DeepStat label={t("analysis.magicDamage")} value={formatCompact(magic)} />
        <DeepStat label={t("analysis.trueDamage")} value={formatCompact(trueDmg)} />
      </div>
    </div>
  );
}

function MultiKillRow({ participant, t }: { participant: PostMatchParticipant; t: T }) {
  const entries: Array<[string, number]> = [
    [t("analysis.doubleKills"), participant.doubleKills],
    [t("analysis.tripleKills"), participant.tripleKills],
    [t("analysis.quadraKills"), participant.quadraKills],
    [t("analysis.pentaKills"), participant.pentaKills],
  ];
  const active = entries.filter(([, count]) => count > 0);
  if (active.length === 0) return null;

  return (
    <div className="mt-1 flex flex-wrap gap-1">
      {active.map(([label, count]) => (
        <span
          key={label}
          className="rounded bg-zinc-100 px-1.5 py-0.5 text-[11px] font-semibold text-zinc-700 dark:bg-zinc-800 dark:text-zinc-200"
        >
          {label} ×{count}
        </span>
      ))}
    </div>
  );
}

function FirstBadge({ label }: { label: string }) {
  return (
    <span className="rounded bg-amber-100 px-1.5 py-0.5 text-[11px] font-semibold text-amber-800 dark:bg-amber-950 dark:text-amber-300">
      {label}
    </span>
  );
}

// ── Sub-cells ─────────────────────────────────────────────────────────────────

function ScoreBadge({ score }: { score: number }) {
  const tone =
    score >= 8
      ? "bg-sky-100 text-sky-800"
      : score >= 6.5
        ? "bg-emerald-100 text-emerald-800"
        : score >= 4.5
          ? "bg-zinc-100 dark:bg-zinc-700 text-zinc-700 dark:text-zinc-300"
          : "bg-rose-100 text-rose-800";

  return <span className={["w-fit rounded-md px-1.5 py-0.5 text-sm font-bold", tone].join(" ")}>{score.toFixed(1)}</span>;
}

function KdaCell({ participant }: { participant: PostMatchParticipant }) {
  return (
    <div className="min-w-0">
      <p className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">
        {participant.kills}/{participant.deaths}/{participant.assists}
      </p>
      <p className="mt-0.5 text-xs text-zinc-500 dark:text-zinc-400">
        {participant.kda === null ? "n/a" : `${participant.kda.toFixed(1)}:1`}
      </p>
    </div>
  );
}

function DamageCell({ damage, maxDamage }: { damage: number; maxDamage: number }) {
  const pct = Math.max(4, Math.round((damage / maxDamage) * 100));

  return (
    <div className="min-w-0">
      <span className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">{formatCompact(damage)}</span>
      <div className="mt-1 h-1.5 overflow-hidden rounded-full bg-zinc-200 dark:bg-zinc-700">
        <div className="h-full rounded-full bg-rose-500" style={{ width: `${pct}%` }} />
      </div>
    </div>
  );
}

function BuildCell({
  assets,
  participant,
  t,
}: {
  assets: Record<string, LeagueGameAssetView>;
  participant: PostMatchParticipant;
  t: T;
}) {
  return (
    <div className="grid gap-1">
      <AssetStrip assetIds={participant.items} assets={assets} iconSize="sm" kind="item" t={t} />
      <div className="flex flex-wrap gap-1">
        <AssetStrip assetIds={participant.runes} assets={assets} iconSize="xs" kind="rune" t={t} />
        <AssetStrip assetIds={participant.spells} assets={assets} iconSize="xs" kind="spell" t={t} />
      </div>
    </div>
  );
}

function AssetStrip({
  assetIds,
  assets,
  iconSize,
  kind,
  t,
}: {
  assetIds: number[];
  assets: Record<string, LeagueGameAssetView>;
  iconSize: "xs" | "sm";
  kind: LeagueGameAssetKind;
  t: T;
}) {
  return (
    <div className="flex min-w-0 flex-wrap gap-0.5">
      {assetIds.length === 0 && kind === "item" && (
        <span className="text-xs text-zinc-400 dark:text-zinc-500">{t("analysis.noItems")}</span>
      )}
      {assetIds.map((assetId, index) => (
        <AssetIcon
          asset={assets[leagueGameAssetKey(kind, assetId)]}
          assetId={assetId}
          iconSize={iconSize}
          key={`${kind}-${assetId}-${index}`}
          kind={kind}
          t={t}
        />
      ))}
    </div>
  );
}

function AssetIcon({
  asset,
  assetId,
  iconSize,
  kind,
  t,
}: {
  asset: LeagueGameAssetView | undefined;
  assetId: number;
  iconSize: "xs" | "sm";
  kind: LeagueGameAssetKind;
  t: T;
}) {
  const label = asset?.name ?? `${assetLabel(kind, t)} ${assetId}`;
  const title = asset?.description ? `${label}\n${asset.description}` : label;
  const sizeClass = iconSize === "sm" ? "h-6 w-6" : "h-4 w-4";

  return (
    <span
      className={[
        "group relative inline-flex shrink-0 items-center justify-center rounded border border-zinc-200 dark:border-zinc-700 bg-zinc-100 dark:bg-zinc-900",
        sizeClass,
      ].join(" ")}
      title={title}
    >
      {asset ? (
        <img alt={label} className="h-full w-full rounded object-cover" src={asset.imageUrl} />
      ) : (
        <span className="text-[8px] font-semibold text-zinc-500 dark:text-zinc-400">{assetId}</span>
      )}
      <span className="pointer-events-none absolute bottom-full left-1/2 z-20 mb-2 hidden w-64 -translate-x-1/2 rounded-md border border-zinc-800 bg-zinc-950 p-3 text-left text-xs text-white shadow-xl group-hover:block">
        <span className="block text-sm font-semibold">{label}</span>
        <span className="mt-1 block text-zinc-300">
          {asset?.description ?? `${assetLabel(kind, t)} ${t("analysis.detailsLoading")}`}
        </span>
      </span>
    </span>
  );
}

// ── Comparison strip ──────────────────────────────────────────────────────────

function ComparisonStrip({
  comparison,
  participantById,
  participantImages,
  t,
}: {
  comparison: PostMatchDetail["comparison"];
  participantById: Map<number, PostMatchParticipant>;
  participantImages: Record<number, string>;
  t: T;
}) {
  const leaderProps = { participantById, participantImages, t };
  return (
    <div className="grid gap-2 md:grid-cols-5">
      <Leader label="KDA" leader={comparison.highestKda} {...leaderProps} />
      <Leader label="CS" leader={comparison.mostCs} {...leaderProps} />
      <Leader label={t("analysis.gold")} leader={comparison.mostGold} {...leaderProps} />
      <Leader label={t("analysis.damage")} leader={comparison.mostDamage} {...leaderProps} />
      <Leader label={t("analysis.vision")} leader={comparison.highestVision} {...leaderProps} />
    </div>
  );
}

function Leader({
  label,
  leader,
  participantById,
  participantImages,
  t,
}: {
  label: string;
  leader: ParticipantMetricLeader | null;
  participantById: Map<number, PostMatchParticipant>;
  participantImages: Record<number, string>;
  t: T;
}) {
  const participant = leader ? participantById.get(leader.participantId) : undefined;
  const imageUrl = participant?.championId ? participantImages[participant.championId] : undefined;

  return (
    <div className="rounded-md border border-zinc-200 dark:border-zinc-700 bg-zinc-50 dark:bg-zinc-800 px-3 py-2">
      <p className="text-xs font-medium uppercase tracking-wide text-zinc-500 dark:text-zinc-400">{label}</p>
      <div className="mt-1 flex min-w-0 items-center gap-2">
        {leader && <ChampionImage championName={participant?.championName ?? ""} imageUrl={imageUrl} size="xs" />}
        <p className="min-w-0 truncate text-sm font-semibold text-zinc-950 dark:text-zinc-50">
          {leader?.displayName ?? t("common.unavailable")}
        </p>
      </div>
      <p className="mt-1 text-xs text-zinc-500 dark:text-zinc-400">
        {leader ? formatLeaderValue(leader.value) : t("common.noData")}
      </p>
    </div>
  );
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function formatCompact(value: number) {
  return value >= 1000 ? `${(value / 1000).toFixed(1)}k` : String(value);
}

function formatLeaderValue(value: number) {
  return Number.isInteger(value) ? String(value) : value.toFixed(1);
}

function assetLabel(kind: LeagueGameAssetKind, t: T) {
  switch (kind) {
    case "item":  return t("analysis.item");
    case "rune":  return t("analysis.rune");
    case "spell": return t("analysis.spell");
  }
}
