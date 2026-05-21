import { useEffect, useMemo, useState } from "react";

import type { AdvisorNamedRef, AdvisorRecord, RankedChampionLane } from "../backend/types";
import { ChampionImage } from "../components/common";
import { useAdvisor, useAppCore, useLeagueAssets } from "../state/AppStateProvider";

const lanes: Array<{ id: RankedChampionLane; label: string }> = [
  { id: "top", label: "Top" },
  { id: "jungle", label: "Jungle" },
  { id: "middle", label: "Mid" },
  { id: "bottom", label: "Bot" },
  { id: "support", label: "Support" },
];

export function Advisor() {
  const { t } = useAppCore();
  const { advisorData, isAdvisorDataLoading, advisorDataError, loadAdvisorData, refreshAdvisorData } = useAdvisor();
  const { leagueImages, loadLeagueChampionIcon } = useLeagueAssets();
  const [lane, setLane] = useState<RankedChampionLane>("top");
  const activeAdvisorData = advisorData?.lane === lane ? advisorData : null;
  const records = activeAdvisorData?.records ?? [];
  const metadata = activeAdvisorData
    ? [activeAdvisorData.patch, activeAdvisorData.region, activeAdvisorData.queue, activeAdvisorData.tier].filter(Boolean).join(" / ")
    : "";

  useEffect(() => {
    void loadAdvisorData({ lane });
  }, [lane, loadAdvisorData]);

  useEffect(() => {
    for (const record of records) {
      void loadLeagueChampionIcon(record.championId);
    }
  }, [loadLeagueChampionIcon, records]);

  return (
    <main className="min-h-0 flex-1 overflow-auto px-8 py-7">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-7">
        <header className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="text-sm font-medium uppercase tracking-wide text-rose-700">{t("advisor.eyebrow")}</p>
            <h1 className="mt-2 text-3xl font-semibold text-zinc-950 dark:text-zinc-50">{t("advisor.title")}</h1>
          </div>
          <div className="flex items-center gap-2">
            <button
              className="inline-flex h-10 items-center gap-2 rounded-md border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-900 px-3 text-sm font-semibold text-zinc-700 dark:text-zinc-300 transition hover:border-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800 disabled:cursor-not-allowed disabled:opacity-60"
              disabled={isAdvisorDataLoading}
              onClick={() => void refreshAdvisorData({ lane })}
              type="button"
            >
              <RefreshIcon />
              <span>{isAdvisorDataLoading ? t("common.refreshing") : t("common.refresh")}</span>
            </button>
            <div className="rounded-md border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 px-3 py-2 text-sm font-medium text-zinc-600 dark:text-zinc-400">
              {records.length} {t("advisor.champions")}
            </div>
          </div>
        </header>

        <section className="flex flex-wrap gap-2">
          {lanes.map((laneOption) => (
            <button
              className={[
                "inline-flex h-10 items-center rounded-md border px-3 text-sm font-semibold transition",
                lane === laneOption.id
                  ? "border-rose-700 bg-rose-700 text-white shadow-sm"
                  : "border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-900 text-zinc-700 dark:text-zinc-300 hover:border-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800",
              ].join(" ")}
              key={laneOption.id}
              onClick={() => setLane(laneOption.id)}
              type="button"
            >
              {laneOption.label}
            </button>
          ))}
        </section>

        <section className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 shadow-sm">
          <div className="flex flex-wrap items-center justify-between gap-3 border-b border-zinc-200 dark:border-zinc-700 px-5 py-4">
            <div>
              <h2 className="text-base font-semibold text-zinc-950 dark:text-zinc-50">{laneLabel(lane)} {t("advisor.recommendations")}</h2>
              <p className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">
                {activeAdvisorData?.source ?? t("advisor.localSample")} {metadata ? `/ ${metadata}` : ""}
              </p>
            </div>
            {activeAdvisorData?.statusMessage && (
              <span className="rounded-md border border-zinc-200 dark:border-zinc-700 bg-zinc-50 dark:bg-zinc-800 px-3 py-1.5 text-xs font-semibold text-zinc-600 dark:text-zinc-400">
                {activeAdvisorData.statusMessage}
              </span>
            )}
          </div>

          {records.length === 0 ? (
            <div className="px-5 py-8 text-sm font-medium text-zinc-500 dark:text-zinc-400">
              {isAdvisorDataLoading ? t("advisor.loading") : advisorDataError ?? t("advisor.noData")}
            </div>
          ) : (
            <div className="grid gap-4 p-4 xl:grid-cols-2">
              {records.map((record) => (
                <AdvisorCard
                  imageUrl={leagueImages.championIcons[record.championId]}
                  key={`${record.lane}-${record.championId}`}
                  record={record}
                  t={t}
                />
              ))}
            </div>
          )}
        </section>
      </div>
    </main>
  );
}

