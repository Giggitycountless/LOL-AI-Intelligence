import { ChampionImage, ResultBadge } from "./common";
import { leagueGameAssetKey, useAppCore, type LeagueGameAssetView } from "../state/AppStateProvider";
import type {
  LeagueGameAssetKind,
  ParticipantMetricLeader,
  PostMatchDetail,
  PostMatchParticipant,
  PostMatchTeam,
} from "../backend/types";
import { formatResult, initials, type T } from "../utils/formatting";

export function PostMatchAnalysis({
  detail,
  gameAssets,
  onParticipantSelect,
  participantImages,
  teamsLayoutClassName = "md:grid-cols-2",
}: {
  detail: PostMatchDetail;
  gameAssets: Record<string, LeagueGameAssetView>;
  onParticipantSelect: (participantId: number) => void;
  participantImages: Record<number, string>;
  teamsLayoutClassName?: string;
}) {
  const { t } = useAppCore();
  const maxDamage = Math.max(
    1,
    ...detail.teams.flatMap((team) => team.participants.map((p) => p.damageToChampions)),
  );

  return (
    <div className="grid gap-4">
      <ComparisonStrip comparison={detail.comparison} t={t} />

      <div className={`grid gap-3 ${teamsLayoutClassName}`}>
        {detail.teams.map((team) => (
          <TeamBlock
            gameAssets={gameAssets}
            key={team.teamId}
            maxDamage={maxDamage}
            onParticipantSelect={onParticipantSelect}
            participantImages={participantImages}
            team={team}
            t={t}
          />
        ))}
      </div>

      {detail.warnings.length > 0 && (
        <div className="rounded-md border border-amber-200 bg-amber-50 p-3 text-sm text-amber-800">
          {detail.warnings.map((warning) => (
            <p key={`${warning.section}-${warning.message}`}>{warning.message}</p>
          ))}
        </div>
      )}
    </div>
  );
}

// ── Team block ────────────────────────────────────────────────────────────────

// Shared column template: Player | Score | KDA | Damage | CS | Gold | Build
const COLS =
  "grid-cols-[minmax(10rem,1.5fr)_3.5rem_4rem_minmax(5.5rem,0.6fr)_2.5rem_3rem_minmax(8rem,0.9fr)]";
const MIN_W = "min-w-[37rem]";

const TEAM_TONE = {
  win: {
    border: "border-emerald-200",
    header: "bg-emerald-50 border-emerald-200",
    accent: "border-l-4 border-l-emerald-400",
    title: "text-emerald-800",
  },
  loss: {
    border: "border-rose-200",
    header: "bg-rose-50 border-rose-200",
    accent: "border-l-4 border-l-rose-400",
    title: "text-rose-700",
  },
  unknown: {
    border: "border-zinc-200 dark:border-zinc-700",
    header: "bg-zinc-50 dark:bg-zinc-800 border-zinc-200 dark:border-zinc-700",
    accent: "",
    title: "text-zinc-950 dark:text-zinc-50",
  },
} as const;

function TeamBlock({
  gameAssets,
  maxDamage,
  onParticipantSelect,
  participantImages,
  team,
  t,
}: {
  gameAssets: Record<string, LeagueGameAssetView>;
  maxDamage: number;
  onParticipantSelect: (participantId: number) => void;
  participantImages: Record<number, string>;
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
        </div>

        <div>
          {team.participants.map((participant) => (
            <ParticipantRow
              gameAssets={gameAssets}
              imageUrl={participant.championId ? participantImages[participant.championId] : undefined}
              key={participant.participantId}
              maxDamage={maxDamage}
              onSelect={() => onParticipantSelect(participant.participantId)}
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
  gameAssets,
  imageUrl,
  maxDamage,
  onSelect,
  participant,
  t,
}: {
  gameAssets: Record<string, LeagueGameAssetView>;
  imageUrl: string | undefined;
  maxDamage: number;
  onSelect: () => void;
  participant: PostMatchParticipant;
  t: T;
}) {
  return (
    <button
      className={`grid ${COLS} ${MIN_W} items-center gap-2 border-b border-zinc-100 dark:border-zinc-700 px-3 py-2 text-left transition last:border-b-0 hover:bg-rose-50 dark:hover:bg-zinc-800`}
      onClick={onSelect}
      type="button"
    >
      {/* Player */}
      <div className="flex min-w-0 items-center gap-2">
        <ChampionImage championName={participant.championName} imageUrl={imageUrl} size="xs" />
        <div className="min-w-0">
          <p className="truncate text-sm font-semibold text-zinc-950 dark:text-zinc-50">{participant.displayName}</p>
          <p className="mt-0.5 truncate text-xs text-zinc-500 dark:text-zinc-400">{participant.championName}</p>
        </div>
      </div>

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
    </button>
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
  t,
}: {
  comparison: PostMatchDetail["comparison"];
  t: T;
}) {
  return (
    <div className="grid gap-2 md:grid-cols-5">
      <Leader label="KDA" leader={comparison.highestKda} t={t} />
      <Leader label="CS" leader={comparison.mostCs} t={t} />
      <Leader label={t("analysis.gold")} leader={comparison.mostGold} t={t} />
      <Leader label={t("analysis.damage")} leader={comparison.mostDamage} t={t} />
      <Leader label={t("analysis.vision")} leader={comparison.highestVision} t={t} />
    </div>
  );
}

function Leader({
  label,
  leader,
  t,
}: {
  label: string;
  leader: ParticipantMetricLeader | null;
  t: T;
}) {
  return (
    <div className="rounded-md border border-zinc-200 dark:border-zinc-700 bg-zinc-50 dark:bg-zinc-800 px-3 py-2">
      <p className="text-xs font-medium uppercase tracking-wide text-zinc-500 dark:text-zinc-400">{label}</p>
      <p className="mt-1 truncate text-sm font-semibold text-zinc-950 dark:text-zinc-50">
        {leader?.displayName ?? t("common.unavailable")}
      </p>
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
