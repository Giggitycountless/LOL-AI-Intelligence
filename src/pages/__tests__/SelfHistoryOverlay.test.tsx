// @vitest-environment jsdom
import { describe, expect, it, vi, beforeEach } from "vitest";
import { act, render, screen } from "@testing-library/react";
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
const mockUseAdvisor = vi.fn();

vi.mock("../../state/AppStateProvider", () => ({
  useAdvisor: () => mockUseAdvisor(),
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
    "advisorTag.strongPick": "Strong pick",
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
    recentStatsStatus: "notRequested",
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

function defaultAdvisor() {
  return {
    advisorData: null,
    champSelectAdvisorSnapshot: null,
    liveOverlaySnapshot: null,
    isAdvisorDataLoading: false,
    loadAdvisorData: vi.fn().mockResolvedValue(true),
    refreshAdvisorData: vi.fn().mockResolvedValue(true),
    refreshChampSelectAdvisorSnapshot: vi.fn().mockResolvedValue(true),
    refreshLiveOverlaySnapshot: vi.fn().mockResolvedValue(true),
  };
}

beforeEach(() => {
  vi.clearAllMocks();
  mockCanOpen.mockResolvedValue(true);
  mockUseAppCore.mockReturnValue(defaultCore());
  mockUseChampSelect.mockReturnValue(defaultChampSelect());
  mockUseLeagueAssets.mockReturnValue(defaultLeagueAssets());
  mockUseAdvisor.mockReturnValue(defaultAdvisor());
});

async function mountOverlay() {
  const { SelfHistoryOverlay } = await import("../SelfHistoryOverlay");
  let view: ReturnType<typeof render> | undefined;
  await act(async () => {
    view = render(React.createElement(SelfHistoryOverlay));
  });
  return view!;
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
      refreshChampSelectSnapshot: vi.fn(() => new Promise<boolean>(() => {})),
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

    // Player names render as initials; verify the overlay structure renders.
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

  it("renders advisor tags and live overlay item value", async () => {
    const ally = createPlayer({
      summonerId: 1,
      displayName: "Ally One",
      team: "ally",
      championId: 86,
      recentStats: {
        matchCount: 1,
        averageKda: 2.5,
        recentChampions: ["Garen"],
        recentMatches: [],
      },
      recentStatsStatus: "loaded",
    });

    mockUseChampSelect.mockReturnValue({
      ...defaultChampSelect(),
      champSelectSnapshot: { players: [ally] },
    });
    mockUseAdvisor.mockReturnValue({
      ...defaultAdvisor(),
      champSelectAdvisorSnapshot: {
        cachedAt: "1",
        advisorSource: "fixture",
        advisorPatch: "26.08",
        dataStatus: "cached",
        players: [
          {
            summonerId: 1,
            displayName: "Ally One",
            championId: 86,
            championName: "Garen",
            team: "ally",
            recentStats: ally.recentStats,
            recentStatsStatus: "loaded",
            tags: [{ kind: "strongPick", value: null, tone: "good" }],
            advisor: null,
            matchupAdvice: "Favorable into Champion 122: punish cooldowns.",
          },
        ],
      },
      liveOverlaySnapshot: {
        gameTimeSeconds: 125,
        gameMode: "CLASSIC",
        mapName: "Summoner's Rift",
        activePlayer: {
          displayName: "Ally One",
          level: 6,
          currentGold: 742.4,
          resourceType: "MANA",
          resourceValue: 100,
          resourceMax: 300,
        },
        players: [],
        events: [{ eventId: 1, eventName: "ChampionKill", eventTime: 120, actor: "Ally One", victim: "Enemy", assistingParticipants: [] }],
        gold: { allyItemValue: 3000, enemyItemValue: 2100, itemValueDiff: 900 },
        refreshedAt: "1",
      },
    });

    await mountOverlay();

    expect(await screen.findByText("Strong pick")).toBeDefined();
    expect(await screen.findByText("+900")).toBeDefined();
    expect(await screen.findByText("742")).toBeDefined();
  });
});
