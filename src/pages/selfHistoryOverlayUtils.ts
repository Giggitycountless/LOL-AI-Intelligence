import type {
  AdvisorPlayerTag,
  AdvisorPlayerTagKind,
  ChampSelectPlayer,
  ChampSelectAdvisorPlayer,
  ChampSelectRecentStatsStatus,
  LiveOverlaySnapshot,
  MatchResult,
  RankedQueueSummary,
  RecentMatchSummary,
} from "../backend/types";
import type { EffectiveLanguage, TranslationKey } from "../i18n";
import type { T } from "../utils/formatting";

export const TEAM_SIZE = 5;
export const MATCH_ROWS = 6;

export type TeamTone = "ally" | "enemy";

export type MatchRowView = {
  id: string;
  imageUrl: string | undefined;
  match: RecentMatchSummary | null;
};

export type PlayerView = {
  id: string;
  averageKda: number | null;
  badge: string;
  championId: number | null | undefined;
  championUrl: string | undefined;
  displayName: string;
  flexRank: string | null;
  flexRankTier: string | null;
  flexRankLP: number | null;
  gameCount: number;
  advisorTags: AdvisorPlayerTag[];
  advisorSummary: string | null;
  isEmpty: boolean;
  masteryLevel: number | null;
  /** 0-based index into the snapshot's premadeGroups, or null when solo. */
  premadeGroup: number | null;
  rows: MatchRowView[];
  score: number | null;
  /** Bar width (0–100) for the score, relative to the highest score in this match. */
  scorePct: number;
  soloRank: string | null;
  soloRankTier: string | null;
  soloRankLP: number | null;
  soloRankWins: number;
  soloRankLosses: number;
  summonerLevel: number | null;
  recentStatsStatus: ChampSelectRecentStatsStatus;
  winCount: number;
  /** Recent-history win rate 0-100, or null with no games. */
  winRate: number | null;
  /** Consecutive wins counting back from the most recent game. */
  winningStreak: number;
  /** Consecutive losses counting back from the most recent game. */
  losingStreak: number;
};

export type TeamSummary = {
  games: number;
  wins: number;
  /** Team-wide recent win rate 0-100, or null with no games. */
  winRate: number | null;
  /** Aggregated (kills+assists)/deaths across every member's recent games. */
  kda: number | null;
};

export type OverlayModel = {
  allies: PlayerView[];
  enemies: PlayerView[];
  allyTeam: TeamSummary;
  enemyTeam: TeamSummary;
  summary: {
    allyGames: number;
    allyWins: number;
    enemyGames: number;
    enemyWins: number;
  };
};

export type InitialSnapshotStatus = "loading" | "ready" | "error";

export function createOverlayModel(
  players: ChampSelectPlayer[],
  advisorPlayers: ChampSelectAdvisorPlayer[],
  imageUrls: Record<number, string>,
  effectiveLanguage: EffectiveLanguage,
  t: T,
  premadeGroups: number[][] = [],
): OverlayModel {
  const advisorBySummonerId = new Map(advisorPlayers.map((player) => [player.summonerId, player]));
  const premadeGroupBySummonerId = new Map<number, number>();
  premadeGroups.forEach((group, groupIndex) => {
    for (const summonerId of group) {
      premadeGroupBySummonerId.set(summonerId, groupIndex);
    }
  });
  const rawAllies = fillTeam(players.filter((player) => player.team === "ally")).map((player, index) =>
    playerView(player, advisorBySummonerId, premadeGroupBySummonerId, index, "ally", imageUrls, effectiveLanguage, t),
  );
  const rawEnemies = fillTeam(players.filter((player) => player.team === "enemy")).map((player, index) =>
    playerView(player, advisorBySummonerId, premadeGroupBySummonerId, index, "enemy", imageUrls, effectiveLanguage, t),
  );

  // Scout Score only carries meaning relative to the other players in this match
  // (see CONTEXT.md), so size each bar against the highest score on the board
  // rather than a fixed absolute scale — the latter saturated strong players and
  // crushed weak ones into an indistinguishable sliver.
  const maxScore = Math.max(0, ...[...rawAllies, ...rawEnemies].map((player) => player.score ?? 0));
  const withScorePct = (player: PlayerView): PlayerView => ({ ...player, scorePct: scoreWidth(player.score, maxScore) });
  const allies = rawAllies.map(withScorePct);
  const enemies = rawEnemies.map(withScorePct);

  return {
    allies,
    enemies,
    allyTeam: teamSummary(allies),
    enemyTeam: teamSummary(enemies),
    summary: {
      allyGames: teamGames(allies),
      allyWins: teamWins(allies),
      enemyGames: teamGames(enemies),
      enemyWins: teamWins(enemies),
    },
  };
}

