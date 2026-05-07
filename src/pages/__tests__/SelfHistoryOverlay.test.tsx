// @vitest-environment jsdom
import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import React from "react";
import type { Mock } from "vitest";

// Mock Tauri window API before anything else
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: vi.fn(() => ({
    hide: vi.fn().mockResolvedValue(undefined),
  })),
}));

// Mock self history overlay window utilities
const mockCanOpen = vi.fn();
const mockDestroy = vi.fn();
vi.mock("../../windows/selfHistoryOverlayWindow", () => ({
  canOpenSelfHistoryOverlayWindow: () => mockCanOpen(),
  destroySelfHistoryOverlayWindow: () => mockDestroy(),
}));

// Mock AppStateProvider hooks
const mockUseAppCore = vi.fn();
const mockUseChampSelect = vi.fn();
const mockUseLeagueAssets = vi.fn();

vi.mock("../../state/AppStateProvider", () => ({
  useAppCore: () => mockUseAppCore(),
  useChampSelect: () => mockUseChampSelect(),
  useLeagueAssets: () => mockUseLeagueAssets(),
}));

// Mock T function
const tMock = vi.fn((key: string) => {
  const labels: Record<string, string> = {
    "common.pending": "Pending...",
    "common.loading": "Loading...",
    "overlay.windowTitle": "Match History",
    "overlay.dragHint": "Drag to reposition",
    "overlay.refresh": "Refresh",
    "overlay.hide": "Hide",
    "overlay.refreshFailed": "Refresh failed",
    "overlay.empty": "Waiting for players",
    "overlay.historyUnavailable": "History unavailable",
    "overlay.score": "Score",
    "overlay.rankUnavailable": "Rank unavailable",
    "overlay.unranked": "Unranked",
    "overlay.allyWins": "Ally Wins",
    "overlay.enemyWins": "Enemy Wins",
  };
  return labels[key] ?? key;
});

// Helper to create a mock player for champ select
function createPlayer(overrides: Record<string, unknown> = {}) {
  return {
    summonerId: 0,
    displayName: "Test Player",
    championId: null,
    team: "ally",
    puuid: "test-puuid",
    gameCount: 0,
    soloRank: null,
    flexRank: null,
    rankedQueues: [],
    recentMatchIds: [],
    recentStats: null,
    ...overrides,
  };
}

function defaultCore() {
  return {
    effectiveLanguage: "en" as const,
    t: tMock,
  };
}

function defaultChampSelect() {
  return {
    champSelectSnapshot: null,
    refreshChampSelectSnapshot: vi.fn().mockResolvedValue(true),
  };
}

function defaultLeagueAssets() {
  return {
    championDetailsById: {},
    leagueImages: { profileIcons: {}, championIcons: {}, gameAssets: {} },
    loadLeagueChampionDetails: vi.fn().mockResolvedValue(false),
  };
}

beforeEach(() => {
  vi.clearAllMocks();
  mockCanOpen.mockResolvedValue(true);
  mockUseAppCore.mockReturnValue(defaultCore());
  mockUseChampSelect.mockReturnValue(defaultChampSelect());
  mockUseLeagueAssets.mockReturnValue(defaultLeagueAssets());
});

async function mountOverlay() {
  const { SelfHistoryOverlay } = await import("../SelfHistoryOverlay");
  return render(React.createElement(SelfHistoryOverlay));
}

describe("SelfHistoryOverlay", () => {
  it("shows pending state while checking overlay permissions", async () => {
    mockCanOpen.mockReturnValue(new Promise(() => {})); // never resolves
    await mountOverlay();
    expect(screen.getByText("Pending...")).toBeDefined();
  });

  it("shows loading message when champ select snapshot is loading", async () => {
    mockUseChampSelect.mockReturnValue({
      ...defaultChampSelect(),
      champSelectSnapshot: { players: [createPlayer({ championId: 1, recentStats: null })] },
    });
    await mountOverlay();
    expect(await screen.findByText("Loading...")).toBeDefined();
  });

  it("shows empty state when there are no players", async () => {
    mockUseChampSelect.mockReturnValue({
      ...defaultChampSelect(),
      champSelectSnapshot: { players: [] },
    });
    await mountOverlay();
    expect(await screen.findByText("Waiting for players")).toBeDefined();
  });

  it("renders team boards with champion portraits for players", async () => {
    const ally = createPlayer({
      summonerId: 1,
      displayName: "Ally One",
      team: "ally",
      championId: 1,
      recentStats: {
        status: "complete" as const,
        recentMatches: [],
      },
    });
    const enemy = createPlayer({
      summonerId: 2,
      displayName: "Enemy One",
      team: "enemy",
      championId: 2,
      recentStats: {
        status: "complete" as const,
        recentMatches: [],
      },
    });

    mockUseChampSelect.mockReturnValue({
      ...defaultChampSelect(),
      champSelectSnapshot: { players: [ally, enemy] },
    });

    await mountOverlay();

    // Player names render as initials — verify the overlay structure renders
    expect(await screen.findByText("Match History")).toBeDefined();
    expect(await screen.findByText("Ally Wins")).toBeDefined();
    expect(await screen.findByText("Enemy Wins")).toBeDefined();
  });

  it("shows refresh button", async () => {
    mockUseChampSelect.mockReturnValue({
      ...defaultChampSelect(),
      champSelectSnapshot: { players: [] },
    });
    await mountOverlay();
    expect(await screen.findByLabelText("Refresh")).toBeDefined();
  });

  it("shows hide button", async () => {
    mockUseChampSelect.mockReturnValue({
      ...defaultChampSelect(),
      champSelectSnapshot: { players: [] },
    });
    await mountOverlay();
    expect(await screen.findByLabelText("Hide")).toBeDefined();
  });
});
