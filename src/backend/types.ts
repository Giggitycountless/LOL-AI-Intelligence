export type ServiceStatus = "ok" | "degraded";
export type DatabaseStatus = "ok" | "unavailable";
export type StartupPage = "dashboard" | "profile" | "matches" | "advisor" | "settings";
export type AppLanguagePreference = "system" | "zh" | "en";
export type AppThemePreference = "light" | "dark";
export type ActivityKind = "note" | "settings" | "system";
export type LeagueClientConnection = "connected" | "unavailable";
export type AutoAcceptStatusState =
  | "disabled"
  | "waitingForClient"
  | "connected"
  | "searching"
  | "readyCheckDetected"
  | "accepting"
  | "accepted"
  | "error";
export type LeagueClientPhase =
  | "notRunning"
  | "lockfileMissing"
  | "connecting"
  | "connected"
  | "unauthorized"
  | "notLoggedIn"
  | "patching"
  | "partialData"
  | "unavailable";
export type LeagueDataSection = "champions" | "ranked" | "advisor" | "matches" | "participants" | "recentStats" | "liveOverlay";
export type RankedQueue = "soloDuo" | "flex" | "other";
export type RankedChampionLane = "top" | "jungle" | "middle" | "bottom" | "support";
export type RankedChampionSort = "overall" | "winRate" | "banRate" | "pickRate";
export type RankedChampionDataStatus = "sample" | "cached" | "fresh" | "staleCache";
export type MatchResult = "win" | "loss" | "unknown";
export type KdaTag = "high" | "standard" | "unavailable";
export type ChampSelectTeam = "ally" | "enemy";

export type HealthcheckResult = {
  status: ServiceStatus;
  databaseStatus: DatabaseStatus;
  schemaVersion: number | null;
};

export type AppSettings = {
  startupPage: StartupPage;
  language: AppLanguagePreference;
  theme: AppThemePreference;
  compactMode: boolean;
  activityLimit: number;
  autoAcceptEnabled: boolean;
  autoPickEnabled: boolean;
  autoPickChampionId: number | null;
  autoPickDelaySeconds: number;
  autoBanEnabled: boolean;
  autoBanChampionId: number | null;
  autoBanDelaySeconds: number;
  aiBaseUrl: string | null;
  aiApiKey: string | null;
  aiModel: string | null;
  updatedAt: string;
};

export type AiConfig = {
  baseUrl: string | null;
  apiKey: string | null;
  model: string | null;
};

export type AiAnalysisCache = {
  scope: string;
  resultText: string;
  gameCountAtAnalysis: number;
  analyzedAt: string;
};

export type ChatPreset = {
  slot: number;
  label: string;
  message: string;
  updatedAt: string;
};

export type SaveSettingsInput = {
  startupPage: StartupPage;
  language: AppLanguagePreference;
  theme: AppThemePreference;
  compactMode: boolean;
  activityLimit: number;
  autoAcceptEnabled: boolean;
  autoPickEnabled: boolean;
  autoPickChampionId: number | null;
  autoPickDelaySeconds: number;
  autoBanEnabled: boolean;
  autoBanChampionId: number | null;
  autoBanDelaySeconds: number;
  aiBaseUrl: string | null;
  aiApiKey: string | null;
  aiModel: string | null;
};

export type RunePage = {
  primaryStyleId: number;
  subStyleId: number;
  selectedPerkIds: number[];
};

export type RuneRecommendation = {
  position: string;
  pickCount: number;
  page: RunePage;
};

export type ChampionRuneConfig = {
  championId: number;
  page: RunePage;
  savedAt: string;
};

export type RunePageSnapshot = {
  championId: number;
  championName: string;
  recommendations: RuneRecommendation[];
  savedConfig: ChampionRuneConfig | null;
  autoApplied: boolean;
};

/** Presence values the League client accepts for `/lol-chat/v1/me`. */
export type ChatAvailability = "chat" | "away" | "dnd" | "offline";

/** The local player's chat presence (signature + availability). */
export type ChatMe = {
  availability: string | null;
  statusMessage: string | null;
};

export type ActivityEntry = {
  id: number;
  kind: ActivityKind;
  title: string;
  body: string | null;
  createdAt: string;
};

export type ActivityEntriesResponse = {
  records: ActivityEntry[];
};

export type ActivityListInput = {
  limit?: number;
  kind?: ActivityKind | null;
};

export type ActivityNoteInput = {
  title: string;
  body?: string | null;
};

export type AppSnapshot = {
  health: HealthcheckResult;
  settings: AppSettings;
  settingsDefaults: SaveSettingsInput;
  recentActivity: ActivityEntry[];
};

