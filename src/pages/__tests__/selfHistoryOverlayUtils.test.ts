import { describe, expect, it } from "vitest";

import type { ChampSelectPlayer, RecentMatchSummary } from "../../backend/types";
import {
  createOverlayModel,
  formatMatchDate,
  kdaToneClass,
  matchStreaks,
  premadeGroupStyle,
  teamSummary,
  winRateToneClass,
} from "../selfHistoryOverlayUtils";

const t = (key: string) => key;

function makeMatch(overrides: Partial<RecentMatchSummary> = {}): RecentMatchSummary {
  return {
    gameId: 1,
    championId: 103,
    championName: "Ahri",
    queueName: "Ranked Solo/Duo",
    result: "win",
    kills: 5,
    deaths: 2,
    assists: 7,
    kda: 6.0,
    playedAt: "2026-07-01T12:30:00Z",
    gameDurationSeconds: 1800,
    ...overrides,
  };
}

function makePlayer(overrides: Partial<ChampSelectPlayer> = {}): ChampSelectPlayer {
  return {
    summonerId: 1,
    displayName: "Player One",
    championId: 103,
    championName: "Ahri",
    team: "ally",
    rankedQueues: [],
    summonerLevel: 100,
    masteryLevel: null,
    recentStats: null,
    recentStatsStatus: "loaded",
    ...overrides,
  };
}

describe("matchStreaks", () => {
  it("counts consecutive wins from the most recent game", () => {
    const matches = [
      makeMatch({ gameId: 1, result: "win" }),
      makeMatch({ gameId: 2, result: "win" }),
      makeMatch({ gameId: 3, result: "loss" }),
      makeMatch({ gameId: 4, result: "win" }),
    ];

    expect(matchStreaks(matches)).toEqual({ winningStreak: 2, losingStreak: 0 });
  });

  it("counts consecutive losses from the most recent game", () => {
    const matches = [
      makeMatch({ gameId: 1, result: "loss" }),
      makeMatch({ gameId: 2, result: "loss" }),
      makeMatch({ gameId: 3, result: "loss" }),
      makeMatch({ gameId: 4, result: "win" }),
    ];

    expect(matchStreaks(matches)).toEqual({ winningStreak: 0, losingStreak: 3 });
  });

  it("breaks streaks on unknown results such as remakes", () => {
    const matches = [
      makeMatch({ gameId: 1, result: "unknown" }),
      makeMatch({ gameId: 2, result: "win" }),
    ];

    expect(matchStreaks(matches)).toEqual({ winningStreak: 0, losingStreak: 0 });
  });

  it("handles empty histories", () => {
    expect(matchStreaks([])).toEqual({ winningStreak: 0, losingStreak: 0 });
  });
});

describe("formatMatchDate", () => {
  it("formats ISO timestamps as MM-DD HH:mm", () => {
    const formatted = formatMatchDate("2026-07-01T12:30:00Z");
    // Local timezone shifts the hour; assert the shape instead of the value.
    expect(formatted).toMatch(/^\d{2}-\d{2} \d{2}:\d{2}$/);
  });

  it("formats epoch-millisecond strings", () => {
    expect(formatMatchDate("1751372400000")).toMatch(/^\d{2}-\d{2} \d{2}:\d{2}$/);
  });

  it("returns null for missing or malformed values", () => {
    expect(formatMatchDate(null)).toBeNull();
    expect(formatMatchDate("not-a-date")).toBeNull();
  });
});

describe("win rate and KDA tone classes", () => {
  it("uses League Akari's thresholds for win rate", () => {
    expect(winRateToneClass(53)).toContain("emerald");
    expect(winRateToneClass(47)).toContain("red");
    expect(winRateToneClass(50)).toContain("zinc-200");
    expect(winRateToneClass(null)).toContain("zinc-500");
  });

  it("colors KDA extremes", () => {
    expect(kdaToneClass(4)).toContain("emerald");
    expect(kdaToneClass(1.5)).toContain("red");
    expect(kdaToneClass(3)).toContain("zinc-200");
  });
});

describe("premadeGroupStyle", () => {
  it("cycles through the palette", () => {
    expect(premadeGroupStyle(0).letter).toBe("A");
    expect(premadeGroupStyle(1).letter).toBe("B");
    expect(premadeGroupStyle(6).letter).toBe("A");
  });
});

describe("createOverlayModel premade groups", () => {
  it("maps snapshot premade groups onto player views", () => {
    const players = [
      makePlayer({ summonerId: 1, team: "ally" }),
      makePlayer({ summonerId: 2, displayName: "Player Two", team: "ally" }),
      makePlayer({ summonerId: 3, displayName: "Player Three", team: "ally" }),
      makePlayer({ summonerId: 9, displayName: "Enemy One", team: "enemy" }),
    ];

    const model = createOverlayModel(players, [], {}, "en", t, [[1, 2]]);

    expect(model.allies[0].premadeGroup).toBe(0);
    expect(model.allies[1].premadeGroup).toBe(0);
    expect(model.allies[2].premadeGroup).toBeNull();
    expect(model.enemies[0].premadeGroup).toBeNull();
  });

  it("computes win rate and streaks per player", () => {
    const players = [
      makePlayer({
        summonerId: 1,
        recentStats: {
          matchCount: 4,
          averageKda: 3.5,
          recentChampions: [],
          recentMatches: [
            makeMatch({ gameId: 1, result: "win" }),
            makeMatch({ gameId: 2, result: "win" }),
            makeMatch({ gameId: 3, result: "win" }),
            makeMatch({ gameId: 4, result: "loss" }),
          ],
        },
      }),
    ];

    const model = createOverlayModel(players, [], {}, "en", t);

    expect(model.allies[0].winRate).toBe(75);
    expect(model.allies[0].winningStreak).toBe(3);
    expect(model.allies[0].losingStreak).toBe(0);
  });
});

describe("teamSummary", () => {
  it("aggregates wins, games, and team KDA across members", () => {
    const players = [
      makePlayer({
        summonerId: 1,
        recentStats: {
          matchCount: 2,
          averageKda: 6.0,
          recentChampions: [],
          recentMatches: [
            makeMatch({ gameId: 1, result: "win", kills: 10, deaths: 2, assists: 10 }),
            makeMatch({ gameId: 2, result: "loss", kills: 2, deaths: 8, assists: 4 }),
          ],
        },
      }),
    ];

    const model = createOverlayModel(players, [], {}, "en", t);
    const team = teamSummary(model.allies);

    expect(team.games).toBe(2);
    expect(team.wins).toBe(1);
    expect(team.winRate).toBe(50);
    // (10+2 kills + 10+4 assists) / (2+8 deaths) = 26/10
    expect(team.kda).toBe(2.6);
    expect(model.allyTeam).toEqual(team);
  });

  it("returns nulls with no games", () => {
    const model = createOverlayModel([makePlayer()], [], {}, "en", t);

    expect(model.allyTeam.winRate).toBeNull();
    expect(model.allyTeam.kda).toBeNull();
  });
});
