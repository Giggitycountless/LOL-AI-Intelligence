import type {
  ActivityEntry,
  ActivityListInput,
  ActivityNoteInput,
  AdvisorDataInput,
  AdvisorDataRefreshInput,
  AdvisorDataResponse,
  AppSnapshot,
  AppLanguagePreference,
  AutoAcceptStatus,
  ChampSelectSnapshot,
  ChampSelectAdvisorSnapshot,
  Feedback,
  LeagueChampionAbility,
  LeagueChampionDetails,
  LeagueGameAsset,
  LeagueGameAssetKind,
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
  SaveSettingsInput,
} from "../backend/types";
import type { EffectiveLanguage, TranslationKey } from "../i18n";

export type LeagueImageUrls = {
  profileIcons: Record<number, string>;
  championIcons: Record<number, string>;
  gameAssets: Record<string, LeagueGameAssetView>;
};

export type AppWindowMode = "main" | "overlay" | "participant";

export type LeagueGameAssetView = Omit<LeagueGameAsset, "image"> & {
  imageUrl: string;
};

export type LeagueChampionAbilityView = Omit<LeagueChampionAbility, "icon"> & {
  iconUrl: string | null;
};

export type LeagueChampionDetailsView = Omit<LeagueChampionDetails, "squarePortrait" | "abilities"> & {
  squarePortraitUrl: string | null;
  abilities: LeagueChampionAbilityView[];
};

export type AppCoreContextValue = {
  snapshot: AppSnapshot | null;
  activityEntries: ActivityEntry[];
  leagueSelfSnapshot: LeagueSelfSnapshot | null;
  rankedChampionStats: RankedChampionStatsResponse | null;
  postMatchDetails: Record<number, PostMatchDetail>;
  participantProfiles: Record<string, ParticipantPublicProfile>;
  autoAcceptStatus: AutoAcceptStatus | null;
  isLoading: boolean;
  isActivityLoading: boolean;
  isLeagueClientLoading: boolean;
  isRankedChampionStatsLoading: boolean;
  feedback: Feedback | null;
  languagePreference: AppLanguagePreference;
  effectiveLanguage: EffectiveLanguage;
  t: (key: TranslationKey) => string;
  clearFeedback: () => void;
  refresh: () => Promise<boolean>;
  loadActivityEntries: (input: ActivityListInput) => Promise<boolean>;
  refreshLeagueClient: (input?: LeagueSelfSnapshotInput) => Promise<boolean>;
  loadRankedChampionStats: (input: RankedChampionStatsInput) => Promise<boolean>;
  refreshRankedChampionStats: (input: RankedChampionRefreshInput) => Promise<boolean>;
  saveSettings: (settings: SaveSettingsInput) => Promise<boolean>;
  setLanguagePreference: (language: AppLanguagePreference) => Promise<boolean>;
  createActivityNote: (input: ActivityNoteInput) => Promise<boolean>;
  clearActivityEntries: (confirm: boolean) => Promise<boolean>;
  exportLocalData: () => Promise<string | null>;
  importLocalData: (json: string) => Promise<boolean>;
  loadPostMatchDetail: (gameId: number) => Promise<boolean>;
  loadParticipantProfile: (input: ParticipantPublicProfileInput) => Promise<boolean>;
  savePlayerNote: (input: SavePlayerNoteInput) => Promise<PlayerNoteView | null>;
  clearPlayerNote: (gameId: number, participantId: number) => Promise<boolean>;
};

export type LeagueAssetsContextValue = {
  championDetailsById: Record<number, LeagueChampionDetailsView>;
  leagueImages: LeagueImageUrls;
  loadLeagueProfileIcon: (profileIconId: number | null | undefined) => Promise<boolean>;
  loadLeagueChampionIcon: (championId: number | null | undefined) => Promise<boolean>;
  loadLeagueChampionDetails: (championId: number | null | undefined) => Promise<boolean>;
  loadLeagueGameAsset: (kind: LeagueGameAssetKind, assetId: number | null | undefined) => Promise<boolean>;
};

export type ChampSelectContextValue = {
  champSelectSnapshot: ChampSelectSnapshot | null;
  refreshChampSelectSnapshot: () => Promise<boolean>;
};

export type AdvisorContextValue = {
  advisorData: AdvisorDataResponse | null;
  champSelectAdvisorSnapshot: ChampSelectAdvisorSnapshot | null;
  liveOverlaySnapshot: LiveOverlaySnapshot | null;
  isAdvisorDataLoading: boolean;
  advisorDataError: string | null;
  loadAdvisorData: (input: AdvisorDataInput) => Promise<boolean>;
  refreshAdvisorData: (input: AdvisorDataRefreshInput) => Promise<boolean>;
  refreshChampSelectAdvisorSnapshot: () => Promise<boolean>;
  refreshLiveOverlaySnapshot: () => Promise<boolean>;
};

/** Compatibility hook shape. Prefer the narrower hooks below for new UI code. */
export type AppStateContextValue = AppCoreContextValue &
  LeagueAssetsContextValue &
  ChampSelectContextValue &
  AdvisorContextValue;
