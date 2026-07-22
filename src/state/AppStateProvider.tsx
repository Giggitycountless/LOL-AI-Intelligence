import { startTransition, useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { emit } from "@tauri-apps/api/event";

import { clearActivityEntries, createActivityNote, listActivityEntries } from "../backend/activity";
import { listenWithCleanup, SETTINGS_CHANGED_EVENT } from "../backend/events";
import { exportLocalData, importLocalData } from "../backend/dataTools";
import {
  fetchLeagueSelfSnapshot,
  fetchRankedChampionStats,
  refreshRankedChampionStats,
  fetchAdvisorData,
  refreshAdvisorData,
  clearPlayerNote as clearPlayerNoteCommand,
  fetchPostMatchDetail,
  fetchPostMatchParticipantProfile,
  savePlayerNote as savePlayerNoteCommand,
  fetchChampSelectSnapshot,
  fetchChampSelectAdvisorSnapshot,
  fetchLiveOverlaySnapshot,
} from "../backend/leagueClient";
import { saveSettings } from "../backend/settings";
import { createTranslator, resolveEffectiveLanguage, type EffectiveLanguage, type TranslationKey } from "../i18n";
import { fetchAppState } from "../backend/system";
import { useAsyncAction } from "./useAsyncAction";
import { participantProfileKey } from "./utils";
import type {
  ActivityEntry,
  ActivityListInput,
  ActivityNoteInput,
  AdvisorDataInput,
  AdvisorDataRefreshInput,
  AdvisorDataResponse,
  AppSnapshot,
  AppLanguagePreference,
  AutoAcceptStatus,
  ChampSelectSnapshot,
  ChampSelectAdvisorSnapshot,
  Feedback,
  LeagueSelfSnapshot,
  LeagueSelfSnapshotInput,
  LiveOverlaySnapshot,
  ParticipantPublicProfile,
  ParticipantPublicProfileInput,
  PlayerNoteView,
  PostMatchDetail,
  RankedChampionRefreshInput,
  RankedChampionStatsInput,
  RankedChampionStatsResponse,
  SavePlayerNoteInput,
  SaveSettingsInput,
} from "../backend/types";
import type {
  AppCoreContextValue,
  LeagueAssetsContextValue,
  ChampSelectContextValue,
  AdvisorContextValue,
  LeagueImageUrls,
  AppWindowMode,
} from "./types";
import { AppCoreContext, LeagueAssetsContext, ChampSelectContext, AdvisorContext, AppStateContext } from "./hooks";
import { useAssetLoader, leagueGameAssetKey } from "./useAssetLoader";
import { useAppEffects } from "./useOverlayEvents";
import { champSelectFingerprint as _champSelectFingerprint, errorMessage } from "./utils";

export function AppStateProvider({ children, mode = "main" }: { children: ReactNode; mode?: AppWindowMode }) {
  const [snapshot, setSnapshot] = useState<AppSnapshot | null>(null);
  const [activityEntries, setActivityEntries] = useState<ActivityEntry[]>([]);
  const [leagueSelfSnapshot, setLeagueSelfSnapshot] = useState<LeagueSelfSnapshot | null>(null);
  const [champSelectSnapshot, setChampSelectSnapshot] = useState<ChampSelectSnapshot | null>(null);
  const [champSelectAdvisorSnapshot, setChampSelectAdvisorSnapshot] = useState<ChampSelectAdvisorSnapshot | null>(null);
  const [liveOverlaySnapshot, setLiveOverlaySnapshot] = useState<LiveOverlaySnapshot | null>(null);
  const [rankedChampionStats, setRankedChampionStats] = useState<RankedChampionStatsResponse | null>(null);
  const [advisorData, setAdvisorData] = useState<AdvisorDataResponse | null>(null);
  const [advisorDataError, setAdvisorDataError] = useState<string | null>(null);
  const [postMatchDetails, setPostMatchDetails] = useState<Record<number, PostMatchDetail>>({});
  const [participantProfiles, setParticipantProfiles] = useState<Record<string, ParticipantPublicProfile>>({});
  const [autoAcceptStatus, setAutoAcceptStatus] = useState<AutoAcceptStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isActivityLoading, setIsActivityLoading] = useState(false);
  const [isLeagueClientLoading, setIsLeagueClientLoading] = useState(false);
  const [isRankedChampionStatsLoading, setIsRankedChampionStatsLoading] = useState(false);
  const [isAdvisorDataLoading, setIsAdvisorDataLoading] = useState(false);
  const [feedback, setFeedback] = useState<Feedback | null>(null);
  const clearFeedback = useCallback(() => setFeedback(null), []);
  const { run } = useAsyncAction(setFeedback);
  const languagePreference = snapshot?.settings.language ?? "system";
  const effectiveLanguage = useMemo(() => resolveEffectiveLanguage(languagePreference), [languagePreference]);
  const t = useMemo(() => createTranslator(effectiveLanguage), [effectiveLanguage]);

  // ── Asset loader ──
  const [championDetailsById, setChampionDetailsById] = useState<Record<number, import("./types").LeagueChampionDetailsView>>({});
  const [leagueImages, setLeagueImages] = useState<LeagueImageUrls>({ profileIcons: {}, championIcons: {}, gameAssets: {}, rankTierIcons: {} });
  const assetLoader = useAssetLoader(setLeagueImages, setChampionDetailsById);

  // ── Data actions ──
  const refresh = useCallback(async () => {
    return run(() => fetchAppState(), setIsLoading, setSnapshot);
  }, [run]);

  const loadActivityEntriesAction = useCallback(async (input: ActivityListInput) => {
    return run(
      () => listActivityEntries(input),
      setIsActivityLoading,
      (response) => startTransition(() => setActivityEntries(response.records)),
    );
  }, [run]);

  const refreshLeagueClientAction = useCallback(async (input: LeagueSelfSnapshotInput = { matchLimit: 6 }) => {
    return run(
      () => fetchLeagueSelfSnapshot(input),
      setIsLeagueClientLoading,
      (response) => startTransition(() => setLeagueSelfSnapshot(response)),
      undefined, // use default 30s timeout
      true,       // suppress error feedback — league client may not be running
    );
  }, [run]);

  const loadRankedChampionStatsAction = useCallback(async (input: RankedChampionStatsInput) => {
    return run(
      () => fetchRankedChampionStats(input),
      setIsRankedChampionStatsLoading,
      (response) => startTransition(() => setRankedChampionStats(response)),
    );
  }, [run]);

  const refreshRankedChampionStatsAction = useCallback(async (input: RankedChampionRefreshInput) => {
    return run(
      () => refreshRankedChampionStats(input),
      setIsRankedChampionStatsLoading,
      (response) => {
        startTransition(() => setRankedChampionStats(response));
        setFeedback({
          kind: response.dataStatus === "staleCache" ? "error" : "success",
          message: response.statusMessage ?? "Ranked champion data refreshed",
        });
      },
    );
  }, [run]);

  const loadAdvisorDataAction = useCallback(async (input: AdvisorDataInput) => {
    setAdvisorDataError(null);
    const ok = await run(
      () => fetchAdvisorData(input),
      setIsAdvisorDataLoading,
      (response) => startTransition(() => setAdvisorData(response)),
    );
    if (!ok) setAdvisorDataError("Failed to load advisor data");
    return ok;
  }, [run]);

  const refreshAdvisorDataAction = useCallback(async (input: AdvisorDataRefreshInput) => {
    setAdvisorDataError(null);
    const ok = await run(
      () => refreshAdvisorData(input),
      setIsAdvisorDataLoading,
      (response) => {
        startTransition(() => setAdvisorData(response));
        setFeedback({
          kind: response.dataStatus === "staleCache" ? "error" : "success",
          message: response.statusMessage ?? "Advisor data refreshed",
        });
      },
    );
    if (!ok) setAdvisorDataError("Failed to refresh advisor data");
    return ok;
  }, [run]);

  // ── Champ select actions ──
  const champSelectFingerprintRef = useRef("");
  // Bumped every time a snapshot with new content is applied, from any
  // source (real-time push or manual refresh). Lets an in-flight manual
  // refresh detect that a newer snapshot already landed while it was
  // waiting, so it doesn't overwrite fresher data with a stale response.
  const champSelectApplySeqRef = useRef(0);

  const applyChampSelectSnapshotAction = useCallback((snapshot: ChampSelectSnapshot) => {
    const nextFingerprint = _champSelectFingerprint(snapshot);
    if (champSelectFingerprintRef.current === nextFingerprint) {
      return;
    }
    champSelectFingerprintRef.current = nextFingerprint;
    champSelectApplySeqRef.current += 1;
    startTransition(() => {
      setChampSelectSnapshot(snapshot);
    });
    for (const player of snapshot.players) {
      void assetLoader.loadLeagueChampionIcon(player.championId);
      for (const match of player.recentStats?.recentMatches ?? []) {
        void assetLoader.loadLeagueChampionIcon(match.championId);
      }
    }
  }, [assetLoader.loadLeagueChampionIcon]);

  const refreshChampSelectSnapshotAction = useCallback(async () => {
    const seqAtStart = champSelectApplySeqRef.current;
    return run(() => fetchChampSelectSnapshot(6), undefined, (snapshot) => {
      if (champSelectApplySeqRef.current !== seqAtStart) return;
      applyChampSelectSnapshotAction(snapshot);
    });
  }, [run, applyChampSelectSnapshotAction]);

  const refreshChampSelectAdvisorSnapshotAction = useCallback(async () => {
    try {
      const snapshot = await fetchChampSelectAdvisorSnapshot(6);
      startTransition(() => setChampSelectAdvisorSnapshot(snapshot));
      for (const player of snapshot.players) {
        void assetLoader.loadLeagueChampionIcon(player.championId);
        for (const match of player.recentStats?.recentMatches ?? []) {
          void assetLoader.loadLeagueChampionIcon(match.championId);
        }
      }
      return true;
    } catch {
      return false;
    }
  }, [assetLoader.loadLeagueChampionIcon]);

  const refreshLiveOverlaySnapshotAction = useCallback(async () => {
    try {
      const snapshot = await fetchLiveOverlaySnapshot();
      startTransition(() => setLiveOverlaySnapshot(snapshot));
      return true;
    } catch {
      startTransition(() => setLiveOverlaySnapshot(null));
      return false;
    }
  }, []);

  // ── Effects ──
  useEffect(() => {
    void refresh();
    if (mode === "main") {
      void refreshLeagueClientAction({ matchLimit: 6 });
    }
  }, [mode, refresh, refreshLeagueClientAction]);

  useEffect(() => {
    return assetLoader.cleanup;
  }, [assetLoader.cleanup]);

  // Non-main windows (e.g. the overlay) don't refresh on their own when the
  // user changes settings in the main window. Listen for the save broadcast and
  // re-pull the snapshot so language/theme stay in sync live. The main window
  // already refreshes inline in saveSettingsAction, so it skips this.
  useEffect(() => {
    if (mode === "main") {
      return;
    }
    return listenWithCleanup(SETTINGS_CHANGED_EVENT, () => {
      void refresh();
    });
  }, [mode, refresh]);

  useAppEffects({
    mode,
    champSelectFingerprintRef,
    setChampSelectSnapshot,
    setChampSelectAdvisorSnapshot,
    setLiveOverlaySnapshot,
    setAutoAcceptStatus,
    setFeedback,
    onChampSelectUpdate: applyChampSelectSnapshotAction,
  });

  // ── Settings / data actions ──
  const saveSettingsAction = useCallback(
    async (settings: SaveSettingsInput) => {
      try {
        await saveSettings(settings);
        // Confirm the save immediately — the snapshot refresh below can take a
        // moment and shouldn't delay the user's feedback.
        setFeedback({ kind: "success", message: t("feedback.settingsSaved") });
        await refresh();
        // Notify other webviews (overlay / aux windows) to re-pull settings so
        // language and theme stay in sync without a reopen.
        void emit(SETTINGS_CHANGED_EVENT);
        return true;
      } catch (caught: unknown) {
        setFeedback({ kind: "error", message: errorMessage(caught) });
        return false;
      }
    },
    [refresh, t],
  );

  const setLanguagePreferenceAction = useCallback(
    async (language: AppLanguagePreference) => {
      const current = snapshot?.settings ?? snapshot?.settingsDefaults;
      if (!current) {
        return false;
      }

      return saveSettingsAction({
        startupPage: current.startupPage,
        language,
        theme: current.theme,
        compactMode: current.compactMode,
        activityLimit: current.activityLimit,
        autoAcceptEnabled: current.autoAcceptEnabled,
        autoPickEnabled: current.autoPickEnabled,
        autoPickChampionId: current.autoPickChampionId,
        autoPickDelaySeconds: current.autoPickDelaySeconds,
        autoBanEnabled: current.autoBanEnabled,
        autoBanChampionId: current.autoBanChampionId,
        autoBanDelaySeconds: current.autoBanDelaySeconds,
        autoHonorEnabled: current.autoHonorEnabled,
        aiBaseUrl: (current as import("../backend/types").AppSettings).aiBaseUrl ?? null,
        aiApiKey: (current as import("../backend/types").AppSettings).aiApiKey ?? null,
        aiModel: (current as import("../backend/types").AppSettings).aiModel ?? null,
      });
    },
    [saveSettingsAction, snapshot?.settings, snapshot?.settingsDefaults],
  );

  const createActivityNoteAction = useCallback(
    async (input: ActivityNoteInput) => {
      try {
        await createActivityNote(input);
        await refresh();
        setFeedback({ kind: "success", message: t("feedback.activityNoteSaved") });
        return true;
      } catch (caught: unknown) {
        setFeedback({ kind: "error", message: errorMessage(caught) });
        return false;
      }
    },
    [refresh, t],
  );

  const clearActivityEntriesAction = useCallback(
    async (confirm: boolean) => {
      try {
        const result = await clearActivityEntries(confirm);
        setActivityEntries([]);
        await refresh();
        setFeedback({ kind: "success", message: t("feedback.activityCleared") });
        return true;
      } catch (caught: unknown) {
        setFeedback({ kind: "error", message: errorMessage(caught) });
        return false;
      }
    },
    [refresh, t],
  );

  const exportLocalDataAction = useCallback(async () => {
    try {
      const data = await exportLocalData();
      setFeedback({ kind: "success", message: t("feedback.localDataExported") });
      return JSON.stringify(data, null, 2);
    } catch (caught: unknown) {
      setFeedback({ kind: "error", message: errorMessage(caught) });
      return null;
    }
  }, [t]);

  const importLocalDataAction = useCallback(
    async (json: string) => {
      try {
        const result = await importLocalData(json);
        await refresh();
        setFeedback({ kind: "success", message: t("feedback.localDataImported") });
        return true;
      } catch (caught: unknown) {
        setFeedback({ kind: "error", message: errorMessage(caught) });
        return false;
      }
    },
    [refresh],
  );

  const loadPostMatchDetailAction = useCallback(async (gameId: number) => {
    return run(
      () => fetchPostMatchDetail(gameId),
      undefined,
      (detail) => startTransition(() => setPostMatchDetails((current) => ({ ...current, [gameId]: detail }))),
    );
  }, [run]);

  const loadParticipantProfileAction = useCallback(async (input: ParticipantPublicProfileInput) => {
    return run(
      () => fetchPostMatchParticipantProfile(input),
      undefined,
      (profile) => startTransition(() => setParticipantProfiles((current) => ({ ...current, [participantProfileKey(input.gameId, input.participantId)]: profile }))),
    );
  }, [run]);

  const savePlayerNoteAction = useCallback(async (input: SavePlayerNoteInput) => {
    try {
      const note = await savePlayerNoteCommand(input);
      await Promise.all([
        loadPostMatchDetailAction(input.gameId),
        loadParticipantProfileAction({ gameId: input.gameId, participantId: input.participantId, recentLimit: 6 }),
      ]);
      setFeedback({ kind: "success", message: t("feedback.playerNoteSaved") });
      return note;
    } catch (caught: unknown) {
      setFeedback({ kind: "error", message: errorMessage(caught) });
      return null;
    }
  }, [loadParticipantProfileAction, loadPostMatchDetailAction, t]);

  const clearPlayerNoteAction = useCallback(async (gameId: number, participantId: number) => {
    try {
      await clearPlayerNoteCommand({ gameId, participantId });
      await Promise.all([
        loadPostMatchDetailAction(gameId),
        loadParticipantProfileAction({ gameId, participantId, recentLimit: 6 }),
      ]);
      setFeedback({ kind: "success", message: t("feedback.playerNoteCleared") });
      return true;
    } catch (caught: unknown) {
      setFeedback({ kind: "error", message: errorMessage(caught) });
      return false;
    }
  }, [loadParticipantProfileAction, loadPostMatchDetailAction, t]);

  // ── Context values ──
  const coreValue = useMemo<AppCoreContextValue>(
    () => ({
      snapshot,
      activityEntries,
      leagueSelfSnapshot,
      rankedChampionStats,
      postMatchDetails,
      participantProfiles,
      autoAcceptStatus,
      isLoading,
      isActivityLoading,
      isLeagueClientLoading,
      isRankedChampionStatsLoading,
      feedback,
      languagePreference,
      effectiveLanguage,
      t,
      clearFeedback,
      refresh,
      loadActivityEntries: loadActivityEntriesAction,
      refreshLeagueClient: refreshLeagueClientAction,
      loadRankedChampionStats: loadRankedChampionStatsAction,
      refreshRankedChampionStats: refreshRankedChampionStatsAction,
      saveSettings: saveSettingsAction,
      setLanguagePreference: setLanguagePreferenceAction,
      createActivityNote: createActivityNoteAction,
      clearActivityEntries: clearActivityEntriesAction,
      exportLocalData: exportLocalDataAction,
      importLocalData: importLocalDataAction,
      loadPostMatchDetail: loadPostMatchDetailAction,
      loadParticipantProfile: loadParticipantProfileAction,
      savePlayerNote: savePlayerNoteAction,
      clearPlayerNote: clearPlayerNoteAction,
    }),
    [
      activityEntries,
      autoAcceptStatus,
      clearFeedback,
      clearActivityEntriesAction,
      clearPlayerNoteAction,
      createActivityNoteAction,
      exportLocalDataAction,
      feedback,
      languagePreference,
      effectiveLanguage,
      importLocalDataAction,
      isActivityLoading,
      isLeagueClientLoading,
      isRankedChampionStatsLoading,
      isLoading,
      leagueSelfSnapshot,
      loadRankedChampionStatsAction,
      loadActivityEntriesAction,
      loadParticipantProfileAction,
      loadPostMatchDetailAction,
      participantProfiles,
      postMatchDetails,
      rankedChampionStats,
      refresh,
      refreshLeagueClientAction,
      refreshRankedChampionStatsAction,
      savePlayerNoteAction,
      saveSettingsAction,
      setLanguagePreferenceAction,
      snapshot,
      t,
    ],
  );

  const assetsValue = useMemo<LeagueAssetsContextValue>(
    () => ({
      championDetailsById,
      leagueImages,
      loadLeagueProfileIcon: assetLoader.loadLeagueProfileIcon,
      loadLeagueChampionIcon: assetLoader.loadLeagueChampionIcon,
      loadLeagueChampionDetails: assetLoader.loadLeagueChampionDetails,
      loadLeagueGameAsset: assetLoader.loadLeagueGameAsset,
      loadLeagueRankTierIcon: assetLoader.loadLeagueRankTierIcon,
    }),
    [
      championDetailsById,
      leagueImages,
      assetLoader.loadLeagueChampionDetails,
      assetLoader.loadLeagueChampionIcon,
      assetLoader.loadLeagueGameAsset,
      assetLoader.loadLeagueProfileIcon,
      assetLoader.loadLeagueRankTierIcon,
    ],
  );

  const champSelectValue = useMemo<ChampSelectContextValue>(
    () => ({
      champSelectSnapshot,
      refreshChampSelectSnapshot: refreshChampSelectSnapshotAction,
    }),
    [champSelectSnapshot, refreshChampSelectSnapshotAction],
  );

  const advisorValue = useMemo<AdvisorContextValue>(
    () => ({
      advisorData,
      champSelectAdvisorSnapshot,
      liveOverlaySnapshot,
      isAdvisorDataLoading,
      advisorDataError,
      loadAdvisorData: loadAdvisorDataAction,
      refreshAdvisorData: refreshAdvisorDataAction,
      refreshChampSelectAdvisorSnapshot: refreshChampSelectAdvisorSnapshotAction,
      refreshLiveOverlaySnapshot: refreshLiveOverlaySnapshotAction,
    }),
    [
      advisorData,
      advisorDataError,
      champSelectAdvisorSnapshot,
      isAdvisorDataLoading,
      liveOverlaySnapshot,
      loadAdvisorDataAction,
      refreshAdvisorDataAction,
      refreshChampSelectAdvisorSnapshotAction,
      refreshLiveOverlaySnapshotAction,
    ],
  );

  const legacyValue = useMemo<import("./types").AppStateContextValue>(
    () => ({
      ...coreValue,
      ...assetsValue,
      ...champSelectValue,
      ...advisorValue,
    }),
    [advisorValue, assetsValue, champSelectValue, coreValue],
  );

  return (
    <AppCoreContext.Provider value={coreValue}>
      <LeagueAssetsContext.Provider value={assetsValue}>
        <ChampSelectContext.Provider value={champSelectValue}>
          <AdvisorContext.Provider value={advisorValue}>
            <AppStateContext.Provider value={legacyValue}>{children}</AppStateContext.Provider>
          </AdvisorContext.Provider>
        </ChampSelectContext.Provider>
      </LeagueAssetsContext.Provider>
    </AppCoreContext.Provider>
  );
}

// ── Re-exports for backwards compatibility ──
export {
  useAppState,
  useAppCore,
  useLeagueAssets,
  useChampSelect,
  useAdvisor,
} from "./hooks";
export type { AppWindowMode, LeagueChampionAbilityView, LeagueChampionDetailsView, LeagueGameAssetView } from "./types";
export { leagueGameAssetKey } from "./useAssetLoader";
