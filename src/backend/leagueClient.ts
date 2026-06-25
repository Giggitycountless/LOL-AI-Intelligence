import { callBackend } from "./commands";
import type {
  AdvisorDataInput,
  ChampionRuneConfig,
  RunePage,
  RuneRecommendation,
  AdvisorDataRefreshInput,
  AdvisorDataResponse,
  ChampSelectSnapshot,
  ChampSelectAdvisorSnapshot,
  ClearPlayerNoteInput,
  ClearPlayerNoteResult,
  AutoAcceptStatus,
  ChatAvailability,
  ChatMe,
  LeagueGameAsset,
  LeagueGameAssetKind,
  LeagueChampionDetails,
  LeagueChampionSummary,
  LeagueClientStatus,
  LeagueImageAsset,
  LeagueSelfSnapshot,
  LeagueSelfSnapshotInput,
  LiveOverlaySnapshot,
  ParticipantPublicProfile,
  ParticipantPublicProfileInput,
  PlayerNoteView,
  PostMatchDetail,
  RankedChampionRefreshInput,
  RankedChampionStatsInput,
  RankedChampionStatsResponse,
  SavePlayerNoteInput,
} from "./types";

export function fetchLeagueClientStatus(): Promise<LeagueClientStatus> {
  return callBackend<LeagueClientStatus>("get_league_client_status");
}

export function fetchAutoAcceptStatus(): Promise<AutoAcceptStatus> {
  return callBackend<AutoAcceptStatus>("get_auto_accept_status");
}

export function fetchLeagueChampionCatalog(): Promise<LeagueChampionSummary[]> {
  return callBackend<LeagueChampionSummary[]>("get_league_champion_catalog");
}

export function fetchLeagueSelfSnapshot(input: LeagueSelfSnapshotInput = { matchLimit: 6 }): Promise<LeagueSelfSnapshot> {
  return callBackend<LeagueSelfSnapshot>("get_league_self_snapshot", {
    input,
  });
}

export function fetchChampSelectSnapshot(recentLimit: number = 6): Promise<ChampSelectSnapshot> {
  return callBackend<ChampSelectSnapshot>("get_champ_select_snapshot", {
    input: { recentLimit },
  });
}

export function fetchRankedChampionStats(input: RankedChampionStatsInput): Promise<RankedChampionStatsResponse> {
  return callBackend<RankedChampionStatsResponse>("get_ranked_champion_stats", {
    input,
  });
}

export function refreshRankedChampionStats(input: RankedChampionRefreshInput): Promise<RankedChampionStatsResponse> {
  return callBackend<RankedChampionStatsResponse>("refresh_ranked_champion_stats", {
    input,
  });
}

export function fetchAdvisorData(input: AdvisorDataInput): Promise<AdvisorDataResponse> {
  return callBackend<AdvisorDataResponse>("get_advisor_data", {
    input,
  });
}

export function refreshAdvisorData(input: AdvisorDataRefreshInput): Promise<AdvisorDataResponse> {
  return callBackend<AdvisorDataResponse>("refresh_advisor_data", {
    input,
  });
}

export function fetchChampSelectAdvisorSnapshot(recentLimit: number = 6): Promise<ChampSelectAdvisorSnapshot> {
  return callBackend<ChampSelectAdvisorSnapshot>("get_champ_select_advisor_snapshot", {
    input: { recentLimit },
  });
}

export function fetchLiveOverlaySnapshot(): Promise<LiveOverlaySnapshot> {
  return callBackend<LiveOverlaySnapshot>("get_live_overlay_snapshot");
}

export function fetchLeagueProfileIcon(profileIconId: number): Promise<LeagueImageAsset> {
  return callBackend<LeagueImageAsset>("get_league_profile_icon", {
    input: { profileIconId },
  });
}

export function fetchLeagueChampionIcon(championId: number): Promise<LeagueImageAsset> {
  return callBackend<LeagueImageAsset>("get_league_champion_icon", {
    input: { championId },
  });
}