export function playerView(
  player: ChampSelectPlayer | null,
  advisorBySummonerId: Map<number, ChampSelectAdvisorPlayer>,
  premadeGroupBySummonerId: Map<number, number>,
  index: number,
  tone: TeamTone,
  imageUrls: Record<number, string>,
  effectiveLanguage: EffectiveLanguage,
  t: T,
): PlayerView {
  const advisorPlayer = player ? advisorBySummonerId.get(player.summonerId) : undefined;
  const rows = fillMatches(player?.recentStats?.recentMatches ?? []).map((match, matchIndex) => ({
    id: match ? `${match.gameId}` : `${tone}-${index}-empty-${matchIndex}`,
    imageUrl: match?.championId ? imageUrls[match.championId] : undefined,
    match,
  }));
  const soloRank = player?.rankedQueues.find((queue) => queue.queue === "soloDuo");
  const flexRank = player?.rankedQueues.find((queue) => queue.queue === "flex");
  const stats = player?.recentStats ?? null;
  const winCount = stats?.recentMatches.filter((match) => match.result === "win").length ?? 0;
  const gameCount = stats?.recentMatches.length ?? 0;
  const streaks = matchStreaks(stats?.recentMatches ?? []);

  return {
    id: player ? `${tone}-${player.summonerId}` : `${tone}-empty-${index}`,
    averageKda: stats?.averageKda ?? null,
    badge: playerBadge(winCount, gameCount, tone),
    championId: player?.championId,
    championUrl: player?.championId ? imageUrls[player.championId] : undefined,
    displayName: player?.displayName ?? t("overlay.unselected"),
    flexRank: rankValue(flexRank, effectiveLanguage, t),
    flexRankTier: flexRank?.isRanked && flexRank.tier ? flexRank.tier.toLowerCase() : null,
    flexRankLP: flexRank?.isRanked ? (flexRank.leaguePoints ?? null) : null,
    gameCount,
    advisorTags: advisorPlayer?.tags ?? [],
    advisorSummary: advisorSummaryText(advisorPlayer),
    isEmpty: !player,
    masteryLevel: player?.masteryLevel ?? null,
    premadeGroup: player ? (premadeGroupBySummonerId.get(player.summonerId) ?? null) : null,
    rows,
    score: playerScore(player),
    scorePct: 0,
    soloRank: rankValue(soloRank, effectiveLanguage, t),
    soloRankTier: soloRank?.isRanked && soloRank.tier ? soloRank.tier.toLowerCase() : null,
    soloRankLP: soloRank?.isRanked ? (soloRank.leaguePoints ?? null) : null,
    soloRankWins: soloRank?.wins ?? 0,
    soloRankLosses: soloRank?.losses ?? 0,
    summonerLevel: player?.summonerLevel ?? null,
    recentStatsStatus: player?.recentStatsStatus ?? "notRequested",
    winCount,
    winRate: gameCount > 0 ? Math.round((winCount / gameCount) * 100) : null,
    winningStreak: streaks.winningStreak,
    losingStreak: streaks.losingStreak,
  };
}

/**
 * Consecutive wins/losses counting back from the most recent game
 * (League Akari's win-loss aggregation): the streak of whichever result the
 * latest game had, stopping at the first opposite result. Unknown results
 * (remakes) break both streaks.
 */
export function matchStreaks(matches: RecentMatchSummary[]): {
  winningStreak: number;
  losingStreak: number;
} {
  let winningStreak = 0;
  let losingStreak = 0;

  for (const match of matches) {
    if (match.result === "win") {
      if (losingStreak > 0) break;
      winningStreak += 1;
      continue;
    }
    if (match.result === "loss") {
      if (winningStreak > 0) break;
      losingStreak += 1;
      continue;
    }
    break;
  }

  return { winningStreak, losingStreak };
}

export function teamSummary(players: PlayerView[]): TeamSummary {
  const games = teamGames(players);
  const wins = teamWins(players);
  let kills = 0;
  let deaths = 0;
  let assists = 0;

  for (const player of players) {
    for (const row of player.rows) {
      if (!row.match) continue;
      kills += row.match.kills;
      deaths += row.match.deaths;
      assists += row.match.assists;
    }
  }

  return {
    games,
    wins,
    winRate: games > 0 ? Math.round((wins / games) * 100) : null,
    // Aggregated team KDA, League Akari-style: total (K+A)/D across members.
    kda: games > 0 ? Math.round(((kills + assists) / Math.max(1, deaths)) * 100) / 100 : null,
  };
}

