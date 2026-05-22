import { startTransition, useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

import { Activity } from "./pages/Activity";
import { Dashboard } from "./pages/Dashboard";
import { Matches } from "./pages/Matches";
import { ParticipantProfileWindow } from "./pages/ParticipantProfileWindow";
import { PostGameNotesWindowRoot } from "./pages/PostGameNotesWindow";
import { Profile } from "./pages/Profile";
import { Advisor } from "./pages/Advisor";
import { RankedChampions } from "./pages/RankedChampions";
import { Rune } from "./pages/Rune";
import { SelfHistoryOverlay } from "./pages/SelfHistoryOverlay";
import { Settings } from "./pages/Settings";
import { AppStateProvider, useAppCore, type AppWindowMode } from "./state/AppStateProvider";
import type { StartupPage } from "./backend/types";
import { oppositeLanguage, type TranslationKey } from "./i18n";
import { selectionFromParticipantProfileHash } from "./windows/participantProfileWindow";
import { isSelfHistoryOverlayHash } from "./windows/selfHistoryOverlayWindow";
import { isPostGameNotesHash, openPostGameNotesWindow } from "./windows/postGameNotesWindow";

type Page = StartupPage | "profile" | "matches" | "ranked" | "advisor" | "rune";

const PERSISTENT_PAGES = new Set<Page>(["matches", "ranked", "activity"]);

const pages: Array<{ id: Page; labelKey: TranslationKey; icon: IconName }> = [
  { id: "dashboard", labelKey: "nav.dashboard", icon: "dashboard" },
  { id: "profile", labelKey: "nav.profile", icon: "profile" },
  { id: "matches", labelKey: "nav.matches", icon: "matches" },
  { id: "advisor", labelKey: "nav.advisor", icon: "advisor" },
  { id: "ranked", labelKey: "nav.ranked", icon: "ranked" },
  { id: "rune", labelKey: "nav.rune", icon: "rune" },
  { id: "activity", labelKey: "nav.activity", icon: "activity" },
  { id: "settings", labelKey: "nav.settings", icon: "settings" },
];

export function App() {
  const participantProfileSelection = selectionFromParticipantProfileHash(window.location.hash);
  const isSelfHistoryOverlay = isSelfHistoryOverlayHash(window.location.hash);
  const isPostGameNotes = isPostGameNotesHash(window.location.hash);
  const mode: AppWindowMode = participantProfileSelection ? "participant" : isSelfHistoryOverlay ? "overlay" : "main";

  useEffect(() => {
    if (mode === "main") {
      void invoke("init_overlay_hotkey");
    }
  }, [mode]);

  // Post-game notes window is a standalone root — no AppStateProvider wrapper needed here
  if (isPostGameNotes) {
    return <PostGameNotesWindowRoot />;
  }

  return (
    <AppStateProvider mode={mode}>
      {participantProfileSelection ? (
        <ParticipantProfileWindow initialSelection={participantProfileSelection} />
      ) : isSelfHistoryOverlay ? (
        <SelfHistoryOverlay />
      ) : (
        <AppShell />
      )}
    </AppStateProvider>
  );
}

export function AppShell() {
  const { snapshot, feedback, clearFeedback, isLoading, refresh, effectiveLanguage, setLanguagePreference, t } = useAppCore();
  const [activePage, setActivePage] = useState<Page>("dashboard");
  const [mountedPages, setMountedPages] = useState<Set<Page>>(() => new Set(["dashboard"]));
  const didApplyStartupPage = useRef(false);
  const didUserNavigate = useRef(false);
  const compactMode = snapshot?.settings.compactMode ?? false;
  const isDark = snapshot?.settings.theme === "dark";

  // Apply/remove `dark` class on <html> for Tailwind dark: variant
  useEffect(() => {
    const root = document.documentElement;
    if (isDark) {
      root.classList.add("dark");
    } else {
      root.classList.remove("dark");
    }
  }, [isDark]);
  const navigateTo = useCallback((page: Page, options?: { isUserInitiated?: boolean }) => {
    if (options?.isUserInitiated) {
      didUserNavigate.current = true;
    }

    startTransition(() => {
      setActivePage(page);
      if (PERSISTENT_PAGES.has(page)) {
        setMountedPages((prev) => prev.has(page) ? prev : new Set([...prev, page]));
      }
    });
  }, []);

  useEffect(() => {
    if (snapshot && !didApplyStartupPage.current && !didUserNavigate.current) {
      navigateTo(snapshot.settings.startupPage);
      didApplyStartupPage.current = true;
    }
  }, [navigateTo, snapshot]);

  // Auto-navigate to rune page on champion lock-in
  useEffect(() => {
    const unlisten = listen("champion-locked-in", () => {
      navigateTo("rune");
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, [navigateTo]);

  // Open post-game notes window when game ends
  useEffect(() => {
    const unlisten = listen("post-game-notes-open", () => {
      void openPostGameNotesWindow();
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <div className="flex h-screen min-h-0 bg-zinc-100 dark:bg-zinc-950 text-zinc-950 dark:text-zinc-50">
      <aside
        className={[
          "flex shrink-0 flex-col border-r border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 transition-[width]",
          compactMode ? "w-20" : "w-64",
        ].join(" ")}
      >
        <div className={["flex h-20 items-center border-b border-zinc-200 dark:border-zinc-700", compactMode ? "justify-center px-3" : "px-5"].join(" ")}>
          <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-md bg-rose-700 text-sm font-bold text-white">
            LoL
          </div>
          {!compactMode && (
            <div className="ml-3 min-w-0">
              <p className="truncate text-sm font-semibold text-zinc-950 dark:text-zinc-50">{t("app.name")}</p>
              <p className="text-xs font-medium text-zinc-500 dark:text-zinc-400">{t("app.milestone")}</p>
            </div>
          )}
        </div>

        <nav className="flex flex-1 flex-col gap-2 px-3 py-4" aria-label="Primary">
          {pages.map((page) => {
            const isActive = page.id === activePage;
            const label = t(page.labelKey);

            return (
              <button
                key={page.id}
                type="button"
                title={compactMode ? label : undefined}
                aria-label={label}
                onClick={() => navigateTo(page.id, { isUserInitiated: true })}
                className={[
                  "flex h-11 w-full items-center gap-3 rounded-md px-3 text-left text-sm font-medium transition",
                  compactMode ? "justify-center" : "",
                  isActive
                    ? "bg-rose-700 text-white shadow-sm"
                    : "text-zinc-600 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800 hover:text-zinc-950 dark:hover:text-zinc-50",
                ].join(" ")}
              >
                <Icon name={page.icon} />
                {!compactMode && <span>{label}</span>}
              </button>
            );
          })}
        </nav>
      </aside>

      <div className="flex min-w-0 flex-1 flex-col">
        <div className="flex h-12 shrink-0 items-center justify-between border-b border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 px-8">
          <p className="text-sm font-medium text-zinc-500 dark:text-zinc-400">{t(pages.find((p) => p.id === activePage)?.labelKey ?? "nav.dashboard")}</p>
          <button
            type="button"
            onClick={() => void setLanguagePreference(oppositeLanguage(effectiveLanguage))}
            disabled={!snapshot}
            className="inline-flex h-8 min-w-12 items-center justify-center rounded-md border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-800 px-3 text-sm font-semibold text-zinc-700 dark:text-zinc-300 transition hover:bg-zinc-50 dark:hover:bg-zinc-700 disabled:cursor-not-allowed disabled:text-zinc-400"
          >
            {t("app.languageToggle")}
          </button>
        </div>
        {feedback && (
          <div
            className={[
              "flex items-center justify-between gap-4 border-b px-8 py-3 text-sm font-medium",
              feedback.kind === "success"
                ? "border-emerald-200 bg-emerald-50 text-emerald-800 dark:border-emerald-800 dark:bg-emerald-950 dark:text-emerald-300"
                : "border-amber-200 bg-amber-50 text-amber-800 dark:border-amber-800 dark:bg-amber-950 dark:text-amber-300",
            ].join(" ")}
          >
            <span>{feedback.message}</span>
            <button type="button" className="font-semibold underline-offset-4 hover:underline" onClick={clearFeedback}>
              {t("app.dismiss")}
            </button>
          </div>
        )}
        {isLoading && !snapshot && (
          <div className="border-b border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-900 px-8 py-3 text-sm font-medium text-zinc-600 dark:text-zinc-400">
            {t("app.loadingState")}
          </div>
        )}
        {!isLoading && !snapshot && (
          <div className="flex items-center justify-between gap-4 border-b border-red-200 bg-red-50 px-8 py-3 text-sm">
            <span className="font-medium text-red-800">{t("app.loadingState")}</span>
            <button
              type="button"
              onClick={() => void refresh()}
              className="inline-flex h-8 items-center rounded-md bg-red-100 px-3 text-sm font-semibold text-red-700 transition hover:bg-red-200"
            >
              {t("common.refresh")}
            </button>
          </div>
        )}
        {activePage === "dashboard" && <Dashboard />}
        {activePage === "profile" && <Profile />}
        {mountedPages.has("matches") && <div className={activePage === "matches" ? "" : "hidden"}><Matches /></div>}
        {activePage === "advisor" && <Advisor />}
        {mountedPages.has("ranked") && <div className={activePage === "ranked" ? "" : "hidden"}><RankedChampions /></div>}
        {activePage === "rune" && <Rune />}
        {mountedPages.has("activity") && <div className={activePage === "activity" ? "" : "hidden"}><Activity /></div>}
        {activePage === "settings" && <Settings />}
      </div>
    </div>
  );
}

type IconName = "dashboard" | "profile" | "matches" | "advisor" | "ranked" | "rune" | "activity" | "settings";

function Icon({ name }: { name: IconName }) {
  const paths: Record<IconName, string> = {
    dashboard: "M4 13h6V4H4v9Zm0 7h6v-5H4v5Zm10 0h6v-9h-6v9Zm0-11h6V4h-6v5Z",
    profile:
      "M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8Zm-8 8a8 8 0 0 1 16 0v1H4v-1Z",
    matches:
      "M5 4h14a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2Zm2 4v3h4V8H7Zm0 5v3h4v-3H7Zm6-5v2h4V8h-4Zm0 5v2h4v-2h-4Z",
    advisor:
      "M12 3 4 7v5c0 5 3.3 8 8 9 4.7-1 8-4 8-9V7l-8-4Zm0 3.2 5 2.5V12c0 3.1-1.8 5.2-5 6-3.2-.8-5-2.9-5-6V8.7l5-2.5Zm-2.5 4.3h5v2h-5v-2Zm0 3.5h5v2h-5v-2Z",
    ranked:
      "M6 20V9h3v11H6Zm5 0V4h3v16h-3Zm5 0v-7h3v7h-3ZM4 20h17v2H4v-2Z",
    rune: "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 14H9V8h2v8zm4 0h-2V8h2v8z",
    activity:
      "M5 4h14v2H5V4Zm0 4h9v2H5V8Zm0 4h14v2H5v-2Zm0 4h9v2H5v-2Zm12-8 4 4-4 4v-3h-5v-2h5V8Z",
    settings:
      "M19.14 12.94c.04-.31.06-.63.06-.94s-.02-.63-.06-.94l2.03-1.58-1.92-3.32-2.39.96a7.13 7.13 0 0 0-1.63-.94L14.87 3h-3.74l-.36 3.18c-.58.23-1.12.54-1.63.94l-2.39-.96-1.92 3.32 2.03 1.58c-.04.31-.06.63-.06.94s.02.63.06.94l-2.03 1.58 1.92 3.32 2.39-.96c.51.4 1.05.71 1.63.94l.36 3.18h3.74l.36-3.18c.58-.23 1.12-.54 1.63-.94l2.39.96 1.92-3.32-2.03-1.58ZM13 15.5A3.5 3.5 0 1 1 13 8a3.5 3.5 0 0 1 0 7.5Z",
  };

  return (
    <svg aria-hidden="true" className="h-5 w-5 shrink-0" viewBox="0 0 24 24" fill="currentColor">
      <path d={paths[name]} />
    </svg>
  );
}
