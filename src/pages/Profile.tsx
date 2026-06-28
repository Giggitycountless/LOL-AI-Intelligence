import { useEffect, useState } from "react";

import { useAppCore, useLeagueAssets } from "../state/AppStateProvider";
import { RefreshIcon } from "../components/common";
import { PlaystyleTags } from "../components/PlaystyleTags";
import { ProfileOverview, StatePanel } from "../components/profile/ProfileOverview";
import { fetchChatMe, fetchLeagueSelfSnapshot, fetchPlaystyleProfile, setChatStatus } from "../backend/leagueClient";
import { type T } from "../utils/formatting";
import type { ChatAvailability, LeagueSelfSnapshot, PlaystyleProfile } from "../backend/types";

// The mastery list is lifetime-deep, but per-champion W/L can only come from
// match history. Pull a wider window than the shared 6-game snapshot so recently
// played mastery champions show a meaningful record.
const RECORD_MATCH_WINDOW = 50;
import type { TranslationKey } from "../i18n";

export function Profile() {
  const {
    leagueSelfSnapshot,
    isLeagueClientLoading,
    refreshLeagueClient,
    effectiveLanguage,
    t,
  } = useAppCore();
  const { leagueImages, loadLeagueChampionIcon, loadLeagueProfileIcon, loadLeagueRankTierIcon } = useLeagueAssets();
  const league = leagueSelfSnapshot;
  const summoner = league?.summoner ?? null;

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
          <ProfileOverview
            championRecords={recordSnapshot?.championRecords ?? league.championRecords}
            effectiveLanguage={effectiveLanguage}
            leagueImages={leagueImages}
            loadLeagueChampionIcon={loadLeagueChampionIcon}
            loadLeagueProfileIcon={loadLeagueProfileIcon}
            loadLeagueRankTierIcon={loadLeagueRankTierIcon}
            personalizationSlot={<PersonalizationCard t={t} />}
            playstyleSlot={<PlaystyleCard enabled={hasSummoner} refreshedAt={league.refreshedAt} t={t} />}
            recentMatches={recordSnapshot?.recentMatches ?? league.recentMatches}
            rankedQueues={league.rankedQueues}
            recentPerformance={league.recentPerformance}
            refreshedAt={league.refreshedAt}
            statusPhase={league.status.phase}
            summoner={summoner}
            t={t}
          />
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

function profileStateTitle(phase: string, t: (key: TranslationKey) => string) {
  if (phase === "notLoggedIn") {
    return t("profile.loginRequired");
  }

  if (phase === "notRunning") {
    return t("profile.clientNotRunning");
  }

  return t("profile.unavailable");
}