export function fetchRankTierIcon(tier: string): Promise<LeagueImageAsset> {
  return callBackend<LeagueImageAsset>("fetch_rank_tier_icon", { tier });
}

export function fetchLeagueChampionDetails(championId: number): Promise<LeagueChampionDetails> {
  return callBackend<LeagueChampionDetails>("get_league_champion_details", {
    input: { championId },
  });
}

export function fetchLeagueGameAsset(kind: LeagueGameAssetKind, assetId: number): Promise<LeagueGameAsset> {
  return callBackend<LeagueGameAsset>("get_league_game_asset", {
    input: { kind, assetId },
  });
}

export function fetchPostMatchDetail(gameId: number): Promise<PostMatchDetail> {
  return callBackend<PostMatchDetail>("get_post_match_detail", {
    input: { gameId },
  });
}

export function fetchPostMatchParticipantProfile(input: ParticipantPublicProfileInput): Promise<ParticipantPublicProfile> {
  return callBackend<ParticipantPublicProfile>("get_post_match_participant_profile", {
    input,
  });
}

export function savePlayerNote(input: SavePlayerNoteInput): Promise<PlayerNoteView> {
  return callBackend<PlayerNoteView>("save_player_note", {
    input,
  });
}

export function clearPlayerNote(input: ClearPlayerNoteInput): Promise<ClearPlayerNoteResult> {
  return callBackend<ClearPlayerNoteResult>("clear_player_note", {
    input,
  });
}

export function fetchRuneRecommendations(championId: number): Promise<RuneRecommendation[]> {
  return callBackend<RuneRecommendation[]>("get_rune_recommendations", {
    input: { championId },
  });
}

export function applyRunePage(championId: number, page: RunePage, championName: string): Promise<void> {
  return callBackend<void>("apply_rune_page", {
    input: { championId, page, championName },
  });
}

export function fetchChatMe(): Promise<ChatMe> {
  return callBackend<ChatMe>("get_chat_me");
}

export function setChatStatus(
  statusMessage: string | null,
  availability: ChatAvailability | null,
): Promise<void> {
  return callBackend<void>("set_chat_status", {
    input: { statusMessage, availability },
  });
}

export function saveChampionRuneConfig(championId: number, page: RunePage): Promise<ChampionRuneConfig> {
  return callBackend<ChampionRuneConfig>("save_champion_rune_config", {
    input: { championId, page },
  });
}

export function fetchChampionRuneConfig(championId: number): Promise<ChampionRuneConfig | null> {
  return callBackend<ChampionRuneConfig | null>("get_champion_rune_config", {
    input: { championId },
  });
}

export function deleteChampionRuneConfig(championId: number): Promise<boolean> {
  return callBackend<boolean>("delete_champion_rune_config", {
    input: { championId },
  });
}

export function fetchAiConfig(): Promise<import("./types").AiConfig> {
  return callBackend<import("./types").AiConfig>("get_ai_config");
}

export function fetchAiAnalysis(
  scope: string,
  tone: string,
): Promise<import("./types").AiAnalysisCache | null> {
  // Cache is keyed per (scope, tone) — must match the backend's `{scope}:{tone}` key.
  return callBackend<import("./types").AiAnalysisCache | null>("get_ai_analysis", {
    scope: `${scope}:${tone}`,
  });
}

export function saveAiAnalysis(scope: string, resultText: string, gameCount: number): Promise<void> {
  return callBackend<void>("save_ai_analysis", {
    input: { scope, resultText, gameCount },
  });
}

export function fetchChatPresets(): Promise<import("./types").ChatPreset[]> {
  return callBackend<import("./types").ChatPreset[]>("list_chat_presets");
}

export function saveChatPreset(slot: number, label: string, message: string): Promise<import("./types").ChatPreset> {
  return callBackend<import("./types").ChatPreset>("save_chat_preset", { slot, label, message });
}

export function deleteChatPreset(slot: number): Promise<boolean> {
  return callBackend<boolean>("delete_chat_preset", { slot });
}
