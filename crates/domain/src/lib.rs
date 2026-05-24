use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthReport {
    pub status: ServiceStatus,
    pub database_status: DatabaseStatus,
    pub schema_version: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    Ok,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseStatus {
    Ok,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub health: HealthReport,
    pub settings: AppSettings,
    pub settings_defaults: SettingsValues,
    pub recent_activity: Vec<ActivityEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub startup_page: StartupPage,
    pub language: AppLanguagePreference,
    pub theme: AppThemePreference,
    pub compact_mode: bool,
    pub activity_limit: i64,
    pub auto_accept_enabled: bool,
    pub auto_pick_enabled: bool,
    pub auto_pick_champion_id: Option<i64>,
    pub auto_pick_delay_seconds: f64,
    pub auto_ban_enabled: bool,
    pub auto_ban_champion_id: Option<i64>,
    pub auto_ban_delay_seconds: f64,
    pub ai_base_url: Option<String>,
    pub ai_api_key: Option<String>,
    pub ai_model: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsValues {
    pub startup_page: StartupPage,
    #[serde(default)]
    pub language: AppLanguagePreference,
    #[serde(default)]
    pub theme: AppThemePreference,
    pub compact_mode: bool,
    pub activity_limit: i64,
    #[serde(default = "default_true")]
    pub auto_accept_enabled: bool,
    #[serde(default)]
    pub auto_pick_enabled: bool,
    #[serde(default)]
    pub auto_pick_champion_id: Option<i64>,
    #[serde(default)]
    pub auto_pick_delay_seconds: f64,
    #[serde(default)]
    pub auto_ban_enabled: bool,
    #[serde(default)]
    pub auto_ban_champion_id: Option<i64>,
    #[serde(default)]
    pub auto_ban_delay_seconds: f64,
    #[serde(default)]
    pub ai_base_url: Option<String>,
    #[serde(default)]
    pub ai_api_key: Option<String>,
    #[serde(default)]
    pub ai_model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConfig {
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiAnalysisCache {
    pub scope: String,
    pub result_text: String,
    pub game_count_at_analysis: i64,
    pub analyzed_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatPreset {
    pub slot: i64,
    pub label: String,
    pub message: String,
    pub updated_at: String,
}

fn default_true() -> bool {
    true
}

impl AppSettings {
    pub fn values(&self) -> SettingsValues {
        SettingsValues {
            startup_page: self.startup_page,
            language: self.language,
            theme: self.theme,
            compact_mode: self.compact_mode,
            activity_limit: self.activity_limit,
            auto_accept_enabled: self.auto_accept_enabled,
            auto_pick_enabled: self.auto_pick_enabled,
            auto_pick_champion_id: self.auto_pick_champion_id,
            auto_pick_delay_seconds: self.auto_pick_delay_seconds,
            auto_ban_enabled: self.auto_ban_enabled,
            auto_ban_champion_id: self.auto_ban_champion_id,
            auto_ban_delay_seconds: self.auto_ban_delay_seconds,
            ai_base_url: self.ai_base_url.clone(),
            ai_api_key: self.ai_api_key.clone(),
            ai_model: self.ai_model.clone(),
        }
    }

    pub fn ai_config(&self) -> AiConfig {
        AiConfig {
            base_url: self.ai_base_url.clone(),
            api_key: self.ai_api_key.clone(),
            model: self.ai_model.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AppLanguagePreference {
    #[default]
    System,
    Zh,
    En,
}

impl AppLanguagePreference {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Zh => "zh",
            Self::En => "en",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "system" => Some(Self::System),
            "zh" => Some(Self::Zh),
            "en" => Some(Self::En),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AppThemePreference {
    #[default]
    Light,
    Dark,
}

impl AppThemePreference {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StartupPage {
    Dashboard,
    Activity,
    Settings,
}

impl StartupPage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dashboard => "dashboard",
            Self::Activity => "activity",
            Self::Settings => "settings",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "dashboard" => Some(Self::Dashboard),
            "activity" => Some(Self::Activity),
            "settings" => Some(Self::Settings),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityEntry {
    pub id: i64,
    pub kind: ActivityKind,
    pub title: String,
    pub body: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewActivityEntry {
    pub kind: ActivityKind,
    pub title: String,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ActivityKind {
    Note,
    Settings,
    System,
}

impl ActivityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Settings => "settings",
            Self::System => "system",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "note" => Some(Self::Note),
            "settings" => Some(Self::Settings),
            "system" => Some(Self::System),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalActivityEntry {
    pub kind: ActivityKind,
    pub title: String,
    pub body: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalDataExport {
    pub format_version: i64,
    pub settings: SettingsValues,
    pub activity_entries: Vec<LocalActivityEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportLocalDataResult {
    pub settings: AppSettings,
    pub imported_activity_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearActivityResult {
    pub deleted_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerNoteSummary {
    pub has_note: bool,
    pub note: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerNoteView {
    pub game_id: i64,
    pub participant_id: i64,
    pub note: Option<String>,
    pub tags: Vec<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearPlayerNoteResult {
    pub cleared: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeagueClientStatus {
    pub is_running: bool,
    pub lockfile_found: bool,
    pub connection: LeagueClientConnection,
    pub phase: LeagueClientPhase,
    pub message: Option<String>,
}

impl LeagueClientStatus {
    pub fn unavailable(phase: LeagueClientPhase, message: impl Into<String>) -> Self {
        Self {
            is_running: false,
            lockfile_found: false,
            connection: LeagueClientConnection::Unavailable,
            phase,
            message: Some(message.into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LeagueClientConnection {
    Connected,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LeagueClientPhase {
    NotRunning,
    LockfileMissing,
    Connecting,
    Connected,
    Unauthorized,
    NotLoggedIn,
    Patching,
    PartialData,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoAcceptStatus {
    pub state: AutoAcceptStatusState,
    pub message: Option<String>,
}

impl AutoAcceptStatus {
    pub fn new(state: AutoAcceptStatusState, message: Option<String>) -> Self {
        Self { state, message }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AutoAcceptStatusState {
    Disabled,
    WaitingForClient,
    Connected,
    Searching,
    ReadyCheckDetected,
    Accepting,
    Accepted,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeagueSelfSnapshot {
    pub status: LeagueClientStatus,
    pub summoner: Option<CurrentSummonerProfile>,
    pub ranked_queues: Vec<RankedQueueSummary>,
    pub recent_matches: Vec<RecentMatchSummary>,
    pub recent_performance: RecentPerformanceSummary,
    pub data_warnings: Vec<LeagueDataWarning>,
    pub refreshed_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeagueSelfData {
    pub status: LeagueClientStatus,
    pub summoner: Option<CurrentSummonerProfile>,
    pub ranked_queues: Vec<RankedQueueSummary>,
    pub recent_matches: Vec<RecentMatchSummary>,
    pub data_warnings: Vec<LeagueDataWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeagueDataWarning {
    pub section: LeagueDataSection,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeagueImageAsset {
    pub mime_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeagueGameAsset {
    pub kind: LeagueGameAssetKind,
    pub asset_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub image: LeagueImageAsset,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeagueChampionSummary {
    pub champion_id: i64,
    pub champion_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeagueChampionDetails {
    pub champion_id: i64,
    pub champion_name: String,
    pub title: Option<String>,
    pub square_portrait: Option<LeagueImageAsset>,
    pub abilities: Vec<LeagueChampionAbility>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeagueChampionAbility {
    pub slot: String,
    pub name: String,
    pub description: String,
    pub summary_description: String,
    pub icon: Option<LeagueImageAsset>,
    pub cooldown: Option<String>,
    pub cost: Option<String>,
    pub range: Option<String>,
    pub cooldown_values: Vec<String>,
    pub cost_values: Vec<String>,
    pub range_values: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stats: Vec<AbilityStat>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbilityStat {
    pub label: String,
    pub values: Vec<f64>,
    pub suffix: String,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LeagueGameAssetKind {
    Item,
    Rune,
    Spell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LeagueDataSection {
    Champions,
    Ranked,
    Advisor,
    Matches,
    Participants,
    RecentStats,
    LiveOverlay,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentSummonerProfile {
    pub display_name: String,
    pub summoner_level: i64,
    pub profile_icon_id: Option<i64>,
    pub honor_level: Option<i64>,
    pub top_mastery: Vec<ChampionMasteryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampionMasteryEntry {
    pub champion_id: i64,
    pub champion_name: String,
    pub mastery_level: i64,
    pub mastery_points: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RankedQueueSummary {
    pub queue: RankedQueue,
    pub tier: Option<String>,
    pub division: Option<String>,
    pub league_points: Option<i64>,
    pub wins: i64,
    pub losses: i64,
    pub is_ranked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RankedQueue {
    SoloDuo,
    Flex,
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RankedChampionStatsResponse {
    pub lane: Option<RankedChampionLane>,
    pub sort_by: RankedChampionSort,
    pub records: Vec<RankedChampionStat>,
    pub source: String,
    pub updated_at: String,
    pub generated_at: Option<String>,
    pub imported_at: Option<String>,
    pub patch: Option<String>,
    pub region: Option<String>,
    pub queue: Option<String>,
    pub tier: Option<String>,
    pub is_cached: bool,
    pub data_status: RankedChampionDataStatus,
    pub status_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RankedChampionStat {
    pub champion_id: i64,
    pub champion_name: String,
    pub champion_alias: Option<String>,
    pub lane: RankedChampionLane,
    pub win_rate: f64,
    pub pick_rate: f64,
    pub ban_rate: f64,
    pub overall_score: f64,
    pub games: i64,
    pub wins: i64,
    pub picks: i64,
    pub bans: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RankedChampionDataSnapshot {
    pub source: String,
    pub patch: Option<String>,
    pub region: Option<String>,
    pub queue: Option<String>,
    pub tier: Option<String>,
    pub generated_at: Option<String>,
    pub imported_at: String,
    pub records: Vec<RankedChampionStat>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvisorDataResponse {
    pub lane: Option<RankedChampionLane>,
    pub champion_id: Option<i64>,
    pub records: Vec<AdvisorRecord>,
    pub source: String,
    pub updated_at: String,
    pub generated_at: Option<String>,
    pub imported_at: Option<String>,
    pub patch: Option<String>,
    pub region: Option<String>,
    pub queue: Option<String>,
    pub tier: Option<String>,
    pub is_cached: bool,
    pub data_status: RankedChampionDataStatus,
    pub status_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvisorDataSnapshot {
    pub source: String,
    pub patch: Option<String>,
    pub region: Option<String>,
    pub queue: Option<String>,
    pub tier: Option<String>,
    pub generated_at: Option<String>,
    pub imported_at: String,
    pub records: Vec<AdvisorRecord>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvisorRecord {
    pub champion_id: i64,
    pub champion_name: String,
    pub champion_alias: Option<String>,
    pub lane: RankedChampionLane,
    pub win_rate: f64,
    pub pick_rate: f64,
    pub ban_rate: f64,
    pub overall_score: f64,
    pub games: i64,
    pub runes: AdvisorRunePage,
    pub summoner_spells: Vec<AdvisorNamedRef>,
    pub skill_order: AdvisorSkillOrder,
    pub item_build: AdvisorItemBuild,
    pub strong_against: Vec<AdvisorMatchup>,
    pub weak_against: Vec<AdvisorMatchup>,
    pub power_spikes: Vec<AdvisorPowerSpike>,
    pub lane_advice: String,
    pub teamfight_advice: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvisorNamedRef {
    pub id: Option<i64>,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvisorRunePage {
    pub primary_style: String,
    pub primary_runes: Vec<AdvisorNamedRef>,
    pub secondary_style: String,
    pub secondary_runes: Vec<AdvisorNamedRef>,
    pub stat_shards: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvisorSkillOrder {
    pub max_order: Vec<String>,
    pub early_order: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvisorItemBuild {
    pub starter: Vec<AdvisorNamedRef>,
    pub core: Vec<AdvisorNamedRef>,
    pub boots: Vec<AdvisorNamedRef>,
    pub late: Vec<AdvisorNamedRef>,
    pub situational: Vec<AdvisorNamedRef>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvisorMatchup {
    pub champion_id: i64,
    pub champion_name: String,
    pub note: String,
    pub win_rate_delta: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvisorPowerSpike {
    pub timing: String,
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectAdvisorSnapshot {
    pub players: Vec<ChampSelectAdvisorPlayer>,
    pub cached_at: String,
    pub advisor_source: String,
    pub advisor_patch: Option<String>,
    pub data_status: RankedChampionDataStatus,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectAdvisorPlayer {
    pub summoner_id: i64,
    pub display_name: String,
    pub champion_id: Option<i64>,
    pub champion_name: Option<String>,
    pub team: ChampSelectTeam,
    pub recent_stats: Option<ParticipantRecentStats>,
    pub recent_stats_status: ChampSelectRecentStatsStatus,
    pub tags: Vec<AdvisorPlayerTag>,
    pub advisor: Option<AdvisorRecord>,
    pub matchup_advice: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvisorPlayerTag {
    pub label: String,
    pub tone: AdvisorTagTone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AdvisorTagTone {
    Good,
    Warn,
    Info,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveOverlaySnapshot {
    pub game_time_seconds: Option<f64>,
    pub game_mode: Option<String>,
    pub map_name: Option<String>,
    pub active_player: Option<LiveOverlayActivePlayer>,
    pub players: Vec<LiveOverlayPlayer>,
    pub events: Vec<LiveOverlayEvent>,
    pub gold: LiveOverlayGoldSummary,
    pub refreshed_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveOverlayActivePlayer {
    pub display_name: String,
    pub level: Option<i64>,
    pub current_gold: Option<f64>,
    pub resource_type: Option<String>,
    pub resource_value: Option<f64>,
    pub resource_max: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveOverlayPlayer {
    pub display_name: String,
    pub champion_name: Option<String>,
    pub team: String,
    pub level: Option<i64>,
    pub position: Option<String>,
    pub is_dead: bool,
    pub respawn_timer: Option<f64>,
    pub items: Vec<LiveOverlayItem>,
    pub scores: Option<LiveOverlayScores>,
    pub summoner_spells: Vec<AdvisorNamedRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveOverlayItem {
    pub item_id: i64,
    pub display_name: String,
    pub price: i64,
    pub count: i64,
    pub slot: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveOverlayScores {
    pub kills: i64,
    pub deaths: i64,
    pub assists: i64,
    pub creep_score: i64,
    pub ward_score: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveOverlayEvent {
    pub event_id: i64,
    pub event_name: String,
    pub event_time: f64,
    pub actor: Option<String>,
    pub victim: Option<String>,
    pub assisting_participants: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveOverlayGoldSummary {
    pub ally_item_value: i64,
    pub enemy_item_value: i64,
    pub item_value_diff: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RankedChampionDataStatus {
    Sample,
    Cached,
    Fresh,
    StaleCache,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RankedChampionLane {
    Top,
    Jungle,
    Middle,
    Bottom,
    Support,
}

impl RankedChampionLane {
    pub const ALL: [Self; 5] = [
        Self::Top,
        Self::Jungle,
        Self::Middle,
        Self::Bottom,
        Self::Support,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Top => "Top",
            Self::Jungle => "Jungle",
            Self::Middle => "Middle",
            Self::Bottom => "Bottom",
            Self::Support => "Support",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Jungle => "jungle",
            Self::Middle => "middle",
            Self::Bottom => "bottom",
            Self::Support => "support",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "top" => Some(Self::Top),
            "jungle" => Some(Self::Jungle),
            "middle" => Some(Self::Middle),
            "bottom" => Some(Self::Bottom),
            "support" => Some(Self::Support),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RankedChampionSort {
    Overall,
    WinRate,
    BanRate,
    PickRate,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentMatchSummary {
    pub game_id: i64,
    pub champion_id: Option<i64>,
    pub champion_name: String,
    pub queue_name: Option<String>,
    pub lane: Option<String>,
    pub result: MatchResult,
    pub kills: i64,
    pub deaths: i64,
    pub assists: i64,
    pub kda: Option<f64>,
    pub played_at: Option<String>,
    pub game_duration_seconds: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchResult {
    Win,
    Loss,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentPerformanceSummary {
    pub match_count: usize,
    pub average_kda: Option<f64>,
    pub kda_tag: KdaTag,
    pub recent_champions: Vec<String>,
    pub top_champions: Vec<RecentChampionSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentChampionSummary {
    pub champion_id: Option<i64>,
    pub champion_name: String,
    pub games: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostMatchDetail {
    pub game_id: i64,
    pub queue_name: Option<String>,
    pub played_at: Option<String>,
    pub game_duration_seconds: Option<i64>,
    pub result: MatchResult,
    pub self_participant_id: Option<i64>,
    pub teams: Vec<PostMatchTeam>,
    pub comparison: PostMatchComparison,
    pub warnings: Vec<LeagueDataWarning>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostMatchTeam {
    pub team_id: i64,
    pub result: MatchResult,
    pub participants: Vec<PostMatchParticipant>,
    pub totals: PostMatchTeamTotals,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostMatchParticipant {
    pub participant_id: i64,
    pub team_id: i64,
    pub display_name: String,
    pub champion_id: Option<i64>,
    pub champion_name: String,
    pub role: Option<String>,
    pub lane: Option<String>,
    pub profile_icon_id: Option<i64>,
    pub result: MatchResult,
    pub kills: i64,
    pub deaths: i64,
    pub assists: i64,
    pub kda: Option<f64>,
    pub performance_score: f64,
    pub cs: i64,
    pub gold_earned: i64,
    pub damage_to_champions: i64,
    pub physical_damage_to_champions: i64,
    pub magic_damage_to_champions: i64,
    pub true_damage_to_champions: i64,
    pub damage_to_objectives: i64,
    pub damage_to_turrets: i64,
    pub damage_taken: i64,
    pub vision_score: i64,
    pub wards_placed: i64,
    pub wards_killed: i64,
    pub control_wards_bought: i64,
    pub time_spent_dead_seconds: i64,
    pub largest_killing_spree: i64,
    pub largest_multi_kill: i64,
    pub double_kills: i64,
    pub triple_kills: i64,
    pub quadra_kills: i64,
    pub penta_kills: i64,
    pub first_blood: bool,
    pub first_tower: bool,
    pub items: Vec<i64>,
    pub runes: Vec<i64>,
    pub spells: Vec<i64>,
    pub note_summary: PlayerNoteSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostMatchTeamTotals {
    pub kills: i64,
    pub deaths: i64,
    pub assists: i64,
    pub gold_earned: i64,
    pub damage_to_champions: i64,
    pub vision_score: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostMatchComparison {
    pub highest_kda: Option<ParticipantMetricLeader>,
    pub most_cs: Option<ParticipantMetricLeader>,
    pub most_gold: Option<ParticipantMetricLeader>,
    pub most_damage: Option<ParticipantMetricLeader>,
    pub highest_vision: Option<ParticipantMetricLeader>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantMetricLeader {
    pub participant_id: i64,
    pub display_name: String,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantPublicProfile {
    pub game_id: i64,
    pub participant_id: i64,
    pub display_name: String,
    pub profile_icon_id: Option<i64>,
    pub recent_stats: Option<ParticipantRecentStats>,
    pub note: Option<PlayerNoteView>,
    pub warnings: Vec<LeagueDataWarning>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantRecentStats {
    pub match_count: usize,
    pub average_kda: Option<f64>,
    pub recent_champions: Vec<String>,
    pub recent_matches: Vec<RecentMatchSummary>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum KdaTag {
    High,
    Standard,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectPlayer {
    pub summoner_id: i64,
    #[serde(default, skip_serializing)]
    pub puuid: String,
    pub display_name: String,
    pub champion_id: Option<i64>,
    pub champion_name: Option<String>,
    pub team: ChampSelectTeam,
    pub ranked_queues: Vec<RankedQueueSummary>,
    pub summoner_level: Option<i64>,
    pub mastery_level: Option<i64>,
    pub recent_stats: Option<ParticipantRecentStats>,
    pub recent_stats_status: ChampSelectRecentStatsStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ChampSelectRecentStatsStatus {
    NotRequested,
    MissingIdentity,
    Loaded,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ChampSelectTeam {
    Ally,
    Enemy,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectSnapshot {
    pub players: Vec<ChampSelectPlayer>,
    pub cached_at: String,
}

// ── Rune system ──────────────────────────────────────────────────────────────

/// A single League of Legends rune page ready to be written to the LCU.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunePage {
    pub primary_style_id: i64,
    pub sub_style_id: i64,
    pub selected_perk_ids: Vec<i64>,
}

/// One rune page recommendation from the Tencent QQ champion-detail API,
/// sorted by pick frequency in descending order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuneRecommendation {
    /// Lane/position: "top", "jungle", "middle", "bottom", "support"
    pub position: String,
    pub pick_count: i64,
    pub page: RunePage,
}

/// A user-saved rune page preference for a specific champion (stored in SQLite).
/// Takes precedence over API recommendations on lock-in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampionRuneConfig {
    pub champion_id: i64,
    pub page: RunePage,
    pub saved_at: String,
}

/// Result of loading rune data for the rune page tab.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunePageSnapshot {
    pub champion_id: i64,
    pub champion_name: String,
    pub recommendations: Vec<RuneRecommendation>,
    pub saved_config: Option<ChampionRuneConfig>,
    pub auto_applied: bool,
}
