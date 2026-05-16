import { useEffect, type Dispatch, type MutableRefObject, type SetStateAction } from "react";

import type { AutoAcceptStatus, ChampSelectAdvisorSnapshot, ChampSelectSnapshot, Feedback, LiveOverlaySnapshot } from "../backend/types";
import { listenWithCleanup } from "../backend/events";
import { fetchAutoAcceptStatus, fetchChampSelectAdvisorSnapshot, fetchChampSelectSnapshot, fetchLiveOverlaySnapshot } from "../backend/leagueClient";
import { openSelfHistoryOverlayWindow } from "../windows/selfHistoryOverlayWindow";
import type { AppWindowMode } from "./types";

const CHAMP_SELECT_ADVISOR_REFRESH_DELAY_MS = 250;
const SELF_HISTORY_OVERLAY_OPEN_DELAY_MS = 500;

interface OverlayEventDeps {
  mode: AppWindowMode;
  champSelectFingerprintRef: MutableRefObject<string>;
  setChampSelectSnapshot: Dispatch<SetStateAction<ChampSelectSnapshot | null>>;
  setChampSelectAdvisorSnapshot: Dispatch<SetStateAction<ChampSelectAdvisorSnapshot | null>>;
  setLiveOverlaySnapshot: Dispatch<SetStateAction<LiveOverlaySnapshot | null>>;
  setAutoAcceptStatus: Dispatch<SetStateAction<AutoAcceptStatus | null>>;
  setFeedback: Dispatch<SetStateAction<Feedback | null>>;
  onChampSelectUpdate: (snapshot: ChampSelectSnapshot) => void;
}

export function useAppEffects(deps: OverlayEventDeps) {
  const {
    mode,
    champSelectFingerprintRef,
    setChampSelectSnapshot,
    setChampSelectAdvisorSnapshot,
    setLiveOverlaySnapshot,
    setAutoAcceptStatus,
    setFeedback,
    onChampSelectUpdate,
  } = deps;

  // ── Overlay mode events ──
  useEffect(() => {
    if (mode !== "overlay") {
      return;
    }

    let advisorRefreshTimer: number | null = null;
    const clearAdvisorRefreshTimer = () => {
      if (advisorRefreshTimer !== null) {
        window.clearTimeout(advisorRefreshTimer);
        advisorRefreshTimer = null;
      }
    };
    const scheduleAdvisorRefresh = () => {
      clearAdvisorRefreshTimer();
      advisorRefreshTimer = window.setTimeout(async () => {
        advisorRefreshTimer = null;
        try {
          const snapshot = await fetchChampSelectAdvisorSnapshot(6);
          setChampSelectAdvisorSnapshot(snapshot);
        } catch {
          // Silently ignore — UI shows stale data
        }
      }, CHAMP_SELECT_ADVISOR_REFRESH_DELAY_MS);
    };

    const cleanupUpdate = listenWithCleanup<ChampSelectSnapshot>("champ-select-update", (event) => {
      onChampSelectUpdate(event.payload);
      scheduleAdvisorRefresh();
    });

    const cleanupClear = listenWithCleanup<void>("champ-select-clear", () => {
      champSelectFingerprintRef.current = "";
      setChampSelectSnapshot(null);
      setChampSelectAdvisorSnapshot(null);
      setLiveOverlaySnapshot(null);
      clearAdvisorRefreshTimer();
    });

    void fetchChampSelectSnapshot(6)
      .then(onChampSelectUpdate)
      .catch(() => {});
    void fetchChampSelectAdvisorSnapshot(6)
      .then(setChampSelectAdvisorSnapshot)
      .catch(() => {});
    void fetchLiveOverlaySnapshot()
      .then(setLiveOverlaySnapshot)
      .catch(() => {});
    const liveTimer = window.setInterval(() => {
      void fetchLiveOverlaySnapshot()
        .then(setLiveOverlaySnapshot)
        .catch(() => {});
    }, 5000);

    return () => {
      clearAdvisorRefreshTimer();
      cleanupUpdate();
      cleanupClear();
      window.clearInterval(liveTimer);
    };
  }, [mode, onChampSelectUpdate, champSelectFingerprintRef, setChampSelectSnapshot, setChampSelectAdvisorSnapshot, setLiveOverlaySnapshot]);

  // ── Main mode events ──
  useEffect(() => {
    if (mode !== "main") {
      return;
    }

    void fetchAutoAcceptStatus()
      .then(setAutoAcceptStatus)
      .catch(() => {
        setAutoAcceptStatus(null);
      });

    const cleanupFeedback = listenWithCleanup<Feedback>("automation-feedback", (event) => {
      setFeedback(event.payload);
    });

    const cleanupStatus = listenWithCleanup<AutoAcceptStatus>("auto-accept-status-update", (event) => {
      setAutoAcceptStatus(event.payload);
    });

    let overlayOpenTimer: number | null = null;
    const clearOverlayOpenTimer = () => {
      if (overlayOpenTimer !== null) {
        window.clearTimeout(overlayOpenTimer);
        overlayOpenTimer = null;
      }
    };
    const scheduleOverlayOpen = () => {
      clearOverlayOpenTimer();
      overlayOpenTimer = window.setTimeout(() => {
        overlayOpenTimer = null;
        void openSelfHistoryOverlayWindow();
      }, SELF_HISTORY_OVERLAY_OPEN_DELAY_MS);
    };

    const cleanupLeaguePhase = listenWithCleanup<string>("league-phase-update", (event) => {
      if (event.payload === "InProgress") {
        scheduleOverlayOpen();
        return;
      }

      clearOverlayOpenTimer();
    });

    const cleanupOverlayShortcut = listenWithCleanup("open-self-history-overlay", () => {
      void openSelfHistoryOverlayWindow();
    });

    void openSelfHistoryOverlayWindow();

    return () => {
      clearOverlayOpenTimer();
      cleanupFeedback();
      cleanupStatus();
      cleanupLeaguePhase();
      cleanupOverlayShortcut();
    };
  }, [mode, setAutoAcceptStatus, setFeedback]);
}
