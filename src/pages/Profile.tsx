import { useEffect, useMemo, useState } from "react";

import { useAppCore, useLeagueAssets } from "../state/AppStateProvider";
import { Metric, RefreshIcon } from "../components/common";
import { PlaystyleTags } from "../components/PlaystyleTags";
import { fetchChatMe, fetchLeagueSelfSnapshot, fetchPlaystyleProfile, setChatStatus } from "../backend/leagueClient";
import { formatTimestamp, formatLeaguePhase, type T } from "../utils/formatting";
import type { ChampionMasteryEntry, ChampionRecordSummary, ChatAvailability, KdaTag, LeagueSelfSnapshot, PlaystyleProfile, RankedQueue, RankedQueueSummary, RecentChampionSummary } from "../backend/types";

// The mastery list is lifetime-deep, but per-champion W/L can only come from
// match history. Pull a wider window than the shared 6-game snapshot so recently
// played mastery champions show a meaningful record.
const RECORD_MATCH_WINDOW = 50;
import type { TranslationKey } from "../i18n";
import type { EffectiveLanguage } from "../i18n";
import { rankTierLabel, romanToNumber } from "./selfHistoryOverlayUtils";

const MASTERY_INITIAL_SHOW = 5;

export function Profile() {
  const {
    leagueSelfSnapshot,
    isLeagueClientLoading,
    refreshLeagueClient,
    effectiveLanguage,
    t,
  } = useAppCore();
  const { leagueImages, loadLeagueChampionIcon, loadLeagueProfileIcon, loadLeagueRankTierIcon } = useLeagueAssets();
  const [masteryExpanded, setMasteryExpanded] = useState(false);
  const league = leagueSelfSnapshot;
  const summoner = league?.summoner ?? null;
  const profileIconId = summoner?.profileIconId ?? null;
  const profileIconUrl = profileIconId ? leagueImages.profileIcons[profileIconId] : undefined;
  const soloDuo = league?.rankedQueues.find((queue) => queue.queue === "soloDuo");
  const flex = league?.rankedQueues.find((queue) => queue.queue === "flex");
  const topChampions = league?.recentPerformance.topChampions ?? [];
  const topMastery = summoner?.topMastery ?? [];
  const visibleMastery = masteryExpanded ? topMastery : topMastery.slice(0, MASTERY_INITIAL_SHOW);

  // Wider-window snapshot fetched only for per-champion W/L. Kept separate from
  // the shared global snapshot so it doesn't disturb other surfaces. Falls back
  // to the global snapshot's (shallow) records until it arrives.
  const [recordSnapshot, setRecordSnapshot] = useState<LeagueSelfSnapshot | null>(null);
  const hasSummoner = Boolean(summoner);
  useEffect(() => {
    if (!hasSummoner) {
      setRecordSnapshot(null);
      return;
    }
    let cancelled = false;
    void (async () => {
      try {
        const snap = await fetchLeagueSelfSnapshot({ matchLimit: RECORD_MATCH_WINDOW });
        if (!cancelled) setRecordSnapshot(snap);
      } catch {
        // Client offline/unavailable — keep falling back to the global snapshot.
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [hasSummoner]);

  const recordByChampion = useMemo(() => {
    const records = recordSnapshot?.championRecords ?? league?.championRecords ?? [];
    const map = new Map<number, ChampionRecordSummary>();
    for (const record of records) {
      map.set(record.championId, record);
    }
    return map;
  }, [recordSnapshot, league]);

  useEffect(() => {
    void loadLeagueProfileIcon(profileIconId);
  }, [loadLeagueProfileIcon, profileIconId]);

  useEffect(() => {
    for (const queue of [soloDuo, flex]) {
      if (queue?.isRanked && queue.tier) {
        void loadLeagueRankTierIcon(queue.tier.toLowerCase());
      }
    }
  }, [loadLeagueRankTierIcon, soloDuo, flex]);

  useEffect(() => {
    for (const champion of topChampions) {
      void loadLeagueChampionIcon(champion.championId);
    }
  }, [loadLeagueChampionIcon, topChampions]);

  useEffect(() => {
    for (const entry of topMastery) {
      void loadLeagueChampionIcon(entry.championId);
    }
  }, [loadLeagueChampionIcon, topMastery]);

  return (
    <main className="min-h-0 flex-1 overflow-auto px-8 py-7">
      <div className="mx-auto flex w-full max-w-6xl flex-col gap-7">
        <header className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="text-sm font-medium uppercase tracking-wide text-rose-700">{t("profile.eyebrow")}</p>
            <h1 className="mt-2 text-3xl font-semibold text-zinc-950 dark:text-zinc-50">{t("profile.title")}</h1>
          </div>
          <div className="flex items-center gap-2">
            <button
              className="inline-flex h-10 items-center gap-2 rounded-md border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-900 px-3 text-sm font-medium text-zinc-800 dark:text-zinc-200 transition hover:border-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800 disabled:cursor-not-allowed disabled:opacity-60"
              disabled={isLeagueClientLoading}
              onClick={() => refreshLeagueClient({ matchLimit: 6 })}
              type="button"
            >
              <RefreshIcon />
              {isLeagueClientLoading ? t("common.refreshing") : t("common.refresh")}
            </button>
          </div>
        </header>

        {!league && isLeagueClientLoading && <StatePanel title={t("profile.loading")} body={t("profile.readingClient")} />}
        {league && !summoner && (
          <StatePanel title={profileStateTitle(league.status.phase, t)} body={league.status.message ?? t("profile.unavailable")} />
        )}

        {league && summoner && (
          <>
            <section className="grid gap-4 lg:grid-cols-[1fr_1.15fr]">
              <div className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 p-5 shadow-sm">
                <div className="flex items-center gap-4">
                  <LeagueImage
                    alt={`${summoner.displayName} profile icon`}
                    fallback={profileIconId ? String(profileIconId) : initials(summoner.displayName)}
                    size="large"
                    src={profileIconUrl}
                  />
                  <div className="min-w-0">
                    <p className="truncate text-2xl font-semibold text-zinc-950 dark:text-zinc-50">{summoner.displayName}</p>
                    <div className="mt-1 flex flex-wrap items-center gap-2">
                      <p className="text-sm font-medium text-zinc-500 dark:text-zinc-400">{t("profile.level")} {summoner.summonerLevel}</p>
                      {summoner.honorLevel !== null && summoner.honorLevel !== undefined && (
                        <HonorBadge level={summoner.honorLevel} t={t} />
                      )}
                    </div>
                  </div>
                </div>

                <div className="mt-5 grid gap-3 sm:grid-cols-2">
                  <Metric label={t("profile.client")} value={formatLeaguePhase(league.status.phase, t)} />
                  <Metric label={t("dashboard.updated")} value={formatTimestamp(league.refreshedAt, t)} />
                </div>
              </div>

              <div className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 p-5 shadow-sm">
                <div className="flex flex-wrap items-center justify-between gap-3">
                  <div>
                    <h2 className="text-base font-semibold text-zinc-950 dark:text-zinc-50">{t("profile.recentPerformance")}</h2>
                    <p className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">{performanceLabel(league.recentPerformance.matchCount, t)}</p>
                  </div>
                  <KdaBadge tag={league.recentPerformance.kdaTag} value={league.recentPerformance.averageKda} t={t} />
                </div>

                <div className="mt-5 grid gap-3 sm:grid-cols-3">
                  {topChampions.length > 0 ? (
                    topChampions.map((champion) => (
                      <ChampionCard
                        champion={champion}
                        imageUrl={champion.championId ? leagueImages.championIcons[champion.championId] : undefined}
                        key={`${champion.championId ?? champion.championName}-${champion.championName}`}
                        t={t}
                      />
                    ))
                  ) : (
                    <p className="text-sm text-zinc-500 dark:text-zinc-400 sm:col-span-3">{t("profile.noRecentChampion")}</p>
                  )}
                </div>
              </div>
            </section>

            <PersonalizationCard t={t} />

            <section className="grid gap-4 md:grid-cols-2">
              <RankedCard effectiveLanguage={effectiveLanguage} label={t("profile.soloDuo")} queue="soloDuo" rankTierIcons={leagueImages.rankTierIcons} summary={soloDuo} t={t} />
              <RankedCard effectiveLanguage={effectiveLanguage} label={t("profile.flex")} queue="flex" rankTierIcons={leagueImages.rankTierIcons} summary={flex} t={t} />
            </section>

            <PlaystyleCard enabled={hasSummoner} refreshedAt={league.refreshedAt} t={t} />

            <section className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 p-5 shadow-sm">
              <div className="flex items-center justify-between gap-3">
                <h2 className="text-base font-semibold text-zinc-950 dark:text-zinc-50">{t("profile.mastery")}</h2>
                {topMastery.length > MASTERY_INITIAL_SHOW && (
                  <button
                    type="button"
                    onClick={() => setMasteryExpanded((v) => !v)}
                    className="text-sm font-medium text-rose-700 hover:underline"
                  >
                    {masteryExpanded ? t("profile.showLess") : t("profile.showAll")}
                  </button>
                )}
              </div>

              {topMastery.length === 0 ? (
                <p className="mt-4 text-sm text-zinc-500 dark:text-zinc-400">{t("profile.noMastery")}</p>
              ) : (
                <div className="mt-4 grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
                  {visibleMastery.map((entry) => (
                    <MasteryCard
                      entry={entry}
                      imageUrl={leagueImages.championIcons[entry.championId]}
                      key={entry.championId}
                      record={recordByChampion.get(entry.championId)}
                      t={t}
                    />
                  ))}
                </div>
              )}
            </section>
          </>
        )}
      </div>
    </main>
  );
}

const AVAILABILITY_OPTIONS: ChatAvailability[] = ["chat", "away", "dnd", "offline"];
const MAX_STATUS_LEN = 100;

const AVAILABILITY_LABEL_KEYS: Record<ChatAvailability, TranslationKey> = {
  chat: "profile.personalization.availability.chat",
  away: "profile.personalization.availability.away",
  dnd: "profile.personalization.availability.dnd",
  offline: "profile.personalization.availability.offline",
};

// Dot colour mirrors how the League client renders each presence.
const AVAILABILITY_DOT: Record<ChatAvailability, string> = {
  chat: "bg-emerald-500",
  away: "bg-amber-500",
  dnd: "bg-rose-500",
  offline: "bg-zinc-400",
};

function PersonalizationCard({ t }: { t: T }) {
  const [statusMessage, setStatusMessage] = useState("");
  const [availability, setAvailability] = useState<ChatAvailability>("chat");
  const [saving, setSaving] = useState(false);
  const [feedback, setFeedback] = useState<{ kind: "success" | "error"; text: string } | null>(null);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const me = await fetchChatMe();
        if (cancelled) return;
        setStatusMessage((me.statusMessage ?? "").slice(0, MAX_STATUS_LEN));
        if (me.availability && (AVAILABILITY_OPTIONS as string[]).includes(me.availability)) {
          setAvailability(me.availability as ChatAvailability);
        }
      } catch {
        // Client offline/unavailable — keep the defaults; saving will surface
        // a clear error if it is still unreachable.
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const save = async () => {
    setSaving(true);
    setFeedback(null);
    try {
      await setChatStatus(statusMessage, availability);
      setFeedback({ kind: "success", text: t("profile.personalization.saved") });
    } catch (error) {
      setFeedback({
        kind: "error",
        text: error instanceof Error ? error.message : t("profile.personalization.failed"),
      });
    } finally {
      setSaving(false);
    }
  };

  return (
    <section className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 p-5 shadow-sm">
      <h2 className="text-base font-semibold text-zinc-950 dark:text-zinc-50">{t("profile.personalization.title")}</h2>
      <p className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">{t("profile.personalization.subtitle")}</p>

      <div className="mt-5 flex flex-col gap-2">
        <label className="text-sm font-medium text-zinc-700 dark:text-zinc-300" htmlFor="profile-status-message">
          {t("profile.personalization.statusLabel")}
        </label>
        <input
          id="profile-status-message"
          className="h-10 rounded-md border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-950 px-3 text-sm text-zinc-900 dark:text-zinc-100 outline-none transition focus:border-rose-500"
          maxLength={MAX_STATUS_LEN}
          onChange={(event) => setStatusMessage(event.target.value)}
          placeholder={t("profile.personalization.statusPlaceholder")}
          type="text"
          value={statusMessage}
        />
        <p className="self-end text-xs text-zinc-400 dark:text-zinc-500">{statusMessage.length} / {MAX_STATUS_LEN}</p>
      </div>

      <div className="mt-3 flex flex-col gap-2">
        <span className="text-sm font-medium text-zinc-700 dark:text-zinc-300">{t("profile.personalization.availabilityLabel")}</span>
        <div className="flex flex-wrap gap-2">
          {AVAILABILITY_OPTIONS.map((option) => {
            const isActive = option === availability;
            return (
              <button
                className={`inline-flex h-9 items-center gap-2 rounded-md border px-3 text-sm font-medium transition ${
                  isActive
                    ? "border-rose-500 bg-rose-50 text-rose-700 dark:bg-rose-950/40 dark:text-rose-300"
                    : "border-zinc-300 dark:border-zinc-600 text-zinc-700 dark:text-zinc-300 hover:border-zinc-400 hover:bg-zinc-50 dark:hover:bg-zinc-800"
                }`}
                key={option}
                onClick={() => setAvailability(option)}
                type="button"
              >
                <span className={`h-2 w-2 rounded-full ${AVAILABILITY_DOT[option]}`} />
                {t(AVAILABILITY_LABEL_KEYS[option])}
              </button>
            );
          })}
        </div>
      </div>

      <div className="mt-5 flex flex-wrap items-center gap-3">
        <button
          className="inline-flex h-10 items-center rounded-md bg-rose-700 px-4 text-sm font-medium text-white transition hover:bg-rose-800 disabled:cursor-not-allowed disabled:opacity-60"
          disabled={saving}
          onClick={() => void save()}
          type="button"
        >
          {saving ? t("common.saving") : t("profile.personalization.apply")}
        </button>
        {feedback && (
          <span className={`text-sm ${feedback.kind === "success" ? "text-emerald-600 dark:text-emerald-400" : "text-rose-600 dark:text-rose-400"}`}>
            {feedback.text}
          </span>
        )}
      </div>
    </section>
  );
}

function HonorBadge({ level, t }: { level: number; t: T }) {
  const colors =
    level >= 5
      ? "border-emerald-200 bg-emerald-50 text-emerald-800"
      : level >= 3
        ? "border-sky-200 bg-sky-50 text-sky-800"
        : "border-zinc-200 dark:border-zinc-700 bg-zinc-50 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400";

  return (
    <span className={["inline-flex items-center gap-1 rounded-md border px-2 py-0.5 text-xs font-semibold", colors].join(" ")}>
      {t("profile.honor")} {level}
    </span>
  );
}

function MasteryCard({ entry, imageUrl, record, t }: { entry: ChampionMasteryEntry; imageUrl: string | undefined; record: ChampionRecordSummary | undefined; t: T }) {
  const decided = record ? record.wins + record.losses : 0;
  const winRate = decided > 0 ? Math.round((record!.wins / decided) * 100) : null;

  return (
    <div className="flex items-center gap-3 rounded-md border border-zinc-200 dark:border-zinc-700 bg-zinc-50 dark:bg-zinc-800 p-3">
      <LeagueImage alt={`${entry.championName} icon`} fallback={initials(entry.championName)} size="small" src={imageUrl} />
      <div className="min-w-0 flex-1">
        <p className="truncate text-sm font-semibold text-zinc-950 dark:text-zinc-50">{entry.championName}</p>
        <div className="mt-1 flex items-center gap-2">
          <span className={[
            "inline-flex items-center rounded px-1.5 py-0.5 text-xs font-semibold",
            entry.masteryLevel === 7
              ? "bg-rose-100 text-rose-700"
              : entry.masteryLevel >= 5
                ? "bg-amber-100 text-amber-700"
                : "bg-zinc-200 dark:bg-zinc-700 text-zinc-600 dark:text-zinc-400",
          ].join(" ")}>
            {t("profile.masteryLevel")} {entry.masteryLevel}
          </span>
          <span className="text-xs text-zinc-500 dark:text-zinc-400">
            {entry.masteryPoints.toLocaleString()} {t("profile.masteryPoints")}
          </span>
        </div>
        {winRate !== null ? (
          <p className="mt-1 text-xs text-zinc-500 dark:text-zinc-400">
            <span className="font-medium text-emerald-600 dark:text-emerald-400">{record!.wins} {t("profile.wins")}</span>
            {" · "}
            <span className="font-medium text-rose-600 dark:text-rose-400">{record!.losses} {t("profile.losses")}</span>
            {" · "}
            {winRate}% {t("profile.winRate")}
          </p>
        ) : (
          <p className="mt-1 text-xs text-zinc-400 dark:text-zinc-500">—</p>
        )}
      </div>
    </div>
  );
}

function ChampionCard({ champion, imageUrl, t }: { champion: RecentChampionSummary; imageUrl: string | undefined; t: (key: TranslationKey) => string }) {
  return (
    <div className="flex min-w-0 items-center gap-3 rounded-md border border-zinc-200 dark:border-zinc-700 bg-zinc-50 dark:bg-zinc-800 p-3">
      <LeagueImage alt={`${champion.championName} icon`} fallback={initials(champion.championName)} size="small" src={imageUrl} />
      <div className="min-w-0">
        <p className="truncate text-sm font-semibold text-zinc-950 dark:text-zinc-50">{champion.championName}</p>
        <p className="mt-1 text-xs font-medium text-zinc-500 dark:text-zinc-400">{champion.games} {t("participant.recentMatches")}</p>
      </div>
    </div>
  );
}

function PlaystyleCard({ enabled, refreshedAt, t }: { enabled: boolean; refreshedAt: string; t: T }) {
  const [profile, setProfile] = useState<PlaystyleProfile | null>(null);

  useEffect(() => {
    if (!enabled) {
      setProfile(null);
      return;
    }
    let cancelled = false;
    void (async () => {
      try {
        const result = await fetchPlaystyleProfile();
        if (!cancelled) setProfile(result);
      } catch {
        if (!cancelled) setProfile(null);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [enabled, refreshedAt]);

  const subtitle = profile && profile.gamesAnalyzed > 0
    ? t("profile.playstyle.subtitle").replace("{n}", String(profile.gamesAnalyzed))
    : null;

  return (
    <section className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 p-5 shadow-sm">
      <div>
        <h2 className="text-base font-semibold text-zinc-950 dark:text-zinc-50">{t("profile.playstyle.title")}</h2>
        {subtitle && <p className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">{subtitle}</p>}
      </div>
      <div className="mt-4">
        <PlaystyleTags profile={profile} t={t} />
      </div>
    </section>
  );
}

function RankedCard({
  effectiveLanguage,
  label,
  queue,
  rankTierIcons,
  summary,
  t,
}: {
  effectiveLanguage: EffectiveLanguage;
  label: string;
  queue: RankedQueue;
  rankTierIcons: Record<string, string>;
  summary: RankedQueueSummary | undefined;
  t: (key: TranslationKey) => string;
}) {
  const tier = summary?.isRanked && summary.tier ? summary.tier : null;
  const tierKey = tier ? tier.toLowerCase() : null;
  const iconUrl = tierKey ? (rankTierIcons[tierKey] ?? null) : null;
  const tierLabel = tier ? rankTierLabel(tier, effectiveLanguage) : null;
  const division = summary?.division ? ` ${romanToNumber(summary.division)}` : "";
  const lp = tier && summary?.leaguePoints !== null && summary?.leaguePoints !== undefined
    ? ` · ${summary.leaguePoints} LP`
    : "";
  const rankText = tierLabel ? `${tierLabel}${division}${lp}` : t("profile.unranked");

  return (
    <div className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 p-5 shadow-sm">
      <p className="text-xs font-medium uppercase tracking-wide text-zinc-500 dark:text-zinc-400">{label}</p>

      <div className="mt-3 flex flex-col items-center gap-2">
        {iconUrl ? (
          <div className="h-36 w-36 shrink-0 overflow-hidden">
            <img alt={tierLabel ?? ""} className="h-full w-full scale-[5.0] object-contain drop-shadow-sm" src={iconUrl} />
          </div>
        ) : (
          <div className="h-36 w-36 shrink-0 rounded-full bg-zinc-100 dark:bg-zinc-800" />
        )}
        <div className="text-center">
          <p className="text-xl font-semibold text-zinc-950 dark:text-zinc-50">{rankText}</p>
          <p className="mt-0.5 text-sm text-zinc-500 dark:text-zinc-400">
            {queue === "soloDuo" ? t("profile.rankedSolo") : t("profile.rankedFlex")}
          </p>
        </div>
      </div>

      <div className="mt-4 grid gap-3 sm:grid-cols-3">
        <Metric label={t("profile.wins")} value={summary ? String(summary.wins) : "0"} />
        <Metric label={t("profile.losses")} value={summary ? String(summary.losses) : "0"} />
        <Metric label={t("profile.winRate")} value={summary ? formatWinRate(summary) : "0%"} />
      </div>
    </div>
  );
}

function LeagueImage({ alt, fallback, size, src }: { alt: string; fallback: string; size: "large" | "small"; src: string | undefined }) {
  const className =
    size === "large"
      ? "h-24 w-24 rounded-lg text-lg"
      : "h-12 w-12 rounded-md text-sm";

  if (src) {
    return <img alt={alt} className={`${className} shrink-0 border border-zinc-200 dark:border-zinc-700 object-cover`} src={src} />;
  }

  return (
    <div className={`${className} flex shrink-0 items-center justify-center border border-zinc-200 dark:border-zinc-700 bg-zinc-100 dark:bg-zinc-900 font-semibold text-zinc-500 dark:text-zinc-400`}>
      {fallback}
    </div>
  );
}

function KdaBadge({ tag, value, t }: { tag: KdaTag; value: number | null; t: (key: TranslationKey) => string }) {
  const tone =
    tag === "high"
      ? "border-emerald-200 bg-emerald-50 text-emerald-800"
      : tag === "standard"
        ? "border-amber-200 bg-amber-50 text-amber-800"
        : "border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 text-zinc-600 dark:text-zinc-400";
  const label = value === null ? `KDA ${t("common.unavailable")}` : `Avg KDA ${value.toFixed(1)}`;

  return <span className={["rounded-md border px-2.5 py-1 text-xs font-semibold", tone].join(" ")}>{label}</span>;
}

function StatePanel({ title, body }: { title: string; body: string }) {
  return (
    <section className="rounded-lg border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 p-6 shadow-sm">
      <h2 className="text-base font-semibold text-zinc-950 dark:text-zinc-50">{title}</h2>
      <p className="mt-2 text-sm text-zinc-500 dark:text-zinc-400">{body}</p>
    </section>
  );
}

function formatWinRate(summary: RankedQueueSummary) {
  const total = summary.wins + summary.losses;

  if (total === 0) {
    return "0%";
  }

  return `${Math.round((summary.wins / total) * 100)}%`;
}

function profileStateTitle(phase: string, t: (key: TranslationKey) => string) {
  if (phase === "notLoggedIn") {
    return t("profile.loginRequired");
  }

  if (phase === "notRunning") {
    return t("profile.clientNotRunning");
  }

  return t("profile.unavailable");
}

function performanceLabel(matchCount: number, t: (key: TranslationKey) => string) {
  if (matchCount === 0) {
    return t("profile.noRecentChampion");
  }

  return `${matchCount} ${t("participant.recentMatches")}`;
}

function initials(value: string) {
  return value
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase() ?? "")
    .join("");
}
