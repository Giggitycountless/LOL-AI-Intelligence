// @vitest-environment jsdom
import { render, screen } from "@testing-library/react";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mockUseAdvisor = vi.fn();
const mockUseLeagueAssets = vi.fn();

vi.mock("../../state/AppStateProvider", () => ({
  useAdvisor: () => mockUseAdvisor(),
  useLeagueAssets: () => mockUseLeagueAssets(),
}));

function advisorRecord() {
  return {
    championId: 103,
    championName: "Ahri",
    championAlias: "Ahri",
    lane: "top",
    winRate: 51.4,
    pickRate: 10.2,
    banRate: 8.1,
    overallScore: 42.0,
    games: 12000,
    runes: {
      primaryStyle: "Domination",
      primaryRunes: [{ id: 8112, name: "Electrocute" }],
      secondaryStyle: "Sorcery",
      secondaryRunes: [{ id: 8210, name: "Transcendence" }],
      statShards: ["Adaptive Force"],
    },
    summonerSpells: [
      { id: 4, name: "Flash" },
      { id: 14, name: "Ignite" },
    ],
    skillOrder: {
      maxOrder: ["Q", "W", "E"],
      earlyOrder: ["Q", "W", "E", "Q", "Q", "R"],
    },
    itemBuild: {
      starter: [{ id: 1056, name: "Doran's Ring" }],
      core: [{ id: 6655, name: "Luden's Companion" }],
      boots: [{ id: 3020, name: "Sorcerer's Shoes" }],
      late: [{ id: 3089, name: "Rabadon's Deathcap" }],
      situational: [{ id: 3157, name: "Zhonya's Hourglass" }],
    },
    strongAgainst: [{ championId: 777, championName: "Yone", note: "Punish Q3.", winRateDelta: 2.1 }],
    weakAgainst: [{ championId: 238, championName: "Zed", note: "Respect level 6.", winRateDelta: -1.8 }],
    powerSpikes: [{ timing: "6", label: "Spirit Rush", description: "Look for roam windows." }],
    laneAdvice: "Push waves before roaming.",
    teamfightAdvice: "Enter after key cooldowns are used.",
  };
}

function defaultAdvisor(records = [advisorRecord()]) {
  return {
    advisorData: {
      lane: "top",
      championId: null,
      records,
      source: "fixture",
      updatedAt: "1",
      generatedAt: null,
      importedAt: "1",
      patch: "26.08",
      region: "KR",
      queue: "RANKED_SOLO_5X5",
      tier: "EMERALD_PLUS",
      isCached: true,
      dataStatus: "cached",
      statusMessage: null,
    },
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
  mockUseAdvisor.mockReturnValue(defaultAdvisor());
  mockUseLeagueAssets.mockReturnValue({
    leagueImages: { profileIcons: {}, championIcons: {}, gameAssets: {} },
    loadLeagueChampionIcon: vi.fn().mockResolvedValue(true),
  });
});

describe("Advisor", () => {
  it("renders runes, summoners, skill order, build, matchups, and advice", async () => {
    const { Advisor } = await import("../Advisor");

    render(React.createElement(Advisor));

    expect(await screen.findByText("Champion Recommendations")).toBeDefined();
    expect(await screen.findByText("Ahri")).toBeDefined();
    expect(screen.getByText(/Electrocute/)).toBeDefined();
    expect(screen.getByText("Flash + Ignite")).toBeDefined();
    expect(screen.getByText(/Max Q > W > E/)).toBeDefined();
    expect(screen.getByText(/Doran's Ring/)).toBeDefined();
    expect(screen.getByText("Rabadon's Deathcap")).toBeDefined();
    expect(screen.getByText("Zhonya's Hourglass")).toBeDefined();
    expect(screen.getByText("Yone")).toBeDefined();
    expect(screen.getByText("Zed")).toBeDefined();
    expect(screen.getByText("Push waves before roaming.")).toBeDefined();
  });

  it("renders an empty state for missing advisor data", async () => {
    mockUseAdvisor.mockReturnValue(defaultAdvisor([]));
    const { Advisor } = await import("../Advisor");

    render(React.createElement(Advisor));

    expect(await screen.findByText("No advisor data is available for this lane.")).toBeDefined();
  });

  it("does not show stale records from another selected lane", async () => {
    mockUseAdvisor.mockReturnValue({
      ...defaultAdvisor([advisorRecord()]),
      advisorData: {
        ...defaultAdvisor([advisorRecord()]).advisorData,
        lane: "middle",
        records: [{ ...advisorRecord(), lane: "middle" }],
      },
    });
    const { Advisor } = await import("../Advisor");

    render(React.createElement(Advisor));

    expect(await screen.findByText("No advisor data is available for this lane.")).toBeDefined();
    expect(screen.queryByText("Ahri")).toBeNull();
  });
});