export type CommandError = {
  code: "validation" | "storage" | "clientUnavailable" | "clientAccess" | "integration" | "internal";
  message: string;
};

export type LeagueClientStatus = {
  isRunning: boolean;
  lockfileFound: boolean;
  connection: LeagueClientConnection;
  phase: LeagueClientPhase;
  message: string | null;
};

export type AutoAcceptStatus = {
  state: AutoAcceptStatusState;
  message: string | null;
};

export type ChampionMasteryEntry = {
  championId: number;
  championName: string;
  masteryLevel: number;
  masteryPoints: number;
};

export type CurrentSummonerProfile = {
  displayName: string;
  summonerLevel: number;
  profileIconId: number | null;
  honorLevel: number | null;
  topMastery: ChampionMasteryEntry[];
};

export type RankedQueueSummary = {
  queue: RankedQueue;
  tier: string | null;
  division: string | null;
  leaguePoints: number | null;
  wins: number;
  losses: number;
  isRanked: boolean;
};

export type RankedChampionStat = {
  championId: number;
  championName: string;
  championAlias: string | null;
  lane: RankedChampionLane;
  winRate: number;
  pickRate: number;
  banRate: number;
  overallScore: number;
  games: number;
  wins: number;
  picks: number;
  bans: number;
};

export type RankedChampionStatsInput = {
  lane?: RankedChampionLane | null;
  sortBy?: RankedChampionSort | null;
};

export type RankedChampionStatsResponse = {
  lane: RankedChampionLane | null;
  sortBy: RankedChampionSort;
  records: RankedChampionStat[];
  source: string;
  updatedAt: string;
  generatedAt: string | null;
  importedAt: string | null;
  patch: string | null;
  region: string | null;
  queue: string | null;
  tier: string | null;
  isCached: boolean;
  dataStatus: RankedChampionDataStatus;
  statusMessage: string | null;
};

export type RankedChampionRefreshInput = RankedChampionStatsInput & {
  tier?: number;
  region?: string;
};

export type AdvisorNamedRef = {
  id: number | null;
  name: string;
};

export type AdvisorRunePage = {
  primaryStyle: string;
  primaryRunes: AdvisorNamedRef[];
  secondaryStyle: string;
  secondaryRunes: AdvisorNamedRef[];
  statShards: string[];
};

export type AdvisorSkillOrder = {
  maxOrder: string[];
  earlyOrder: string[];
};

export type AdvisorItemBuild = {
  starter: AdvisorNamedRef[];
  core: AdvisorNamedRef[];
  boots: AdvisorNamedRef[];
  late: AdvisorNamedRef[];
  situational: AdvisorNamedRef[];
};

export type AdvisorMatchup = {
  championId: number;
  championName: string;
  note: string;
  winRateDelta: number | null;
};

export type AdvisorPowerSpike = {
  timing: string;
  label: string;
  description: string;
};

export type AdvisorRecord = {
  championId: number;
  championName: string;
  championAlias: string | null;
  lane: RankedChampionLane;
  winRate: number;
  pickRate: number;
  banRate: number;
  overallScore: number;
  games: number;
  runes: AdvisorRunePage;
  summonerSpells: AdvisorNamedRef[];
  skillOrder: AdvisorSkillOrder;
  itemBuild: AdvisorItemBuild;
  strongAgainst: AdvisorMatchup[];
  weakAgainst: AdvisorMatchup[];
  powerSpikes: AdvisorPowerSpike[];
  laneAdvice: string;
  teamfightAdvice: string;
};

export type AdvisorDataInput = {
  lane?: RankedChampionLane | null;
  championId?: number | null;
};

export type AdvisorDataResponse = {
  lane: RankedChampionLane | null;
  championId: number | null;
  records: AdvisorRecord[];
  source: string;
  updatedAt: string;
  generatedAt: string | null;
  importedAt: string | null;
  patch: string | null;
  region: string | null;
  queue: string | null;
  tier: string | null;
  isCached: boolean;
  dataStatus: RankedChampionDataStatus;
  statusMessage: string | null;
};

export type AdvisorDataRefreshInput = AdvisorDataInput & {
  url?: string | null;
};

export type AdvisorTagTone = "good" | "warn" | "info";

export type AdvisorPlayerTagKind =
  | "oneTrick"
  | "lossStreak"
  | "strongPick"
  | "lowWinRate"
  | "stable"
  | "spike";

