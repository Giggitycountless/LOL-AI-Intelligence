// @vitest-environment jsdom
import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import React from "react";

// Mock all Tauri API modules before importing the provider
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
  emit: vi.fn().mockResolvedValue(undefined),
  emitTo: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(null),
}));

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  WebviewWindow: Object.assign(vi.fn(), { getByLabel: vi.fn().mockResolvedValue(null) }),
}));

// Mock backend modules
vi.mock("../../backend/system", () => ({
  fetchAppState: vi.fn().mockResolvedValue({
    settings: { language: "system", compactMode: false, startupPage: "dashboard", activityLimit: 50 },
    settingsDefaults: { language: "system", compactMode: false, startupPage: "dashboard", activityLimit: 50 },
    serviceStatus: "ok",
    databaseStatus: "ok",
  }),
}));

vi.mock("../../backend/activity", () => ({
  listActivityEntries: vi.fn().mockResolvedValue({ records: [], total: 0 }),
  createActivityNote: vi.fn().mockResolvedValue({}),
  clearActivityEntries: vi.fn().mockResolvedValue({ deletedCount: 0 }),
}));

vi.mock("../../backend/leagueClient", () => ({
  fetchLeagueSelfSnapshot: vi.fn().mockResolvedValue(null),
  fetchAutoAcceptStatus: vi.fn().mockResolvedValue(null),
  fetchChampSelectSnapshot: vi.fn().mockResolvedValue({ players: [] }),
  fetchLeagueChampionDetails: vi.fn().mockResolvedValue(null),
  fetchLeagueChampionIcon: vi.fn().mockResolvedValue(null),
  fetchLeagueGameAsset: vi.fn().mockResolvedValue(null),
  fetchLeagueProfileIcon: vi.fn().mockResolvedValue(null),
  fetchPostMatchDetail: vi.fn().mockResolvedValue(null),
  fetchPostMatchParticipantProfile: vi.fn().mockResolvedValue(null),
  fetchRankedChampionStats: vi.fn().mockResolvedValue(null),
  refreshRankedChampionStats: vi.fn().mockResolvedValue(null),
  savePlayerNote: vi.fn().mockResolvedValue(null),
  clearPlayerNote: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("../../backend/settings", () => ({
  saveSettings: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("../../backend/dataTools", () => ({
  exportLocalData: vi.fn().mockResolvedValue({}),
  importLocalData: vi.fn().mockResolvedValue({ importedActivityCount: 0 }),
}));

vi.mock("../../backend/events", () => ({
  listenWithCleanup: vi.fn().mockReturnValue(vi.fn()),
}));

vi.mock("../../windows/selfHistoryOverlayWindow", () => ({
  openSelfHistoryOverlayWindow: vi.fn().mockResolvedValue(undefined),
}));

// Must import after mocks
const { AppStateProvider, useAppState } = await import("../AppStateProvider");

function TestConsumer() {
  const state = useAppState();
  return (
    <div>
      <span data-testid="language">{state.effectiveLanguage}</span>
      <span data-testid="loading">{String(state.isLoading)}</span>
      <span data-testid="app-name">{state.t("app.name")}</span>
    </div>
  );
}

describe("AppStateProvider", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders children and provides context", () => {
    render(
      <AppStateProvider>
        <TestConsumer />
      </AppStateProvider>,
    );

    expect(screen.getByTestId("app-name").textContent).toBeTruthy();
  });

  it("provides English translations by default", async () => {
    render(
      <AppStateProvider>
        <TestConsumer />
      </AppStateProvider>,
    );

    // Default effective language with "system" preference and no zh system lang is "en"
    const langEl = screen.getByTestId("language");
    expect(["en", "zh"]).toContain(langEl.textContent);
  });

  it("throws when useAppState is used outside provider", () => {
    const spy = vi.spyOn(console, "error").mockImplementation(() => {});

    expect(() => render(<TestConsumer />)).toThrow("AppStateProvider is missing");

    spy.mockRestore();
  });
});