export function advisorSummaryText(player: ChampSelectAdvisorPlayer | undefined) {
  if (!player) {
    return null;
  }

  return player.matchupAdvice ?? player.advisor?.laneAdvice ?? player.advisor?.powerSpikes[0]?.description ?? null;
}

const ADVISOR_TAG_KEY: Record<AdvisorPlayerTagKind, TranslationKey> = {
  oneTrick: "advisorTag.oneTrick",
  lossStreak: "advisorTag.lossStreak",
  strongPick: "advisorTag.strongPick",
  lowWinRate: "advisorTag.lowWinRate",
  stable: "advisorTag.stable",
  spike: "advisorTag.spike",
};

/**
 * Localize an advisor tag in the current UI language. The backend ships a
 * stable `kind` plus an optional numeric `value` (loss-streak count, spike
 * timing) so the label re-translates live when the language changes, rather
 * than being baked into one language at fetch time.
 */
export function advisorTagLabel(tag: AdvisorPlayerTag, t: T): string {
  const label = t(ADVISOR_TAG_KEY[tag.kind]);
  return tag.value ? label.replace("{n}", tag.value) : label;
}

export function advisorTagClass(tone: AdvisorPlayerTag["tone"]) {
  switch (tone) {
    case "good":
      return "bg-emerald-100 text-emerald-800";
    case "warn":
      return "bg-amber-100 text-amber-800";
    case "info":
      return "bg-zinc-100 text-zinc-700";
  }
}

export function fillTeam(players: ChampSelectPlayer[]) {
  return Array.from({ length: TEAM_SIZE }, (_, index) => players[index] ?? null);
}

export function fillMatches(matches: RecentMatchSummary[]) {
  return Array.from({ length: MATCH_ROWS }, (_, index) => matches[index] ?? null);
}

export function teamWins(players: PlayerView[]) {
  return players.reduce((total, player) => total + player.winCount, 0);
}

export function teamGames(players: PlayerView[]) {
  return players.reduce((total, player) => total + player.gameCount, 0);
}

export function playerScore(player: ChampSelectPlayer | null) {
  const stats = player?.recentStats;
  if (!stats || stats.recentMatches.length === 0) {
    return null;
  }

  const wins = stats.recentMatches.filter((match) => match.result === "win").length;
  const kda = stats.averageKda ?? 0;
  const volume = stats.matchCount * 408;

  return Math.round(volume + wins * 777 + kda * 1200);
}

export function playerBadge(wins: number, games: number, tone: TeamTone) {
  if (games === 0) {
    return "";
  }

  if (tone === "ally") {
    return String(Math.max(1, wins));
  }

  return String(Math.max(1, games - wins));
}

export function scoreWidth(score: number | null, maxScore: number) {
  if (score === null || maxScore <= 0) {
    return 0;
  }

  const pct = Math.round((score / maxScore) * 100);
  return Math.max(8, Math.min(100, pct));
}

export function rankValue(summary: RankedQueueSummary | undefined, language: EffectiveLanguage, t: T) {
  if (!summary) {
    return null;
  }

  if (!summary.isRanked || !summary.tier) {
    return t("overlay.unranked");
  }

  const tier = rankTierLabel(summary.tier, language);
  const division = summary.division ? romanToNumber(summary.division) : "";

  return `${tier}${division}`;
}

export function initialSnapshotMessage(status: InitialSnapshotStatus, t: T) {
  if (status === "loading") {
    return t("common.loading");
  }

  if (status === "error") {
    return t("overlay.historyUnavailable");
  }

  return t("overlay.empty");
}

export function recentStatsStatusMessage(status: ChampSelectRecentStatsStatus, t: T) {
  if (status === "missingIdentity") {
    return t("overlay.historyIdentityUnavailable");
  }

  if (status === "unavailable") {
    return t("overlay.historyUnavailableShort");
  }

  if (status === "notRequested") {
    return "--";
  }

  return "0/0";
}

export function rankTierLabel(tier: string, language: EffectiveLanguage) {
  const zhLabels: Record<string, string> = {
    IRON: "黑铁",
    BRONZE: "青铜",
    SILVER: "白银",
    GOLD: "黄金",
    PLATINUM: "铂金",
    EMERALD: "翡翠",
    DIAMOND: "钻石",
    MASTER: "大师",
    GRANDMASTER: "宗师",
    CHALLENGER: "王者",
  };

  const enLabels: Record<string, string> = {
    IRON: "Iron",
    BRONZE: "Bronze",
    SILVER: "Silver",
    GOLD: "Gold",
    PLATINUM: "Platinum",
    EMERALD: "Emerald",
    DIAMOND: "Diamond",
    MASTER: "Master",
    GRANDMASTER: "Grandmaster",
    CHALLENGER: "Challenger",
  };

  const labels = language === "zh" ? zhLabels : enLabels;
  return labels[tier.toUpperCase()] ?? tier;
}