export type AdvisorPlayerTag = {
  kind: AdvisorPlayerTagKind;
  // Numeric detail for tags that carry one (loss-streak count, spike timing);
  // interpolated into the localized label via the {n} placeholder.
  value?: string | null;
  tone: AdvisorTagTone;
};

export type ChampSelectAdvisorPlayer = {
  summonerId: number;
  displayName: string;
  championId: number | null;
  championName: string | null;
  team: ChampSelectTeam;
  recentStats: ParticipantRecentStats | null;
  recentStatsStatus: ChampSelectRecentStatsStatus;
  tags: AdvisorPlayerTag[];
  advisor: AdvisorRecord | null;
  matchupAdvice: string | null;
};

export type ChampSelectAdvisorSnapshot = {
  players: ChampSelectAdvisorPlayer[];
  cachedAt: string;
  advisorSource: string;
  advisorPatch: string | null;
  dataStatus: RankedChampionDataStatus;
};

export type LiveOverlaySnapshot = {
  gameTimeSeconds: number | null;
  gameMode: string | null;
  mapName: string | null;
  activePlayer: LiveOverlayActivePlayer | null;
  players: LiveOverlayPlayer[];
  events: LiveOverlayEvent[];
  gold: LiveOverlayGoldSummary;
  refreshedAt: string;
};

export type LiveOverlayActivePlayer = {
  displayName: string;
  level: number | null;
  currentGold: number | null;
  resourceType: string | null;
  resourceValue: number | null;
  resourceMax: number | null;
};

export type LiveOverlayPlayer = {
  displayName: string;
  championName: string | null;
  team: string;
  level: number | null;
  position: string | null;
  isDead: boolean;
  respawnTimer: number | null;
  items: LiveOverlayItem[];
  scores: LiveOverlayScores | null;
  summonerSpells: AdvisorNamedRef[];
};

export type LiveOverlayItem = {
  itemId: number;
  displayName: string;
  price: number;
  count: number;
  slot: number | null;
};

export type LiveOverlayScores = {
  kills: number;
  deaths: number;
  assists: number;
  creepScore: number;
  wardScore: number;
};

export type LiveOverlayEvent = {
  eventId: number;
  eventName: string;
  eventTime: number;
  actor: string | null;
  victim: string | null;
  assistingParticipants: string[];
};

export type LiveOverlayGoldSummary = {
  allyItemValue: number;
  enemyItemValue: number;
  itemValueDiff: number;
};

export type RecentMatchSummary = {
  gameId: number;
  championId: number | null;
  championName: string;
  queueName: string | null;
  result: MatchResult;
  kills: number;
  deaths: number;
  assists: number;
  kda: number | null;
  playedAt: string | null;
  gameDurationSeconds: number | null;
};

export type RecentChampionSummary = {
  championId: number | null;
  championName: string;
  games: number;
};

export type RecentPerformanceSummary = {
  matchCount: number;
  averageKda: number | null;
  kdaTag: KdaTag;
  recentChampions: string[];
  topChampions: RecentChampionSummary[];
};

export type ChampionRecordSummary = {
  championId: number;
  wins: number;
  losses: number;
  games: number;
};

export type LeagueDataWarning = {
  section: LeagueDataSection;
  message: string;
};

export type LeagueSelfSnapshot = {
  status: LeagueClientStatus;
  summoner: CurrentSummonerProfile | null;
  rankedQueues: RankedQueueSummary[];
  recentMatches: RecentMatchSummary[];
  recentPerformance: RecentPerformanceSummary;
  championRecords: ChampionRecordSummary[];
  dataWarnings: LeagueDataWarning[];
  refreshedAt: string;
};

export type LeagueSelfSnapshotInput = {
  matchLimit?: number;
};

export type LeagueChampionSummary = {
  championId: number;
  championName: string;
  championAlias: string | null;
};

export type AbilityStat = {
  label: string;
  values: number[];
  suffix: string;
};

export type LeagueChampionAbility = {
  slot: string;
  name: string;
  description: string;
  summaryDescription: string;
  icon: LeagueImageAsset | null;
  cooldown: string | null;
  cost: string | null;
  range: string | null;
  cooldownValues: string[];
  costValues: string[];
  rangeValues: string[];
  stats?: AbilityStat[];
};

export type LeagueChampionDetails = {
  championId: number;
  championName: string;
  title: string | null;
  squarePortrait: LeagueImageAsset | null;
  abilities: LeagueChampionAbility[];
};

export type LeagueImageAsset = {
  mimeType: string;
  /** Base64-encoded image bytes (see domain::LeagueImageAsset). */
  bytes: string;
};

export type LeagueGameAssetKind = "item" | "rune" | "spell";

