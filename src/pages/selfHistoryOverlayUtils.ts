import type {
  AdvisorPlayerTag,
  ChampSelectPlayer,
  ChampSelectAdvisorPlayer,
  ChampSelectRecentStatsStatus,
  LiveOverlaySnapshot,
  MatchResult,
  RankedQueueSummary,
  RecentMatchSummary,
} from "../backend/types";
import type { EffectiveLanguage } from "../i18n";
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
};

export type OverlayModel = {
  allies: PlayerView[];
  enemies: PlayerView[];
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
): OverlayModel {
  const advisorBySummonerId = new Map(advisorPlayers.map((player) => [player.summonerId, player]));
  const rawAllies = fillTeam(players.filter((player) => player.team === "ally")).map((player, index) =>
    playerView(player, advisorBySummonerId, index, "ally", imageUrls, effectiveLanguage, t),
  );
  const rawEnemies = fillTeam(players.filter((player) => player.team === "enemy")).map((player, index) =>
    playerView(player, advisorBySummonerId, index, "enemy", imageUrls, effectiveLanguage, t),
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
  };
}

export function advisorSummaryText(player: ChampSelectAdvisorPlayer | undefined) {
  if (!player) {
    return null;
  }

  return player.matchupAdvice ?? player.advisor?.laneAdvice ?? player.advisor?.powerSpikes[0]?.description ?? null;
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

export function rankTierIconUrl(tier: string | null): string | null {
  if (!tier) return null;
  return `https://cdn.mobalytics.gg/assets/lol/images/rank-icon/summoner-tier/${tier}.png`;
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

export function eventSummary(event: LiveOverlaySnapshot["events"][number]) {
  if (event.actor && event.victim) {
    return `${event.eventName}: ${event.actor} -> ${event.victim}`;
  }

  return event.eventName;
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
