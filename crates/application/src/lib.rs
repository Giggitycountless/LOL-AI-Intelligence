use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    error::Error,
    fmt, thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use domain::{
    ActivityEntry, ActivityKind, AdvisorDataResponse, AdvisorDataSnapshot, AdvisorItemBuild,
    AdvisorMatchup, AdvisorNamedRef, AdvisorPlayerTag, AdvisorPowerSpike, AdvisorRecord,
    AdvisorRunePage, AdvisorSkillOrder, AdvisorTagTone, AppLanguagePreference, AppSettings,
    AppSnapshot, AppThemePreference, ChampSelectAdvisorPlayer, ChampSelectAdvisorSnapshot,
    ChampSelectRecentStatsStatus, ChampionRuneConfig, ClearActivityResult, ClearPlayerNoteResult,
    DatabaseStatus,
    HealthReport, ImportLocalDataResult, KdaTag, LeagueChampionDetails, LeagueChampionSummary,
    LeagueClientStatus, LeagueDataSection, LeagueDataWarning, LeagueGameAsset, LeagueGameAssetKind,
    ChampionRecordSummary, LeagueImageAsset, LeagueSelfData, LeagueSelfSnapshot, LiveOverlaySnapshot,
    LocalActivityEntry,
    LocalDataExport, MatchResult, NewActivityEntry, ParticipantMetricLeader,
    ParticipantPublicProfile, ParticipantRecentStats, PlayerNoteSummary, PlayerNoteView,
    PostMatchComparison, PostMatchDetail, PostMatchParticipant, PostMatchTeam, PostMatchTeamTotals,
    RankedChampionDataSnapshot, RankedChampionDataStatus, RankedChampionLane, RankedChampionSort,
    RankedChampionStat, RankedChampionStatsResponse, RankedQueueSummary, RecentChampionSummary, RecentMatchSummary,
    RecentPerformanceSummary, RunePage, ServiceStatus, SettingsValues, StartupPage,
};

mod constants;
mod ranked_seeds;
use constants::*;

fn log_auto_accept_event(message: &str) {
    eprintln!("[auto-accept] {message}");
}

fn log_auto_accept_attempt(attempt: usize, message: &str) {
    eprintln!("[auto-accept] attempt {attempt}: {message}");
}

use ranked_seeds::{RANKED_CHAMPION_SEEDS, RankedChampionSeed};

pub trait AppStore {
    fn schema_version(&self) -> Result<i64, String>;
    fn get_settings(&self) -> Result<AppSettings, String>;
    fn save_settings(&self, settings: SettingsValues) -> Result<AppSettings, String>;
    fn list_activity_entries(
        &self,
        limit: i64,
        kind: Option<ActivityKind>,
    ) -> Result<Vec<ActivityEntry>, String>;
    fn list_all_activity_entries(&self) -> Result<Vec<ActivityEntry>, String>;
    fn create_activity_entry(&self, entry: NewActivityEntry) -> Result<ActivityEntry, String>;
    fn import_local_data(
        &self,
        settings: SettingsValues,
        activity_entries: Vec<LocalActivityEntry>,
    ) -> Result<ImportLocalDataResult, String>;
    fn clear_activity_entries(&self) -> Result<i64, String>;
    fn get_player_note(&self, player_puuid: &str) -> Result<Option<StoredPlayerNote>, String>;
    fn save_player_note(&self, note: StoredPlayerNoteInput) -> Result<StoredPlayerNote, String>;
    fn clear_player_note(&self, player_puuid: &str) -> Result<bool, String>;
    fn latest_ranked_champion_snapshot(&self)
    -> Result<Option<RankedChampionDataSnapshot>, String>;
    fn replace_ranked_champion_snapshot(
        &self,
        snapshot: RankedChampionDataSnapshot,
    ) -> Result<RankedChampionDataSnapshot, String>;
    fn latest_advisor_snapshot(&self) -> Result<Option<AdvisorDataSnapshot>, String>;
    fn replace_advisor_snapshot(
        &self,
        snapshot: AdvisorDataSnapshot,
    ) -> Result<AdvisorDataSnapshot, String>;
    fn get_champion_rune_config(
        &self,
        champion_id: i64,
    ) -> Result<Option<ChampionRuneConfig>, String>;
    fn save_champion_rune_config(
        &self,
        champion_id: i64,
        page: RunePage,
    ) -> Result<ChampionRuneConfig, String>;
    fn delete_champion_rune_config(&self, champion_id: i64) -> Result<bool, String>;
    fn get_ai_analysis(&self, scope: &str) -> Result<Option<domain::AiAnalysisCache>, String>;
    fn save_ai_analysis(&self, scope: &str, result_text: &str, game_count: i64) -> Result<(), String>;
    fn list_chat_presets(&self) -> Result<Vec<domain::ChatPreset>, String>;
    fn save_chat_preset(
        &self,
        slot: i64,
        label: &str,
        message: &str,
    ) -> Result<domain::ChatPreset, String>;
    fn delete_chat_preset(&self, slot: i64) -> Result<bool, String>;
}

pub trait LeagueClientReader {
    fn status(&self) -> Result<LeagueClientStatus, LeagueClientReadError>;
    fn self_data(&self, match_limit: i64) -> Result<LeagueSelfData, LeagueClientReadError>;
    fn profile_icon(&self, profile_icon_id: i64)
    -> Result<LeagueImageAsset, LeagueClientReadError>;
    fn champion_icon(&self, champion_id: i64) -> Result<LeagueImageAsset, LeagueClientReadError>;
    fn game_asset(
        &self,
        kind: LeagueGameAssetKind,
        asset_id: i64,
    ) -> Result<LeagueGameAsset, LeagueClientReadError>;
    fn completed_match(&self, game_id: i64) -> Result<LeagueCompletedMatch, LeagueClientReadError>;
    fn participant_recent_stats(
        &self,
        player_puuid: &str,
        limit: i64,
    ) -> Result<ParticipantRecentStats, LeagueClientReadError>;
    fn participant_recent_stats_batch(
        &self,
        player_puuids: &[String],
        limit: i64,
    ) -> HashMap<String, Result<ParticipantRecentStats, LeagueClientReadError>> {
        player_puuids
            .iter()
            .map(|player_puuid| {
                (
                    player_puuid.clone(),
                    self.participant_recent_stats(player_puuid, limit),
                )
            })
            .collect()
    }
    fn champ_select_session(&self) -> Result<ChampSelectSessionData, LeagueClientReadError>;
    fn summoners_by_ids(&self, ids: &[i64]) -> Vec<SummonerBatchEntry>;
    fn summoners_by_names(&self, names: &[String]) -> Vec<SummonerBatchEntry>;
    fn participant_ranked_stats_batch(
        &self,
        puuids: &[String],
    ) -> HashMap<String, Vec<RankedQueueSummary>> {
        puuids
            .iter()
            .map(|puuid| (puuid.clone(), Vec::new()))
            .collect()
    }
    /// Per-player champion mastery level. `entries` are `(puuid, champion_id)`;
    /// the result is keyed by puuid.
    fn champion_mastery_batch(
        &self,
        _entries: &[(String, i64)],
    ) -> HashMap<String, Option<i64>> {
        HashMap::new()
    }
    fn champion_catalog(&self) -> Result<Vec<LeagueChampionSummary>, LeagueClientReadError>;
    fn champion_details(
        &self,
        champion_id: i64,
    ) -> Result<LeagueChampionDetails, LeagueClientReadError>;
    fn gameflow_phase(&self) -> Result<String, LeagueClientReadError>;
    fn live_overlay(&self) -> Result<LiveOverlaySnapshot, LeagueClientReadError>;
    fn accept_ready_check(&self) -> Result<(), LeagueClientReadError>;
    fn apply_rune_page(
        &self,
        page: &domain::RunePage,
        champion_name: &str,
    ) -> Result<(), LeagueClientReadError>;
    fn apply_champ_select_preferences(
        &self,
        pick_champion_id: Option<i64>,
        ban_champion_id: Option<i64>,
    ) -> Result<(), LeagueClientReadError>;
}

pub trait RankedChampionDataProvider {
    fn fetch_ranked_champion_snapshot(
        &self,
        input: RankedChampionRefreshInput,
    ) -> Result<RankedChampionDataSnapshot, RankedChampionDataError>;
}

pub trait AdvisorDataProvider {
    fn fetch_advisor_snapshot(
        &self,
        input: AdvisorDataRefreshInput,
    ) -> Result<AdvisorDataSnapshot, RankedChampionDataError>;
}

