// @vitest-environment jsdom
import { describe, expect, it, vi, beforeEach } from "vitest";
import { act, fireEvent, render, screen } from "@testing-library/react";
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
    "playerNotes.eyebrow": "Player Notes",
    "participant.note": "Note",
    "participant.tags": "Tags",
  };
  return labels[key] ?? key;
});

// Resolves out of call order, so a request that started earlier can be made
// to resolve after one that started later — reproducing the race the
// matchRequestIdRef/championRequestIdRef guards protect against.
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((res) => {
    resolve = res;
  });
  return { promise, resolve };
}

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

  it("shows a note badge for a champ-select player with a saved note", async () => {
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
    const enemy = createPlayer({
      summonerId: 2,
      displayName: "Enemy One",
      team: "enemy",
      championId: 122,
      recentStats: null,
      recentStatsStatus: "notRequested",
    });

    mockUseChampSelect.mockReturnValue({
      ...defaultChampSelect(),
      champSelectSnapshot: { players: [ally, enemy] },
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
            tags: [],
            advisor: null,
            matchupAdvice: null,
            noteSummary: { hasNote: true, note: "Strong laner", tags: ["lane"] },
          },
          {
            summonerId: 2,
            displayName: "Enemy One",
            championId: 122,
            championName: "Darius",
            team: "enemy",
            recentStats: null,
            recentStatsStatus: "notRequested",
            tags: [],
            advisor: null,
            matchupAdvice: null,
            noteSummary: { hasNote: false, note: null, tags: [] },
          },
        ],
      },
    });

    await mountOverlay();

    const badge = await screen.findByLabelText("Player Notes");
    expect(badge.getAttribute("title")).toBe("Note: Strong laner · Tags: lane");
  });

  it("ignores a stale match-detail response after a newer match was selected", async () => {
    const matchA = {
      gameId: 1001,
      championId: 1,
      championName: "Ahri",
      queueName: "Ranked Solo",
      result: "win" as const,
      kills: 5,
      deaths: 2,
      assists: 7,
      kda: 6,
      playedAt: "2026-08-01T00:00:00Z",
      gameDurationSeconds: 1800,
    };
    const matchB = { ...matchA, gameId: 1002, result: "loss" as const };
    const player = createPlayer({
      summonerId: 1,
      displayName: "Ally One",
      team: "ally",
      championId: 1,
      recentStats: { matchCount: 2, averageKda: 3, recentChampions: ["Ahri"], recentMatches: [matchA, matchB] },
      recentStatsStatus: "loaded",
    });

    mockUseChampSelect.mockReturnValue({
      ...defaultChampSelect(),
      champSelectSnapshot: { players: [player] },
    });

    const deferredA = deferred<boolean>();
    const deferredB = deferred<boolean>();
    const loadPostMatchDetail = vi.fn((gameId: number) => (gameId === 1001 ? deferredA.promise : deferredB.promise));
    mockUseAppCore.mockReturnValue({
      ...defaultCore(),
      postMatchDetails: {},
      loadPostMatchDetail,
    });

    await mountOverlay();

    const rows = await screen.findAllByTitle("overlay.viewMatchDetails");
    expect(rows).toHaveLength(2);

    // Select the older match, then the newer one before the older fetch resolves.
    fireEvent.click(rows[0]);
    fireEvent.click(rows[1]);

    expect(loadPostMatchDetail).toHaveBeenCalledWith(1001);
    expect(loadPostMatchDetail).toHaveBeenCalledWith(1002);
    // The header subtitle and the body both fall back to this text whenever
    // no detail has loaded yet, so two elements is the expected loading state.
    expect(await screen.findAllByText("overlay.loadingMatchDetails")).toHaveLength(2);

    // The older (1001) fetch resolves last-selection is now 1002 — it must
    // be ignored, not clear the loading state for the still-pending newer fetch.
    await act(async () => {
      deferredA.resolve(true);
      await Promise.resolve();
    });
    expect(screen.getAllByText("overlay.loadingMatchDetails")).toHaveLength(2);
    expect(screen.queryByText("overlay.matchDetailsUnavailable")).toBeNull();

    // The newer (1002) fetch resolves — this is the one that should drive the UI.
    // Only the body's loading paragraph is gated on isLoading; the header
    // subtitle has no detail to show either way, so one match remains.
    await act(async () => {
      deferredB.resolve(false);
      await Promise.resolve();
    });
    expect(screen.getAllByText("overlay.loadingMatchDetails")).toHaveLength(1);
    expect(await screen.findByText("overlay.matchDetailsUnavailable")).toBeDefined();
  });

  it("ignores a stale champion-detail response after a newer champion was selected", async () => {
    const allyA = createPlayer({
      summonerId: 1,
      displayName: "Ally One",
      team: "ally",
      championId: 10,
      recentStats: { matchCount: 0, averageKda: null, recentChampions: [], recentMatches: [] },
      recentStatsStatus: "loaded",
    });
    const allyB = createPlayer({
      summonerId: 2,
      displayName: "Ally Two",
      team: "ally",
      championId: 20,
      recentStats: { matchCount: 0, averageKda: null, recentChampions: [], recentMatches: [] },
      recentStatsStatus: "loaded",
    });

    mockUseChampSelect.mockReturnValue({
      ...defaultChampSelect(),
      champSelectSnapshot: { players: [allyA, allyB] },
    });

    const deferredA = deferred<boolean>();
    const deferredB = deferred<boolean>();
    const loadLeagueChampionDetails = vi.fn((championId: number) => (championId === 10 ? deferredA.promise : deferredB.promise));
    mockUseLeagueAssets.mockReturnValue({
      ...defaultLeagueAssets(),
      loadLeagueChampionDetails,
    });

    await mountOverlay();

    const portraitA = screen.getByRole("button", { name: "overlay.viewAbilities Ally One" });
    const portraitB = screen.getByRole("button", { name: "overlay.viewAbilities Ally Two" });

    // Select the first champion, then the second before the first fetch resolves.
    fireEvent.click(portraitA);
    fireEvent.click(portraitB);

    expect(loadLeagueChampionDetails).toHaveBeenCalledWith(10);
    expect(loadLeagueChampionDetails).toHaveBeenCalledWith(20);
    expect(await screen.findByText("overlay.loadingAbilities")).toBeDefined();

    // The first (10) fetch resolves last-selection is now 20 — it must be
    // ignored, not clear the loading state for the still-pending newer fetch.
    await act(async () => {
      deferredA.resolve(true);
      await Promise.resolve();
    });
    expect(screen.getByText("overlay.loadingAbilities")).toBeDefined();
    expect(screen.queryByText("overlay.abilitiesUnavailable")).toBeNull();

    // The newer (20) fetch resolves — this is the one that should drive the UI.
    await act(async () => {
      deferredB.resolve(false);
      await Promise.resolve();
    });
    expect(screen.queryByText("overlay.loadingAbilities")).toBeNull();
    expect(await screen.findByText("overlay.abilitiesUnavailable")).toBeDefined();
  });
});