export function formatGameTime(seconds: number) {
  const totalSeconds = Math.max(0, Math.floor(seconds));
  const minutes = Math.floor(totalSeconds / 60);
  const remainder = totalSeconds % 60;
  return `${minutes}:${String(remainder).padStart(2, "0")}`;
}

export function formatNumber(value: number) {
  return new Intl.NumberFormat("en-US").format(value);
}

export function formatSignedNumber(value: number) {
  const prefix = value > 0 ? "+" : "";
  return `${prefix}${formatNumber(value)}`;
}

const LIVE_EVENT_LABEL_KEYS: Partial<Record<string, TranslationKey>> = {
  ChampionKill: "live.eventChampionKill",
  FirstBlood: "live.eventFirstBlood",
  Multikill: "live.eventMultikill",
  Ace: "live.eventAce",
  TurretKilled: "live.eventTurretKilled",
  InhibKilled: "live.eventInhibKilled",
  DragonKill: "live.eventDragonKill",
  HeraldKill: "live.eventHeraldKill",
  BaronKill: "live.eventBaronKill",
  GameStart: "live.eventGameStart",
  MinionsSpawning: "live.eventMinionsSpawning",
  FirstBrick: "live.eventFirstBrick",
};

export function eventSummary(event: LiveOverlaySnapshot["events"][number], t: T) {
  const key = LIVE_EVENT_LABEL_KEYS[event.eventName];
  const label = key ? t(key) : event.eventName;
  if (event.actor && event.victim) {
    return `${label}: ${event.actor} → ${event.victim}`;
  }

  return label;
}

export function romanToNumber(value: string) {
  const labels: Record<string, string> = {
    I: "I",
    II: "II",
    III: "III",
    IV: "IV",
  };

  return labels[value.toUpperCase()] ?? value;
}

export function resultClass(result: MatchResult) {
  if (result === "win") {
    return "border-emerald-200 bg-emerald-50 text-emerald-800";
  }
  if (result === "loss") {
    return "border-rose-200 bg-rose-50 text-rose-700";
  }

  return "border-zinc-200 bg-zinc-50 text-zinc-500";
}

export function matchResultLabel(result: MatchResult, t: T) {
  if (result === "win") {
    return t("overlay.matchWin");
  }
  if (result === "loss") {
    return t("overlay.matchLoss");
  }

  return t("overlay.matchUnknown");
}

/**
 * Compact `MM-DD HH:mm` timestamp for a match row (League Akari's format).
 * Accepts the two shapes the backend emits: an ISO date string or an
 * epoch-milliseconds string.
 */
export function formatMatchDate(playedAt: string | null): string | null {
  if (!playedAt) {
    return null;
  }

  const date = /^\d+$/.test(playedAt) ? new Date(Number(playedAt)) : new Date(playedAt);
  if (Number.isNaN(date.getTime())) {
    return null;
  }

  const pad = (value: number) => String(value).padStart(2, "0");
  return `${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

/**
 * League Akari's win-rate coloring: >=53% reads good, <=47% reads bad, the
 * band in between is neutral noise.
 */
export function winRateToneClass(winRate: number | null) {
  if (winRate === null) {
    return "text-zinc-500";
  }
  if (winRate >= 53) {
    return "text-emerald-400";
  }
  if (winRate <= 47) {
    return "text-red-400";
  }

  return "text-zinc-200";
}

export function kdaToneClass(kda: number | null) {
  if (kda === null) {
    return "text-zinc-500";
  }
  if (kda >= 4) {
    return "text-emerald-400";
  }
  if (kda <= 2) {
    return "text-red-400";
  }

  return "text-zinc-200";
}

/**
 * Premade badge palette (League Akari's PREMADE_TEAM_COLORS, dark theme).
 * Indexed by premade-group position; letters label the groups in the UI.
 */
export const PREMADE_GROUP_STYLES: { letter: string; background: string; color: string }[] = [
  { letter: "A", background: "#48e5db", color: "#000000" },
  { letter: "B", background: "#628aff", color: "#000000" },
  { letter: "C", background: "#d4de17", color: "#000000" },
  { letter: "D", background: "#2eda3e", color: "#000000" },
  { letter: "E", background: "#ff9f1c", color: "#000000" },
  { letter: "F", background: "#da4e2e", color: "#ffffff" },
];

export function premadeGroupStyle(groupIndex: number) {
  return PREMADE_GROUP_STYLES[groupIndex % PREMADE_GROUP_STYLES.length];
}