pub trait RuneRecommendationProvider {
    fn fetch_rune_recommendations(
        &self,
        champion_id: i64,
    ) -> Vec<domain::RuneRecommendation>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeagueClientReadError {
    ClientUnavailable(String),
    ClientAccess(String),
    Integration(String),
}

impl fmt::Display for LeagueClientReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClientUnavailable(message)
            | Self::ClientAccess(message)
            | Self::Integration(message) => formatter.write_str(message),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChampSelectSessionData {
    pub ally_ids: Vec<i64>,
    pub enemy_ids: Vec<i64>,
    pub champion_selections: std::collections::HashMap<i64, i64>,
    pub ally_names: Vec<String>,
    pub enemy_names: Vec<String>,
    pub champion_selections_by_name: std::collections::HashMap<String, i64>,
    pub source: ChampSelectSessionSource,
    pub players: Vec<ChampSelectSessionPlayer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChampSelectSessionSource {
    ChampSelect,
    GameflowSession,
    LiveClient,
}

impl ChampSelectSessionSource {
    fn as_log_label(self) -> &'static str {
        match self {
            Self::ChampSelect => "champ-select",
            Self::GameflowSession => "gameflow-session",
            Self::LiveClient => "live-client",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChampSelectSessionPlayer {
    pub summoner_id: Option<i64>,
    pub puuid: Option<String>,
    pub display_name: String,
    pub champion_id: Option<i64>,
    pub team: domain::ChampSelectTeam,
}

#[derive(Debug, Clone)]
pub struct SummonerBatchEntry {
    pub summoner_id: i64,
    pub puuid: String,
    pub display_name: String,
    pub summoner_level: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettingsInput {
    pub startup_page: String,
    pub language: String,
    pub theme: String,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityListInput {
    pub limit: Option<i64>,
    pub kind: Option<ActivityKind>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityNoteInput {
    pub title: String,
    pub body: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityEntries {
    pub records: Vec<ActivityEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeagueSelfSnapshotInput {
    pub match_limit: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankedChampionStatsInput {
    pub lane: Option<RankedChampionLane>,
    pub sort_by: Option<RankedChampionSort>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum RankedChampionDataSource {
    #[default]
    GitHubJson,
    Tencent,
    KoreaKr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChampionHint {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankedChampionRefreshInput {
    pub url: Option<String>,
    pub source: RankedChampionDataSource,
    pub champion_hints: Vec<ChampionHint>,
    pub tier: u32,
    pub lane: Option<RankedChampionLane>,
    pub patch_version: Option<String>,
}

impl Default for RankedChampionRefreshInput {
    fn default() -> Self {
        Self {
            url: None,
            source: RankedChampionDataSource::GitHubJson,
            champion_hints: vec![],
            tier: 200,
            lane: None,
            patch_version: None,
        }
    }
}

pub fn seed_champion_hints() -> Vec<ChampionHint> {
    let mut seen = std::collections::HashSet::new();
    RANKED_CHAMPION_SEEDS
        .iter()
        .filter_map(|seed| {
            if seen.insert(seed.champion_id) {
                Some(ChampionHint {
                    id: seed.champion_id,
                    name: seed.champion_name.to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvisorDataInput {
    pub lane: Option<RankedChampionLane>,
    pub champion_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvisorDataRefreshInput {
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RankedChampionDataError {
    Unavailable(String),
    InvalidData(String),
}

impl fmt::Display for RankedChampionDataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(message) | Self::InvalidData(message) => formatter.write_str(message),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeagueProfileIconInput {
    pub profile_icon_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeagueChampionIconInput {
    pub champion_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeagueChampionDetailsInput {
    pub champion_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeagueGameAssetInput {
    pub kind: LeagueGameAssetKind,
    pub asset_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostMatchDetailInput {
    pub game_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticipantPublicProfileInput {
    pub game_id: i64,
    pub participant_id: i64,
    pub recent_limit: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavePlayerNoteInput {
    pub game_id: i64,
    pub participant_id: i64,
    pub note: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClearPlayerNoteInput {
    pub game_id: i64,
    pub participant_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredPlayerNote {
    pub player_puuid: String,
    pub last_display_name: String,
    pub note: Option<String>,
    pub tags: Vec<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredPlayerNoteInput {
    pub player_puuid: String,
    pub last_display_name: String,
    pub note: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LeagueCompletedMatch {
    pub game_id: i64,
    pub queue_name: Option<String>,
    pub played_at: Option<String>,
    pub game_duration_seconds: Option<i64>,
    pub result: MatchResult,
    pub self_participant_id: Option<i64>,
    pub participants: Vec<LeagueCompletedParticipant>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LeagueCompletedParticipant {
    pub participant_id: i64,
    pub team_id: i64,
    pub display_name: String,
    pub player_puuid: Option<String>,
    pub profile_icon_id: Option<i64>,
    pub champion_id: Option<i64>,
    pub champion_name: String,
    pub role: Option<String>,
    pub lane: Option<String>,
    pub result: MatchResult,
    pub kills: i64,
    pub deaths: i64,
    pub assists: i64,
    pub kda: Option<f64>,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationError {
    Validation(String),
    Storage(String),
    ClientUnavailable(String),
    ClientAccess(String),
    Integration(String),
}

impl ApplicationError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "validation",
            Self::Storage(_) => "storage",
            Self::ClientUnavailable(_) => "clientUnavailable",
            Self::ClientAccess(_) => "clientAccess",
            Self::Integration(_) => "integration",
        }
    }
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(message)
            | Self::Storage(message)
            | Self::ClientUnavailable(message)
            | Self::ClientAccess(message)
            | Self::Integration(message) => formatter.write_str(message),
        }
    }
}

impl Error for ApplicationError {}

fn storage_failure(operation: &'static str, error: String) -> ApplicationError {
    ApplicationError::Storage(format!("{operation} failed: {error}"))
}

impl From<LeagueClientReadError> for ApplicationError {
    fn from(error: LeagueClientReadError) -> Self {
        match error {
            LeagueClientReadError::ClientUnavailable(message) => Self::ClientUnavailable(message),
            LeagueClientReadError::ClientAccess(message) => Self::ClientAccess(message),
            LeagueClientReadError::Integration(message) => Self::Integration(message),
        }
    }
}

pub fn health_report(database_status: DatabaseStatus, schema_version: Option<i64>) -> HealthReport {
    let status = match database_status {
        DatabaseStatus::Ok => ServiceStatus::Ok,
        DatabaseStatus::Unavailable => ServiceStatus::Degraded,
    };

    HealthReport {
        status,
        database_status,
        schema_version,
    }
}

pub fn settings_defaults() -> SettingsValues {
    SettingsValues {
        startup_page: StartupPage::Dashboard,
        language: AppLanguagePreference::System,
        theme: AppThemePreference::Light,
        compact_mode: false,
        activity_limit: DEFAULT_ACTIVITY_LIMIT,
        auto_accept_enabled: true,
        auto_pick_enabled: false,
        auto_pick_champion_id: None,
        auto_pick_delay_seconds: 0.0,
        auto_ban_enabled: false,
        auto_ban_champion_id: None,
        auto_ban_delay_seconds: 0.0,
        ai_base_url: None,
        ai_api_key: None,
        ai_model: None,
    }
}

pub fn app_snapshot(store: &impl AppStore) -> Result<AppSnapshot, ApplicationError> {
    let schema_version = store
        .schema_version()
        .map_err(|error| storage_failure("read schema version", error))?;
    let settings = get_settings(store)?;
    let recent_activity = list_activity_entries(
        store,
        ActivityListInput {
            limit: Some(settings.activity_limit),
            kind: None,
        },
    )?
    .records;

    Ok(AppSnapshot {
        health: health_report(DatabaseStatus::Ok, Some(schema_version)),
        settings,
        settings_defaults: settings_defaults(),
        recent_activity,
    })
}

pub fn get_settings(store: &impl AppStore) -> Result<AppSettings, ApplicationError> {
    store
        .get_settings()
        .map_err(|error| storage_failure("load settings", error))
}

pub fn save_settings(
    store: &impl AppStore,
    input: SettingsInput,
) -> Result<AppSettings, ApplicationError> {
    let next_settings = validate_settings(input)?;
    let current_settings = store
        .get_settings()
        .map_err(|error| storage_failure("load current settings", error))?;

    if current_settings.values() == next_settings {
        return Ok(current_settings);
    }

    let saved_settings = store
        .save_settings(next_settings)
        .map_err(|error| storage_failure("save settings", error))?;

    store
        .create_activity_entry(NewActivityEntry {
            kind: ActivityKind::Settings,
            title: "Settings updated".to_string(),
            body: Some("Application preferences changed".to_string()),
        })
        .map_err(|error| storage_failure("create settings activity entry", error))?;

    Ok(saved_settings)
}

pub fn list_activity_entries(
    store: &impl AppStore,
    input: ActivityListInput,
) -> Result<ActivityEntries, ApplicationError> {
    let limit = normalize_activity_limit(input.limit.unwrap_or(DEFAULT_ACTIVITY_LIMIT))?;
    let records = store
        .list_activity_entries(limit, input.kind)
        .map_err(|error| storage_failure("list activity entries", error))?;

    Ok(ActivityEntries { records })
}

pub fn create_activity_note(
    store: &impl AppStore,
    input: ActivityNoteInput,
) -> Result<ActivityEntry, ApplicationError> {
    let entry = validate_activity_note(input)?;

    store
        .create_activity_entry(entry)
        .map_err(|error| storage_failure("create activity note", error))
}

pub fn export_local_data(store: &impl AppStore) -> Result<LocalDataExport, ApplicationError> {
    let settings = store
        .get_settings()
        .map_err(|error| storage_failure("load settings for export", error))?
        .values();
    let activity_entries = store
        .list_all_activity_entries()
        .map_err(|error| storage_failure("list activity entries for export", error))?
        .into_iter()
        .map(|entry| LocalActivityEntry {
            kind: entry.kind,
            title: entry.title,
            body: entry.body,
            created_at: entry.created_at,
        })
        .collect();

    Ok(LocalDataExport {
        format_version: LOCAL_DATA_FORMAT_VERSION,
        settings,
        activity_entries,
    })
}

pub fn import_local_data(
    store: &impl AppStore,
    json: &str,
) -> Result<ImportLocalDataResult, ApplicationError> {
    let data: LocalDataExport = serde_json::from_str(json).map_err(|error| {
        ApplicationError::Validation(format!("Import JSON is invalid: {error}"))
    })?;

    if data.format_version != LOCAL_DATA_FORMAT_VERSION {
        return Err(ApplicationError::Validation(format!(
            "Unsupported import format version: {}",
            data.format_version
        )));
    }

    validate_settings_values(&data.settings)?;
    for entry in &data.activity_entries {
        validate_local_activity_entry(entry)?;
    }

    store
        .import_local_data(data.settings, data.activity_entries)
        .map_err(|error| storage_failure("import local data", error))
}

pub fn clear_activity_entries(
    store: &impl AppStore,
    confirm: bool,
) -> Result<ClearActivityResult, ApplicationError> {
    if !confirm {
        return Err(ApplicationError::Validation(
            "Activity clear confirmation is required".to_string(),
        ));
    }

    let deleted_count = store
        .clear_activity_entries()
        .map_err(|error| storage_failure("clear activity entries", error))?;

    Ok(ClearActivityResult { deleted_count })
}

// ── Rune system ──────────────────────────────────────────────────────────────

pub fn get_champion_rune_recommendations(
    provider: &impl RuneRecommendationProvider,
    champion_id: i64,
) -> Vec<domain::RuneRecommendation> {
    provider.fetch_rune_recommendations(champion_id)
}

pub fn auto_apply_rune_on_lock_in(
    store: &impl AppStore,
    reader: &impl LeagueClientReader,
    provider: &impl RuneRecommendationProvider,
    champion_id: i64,
    champion_name: &str,
) -> Result<bool, ApplicationError> {
    let page = if let Some(saved) = store
        .get_champion_rune_config(champion_id)
        .map_err(ApplicationError::Storage)?
    {
        saved.page
    } else {
        let recs = provider.fetch_rune_recommendations(champion_id);
        match recs.into_iter().next() {
            Some(rec) => rec.page,
            None => return Ok(false),
        }
    };

    reader
        .apply_rune_page(&page, champion_name)
        .map_err(ApplicationError::from)?;

    Ok(true)
}

pub fn apply_specific_rune_page(
    reader: &impl LeagueClientReader,
    page: RunePage,
    champion_name: &str,
) -> Result<(), ApplicationError> {
    reader
        .apply_rune_page(&page, champion_name)
        .map_err(ApplicationError::from)
}

pub fn get_stored_rune_config(
    store: &impl AppStore,
    champion_id: i64,
) -> Result<Option<ChampionRuneConfig>, ApplicationError> {
    store
        .get_champion_rune_config(champion_id)
        .map_err(ApplicationError::Storage)
}

pub fn save_rune_config(
    store: &impl AppStore,
    champion_id: i64,
    page: RunePage,
) -> Result<ChampionRuneConfig, ApplicationError> {
    if champion_id <= 0 {
        return Err(ApplicationError::Validation(
            "Champion id must be greater than 0".to_string(),
        ));
    }
    store
        .save_champion_rune_config(champion_id, page)
        .map_err(ApplicationError::Storage)
}

pub fn delete_rune_config(
    store: &impl AppStore,
    champion_id: i64,
) -> Result<bool, ApplicationError> {
    if champion_id <= 0 {
        return Err(ApplicationError::Validation(
            "Champion id must be greater than 0".to_string(),
        ));
    }
    store
        .delete_champion_rune_config(champion_id)
        .map_err(ApplicationError::Storage)
}

pub fn get_league_client_status(
    reader: &impl LeagueClientReader,
) -> Result<LeagueClientStatus, ApplicationError> {
    reader.status().map_err(ApplicationError::from)
}

pub fn get_league_self_snapshot(
    reader: &impl LeagueClientReader,
    input: LeagueSelfSnapshotInput,
) -> Result<LeagueSelfSnapshot, ApplicationError> {
    let match_limit = normalize_match_limit(input.match_limit.unwrap_or(DEFAULT_MATCH_LIMIT))?;
    let data = reader
        .self_data(match_limit)
        .map_err(ApplicationError::from)?;

    Ok(LeagueSelfSnapshot {
        recent_performance: summarize_recent_performance(&data.recent_matches),
        champion_records: summarize_champion_records(&data.recent_matches),
        status: data.status,
        summoner: data.summoner,
        ranked_queues: data.ranked_queues,
        recent_matches: data.recent_matches,
        data_warnings: data.data_warnings,
        refreshed_at: unix_timestamp_seconds(),
    })
}

pub fn get_ranked_champion_stats(input: RankedChampionStatsInput) -> RankedChampionStatsResponse {
    let sort_by = input.sort_by.unwrap_or(RankedChampionSort::Overall);
    let mut records: Vec<RankedChampionStat> = RANKED_CHAMPION_SEEDS
        .iter()
        .filter(|seed| input.lane.is_none_or(|lane| seed.lane == lane))
        .map(ranked_champion_stat)
        .collect();

    records.sort_by(|left, right| compare_ranked_champions(left, right, sort_by));

    RankedChampionStatsResponse {
        lane: input.lane,
        sort_by,
        records,
        source: "Local ranked data sample".to_string(),
        updated_at: "2026-04-24".to_string(),
        generated_at: None,
        imported_at: None,
        patch: None,
        region: None,
        queue: Some("RANKED_SOLO_5x5".to_string()),
        tier: Some("sample".to_string()),
        is_cached: false,
        data_status: RankedChampionDataStatus::Sample,
        status_message: Some(
            "Sample data is shown until ranked champion data is refreshed".to_string(),
        ),
    }
}

pub fn get_ranked_champion_stats_from_store(
    store: &impl AppStore,
    input: RankedChampionStatsInput,
) -> Result<RankedChampionStatsResponse, ApplicationError> {
    match store
        .latest_ranked_champion_snapshot()
        .map_err(ApplicationError::Storage)?
    {
        Some(snapshot) => Ok(ranked_response_from_snapshot(
            snapshot,
            input,
            true,
            RankedChampionDataStatus::Cached,
            None,
        )),
        None => Ok(get_ranked_champion_stats(input)),
    }
}

pub fn refresh_ranked_champion_stats(
    store: &impl AppStore,
    provider: &impl RankedChampionDataProvider,
    input: RankedChampionRefreshInput,
    stats_input: RankedChampionStatsInput,
) -> Result<RankedChampionStatsResponse, ApplicationError> {
    let mut snapshot = match provider.fetch_ranked_champion_snapshot(input) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            let error_message = match &error {
                RankedChampionDataError::Unavailable(msg)
                | RankedChampionDataError::InvalidData(msg) => msg.clone(),
            };
            eprintln!("[ranked] refresh failed: {error_message}");

            let cached_snapshot = store
                .latest_ranked_champion_snapshot()
                .map_err(ApplicationError::Storage)?;

            return cached_snapshot.map_or_else(
                || Err(match error {
                    RankedChampionDataError::Unavailable(message)
                    | RankedChampionDataError::InvalidData(message) => {
                        ApplicationError::Integration(message)
                    }
                }),
                |snapshot| {
                    Ok(ranked_response_from_snapshot(
                        snapshot,
                        stats_input,
                        true,
                        RankedChampionDataStatus::StaleCache,
                        Some(format!(
                            "Remote ranked champion data could not be refreshed: {error_message}"
                        )),
                    ))
                },
            );
        }
    };
    snapshot.imported_at = unix_timestamp_seconds();

    let saved = store
        .replace_ranked_champion_snapshot(snapshot)
        .map_err(ApplicationError::Storage)?;

    Ok(ranked_response_from_snapshot(
        saved,
        stats_input,
        true,
        RankedChampionDataStatus::Fresh,
        Some("Ranked champion data refreshed".to_string()),
    ))
}

pub fn get_advisor_data_from_store(
    store: &impl AppStore,
    input: AdvisorDataInput,
) -> Result<AdvisorDataResponse, ApplicationError> {
    let snapshot = store
        .latest_advisor_snapshot()
        .map_err(|error| storage_failure("load advisor data", error))?
        .unwrap_or_else(sample_advisor_snapshot);

    Ok(advisor_response_from_snapshot(
        snapshot,
        input,
        true,
        RankedChampionDataStatus::Cached,
        None,
    ))
}

pub fn refresh_advisor_data(
    store: &impl AppStore,
    provider: &impl AdvisorDataProvider,
    input: AdvisorDataRefreshInput,
    data_input: AdvisorDataInput,
) -> Result<AdvisorDataResponse, ApplicationError> {
    let mut snapshot = match provider.fetch_advisor_snapshot(input) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            let cached_snapshot = store
                .latest_advisor_snapshot()
                .map_err(|error| storage_failure("load cached advisor data", error))?;

            return cached_snapshot.map_or_else(
                || {
                    Err(match error {
                        RankedChampionDataError::Unavailable(message)
                        | RankedChampionDataError::InvalidData(message) => {
                            ApplicationError::Integration(message)
                        }
                    })
                },
                |snapshot| {
                    Ok(advisor_response_from_snapshot(
                        snapshot,
                        data_input,
                        true,
                        RankedChampionDataStatus::StaleCache,
                        Some(
                            "Remote advisor data could not be refreshed; showing cached data"
                                .to_string(),
                        ),
                    ))
                },
            );
        }
    };
    snapshot.imported_at = unix_timestamp_seconds();

    let saved = store
        .replace_advisor_snapshot(snapshot)
        .map_err(|error| storage_failure("save advisor data", error))?;

    Ok(advisor_response_from_snapshot(
        saved,
        data_input,
        true,
        RankedChampionDataStatus::Fresh,
        Some("Advisor data refreshed".to_string()),
    ))
}

pub fn get_champ_select_advisor_snapshot(
    store: &impl AppStore,
    reader: &(impl LeagueClientReader + Sync),
    recent_limit: i64,
) -> Result<ChampSelectAdvisorSnapshot, ApplicationError> {
    let snapshot = get_champ_select_snapshot(reader, recent_limit)?;
    let advisor_snapshot = store
        .latest_advisor_snapshot()
        .map_err(|error| storage_failure("load advisor data", error))?
        .unwrap_or_else(sample_advisor_snapshot);
    // Each stored advisor snapshot is lane-specific, so champion_id collisions
    // are unlikely. Using champion_id alone as the key is intentional since
    // ChampSelectSessionPlayer does not carry lane information.
    let advisors_by_champion: HashMap<i64, AdvisorRecord> = advisor_snapshot
        .records
        .iter()
        .map(|record| (record.champion_id, record.clone()))
        .collect();
    let mut advisor_players: Vec<ChampSelectAdvisorPlayer> = snapshot
        .players
        .into_iter()
        .map(|player| {
            let advisor = player
                .champion_id
                .and_then(|champion_id| advisors_by_champion.get(&champion_id).cloned());
            let tags = player_advisor_tags(&player.recent_stats, advisor.as_ref());

            ChampSelectAdvisorPlayer {
                summoner_id: player.summoner_id,
                display_name: player.display_name,
                champion_id: player.champion_id,
                champion_name: player.champion_name,
                team: player.team,
                recent_stats: player.recent_stats,
                recent_stats_status: player.recent_stats_status,
                tags,
                advisor,
                matchup_advice: None,
            }
        })
        .collect();

    let matchups: Vec<(usize, String)> = advisor_players
        .iter()
        .enumerate()
        .filter_map(|(index, player)| {
            let advisor = player.advisor.as_ref()?;
            let opponent = advisor_players.iter().find(|candidate| {
                candidate.team != player.team
                    && candidate
                        .advisor
                        .as_ref()
                        .is_some_and(|candidate_advisor| candidate_advisor.lane == advisor.lane)
            })?;
            matchup_advice(advisor, opponent).map(|advice| (index, advice))
        })
        .collect();

    for (index, advice) in matchups {
        if let Some(player) = advisor_players.get_mut(index) {
            player.matchup_advice = Some(advice);
        }
    }

    Ok(ChampSelectAdvisorSnapshot {
        players: advisor_players,
        cached_at: unix_timestamp_seconds(),
        advisor_source: advisor_snapshot.source,
        advisor_patch: advisor_snapshot.patch,
        data_status: RankedChampionDataStatus::Cached,
    })
}

pub fn get_live_overlay_snapshot(
    reader: &impl LeagueClientReader,
) -> Result<LiveOverlaySnapshot, ApplicationError> {
    reader.live_overlay().map_err(ApplicationError::from)
}

pub fn get_league_profile_icon(
    reader: &impl LeagueClientReader,
    input: LeagueProfileIconInput,
) -> Result<LeagueImageAsset, ApplicationError> {
    let profile_icon_id = normalize_league_asset_id(input.profile_icon_id, "Profile icon id")?;

    reader
        .profile_icon(profile_icon_id)
        .map_err(ApplicationError::from)
}

pub fn get_league_champion_icon(
    reader: &impl LeagueClientReader,
    input: LeagueChampionIconInput,
) -> Result<LeagueImageAsset, ApplicationError> {
    let champion_id = normalize_league_asset_id(input.champion_id, "Champion id")?;

    reader
        .champion_icon(champion_id)
        .map_err(ApplicationError::from)
}

pub fn get_league_champion_details(
    reader: &impl LeagueClientReader,
    input: LeagueChampionDetailsInput,
) -> Result<LeagueChampionDetails, ApplicationError> {
    let champion_id = normalize_league_asset_id(input.champion_id, "Champion id")?;

    reader
        .champion_details(champion_id)
        .map_err(ApplicationError::from)
}

pub fn get_league_game_asset(
    reader: &impl LeagueClientReader,
    input: LeagueGameAssetInput,
) -> Result<LeagueGameAsset, ApplicationError> {
    let asset_id = normalize_league_asset_id(input.asset_id, "League game asset id")?;

    reader
        .game_asset(input.kind, asset_id)
        .map_err(ApplicationError::from)
}

pub fn get_post_match_detail(
    store: &impl AppStore,
    reader: &impl LeagueClientReader,
    input: PostMatchDetailInput,
) -> Result<PostMatchDetail, ApplicationError> {
    validate_game_id(input.game_id)?;
    let completed_match = reader
        .completed_match(input.game_id)
        .map_err(ApplicationError::from)?;

    post_match_detail_from_completed_match(store, completed_match)
}

pub fn get_post_match_participant_profile(
    store: &impl AppStore,
    reader: &impl LeagueClientReader,
    input: ParticipantPublicProfileInput,
) -> Result<ParticipantPublicProfile, ApplicationError> {
    validate_game_and_participant_ids(input.game_id, input.participant_id)?;
    let recent_limit =
        normalize_match_limit(input.recent_limit.unwrap_or(DEFAULT_PUBLIC_RECENT_LIMIT))?;
    let completed_match = reader
        .completed_match(input.game_id)
        .map_err(ApplicationError::from)?;
    let participant = completed_match
        .participants
        .iter()
        .find(|participant| participant.participant_id == input.participant_id)
        .ok_or_else(|| {
            ApplicationError::Validation(
                "Participant was not found in the completed match".to_string(),
            )
        })?;
    let note = match participant.player_puuid.as_deref() {
        Some(player_puuid) => store
            .get_player_note(player_puuid)
            .map_err(ApplicationError::Storage)?,
        None => None,
    };
    let mut warnings = Vec::new();
    let recent_stats = match participant.player_puuid.as_deref() {
        Some(player_puuid) => match reader.participant_recent_stats(player_puuid, recent_limit) {
            Ok(stats) => Some(stats),
            Err(_) => {
                warnings.push(LeagueDataWarning {
                    section: LeagueDataSection::RecentStats,
                    message: "Participant recent stats are unavailable from the local client"
                        .to_string(),
                });
                None
            }
        },
        None => {
            warnings.push(LeagueDataWarning {
                section: LeagueDataSection::Participants,
                message: "Participant public profile identity is unavailable".to_string(),
            });
            None
        }
    };

    Ok(ParticipantPublicProfile {
        game_id: input.game_id,
        participant_id: input.participant_id,
        display_name: participant.display_name.clone(),
        profile_icon_id: participant.profile_icon_id,
        recent_stats,
        note: note.map(|note| player_note_view(input.game_id, input.participant_id, Some(note))),
        warnings,
    })
}

pub fn save_player_note(
    store: &impl AppStore,
    reader: &impl LeagueClientReader,
    input: SavePlayerNoteInput,
) -> Result<PlayerNoteView, ApplicationError> {
    let (player_puuid, display_name) =
        resolve_post_match_participant_identity(reader, input.game_id, input.participant_id)?;

    save_player_note_for_resolved_player(store, input, player_puuid, display_name)
}

pub fn clear_player_note(
    store: &impl AppStore,
    reader: &impl LeagueClientReader,
    input: ClearPlayerNoteInput,
) -> Result<ClearPlayerNoteResult, ApplicationError> {
    let (player_puuid, _) =
        resolve_post_match_participant_identity(reader, input.game_id, input.participant_id)?;

    clear_player_note_for_resolved_player(store, input, player_puuid.as_str())
}

pub fn save_player_note_for_resolved_player(
    store: &impl AppStore,
    input: SavePlayerNoteInput,
    player_puuid: String,
    display_name: String,
) -> Result<PlayerNoteView, ApplicationError> {
    validate_game_and_participant_ids(input.game_id, input.participant_id)?;
    let note = normalize_player_note(input.note)?;
    let tags = normalize_player_tags(input.tags)?;

    let saved = store
        .save_player_note(StoredPlayerNoteInput {
            player_puuid,
            last_display_name: display_name,
            note,
            tags,
        })
        .map_err(ApplicationError::Storage)?;

    Ok(player_note_view(
        input.game_id,
        input.participant_id,
        Some(saved),
    ))
}

pub fn clear_player_note_for_resolved_player(
    store: &impl AppStore,
    input: ClearPlayerNoteInput,
    player_puuid: &str,
) -> Result<ClearPlayerNoteResult, ApplicationError> {
    validate_game_and_participant_ids(input.game_id, input.participant_id)?;
    let cleared = store
        .clear_player_note(player_puuid)
        .map_err(ApplicationError::Storage)?;

    Ok(ClearPlayerNoteResult { cleared })
}

pub fn player_note_summary(
    store: &impl AppStore,
    player_puuid: Option<&str>,
) -> Result<PlayerNoteSummary, ApplicationError> {
    let Some(player_puuid) = player_puuid else {
        return Ok(PlayerNoteSummary {
            has_note: false,
            note: None,
            tags: Vec::new(),
        });
    };
    let note = store
        .get_player_note(player_puuid)
        .map_err(ApplicationError::Storage)?;

    Ok(PlayerNoteSummary {
        has_note: note.as_ref().is_some_and(|value| value.note.is_some()),
        note: note.as_ref().and_then(|value| value.note.clone()),
        tags: note.map(|value| value.tags).unwrap_or_default(),
    })
}

pub fn player_note_view(
    game_id: i64,
    participant_id: i64,
    note: Option<StoredPlayerNote>,
) -> PlayerNoteView {
    match note {
        Some(note) => PlayerNoteView {
            game_id,
            participant_id,
            note: note.note,
            tags: note.tags,
            updated_at: Some(note.updated_at),
        },
        None => PlayerNoteView {
            game_id,
            participant_id,
            note: None,
            tags: Vec::new(),
            updated_at: None,
        },
    }
}

fn resolve_post_match_participant_identity(
    reader: &impl LeagueClientReader,
    game_id: i64,
    participant_id: i64,
) -> Result<(String, String), ApplicationError> {
    validate_game_and_participant_ids(game_id, participant_id)?;
    let completed_match = reader
        .completed_match(game_id)
        .map_err(ApplicationError::from)?;
    let participant = completed_match
        .participants
        .iter()
        .find(|participant| participant.participant_id == participant_id)
        .ok_or_else(|| {
            ApplicationError::Validation(
                "Participant was not found in the completed match".to_string(),
            )
        })?;
    let player_puuid = participant.player_puuid.clone().ok_or_else(|| {
        ApplicationError::Validation("Participant cannot be linked to local notes".to_string())
    })?;

    Ok((player_puuid, participant.display_name.clone()))
}

fn post_match_detail_from_completed_match(
    store: &impl AppStore,
    completed_match: LeagueCompletedMatch,
) -> Result<PostMatchDetail, ApplicationError> {
    let mut participants = Vec::new();

    for participant in completed_match.participants {
        let note_summary = player_note_summary(store, participant.player_puuid.as_deref())?;
        participants.push(PostMatchParticipant {
            participant_id: participant.participant_id,
            team_id: participant.team_id,
            display_name: participant.display_name,
            champion_id: participant.champion_id,
            champion_name: participant.champion_name,
            role: participant.role,
            lane: participant.lane,
            profile_icon_id: participant.profile_icon_id,
            result: participant.result,
            kills: participant.kills,
            deaths: participant.deaths,
            assists: participant.assists,
            kda: participant.kda,
            performance_score: 0.0,
            cs: participant.cs,
            gold_earned: participant.gold_earned,
            damage_to_champions: participant.damage_to_champions,
            physical_damage_to_champions: participant.physical_damage_to_champions,
            magic_damage_to_champions: participant.magic_damage_to_champions,
            true_damage_to_champions: participant.true_damage_to_champions,
            damage_to_objectives: participant.damage_to_objectives,
            damage_to_turrets: participant.damage_to_turrets,
            damage_taken: participant.damage_taken,
            vision_score: participant.vision_score,
            wards_placed: participant.wards_placed,
            wards_killed: participant.wards_killed,
            control_wards_bought: participant.control_wards_bought,
            time_spent_dead_seconds: participant.time_spent_dead_seconds,
            largest_killing_spree: participant.largest_killing_spree,
            largest_multi_kill: participant.largest_multi_kill,
            double_kills: participant.double_kills,
            triple_kills: participant.triple_kills,
            quadra_kills: participant.quadra_kills,
            penta_kills: participant.penta_kills,
            first_blood: participant.first_blood,
            first_tower: participant.first_tower,
            items: participant.items,
            runes: participant.runes,
            spells: participant.spells,
            note_summary,
        });
    }

    score_post_match_participants(&mut participants, completed_match.game_duration_seconds);

    let teams = post_match_teams(&participants);
    let comparison = post_match_comparison(&participants);
    let warnings = if participants.len() < 2 {
        vec![LeagueDataWarning {
            section: LeagueDataSection::Participants,
            message: "Only partial participant details were available from the local client"
                .to_string(),
        }]
    } else {
        Vec::new()
    };

    Ok(PostMatchDetail {
        game_id: completed_match.game_id,
        queue_name: completed_match.queue_name,
        played_at: completed_match.played_at,
        game_duration_seconds: completed_match.game_duration_seconds,
        result: completed_match.result,
        self_participant_id: completed_match.self_participant_id,
        teams,
        comparison,
        warnings,
    })
}

fn post_match_teams(participants: &[PostMatchParticipant]) -> Vec<PostMatchTeam> {
    let mut team_ids: Vec<i64> = participants
        .iter()
        .map(|participant| participant.team_id)
        .collect();
    team_ids.sort_unstable();
    team_ids.dedup();

    team_ids
        .into_iter()
        .map(|team_id| {
            let team_participants: Vec<PostMatchParticipant> = participants
                .iter()
                .filter(|participant| participant.team_id == team_id)
                .cloned()
                .collect();
            let totals = team_totals(&team_participants);

            PostMatchTeam {
                team_id,
                result: team_participants
                    .first()
                    .map(|participant| participant.result)
                    .unwrap_or(MatchResult::Unknown),
                participants: team_participants,
                totals,
            }
        })
        .collect()
}

fn score_post_match_participants(
    participants: &mut [PostMatchParticipant],
    game_duration_seconds: Option<i64>,
) {
    // Pre-compute per-team aggregates once (O(n)) instead of per-participant (O(n²))
    let team_aggregates: Vec<(i64, i64, i64, i64)> = {
        let mut agg = Vec::new();
        for p in participants.iter() {
            let id = p.team_id;
            if let Some(entry) = agg.iter_mut().find(|(tid, _, _, _)| *tid == id) {
                entry.1 += p.kills;
                entry.2 += p.damage_to_champions;
                entry.3 += p.gold_earned;
            } else {
                agg.push((id, p.kills, p.damage_to_champions, p.gold_earned));
            }
        }
        agg
    };

    for participant in participants.iter_mut() {
        let (_, t_kills, t_damage, t_gold) = team_aggregates
            .iter()
            .find(|(id, _, _, _)| *id == participant.team_id)
            .expect("participant has a valid team_id");

        participant.performance_score = participant_performance_score(
            participant,
            *t_kills,
            *t_damage,
            *t_gold,
            game_duration_seconds,
        );
    }
}

fn participant_performance_score(
    participant: &PostMatchParticipant,
    team_kills: i64,
    team_damage: i64,
    team_gold: i64,
    game_duration_seconds: Option<i64>,
) -> f64 {
    let kda = participant.kda.unwrap_or_else(|| {
        calculate_kda(participant.kills, participant.deaths, participant.assists)
    });
    let duration_minutes = game_duration_seconds
        .filter(|seconds| *seconds > 0)
        .map(|seconds| seconds as f64 / 60.0);
    let kill_participation = if team_kills > 0 {
        ((participant.kills + participant.assists) as f64 / team_kills as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let damage_share = if team_damage > 0 {
        capped_ratio(
            participant.damage_to_champions as f64 / team_damage as f64,
            0.35,
        )
    } else {
        0.0
    };
    let gold_share = if team_gold > 0 {
        capped_ratio(participant.gold_earned as f64 / team_gold as f64, 0.28)
    } else {
        0.0
    };
    let cs_pace = duration_minutes
        .map(|minutes| capped_ratio(participant.cs as f64 / minutes, 10.0))
        .unwrap_or(0.0);
    let vision_pace = duration_minutes
        .map(|minutes| capped_ratio(participant.vision_score as f64 / minutes, 2.0))
        .unwrap_or(0.0);
    let result_value = match participant.result {
        MatchResult::Win => 1.0,
        MatchResult::Loss => 0.35,
        MatchResult::Unknown => 0.5,
    };
    let weighted_score = capped_ratio(kda, 12.0) * SCORE_KDA_WEIGHT
        + kill_participation * SCORE_KILL_PARTICIPATION_WEIGHT
        + damage_share * SCORE_DAMAGE_WEIGHT
        + gold_share * SCORE_GOLD_WEIGHT
        + cs_pace * SCORE_CS_WEIGHT
        + vision_pace * SCORE_VISION_WEIGHT
        + result_value * SCORE_RESULT_WEIGHT;

    round_to_tenth((1.0 + weighted_score * 9.0).clamp(0.0, 10.0))
}

fn capped_ratio(value: f64, cap: f64) -> f64 {
    if cap <= 0.0 {
        0.0
    } else {
        (value / cap).clamp(0.0, 1.0)
    }
}

fn team_totals(participants: &[PostMatchParticipant]) -> PostMatchTeamTotals {
    let mut kills = 0i64;
    let mut deaths = 0i64;
    let mut assists = 0i64;
    let mut gold_earned = 0i64;
    let mut damage_to_champions = 0i64;
    let mut vision_score = 0i64;
    for p in participants {
        kills += p.kills;
        deaths += p.deaths;
        assists += p.assists;
        gold_earned += p.gold_earned;
        damage_to_champions += p.damage_to_champions;
        vision_score += p.vision_score;
    }
    PostMatchTeamTotals {
        kills,
        deaths,
        assists,
        gold_earned,
        damage_to_champions,
        vision_score,
    }
}

fn post_match_comparison(participants: &[PostMatchParticipant]) -> PostMatchComparison {
    PostMatchComparison {
        highest_kda: metric_leader(participants, |participant| participant.kda.unwrap_or(0.0)),
        most_cs: metric_leader(participants, |participant| participant.cs as f64),
        most_gold: metric_leader(participants, |participant| participant.gold_earned as f64),
        most_damage: metric_leader(participants, |participant| {
            participant.damage_to_champions as f64
        }),
        highest_vision: metric_leader(participants, |participant| participant.vision_score as f64),
    }
}

fn metric_leader(
    participants: &[PostMatchParticipant],
    metric: impl Fn(&PostMatchParticipant) -> f64,
) -> Option<ParticipantMetricLeader> {
    participants
        .iter()
        .map(|participant| (participant, metric(participant)))
        .max_by(|left, right| {
            left.1
                .partial_cmp(&right.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(participant, value)| ParticipantMetricLeader {
            participant_id: participant.participant_id,
            display_name: participant.display_name.clone(),
            value,
        })
}

fn validate_settings(input: SettingsInput) -> Result<SettingsValues, ApplicationError> {
    let startup_page = StartupPage::parse(input.startup_page.as_str()).ok_or_else(|| {
        ApplicationError::Validation(
            "Startup page must be dashboard, profile, matches, advisor, or settings".into(),
        )
    })?;
    let language = AppLanguagePreference::parse(input.language.as_str())
        .ok_or_else(|| ApplicationError::Validation("Language must be system, zh, or en".into()))?;
    let theme = AppThemePreference::parse(input.theme.as_str())
        .ok_or_else(|| ApplicationError::Validation("Theme must be light or dark".into()))?;

    let ai_base_url = normalize_optional_string(input.ai_base_url);
    let ai_api_key = normalize_optional_string(input.ai_api_key);
    let ai_model = normalize_optional_string(input.ai_model);

    let values = SettingsValues {
        startup_page,
        language,
        theme,
        compact_mode: input.compact_mode,
        activity_limit: input.activity_limit,
        auto_accept_enabled: input.auto_accept_enabled,
        auto_pick_enabled: input.auto_pick_enabled,
        auto_pick_champion_id: input.auto_pick_champion_id,
        auto_pick_delay_seconds: normalize_delay_seconds(input.auto_pick_delay_seconds),
        auto_ban_enabled: input.auto_ban_enabled,
        auto_ban_champion_id: input.auto_ban_champion_id,
        auto_ban_delay_seconds: normalize_delay_seconds(input.auto_ban_delay_seconds),
        ai_base_url,
        ai_api_key,
        ai_model,
    };

    validate_settings_values(&values)?;
    Ok(values)
}

fn validate_settings_values(settings: &SettingsValues) -> Result<(), ApplicationError> {
    normalize_activity_limit(settings.activity_limit)?;
    validate_optional_champion_id(settings.auto_pick_champion_id, "Auto pick champion")?;
    validate_optional_champion_id(settings.auto_ban_champion_id, "Auto ban champion")?;

    if settings.auto_pick_enabled && settings.auto_pick_champion_id.is_none() {
        return Err(ApplicationError::Validation(
            "Auto pick requires a champion".to_string(),
        ));
    }

    if settings.auto_ban_enabled && settings.auto_ban_champion_id.is_none() {
        return Err(ApplicationError::Validation(
            "Auto ban requires a champion".to_string(),
        ));
    }

    Ok(())
}

fn validate_optional_champion_id(
    champion_id: Option<i64>,
    label: &str,
) -> Result<(), ApplicationError> {
    if let Some(champion_id) = champion_id {
        normalize_league_asset_id(champion_id, label)?;
    }

    Ok(())
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|s| {
        let trimmed = s.trim().to_string();
        if trimmed.is_empty() { None } else { Some(trimmed) }
    })
}

fn normalize_delay_seconds(value: f64) -> f64 {
    // Clamp to 0.0–5.0 and round to the nearest 0.5-second step.
    let clamped = value.clamp(0.0, 5.0);
    (clamped * 2.0).round() / 2.0
}

fn normalize_activity_limit(limit: i64) -> Result<i64, ApplicationError> {
    if (MIN_ACTIVITY_LIMIT..=MAX_ACTIVITY_LIMIT).contains(&limit) {
        Ok(limit)
    } else {
        Err(ApplicationError::Validation(format!(
            "Activity limit must be between {MIN_ACTIVITY_LIMIT} and {MAX_ACTIVITY_LIMIT}"
        )))
    }
}

fn normalize_match_limit(limit: i64) -> Result<i64, ApplicationError> {
    if (1..=MAX_MATCH_LIMIT).contains(&limit) {
        Ok(limit)
    } else {
        Err(ApplicationError::Validation(format!(
            "Match limit must be between 1 and {MAX_MATCH_LIMIT}"
        )))
    }
}

fn normalize_league_asset_id(id: i64, label: &str) -> Result<i64, ApplicationError> {
    if (1..=MAX_LEAGUE_ASSET_ID).contains(&id) {
        Ok(id)
    } else {
        Err(ApplicationError::Validation(format!(
            "{label} must be between 1 and {MAX_LEAGUE_ASSET_ID}"
        )))
    }
}

fn validate_game_and_participant_ids(
    game_id: i64,
    participant_id: i64,
) -> Result<(), ApplicationError> {
    validate_game_id(game_id)?;

    if participant_id <= 0 {
        return Err(ApplicationError::Validation(
            "Participant id must be greater than 0".to_string(),
        ));
    }

    Ok(())
}

fn validate_game_id(game_id: i64) -> Result<(), ApplicationError> {
    if game_id <= 0 {
        return Err(ApplicationError::Validation(
            "Game id must be greater than 0".to_string(),
        ));
    }

    Ok(())
}

fn normalize_player_note(note: Option<String>) -> Result<Option<String>, ApplicationError> {
    let note = note
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    if let Some(value) = &note
        && value.chars().count() > MAX_PLAYER_NOTE_LEN
    {
        return Err(ApplicationError::Validation(format!(
            "Player note must be {MAX_PLAYER_NOTE_LEN} characters or fewer"
        )));
    }

    Ok(note)
}

fn normalize_player_tags(tags: Vec<String>) -> Result<Vec<String>, ApplicationError> {
    let mut normalized = Vec::new();

    for tag in tags {
        let tag = tag.trim().to_string();
        if tag.is_empty() || normalized.iter().any(|value| value == &tag) {
            continue;
        }

        if tag.chars().count() > MAX_PLAYER_TAG_LEN {
            return Err(ApplicationError::Validation(format!(
                "Player tags must be {MAX_PLAYER_TAG_LEN} characters or fewer"
            )));
        }

        normalized.push(tag);
    }

    if normalized.len() > MAX_PLAYER_TAGS {
        return Err(ApplicationError::Validation(format!(
            "Player tags must include {MAX_PLAYER_TAGS} entries or fewer"
        )));
    }

    Ok(normalized)
}

fn summarize_recent_performance(matches: &[RecentMatchSummary]) -> RecentPerformanceSummary {
    let recent_matches = matches.iter().take(PERFORMANCE_MATCH_COUNT);
    let mut total_kda = 0.0;
    let mut match_count = 0;
    let mut recent_champions = Vec::new();

    for match_summary in recent_matches {
        match_count += 1;
        total_kda += calculate_kda(
            match_summary.kills,
            match_summary.deaths,
            match_summary.assists,
        );
        recent_champions.push(match_summary.champion_name.clone());
    }

    let average_kda = if match_count == 0 {
        None
    } else {
        Some(round_to_tenth(total_kda / match_count as f64))
    };

    let kda_tag = match average_kda {
        Some(value) if value >= HIGH_KDA_THRESHOLD => KdaTag::High,
        Some(_) => KdaTag::Standard,
        None => KdaTag::Unavailable,
    };

    RecentPerformanceSummary {
        match_count,
        average_kda,
        kda_tag,
        recent_champions,
        top_champions: summarize_top_champions(matches),
    }
}

fn summarize_top_champions(matches: &[RecentMatchSummary]) -> Vec<RecentChampionSummary> {
    let mut counts: Vec<(Option<i64>, String, usize, usize)> = Vec::new();

    for (index, match_summary) in matches.iter().take(PERFORMANCE_MATCH_COUNT).enumerate() {
        if let Some((_, _, games, _)) =
            counts
                .iter_mut()
                .find(|(champion_id, champion_name, _, _)| {
                    *champion_id == match_summary.champion_id
                        && champion_name == &match_summary.champion_name
                })
        {
            *games += 1;
            continue;
        }

        counts.push((
            match_summary.champion_id,
            match_summary.champion_name.clone(),
            1,
            index,
        ));
    }

    counts.sort_by(|left, right| right.2.cmp(&left.2).then_with(|| left.3.cmp(&right.3)));
    counts
        .into_iter()
        .take(3)
        .map(
            |(champion_id, champion_name, games, _)| RecentChampionSummary {
                champion_id,
                champion_name,
                games,
            },
        )
        .collect()
}

/// Aggregates win/loss per champion over the FULL match window provided (not
/// capped at `PERFORMANCE_MATCH_COUNT`, unlike `summarize_top_champions`). The
/// caller controls depth via the fetched `match_limit`; the Profile page asks
/// for a wide window so mastery champions show a meaningful recent record.
/// Matches with an unknown result still count toward `games` but not W/L.
fn summarize_champion_records(matches: &[RecentMatchSummary]) -> Vec<ChampionRecordSummary> {
    let mut records: Vec<ChampionRecordSummary> = Vec::new();

    for match_summary in matches {
        let Some(champion_id) = match_summary.champion_id else {
            continue;
        };
        let record = match records.iter_mut().find(|r| r.champion_id == champion_id) {
            Some(existing) => existing,
            None => {
                records.push(ChampionRecordSummary {
                    champion_id,
                    wins: 0,
                    losses: 0,
                    games: 0,
                });
                records.last_mut().expect("just pushed")
            }
        };
        record.games += 1;
        match match_summary.result {
            MatchResult::Win => record.wins += 1,
            MatchResult::Loss => record.losses += 1,
            MatchResult::Unknown => {}
        }
    }

    records
}

fn calculate_kda(kills: i64, deaths: i64, assists: i64) -> f64 {
    let contribution = (kills + assists) as f64;

    if deaths <= 0 {
        contribution
    } else {
        contribution / deaths as f64
    }
}

fn round_to_tenth(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn ranked_champion_stat(seed: &RankedChampionSeed) -> RankedChampionStat {
    RankedChampionStat {
        champion_id: seed.champion_id,
        champion_name: seed.champion_name.to_string(),
        champion_alias: None,
        lane: seed.lane,
        win_rate: seed.win_rate,
        pick_rate: seed.pick_rate,
        ban_rate: seed.ban_rate,
        overall_score: ranked_overall_score(seed.win_rate, seed.pick_rate, seed.ban_rate),
        games: seed.games,
        wins: ((seed.games as f64) * (seed.win_rate / 100.0)).round() as i64,
        picks: seed.games,
        bans: ((seed.games as f64) * (seed.ban_rate / 100.0)).round() as i64,
    }
}

fn ranked_response_from_snapshot(
    snapshot: RankedChampionDataSnapshot,
    input: RankedChampionStatsInput,
    is_cached: bool,
    data_status: RankedChampionDataStatus,
    status_message: Option<String>,
) -> RankedChampionStatsResponse {
    let sort_by = input.sort_by.unwrap_or(RankedChampionSort::Overall);
    let mut records: Vec<RankedChampionStat> = snapshot
        .records
        .into_iter()
        .filter(|record| input.lane.is_none_or(|lane| record.lane == lane))
        .collect();

    records.sort_by(|left, right| compare_ranked_champions(left, right, sort_by));

    RankedChampionStatsResponse {
        lane: input.lane,
        sort_by,
        records,
        source: snapshot.source,
        updated_at: snapshot
            .generated_at
            .clone()
            .unwrap_or_else(|| snapshot.imported_at.clone()),
        generated_at: snapshot.generated_at,
        imported_at: Some(snapshot.imported_at),
        patch: snapshot.patch,
        region: snapshot.region,
        queue: snapshot.queue,
        tier: snapshot.tier,
        is_cached,
        data_status,
        status_message,
    }
}

fn advisor_response_from_snapshot(
    snapshot: AdvisorDataSnapshot,
    input: AdvisorDataInput,
    is_cached: bool,
    data_status: RankedChampionDataStatus,
    status_message: Option<String>,
) -> AdvisorDataResponse {
    let records = snapshot
        .records
        .into_iter()
        .filter(|record| input.lane.is_none_or(|lane| record.lane == lane))
        .filter(|record| {
            input
                .champion_id
                .is_none_or(|champion_id| record.champion_id == champion_id)
        })
        .collect();

    AdvisorDataResponse {
        lane: input.lane,
        champion_id: input.champion_id,
        records,
        source: snapshot.source,
        updated_at: snapshot
            .generated_at
            .clone()
            .unwrap_or_else(|| snapshot.imported_at.clone()),
        generated_at: snapshot.generated_at,
        imported_at: Some(snapshot.imported_at),
        patch: snapshot.patch,
        region: snapshot.region,
        queue: snapshot.queue,
        tier: snapshot.tier,
        is_cached,
        data_status,
        status_message,
    }
}

fn player_advisor_tags(
    recent_stats: &Option<ParticipantRecentStats>,
    advisor: Option<&AdvisorRecord>,
) -> Vec<AdvisorPlayerTag> {
    let mut tags = Vec::new();

    if let Some(recent_stats) = recent_stats {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for match_summary in recent_stats
            .recent_matches
            .iter()
            .take(PERFORMANCE_MATCH_COUNT)
        {
            *counts
                .entry(match_summary.champion_name.to_ascii_lowercase())
                .or_default() += 1;
        }
        if counts.values().copied().max().unwrap_or(0) >= 3 {
            tags.push(AdvisorPlayerTag {
                label: "One-trick".to_string(),
                tone: AdvisorTagTone::Good,
            });
        }

        let loss_streak = recent_stats
            .recent_matches
            .iter()
            .take(PERFORMANCE_MATCH_COUNT)
            .take_while(|match_summary| match_summary.result == MatchResult::Loss)
            .count();
        if loss_streak >= 3 {
            tags.push(AdvisorPlayerTag {
                label: format!("{loss_streak} loss streak"),
                tone: AdvisorTagTone::Warn,
            });
        }
    }

    if let Some(advisor) = advisor {
        let (label, tone) = if advisor.win_rate >= 52.0 || advisor.overall_score >= 55.0 {
            ("Strong pick", AdvisorTagTone::Good)
        } else if advisor.win_rate < 49.0 {
            ("Low WR", AdvisorTagTone::Warn)
        } else {
            ("Stable", AdvisorTagTone::Info)
        };
        tags.push(AdvisorPlayerTag {
            label: label.to_string(),
            tone,
        });

        if let Some(spike) = advisor.power_spikes.first() {
            tags.push(AdvisorPlayerTag {
                label: format!("Spike {}", spike.timing),
                tone: AdvisorTagTone::Info,
            });
        }
    }

    tags
}

fn matchup_advice(advisor: &AdvisorRecord, opponent: &ChampSelectAdvisorPlayer) -> Option<String> {
    let opponent_id = opponent.champion_id?;
    advisor
        .strong_against
        .iter()
        .find(|matchup| matchup.champion_id == opponent_id)
        .map(|matchup| format!("Favorable into {}: {}", matchup.champion_name, matchup.note))
        .or_else(|| {
            advisor
                .weak_against
                .iter()
                .find(|matchup| matchup.champion_id == opponent_id)
                .map(|matchup| {
                    format!("Difficult into {}: {}", matchup.champion_name, matchup.note)
                })
        })
}

fn sample_advisor_snapshot() -> AdvisorDataSnapshot {
    AdvisorDataSnapshot {
        source: "Local advisor sample".to_string(),
        patch: Some("26.08".to_string()),
        region: Some("KR".to_string()),
        queue: Some("RANKED_SOLO_5X5".to_string()),
        tier: Some("EMERALD_PLUS".to_string()),
        generated_at: Some("2026-04-25T00:00:00Z".to_string()),
        imported_at: "2026-04-25 00:00:00".to_string(),
        records: vec![
            sample_advisor_record(
                86,
                "Garen",
                "Garen",
                RankedChampionLane::Top,
                52.1,
                vec![122, 266],
                vec![164],
                "Short trades are strong when Q is ready. Hold W for enemy burst windows.",
                "Front-to-back fights are best. Silence the nearest carry or diver before using R.",
            ),
            sample_advisor_record(
                5,
                "Xin Zhao",
                "XinZhao",
                RankedChampionLane::Jungle,
                52.3,
                vec![64],
                vec![234],
                "Path toward early skirmishes and protect lanes with setup CC.",
                "Use R to isolate priority targets and deny ranged follow-up.",
            ),
            sample_advisor_record(
                103,
                "Ahri",
                "Ahri",
                RankedChampionLane::Middle,
                51.1,
                vec![134],
                vec![777],
                "Play around charm threat and push waves before roaming.",
                "Enter after cooldowns are used, then use R charges to clean up fights.",
            ),
            sample_advisor_record(
                222,
                "Jinx",
                "Jinx",
                RankedChampionLane::Bottom,
                51.9,
                vec![145],
                vec![81],
                "Scale safely and trade when rockets can hit both ADC and support.",
                "Reset fights are the win condition. Stay behind peel until passive procs.",
            ),
            sample_advisor_record(
                412,
                "Thresh",
                "Thresh",
                RankedChampionLane::Support,
                50.5,
                vec![497],
                vec![555],
                "Keep lane brushes warded and threaten hook when enemy last-hits.",
                "Hold lantern for carries instead of forcing every engage.",
            ),
        ],
    }
}

#[allow(clippy::too_many_arguments)]
fn sample_advisor_record(
    champion_id: i64,
    champion_name: &str,
    champion_alias: &str,
    lane: RankedChampionLane,
    win_rate: f64,
    strong_against_ids: Vec<i64>,
    weak_against_ids: Vec<i64>,
    lane_advice: &str,
    teamfight_advice: &str,
) -> AdvisorRecord {
    AdvisorRecord {
        champion_id,
        champion_name: champion_name.to_string(),
        champion_alias: Some(champion_alias.to_string()),
        lane,
        win_rate,
        pick_rate: 7.5,
        ban_rate: 8.0,
        overall_score: ranked_overall_score(win_rate, 7.5, 8.0),
        games: 100_000,
        runes: AdvisorRunePage {
            primary_style: "Precision".to_string(),
            primary_runes: vec![
                named_ref(Some(8010), "Conqueror"),
                named_ref(Some(9111), "Triumph"),
                named_ref(Some(9104), "Legend: Alacrity"),
                named_ref(Some(8299), "Last Stand"),
            ],
            secondary_style: "Resolve".to_string(),
            secondary_runes: vec![
                named_ref(Some(8444), "Second Wind"),
                named_ref(Some(8451), "Overgrowth"),
            ],
            stat_shards: vec![
                "Attack Speed".to_string(),
                "Adaptive Force".to_string(),
                "Health Scaling".to_string(),
            ],
        },
        summoner_spells: vec![named_ref(Some(4), "Flash"), named_ref(Some(14), "Ignite")],
        skill_order: AdvisorSkillOrder {
            max_order: vec!["Q".to_string(), "E".to_string(), "W".to_string()],
            early_order: vec![
                "Q".to_string(),
                "E".to_string(),
                "W".to_string(),
                "Q".to_string(),
                "Q".to_string(),
                "R".to_string(),
            ],
        },
        item_build: AdvisorItemBuild {
            starter: vec![
                named_ref(Some(1055), "Doran's Blade"),
                named_ref(Some(2003), "Health Potion"),
            ],
            core: vec![
                named_ref(Some(6631), "Stridebreaker"),
                named_ref(Some(3053), "Sterak's Gage"),
            ],
            boots: vec![named_ref(Some(3047), "Plated Steelcaps")],
            late: vec![
                named_ref(Some(6333), "Death's Dance"),
                named_ref(Some(3075), "Thornmail"),
            ],
            situational: vec![named_ref(Some(3156), "Maw of Malmortius")],
        },
        strong_against: strong_against_ids
            .into_iter()
            .map(|id| AdvisorMatchup {
                champion_id: id,
                champion_name: format!("Champion {id}"),
                note: "Can contest short trades and punish cooldowns.".to_string(),
                win_rate_delta: Some(2.0),
            })
            .collect(),
        weak_against: weak_against_ids
            .into_iter()
            .map(|id| AdvisorMatchup {
                champion_id: id,
                champion_name: format!("Champion {id}"),
                note: "Respect range or all-in timing before first item.".to_string(),
                win_rate_delta: Some(-2.0),
            })
            .collect(),
        power_spikes: vec![
            AdvisorPowerSpike {
                timing: "6".to_string(),
                label: "Ultimate threat".to_string(),
                description: "Look for an all-in after unlocking R.".to_string(),
            },
            AdvisorPowerSpike {
                timing: "1 item".to_string(),
                label: "Core item".to_string(),
                description: "First completed item opens stronger skirmishes.".to_string(),
            },
        ],
        lane_advice: lane_advice.to_string(),
        teamfight_advice: teamfight_advice.to_string(),
    }
}

fn named_ref(id: Option<i64>, name: &str) -> AdvisorNamedRef {
    AdvisorNamedRef {
        id,
        name: name.to_string(),
    }
}

fn ranked_overall_score(win_rate: f64, pick_rate: f64, ban_rate: f64) -> f64 {
    round_to_tenth((win_rate * 0.55) + (pick_rate * 0.25) + (ban_rate * 0.20))
}

fn compare_ranked_champions(
    left: &RankedChampionStat,
    right: &RankedChampionStat,
    sort_by: RankedChampionSort,
) -> Ordering {
    let left_value = ranked_sort_value(left, sort_by);
    let right_value = ranked_sort_value(right, sort_by);

    right_value
        .partial_cmp(&left_value)
        .unwrap_or(Ordering::Equal)
        .then_with(|| {
            right
                .overall_score
                .partial_cmp(&left.overall_score)
                .unwrap_or(Ordering::Equal)
        })
        .then_with(|| left.champion_name.cmp(&right.champion_name))
}

fn ranked_sort_value(record: &RankedChampionStat, sort_by: RankedChampionSort) -> f64 {
    match sort_by {
        RankedChampionSort::Overall => record.overall_score,
        RankedChampionSort::WinRate => record.win_rate,
        RankedChampionSort::BanRate => record.ban_rate,
        RankedChampionSort::PickRate => record.pick_rate,
    }
}

fn unix_timestamp_seconds() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn validate_activity_title(title: &str, label: &str) -> Result<String, ApplicationError> {
    let trimmed = title.trim().to_string();
    if trimmed.is_empty() {
        return Err(ApplicationError::Validation(format!("{label} is required")));
    }
    if trimmed.chars().count() > MAX_ACTIVITY_TITLE_LEN {
        return Err(ApplicationError::Validation(format!(
            "{label} must be {MAX_ACTIVITY_TITLE_LEN} characters or fewer"
        )));
    }
    Ok(trimmed)
}

fn validate_activity_body(
    body: &Option<String>,
    label: &str,
) -> Result<Option<String>, ApplicationError> {
    let trimmed = body
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    if let Some(ref value) = trimmed
        && value.chars().count() > MAX_ACTIVITY_BODY_LEN
    {
        return Err(ApplicationError::Validation(format!(
            "{label} must be {MAX_ACTIVITY_BODY_LEN} characters or fewer"
        )));
    }
    Ok(trimmed)
}

fn validate_activity_note(input: ActivityNoteInput) -> Result<NewActivityEntry, ApplicationError> {
    let title = validate_activity_title(&input.title, "Activity note title")?;
    let body = validate_activity_body(&input.body, "Activity note body")?;

    Ok(NewActivityEntry {
        kind: ActivityKind::Note,
        title,
        body,
    })
}

fn validate_local_activity_entry(entry: &LocalActivityEntry) -> Result<(), ApplicationError> {
    validate_activity_title(&entry.title, "Imported activity title")?;
    validate_activity_body(&entry.body, "Imported activity body")?;

    if entry.created_at.trim().is_empty() {
        return Err(ApplicationError::Validation(
            "Imported activity createdAt is required".to_string(),
        ));
    }

    Ok(())
}

pub fn get_champ_select_snapshot(
    reader: &(impl LeagueClientReader + Sync),
    recent_limit: i64,
) -> Result<domain::ChampSelectSnapshot, ApplicationError> {
    #[derive(Debug)]
    struct PlayerSeed {
        summoner_id: i64,
        puuid: String,
        display_name: String,
        champion_id: Option<i64>,
        team: domain::ChampSelectTeam,
        summoner_level: Option<i64>,
    }

    let session = reader.champ_select_session()?;
    let mut all_ids: Vec<i64> = session
        .ally_ids
        .iter()
        .chain(session.enemy_ids.iter())
        .copied()
        .collect();
    all_ids.sort_unstable();
    all_ids.dedup();
    let summoners = reader.summoners_by_ids(&all_ids);
    let summoners_by_id: HashMap<i64, SummonerBatchEntry> = summoners
        .into_iter()
        .map(|summoner| (summoner.summoner_id, summoner))
        .collect();
    let all_names: Vec<String> = session
        .ally_names
        .iter()
        .chain(session.enemy_names.iter())
        .filter(|name| !name.trim().is_empty())
        .cloned()
        .collect();
    let summoners_by_name: HashMap<String, SummonerBatchEntry> = reader
        .summoners_by_names(&all_names)
        .into_iter()
        .flat_map(|summoner| {
            summoner_name_lookup_keys(summoner.display_name.as_str())
                .into_iter()
                .map(move |name| (name, summoner.clone()))
        })
        .collect();

    let mut seeds = Vec::new();
    let mut seen_ids = HashSet::new();
    let mut seen_names = HashSet::new();
    let mut seen_puuids = HashSet::new();

    for (index, player) in session.players.iter().enumerate() {
        let display_name = if player.display_name.trim().is_empty() {
            player
                .summoner_id
                .map(|id| format!("Summoner {id}"))
                .unwrap_or_else(|| format!("Player {}", index + 1))
        } else {
            player.display_name.clone()
        };
        let normalized_name = normalize_player_name(display_name.as_str());
        let summoner_id = player
            .summoner_id
            .filter(|id| *id > 0)
            .unwrap_or_else(|| negative_stable_id(display_name.as_str()));
        let puuid = player
            .puuid
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .cloned()
            .unwrap_or_default();

        if !puuid.is_empty() && !seen_puuids.insert(puuid.clone()) {
            continue;
        }
        if summoner_id > 0 && seen_ids.contains(&summoner_id) {
            continue;
        }
        if !normalized_name.is_empty() && seen_names.contains(&normalized_name) {
            continue;
        }

        if summoner_id > 0 {
            seen_ids.insert(summoner_id);
        }
        if !normalized_name.is_empty() {
            seen_names.insert(normalized_name);
        }
        let summoner_level = summoners_by_id.get(&summoner_id).and_then(|s| s.summoner_level);
        seeds.push(PlayerSeed {
            summoner_id,
            puuid,
            display_name,
            champion_id: player.champion_id,
            team: player.team.clone(),
            summoner_level,
        });
    }

    for summoner_id in all_ids {
        if seen_ids.contains(&summoner_id) {
            continue;
        }
        let summoner = summoners_by_id.get(&summoner_id);
        let team = if session.ally_ids.contains(&summoner_id) {
            domain::ChampSelectTeam::Ally
        } else {
            domain::ChampSelectTeam::Enemy
        };
        let champion_id = session.champion_selections.get(&summoner_id).copied();
        let puuid = summoner
            .map(|value| value.puuid.clone())
            .unwrap_or_default();
        let display_name = summoner
            .map(|value| value.display_name.clone())
            .unwrap_or_else(|| format!("Summoner {summoner_id}"));

        if !puuid.is_empty() {
            seen_puuids.insert(puuid.clone());
        }
        seen_ids.insert(summoner_id);
        seen_names.insert(normalize_player_name(display_name.as_str()));
        let summoner_level = summoner.and_then(|s| s.summoner_level);
        seeds.push(PlayerSeed {
            summoner_id,
            puuid,
            display_name,
            champion_id,
            team,
            summoner_level,
        });
    }

    for (name, team) in session
        .ally_names
        .iter()
        .map(|name| (name, domain::ChampSelectTeam::Ally))
        .chain(
            session
                .enemy_names
                .iter()
                .map(|name| (name, domain::ChampSelectTeam::Enemy)),
        )
    {
        let normalized_name = normalize_player_name(name.as_str());
        if normalized_name.is_empty() || seen_names.contains(&normalized_name) {
            continue;
        }

        let summoner = summoners_by_name.get(&normalized_name);
        if let Some(summoner) = summoner {
            if seen_ids.contains(&summoner.summoner_id) {
                continue;
            }
            seen_ids.insert(summoner.summoner_id);
        }

        let summoner_id = summoner
            .map(|value| value.summoner_id)
            .unwrap_or_else(|| negative_stable_id(name.as_str()));
        let puuid = summoner
            .map(|value| value.puuid.clone())
            .unwrap_or_default();
        let display_name = summoner
            .map(|value| value.display_name.clone())
            .unwrap_or_else(|| name.clone());
        let champion_id = session
            .champion_selections_by_name
            .get(&normalized_name)
            .copied();

        let summoner_level = summoner.and_then(|s| s.summoner_level);
        seen_names.insert(normalized_name);
        seeds.push(PlayerSeed {
            summoner_id,
            puuid,
            display_name,
            champion_id,
            team,
            summoner_level,
        });
    }

    // Enrich seeds with empty PUUIDs via the name-based summoner lookup.
    // session.players can include enemy players with empty PUUIDs (common in
    // the Tencent client), and since those players are already in seen_names,
    // the name loop above skips them — leaving their PUUID empty even though
    // summoners_by_name may already have a resolved PUUID for them.
    for seed in &mut seeds {
        let needs_puuid = seed.puuid.is_empty();
        let needs_level = seed.summoner_level.is_none();
        if !needs_puuid && !needs_level {
            continue;
        }
        for key in summoner_name_lookup_keys(seed.display_name.as_str()) {
            if let Some(summoner) = summoners_by_name.get(&key) {
                if needs_puuid && !summoner.puuid.is_empty() {
                    seed.puuid = summoner.puuid.clone();
                }
                if needs_level && summoner.summoner_level.is_some() {
                    seed.summoner_level = summoner.summoner_level;
                }
                if !seed.puuid.is_empty() && seed.summoner_level.is_some() {
                    break;
                }
            }
        }
    }

    let player_count = seeds.len();
    let missing_identity_count = seeds.iter().filter(|seed| seed.puuid.is_empty()).count();
    let resolved_identity_count = player_count.saturating_sub(missing_identity_count);
    let mut recent_stats_requested = 0;
    let recent_stats_by_puuid = if recent_limit <= 0 {
        HashMap::new()
    } else {
        let mut puuids: Vec<String> = seeds
            .iter()
            .map(|seed| seed.puuid.clone())
            .filter(|puuid| !puuid.is_empty())
            .collect();
        puuids.sort_unstable();
        puuids.dedup();
        recent_stats_requested = puuids.len();
        if puuids.is_empty() {
            HashMap::new()
        } else {
            reader.participant_recent_stats_batch(&puuids, recent_limit)
        }
    };
    let recent_stats_success = recent_stats_by_puuid
        .values()
        .filter(|result| result.is_ok())
        .count();
    let recent_stats_failed = recent_stats_by_puuid
        .values()
        .filter(|result| result.is_err())
        .count();
    log_overlay_history_snapshot(OverlayHistoryLogArgs {
        source: session.source,
        player_count,
        resolved_identity_count,
        missing_identity_count,
        recent_stats_requested,
        recent_stats_success,
        recent_stats_failed,
        recent_limit,
    });

    // Ranked stats and mastery are only fetched on full builds (recent_limit > 0).
    // Light builds (event-driven, limit=0) skip these to avoid firing 10-20 LCU
    // requests on every champ-select event. The platform layer carries the data
    // forward from the cache via merge_cached_player_data.
    let ranked_by_puuid: HashMap<String, Vec<RankedQueueSummary>> = if recent_limit > 0 {
        let mut ranked_puuids: Vec<String> = seeds
            .iter()
            .filter(|seed| !seed.puuid.is_empty())
            .map(|seed| seed.puuid.clone())
            .collect();
        ranked_puuids.sort_unstable();
        ranked_puuids.dedup();
        if ranked_puuids.is_empty() {
            HashMap::new()
        } else {
            reader.participant_ranked_stats_batch(&ranked_puuids)
        }
    } else {
        HashMap::new()
    };

    let mastery_by_puuid: HashMap<String, Option<i64>> = if recent_limit > 0 {
        let mastery_entries: Vec<(String, i64)> = seeds
            .iter()
            .filter(|seed| !seed.puuid.is_empty())
            .filter_map(|seed| seed.champion_id.map(|cid| (seed.puuid.clone(), cid)))
            .collect();
        if mastery_entries.is_empty() {
            HashMap::new()
        } else {
            reader.champion_mastery_batch(&mastery_entries)
        }
    } else {
        HashMap::new()
    };

    let players = seeds
        .into_iter()
        .map(|seed| {
            let recent_stats_result = recent_stats_by_puuid.get(seed.puuid.as_str());
            let diagnosis = diagnose_recent_stats(recent_limit, seed.puuid.as_str(), recent_stats_result);
            if diagnosis.should_log_to_stderr() {
                eprintln!(
                    "[overlay-history] player team={team} name={name:?} championId={champion:?} -> {phrase}",
                    team = champ_select_team_log_label(&seed.team),
                    name = seed.display_name,
                    champion = seed.champion_id,
                    phrase = diagnosis.log_phrase(),
                );
            }
            let recent_stats = recent_stats_result.and_then(|result| result.clone().ok());
            let recent_stats_status = diagnosis.public_status();
            let ranked_queues = ranked_by_puuid
                .get(seed.puuid.as_str())
                .cloned()
                .unwrap_or_default();
            let mastery_level = mastery_by_puuid
                .get(seed.puuid.as_str())
                .and_then(|v| *v);

            domain::ChampSelectPlayer {
                summoner_id: seed.summoner_id,
                puuid: seed.puuid,
                display_name: seed.display_name,
                champion_id: seed.champion_id,
                champion_name: None,
                team: seed.team,
                ranked_queues,
                summoner_level: seed.summoner_level,
                mastery_level,
                recent_stats,
                recent_stats_status,
            }
        })
        .collect();

    Ok(domain::ChampSelectSnapshot {
        players,
        cached_at: unix_timestamp_seconds(),
    })
}

struct OverlayHistoryLogArgs {
    source: ChampSelectSessionSource,
    player_count: usize,
    resolved_identity_count: usize,
    missing_identity_count: usize,
    recent_stats_requested: usize,
    recent_stats_success: usize,
    recent_stats_failed: usize,
    recent_limit: i64,
}

fn log_overlay_history_snapshot(args: OverlayHistoryLogArgs) {
    eprintln!(
        "[overlay-history] roster source={} players={} puuidResolved={} missingIdentity={} recentLimit={}",
        args.source.as_log_label(),
        args.player_count,
        args.resolved_identity_count,
        args.missing_identity_count,
        args.recent_limit
    );
    eprintln!(
        "[overlay-history] recent stats requested={} success={} failed={} missingIdentity={}",
        args.recent_stats_requested,
        args.recent_stats_success,
        args.recent_stats_failed,
        args.missing_identity_count
    );
}

/// Per-player explanation for why recent_stats is or is not populated.
/// Derived from the same inputs the public `ChampSelectRecentStatsStatus` uses,
/// but carries the underlying LCU error and match count so failures can be
/// diagnosed at runtime from logs alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RecentStatsDiagnosis {
    /// `recent_limit <= 0` — light/event-driven build path skipped fetching.
    NotRequested,
    /// PUUID was empty after both the champ-select session and name-lookup
    /// fallback. Typically Riot anti-dodge anonymization of `their_team`
    /// during ChampSelect, or an unresolved display name.
    MissingIdentity,
    /// LCU returned a successful response containing N games.
    Loaded(usize),
    /// LCU returned `Ok` but with zero games (account exists but no history).
    LoadedEmpty,
    /// LCU rejected the match-history request for this PUUID. Carries the
    /// rendered error message so it shows up in the log line.
    LcuError(String),
    /// The batch HashMap had no entry for this PUUID at all — should not
    /// normally happen and indicates an upstream batching bug.
    NoResult,
}

impl RecentStatsDiagnosis {
    pub(crate) fn public_status(&self) -> ChampSelectRecentStatsStatus {
        match self {
            Self::NotRequested => ChampSelectRecentStatsStatus::NotRequested,
            Self::MissingIdentity => ChampSelectRecentStatsStatus::MissingIdentity,
            Self::Loaded(_) | Self::LoadedEmpty => ChampSelectRecentStatsStatus::Loaded,
            Self::LcuError(_) | Self::NoResult => ChampSelectRecentStatsStatus::Unavailable,
        }
    }

    pub(crate) fn should_log_to_stderr(&self) -> bool {
        // Suppress the noisy happy path; surface only states the user/
        // developer would want to know about when triaging.
        !matches!(self, Self::NotRequested | Self::Loaded(_))
    }

    pub(crate) fn log_phrase(&self) -> String {
        match self {
            Self::NotRequested => "skipped (recentLimit<=0)".to_string(),
            Self::MissingIdentity => {
                "MISSING_IDENTITY — LCU never exposed a PUUID for this player (Riot anti-dodge anonymization of enemies during ChampSelect, or summoner-name lookup did not resolve)".to_string()
            }
            Self::Loaded(count) => format!("OK matches={count}"),
            Self::LoadedEmpty => {
                "EMPTY_HISTORY — LCU returned 0 games for this PUUID (new account or hidden history)".to_string()
            }
            Self::LcuError(message) => {
                format!("LCU_ERROR — match-history request failed: {message}")
            }
            Self::NoResult => {
                "NO_RESULT — recent_stats batch returned no entry for this PUUID (upstream batching bug)".to_string()
            }
        }
    }
}

pub(crate) fn diagnose_recent_stats(
    recent_limit: i64,
    puuid: &str,
    result: Option<&Result<ParticipantRecentStats, LeagueClientReadError>>,
) -> RecentStatsDiagnosis {
    if recent_limit <= 0 {
        return RecentStatsDiagnosis::NotRequested;
    }
    if puuid.trim().is_empty() {
        return RecentStatsDiagnosis::MissingIdentity;
    }
    match result {
        Some(Ok(stats)) if stats.recent_matches.is_empty() => RecentStatsDiagnosis::LoadedEmpty,
        Some(Ok(stats)) => RecentStatsDiagnosis::Loaded(stats.recent_matches.len()),
        Some(Err(error)) => RecentStatsDiagnosis::LcuError(error.to_string()),
        None => RecentStatsDiagnosis::NoResult,
    }
}

fn champ_select_team_log_label(team: &domain::ChampSelectTeam) -> &'static str {
    match team {
        domain::ChampSelectTeam::Ally => "ally",
        domain::ChampSelectTeam::Enemy => "enemy",
    }
}

pub fn get_league_champion_catalog(
    reader: &impl LeagueClientReader,
) -> Result<Vec<LeagueChampionSummary>, ApplicationError> {
    let mut champions = reader.champion_catalog()?;
    champions.sort_by(|left, right| {
        left.champion_name
            .to_ascii_lowercase()
            .cmp(&right.champion_name.to_ascii_lowercase())
            .then(left.champion_id.cmp(&right.champion_id))
    });
    Ok(champions)
}

pub fn run_lobby_automation(
    store: &impl AppStore,
    reader: &impl LeagueClientReader,
) -> Result<(), ApplicationError> {
    run_ready_check_automation(store, reader)?;
    run_champ_select_automation(store, reader)
}

pub fn run_ready_check_automation(
    store: &impl AppStore,
    reader: &impl LeagueClientReader,
) -> Result<(), ApplicationError> {
    let settings = store.get_settings().map_err(ApplicationError::Storage)?;

    if !settings.auto_accept_enabled {
        log_auto_accept_event("skipped because setting is disabled");
        return Ok(());
    }

    if !is_ready_check_active(reader)? {
        log_auto_accept_event("skipped because ReadyCheck is not active");
        return Ok(());
    }
    log_auto_accept_event("ready check detected");

    for (attempt, delay_ms) in READY_CHECK_AUTOMATION_RETRY_DELAYS_MS
        .iter()
        .copied()
        .chain(std::iter::once(0))
        .enumerate()
    {
        let attempt_number = attempt + 1;
        if !is_ready_check_active(reader)? {
            log_auto_accept_attempt(attempt_number, "skipped because phase moved before request");
            return Ok(());
        }

        log_auto_accept_attempt(attempt_number, "sending accept request");
        if let Err(error) = reader.accept_ready_check() {
            log_auto_accept_attempt(attempt_number, "accept request failed");
            if !is_ready_check_active(reader)? {
                log_auto_accept_attempt(
                    attempt_number,
                    "accept response was uncertain but phase moved",
                );
                return Ok(());
            }

            record_system_activity(
                store,
                "Lobby automation accept failed",
                format!("Auto-accept could not reach the League Client: {error}").as_str(),
            );
            return Err(error.into());
        }

        if !is_ready_check_active(reader)? {
            log_auto_accept_attempt(attempt_number, "verified phase moved after accept");
            return Ok(());
        }
        log_auto_accept_attempt(attempt_number, "phase still ReadyCheck after accept");

        if delay_ms > 0 {
            log_auto_accept_attempt(attempt_number, "waiting before retry verification");
            thread::sleep(Duration::from_millis(delay_ms));
        }

        if !is_ready_check_active(reader)? {
            log_auto_accept_attempt(attempt_number, "verified phase moved after delay");
            return Ok(());
        }
        log_auto_accept_attempt(attempt_number, "phase still ReadyCheck after delay");

        if attempt + 1 == READY_CHECK_AUTOMATION_RETRY_DELAYS_MS.len() + 1 {
            break;
        }
    }

    let message =
        "Auto-accept did not move the client out of ReadyCheck after multiple verification attempts"
            .to_string();
    record_system_activity(
        store,
        "Lobby automation requires manual accept",
        "Auto-accept retried, but the client stayed in ReadyCheck. Manual confirmation may still be needed.",
    );
    log_auto_accept_event("failed because phase stayed ReadyCheck after retries");
    Err(ApplicationError::Integration(message))
}

pub fn run_champ_select_automation(
    store: &impl AppStore,
    reader: &impl LeagueClientReader,
) -> Result<(), ApplicationError> {
    let settings = store.get_settings().map_err(ApplicationError::Storage)?;

    let pick = if settings.auto_pick_enabled { settings.auto_pick_champion_id } else { None };
    let ban = if settings.auto_ban_enabled { settings.auto_ban_champion_id } else { None };

    if pick.is_none() && ban.is_none() {
        return Ok(());
    }

    reader
        .apply_champ_select_preferences(pick, ban)
        .map_err(ApplicationError::from)
}

pub fn normalize_player_name(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn summoner_name_lookup_keys(value: &str) -> Vec<String> {
    let normalized = normalize_player_name(value);
    let mut keys = Vec::new();
    if !normalized.is_empty() {
        keys.push(normalized.clone());
    }

    if let Some((game_name, _)) = value.split_once('#') {
        let normalized_game_name = normalize_player_name(game_name);
        if !normalized_game_name.is_empty() && normalized_game_name != normalized {
            keys.push(normalized_game_name);
        }
    }

    keys
}

fn is_ready_check_active(reader: &impl LeagueClientReader) -> Result<bool, ApplicationError> {
    Ok(reader.gameflow_phase()?.as_str() == "ReadyCheck")
}

fn record_system_activity(store: &impl AppStore, title: &str, body: &str) {
    let _ = store.create_activity_entry(NewActivityEntry {
        kind: ActivityKind::System,
        title: title.to_string(),
        body: Some(body.to_string()),
    });
}

fn negative_stable_id(value: &str) -> i64 {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    -((hasher.finish() & 0x3fff_ffff_ffff) as i64) - 1
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