function AdvisorCard({ imageUrl, record, t }: { imageUrl: string | undefined; record: AdvisorRecord; t: (key: string) => string }) {
  const spellNames = record.summonerSpells.map((spell) => spell.name).join(" + ");
  const runeText = [
    `${record.runes.primaryStyle}: ${names(record.runes.primaryRunes)}`,
    `${record.runes.secondaryStyle}: ${names(record.runes.secondaryRunes)}`,
    record.runes.statShards.length > 0 ? `Shards: ${record.runes.statShards.join(" / ")}` : "",
  ].filter(Boolean).join(" / ");

  return (
    <article className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 p-4 shadow-sm">
      <div className="flex min-w-0 items-start gap-3">
        <ChampionImage championName={record.championName} imageUrl={imageUrl} size="lg" />
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <h3 className="truncate text-base font-semibold text-zinc-950 dark:text-zinc-50">{record.championName}</h3>
            <span className="rounded bg-rose-50 px-2 py-0.5 text-xs font-bold uppercase text-rose-700">{laneLabel(record.lane)}</span>
          </div>
          <div className="mt-2 grid gap-2 text-xs font-semibold text-zinc-600 dark:text-zinc-400 sm:grid-cols-5">
            <Metric label="Score" value={formatDecimal(record.overallScore)} />
            <Metric label={t("advisor.winRate")} value={`${record.winRate.toFixed(1)}%`} />
            <Metric label={t("advisor.pickRate")} value={`${record.pickRate.toFixed(1)}%`} />
            <Metric label={t("advisor.banRate")} value={`${record.banRate.toFixed(1)}%`} />
            <Metric label={t("advisor.games")} value={formatGames(record.games)} />
          </div>
        </div>
      </div>

      <div className="mt-4 grid gap-3 lg:grid-cols-2">
        <InfoBlock label={t("advisor.runes")} value={runeText} />
        <InfoBlock label={t("advisor.spells")} value={spellNames || "-"} />
        <InfoBlock label={t("advisor.skillOrder")} value={`Max ${record.skillOrder.maxOrder.join(" > ")} / Early ${record.skillOrder.earlyOrder.join(" > ")}`} />
        <InfoBlock label={t("advisor.items")} value={buildPath(record.itemBuild.starter, record.itemBuild.core, record.itemBuild.boots)} />
        <InfoBlock label="Late build" value={names(record.itemBuild.late)} />
        <InfoBlock label="Situational" value={names(record.itemBuild.situational)} />
      </div>

      <div className="mt-3 grid gap-3 lg:grid-cols-2">
        <AdviceBlock title="Lane" body={record.laneAdvice} />
        <AdviceBlock title="Teamfights" body={record.teamfightAdvice} />
      </div>

      <div className="mt-3 grid gap-3 lg:grid-cols-3">
        <ListBlock title={t("advisor.powerSpikes")} values={record.powerSpikes.map((spike) => `${spike.timing}: ${spike.label}`)} />
        <ListBlock title={t("advisor.strongAgainst")} values={record.strongAgainst.map((matchup) => matchup.championName)} />
        <ListBlock title={t("advisor.weakAgainst")} values={record.weakAgainst.map((matchup) => matchup.championName)} />
      </div>
    </article>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-md border border-zinc-200 dark:border-zinc-700 bg-zinc-50 dark:bg-zinc-800 px-3 py-2">
      <p className="text-[11px] uppercase tracking-wide text-zinc-500 dark:text-zinc-400">{label}</p>
      <p className="mt-1 text-sm text-zinc-950 dark:text-zinc-50">{value}</p>
    </div>
  );
}

function InfoBlock({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-md border border-zinc-200 dark:border-zinc-700 bg-zinc-50 dark:bg-zinc-800 px-3 py-2">
      <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-500 dark:text-zinc-400">{label}</p>
      <p className="mt-1 text-sm font-medium leading-relaxed text-zinc-800 dark:text-zinc-200">{value || "-"}</p>
    </div>
  );
}

function AdviceBlock({ title, body }: { title: string; body: string }) {
  return (
    <div className="rounded-md border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 px-3 py-2">
      <p className="text-[11px] font-semibold uppercase tracking-wide text-rose-700">{title}</p>
      <p className="mt-1 text-sm font-medium leading-relaxed text-zinc-700 dark:text-zinc-300">{body}</p>
    </div>
  );
}

function ListBlock({ title, values }: { title: string; values: string[] }) {
  const displayValues = useMemo(() => (values.length > 0 ? values : ["-"]), [values]);

  return (
    <div className="rounded-md border border-zinc-200 dark:border-zinc-700 bg-zinc-50 dark:bg-zinc-800 px-3 py-2">
      <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-500 dark:text-zinc-400">{title}</p>
      <div className="mt-2 flex flex-wrap gap-1.5">
        {displayValues.map((value, index) => (
          <span className="rounded bg-white dark:bg-zinc-900 px-2 py-1 text-xs font-semibold text-zinc-700 dark:text-zinc-300" key={`${value}-${index}`}>
            {value}
          </span>
        ))}
      </div>
    </div>
  );
}

function names(values: AdvisorNamedRef[]) {
  return values.map((value) => value.name).join(" / ");
}

function buildPath(...groups: AdvisorNamedRef[][]) {
  return groups.map((group) => names(group)).filter(Boolean).join(" -> ");
}

function formatDecimal(value: number) {
  return value.toFixed(1).replace(/\.0$/, "");
}

function laneLabel(lane: RankedChampionLane) {
  switch (lane) {
    case "top":
      return "Top";
    case "jungle":
      return "Jungle";
    case "middle":
      return "Mid";
    case "bottom":
      return "Bot";
    case "support":
      return "Support";
  }
}

function formatGames(value: number) {
  if (value >= 1000) {
    return `${(value / 1000).toFixed(value >= 10000 ? 0 : 1)}k`;
  }

  return String(value);
}

function RefreshIcon() {
  return (
    <svg aria-hidden="true" className="h-4 w-4 shrink-0" viewBox="0 0 24 24" fill="none">
      <path d="M20 12a8 8 0 0 1-13.4 5.9" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
      <path d="M4 12A8 8 0 0 1 17.4 6.1" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
      <path d="M7 18H4v-3M17 6h3v3" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}