export type LeagueGameAsset = {
  kind: LeagueGameAssetKind;
  assetId: number;
  name: string;
  description: string | null;
  image: LeagueImageAsset;
};

export type PlayerNoteSummary = {
  hasNote: boolean;
  note: string | null;
  tags: string[];
};

export type PlayerNoteView = {
  gameId: number;
  participantId: number;
  note: string | null;
  tags: string[];
  updatedAt: string | null;
};

export type ClearPlayerNoteResult = {
  cleared: boolean;
};

export type PostMatchDetail = {
  gameId: number;
  queueName: string | null;
  playedAt: string | null;
  gameDurationSeconds: number | null;
  result: MatchResult;
  selfParticipantId: number | null;
  teams: PostMatchTeam[];
  comparison: PostMatchComparison;
  warnings: LeagueDataWarning[];
};

export type PostMatchTeam = {
  teamId: number;
  result: MatchResult;
  participants: PostMatchParticipant[];
  totals: PostMatchTeamTotals;
};

export type PostMatchParticipant = {
  participantId: number;
  teamId: number;
  displayName: string;
  championId: number | null;
  championName: string;
  role: string | null;
  lane: string | null;
  profileIconId: number | null;
  result: MatchResult;
  kills: number;
  deaths: number;
  assists: number;
  kda: number | null;
  performanceScore: number;
  cs: number;
  goldEarned: number;
  damageToChampions: number;
  physicalDamageToChampions: number;
  magicDamageToChampions: number;
  trueDamageToChampions: number;
  damageToObjectives: number;
  damageToTurrets: number;
  damageTaken: number;
  visionScore: number;
  wardsPlaced: number;
  wardsKilled: number;
  controlWardsBought: number;
  timeSpentDeadSeconds: number;
  largestKillingSpree: number;
  largestMultiKill: number;
  doubleKills: number;
  tripleKills: number;
  quadraKills: number;
  pentaKills: number;
  firstBlood: boolean;
  firstTower: boolean;
  items: number[];
  runes: number[];
  spells: number[];
  noteSummary: PlayerNoteSummary;
};

export type PostMatchTeamTotals = {
  kills: number;
  deaths: number;
  assists: number;
  goldEarned: number;
  damageToChampions: number;
  visionScore: number;
};

export type PostMatchComparison = {
  highestKda: ParticipantMetricLeader | null;
  mostCs: ParticipantMetricLeader | null;
  mostGold: ParticipantMetricLeader | null;
  mostDamage: ParticipantMetricLeader | null;
  highestVision: ParticipantMetricLeader | null;
};

export type ParticipantMetricLeader = {
  participantId: number;
  displayName: string;
  value: number;
};

export type ParticipantRecentStats = {
  matchCount: number;
  averageKda: number | null;
  recentChampions: string[];
  recentMatches: RecentMatchSummary[];
};

export type ChampSelectRecentStatsStatus = "notRequested" | "missingIdentity" | "loaded" | "unavailable";

export type ParticipantPublicProfile = {
  gameId: number;
  participantId: number;
  displayName: string;
  profileIconId: number | null;
  recentStats: ParticipantRecentStats | null;
  note: PlayerNoteView | null;
  warnings: LeagueDataWarning[];
};

export type ChampSelectPlayer = {
  summonerId: number;
  displayName: string;
  championId: number | null;
  championName: string | null;
  team: ChampSelectTeam;
  rankedQueues: RankedQueueSummary[];
  summonerLevel: number | null;
  masteryLevel: number | null;
  recentStats: ParticipantRecentStats | null;
  recentStatsStatus: ChampSelectRecentStatsStatus;
};

export type ChampSelectSnapshot = {
  players: ChampSelectPlayer[];
  cachedAt: string;
};

export type ParticipantPublicProfileInput = {
  gameId: number;
  participantId: number;
  recentLimit?: number;
};

export type SavePlayerNoteInput = {
  gameId: number;
  participantId: number;
  note: string | null;
  tags: string[];
};

export type ClearPlayerNoteInput = {
  gameId: number;
  participantId: number;
};

export type LocalDataExport = {
  formatVersion: 1;
  settings: SaveSettingsInput;
  activityEntries: Array<{
    kind: ActivityKind;
    title: string;
    body: string | null;
    createdAt: string;
  }>;
};

export type ImportLocalDataResult = {
  settings: AppSettings;
  importedActivityCount: number;
};

export type ClearActivityResult = {
  deletedCount: number;
};

export type Feedback = {
  kind: "success" | "error";
  message: string;
};
