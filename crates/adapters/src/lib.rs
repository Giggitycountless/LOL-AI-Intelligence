use std::{
    collections::{HashMap, HashSet}, fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use application::{
    ChampSelectSessionData, ChampSelectSessionPlayer,
    ChampSelectSessionSource, LeagueClientReadError, LeagueClientReader, SummonerBatchEntry,
};
use domain::{
    AdvisorNamedRef, ChampSelectTeam, CurrentSummonerProfile,
    LeagueChampionAbility, LeagueChampionDetails, LeagueChampionSummary, LeagueClientConnection,
    LeagueClientPhase, LeagueClientStatus, LeagueDataSection, LeagueDataWarning, LeagueGameAsset,
    LeagueGameAssetKind, LeagueImageAsset, LeagueSelfData, LiveOverlayActivePlayer,
    LiveOverlayEvent, LiveOverlayGoldSummary, LiveOverlayItem, LiveOverlayPlayer,
    LiveOverlayScores, LiveOverlaySnapshot, MatchResult, ParticipantRecentStats, RankedQueue,
    RankedQueueSummary, RecentMatchSummary, AbilityStat,
};
use percent_encoding::{AsciiSet, CONTROLS, percent_encode};
use rayon::prelude::*;
use reqwest::blocking::Client;
use serde_json::Value;
use sysinfo::{ProcessesToUpdate, System};

mod community_dragon;
mod constants;
use constants::*;

pub mod data_providers;
pub use data_providers::{RemoteRankedChampionJsonProvider, RemoteAdvisorJsonProvider, parse_advisor_snapshot_json, parse_ranked_champion_snapshot_json};
use data_providers::unix_timestamp_seconds;

mod game_client_types;
pub(crate) use game_client_types::*;

mod lcu_types;
pub(crate) use lcu_types::*;

pub fn layer_name() -> &'static str {
    "adapters"
}

pub(crate) fn log_lcu_adapter_event(message: &str) {
    eprintln!("[lcu-adapter] {message}");
}



fn map_ranked_queues(stats: LcuRankedStats) -> Vec<RankedQueueSummary> {
    stats
        .queues
        .into_iter()
        .filter_map(|queue| {
            let queue_type = queue.queue_type.as_deref()?;
            let queue_kind = match queue_type {
                "RANKED_SOLO_5x5" => RankedQueue::SoloDuo,
                "RANKED_FLEX_SR" => RankedQueue::Flex,
                _ => RankedQueue::Other,
            };
            let tier = queue.tier.and_then(non_empty_owned);
            let division = queue.division.and_then(non_empty_owned);
            let is_ranked = tier
                .as_deref()
                .is_some_and(|value| value != "NONE" && value != "UNRANKED");

            Some(RankedQueueSummary {
                queue: queue_kind,
                tier,
                division,
                league_points: queue.league_points,
                wins: queue.wins.unwrap_or_default(),
                losses: queue.losses.unwrap_or_default(),
                is_ranked,
            })
        })
        .collect()
}
#[derive(Debug, Default)]
pub struct LocalLeagueClient {
    lockfile_override: Option<PathBuf>,
    community_dragon: community_dragon::CommunityDragonClient,
    cached_credentials: Arc<Mutex<Option<(Instant, LockfileCredentials)>>>,
    session_cache: Arc<Mutex<Option<LcuSession>>>,
}

impl Clone for LocalLeagueClient {
    fn clone(&self) -> Self {
        Self {
            lockfile_override: self.lockfile_override.clone(),
            community_dragon: self.community_dragon.clone(),
            cached_credentials: self.cached_credentials.clone(),
            session_cache: self.session_cache.clone(),
        }
    }
}

impl LocalLeagueClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_lockfile_path(path: impl Into<PathBuf>) -> Self {
        Self {
            lockfile_override: Some(path.into()),
            community_dragon: community_dragon::CommunityDragonClient::default(),
            cached_credentials: Arc::default(),
            session_cache: Arc::default(),
        }
    }

    pub fn gameflow_phase(&self) -> Result<String, LeagueClientReadError> {
        let session = match self.open_session() {
            SessionOpenResult::Ready(session) => session,
            SessionOpenResult::Status(status) => return Err(read_error_from_status(status)),
        };

        session
            .get_json::<String>("/lol-gameflow/v1/gameflow-phase")
            .map_err(read_error_from_request)
    }

    pub async fn open_websocket(&self) -> Result<LcuWebSocketClient, LcuWebSocketError> {
        let credentials = self
            .read_lockfile_credentials()
            .map_err(|_| LcuWebSocketError::Unavailable)?;
        log_lcu_adapter_event("websocket connecting");
        match LcuWebSocketClient::connect(credentials).await {
            Ok(client) => {
                log_lcu_adapter_event("websocket connected");
                Ok(client)
            }
            Err(error) => {
                log_lcu_adapter_event("websocket connection failed");
                Err(error)
            }
        }
    }

    fn read_status(&self) -> LeagueClientStatus {
        let session = match self.open_session() {
            SessionOpenResult::Ready(session) => session,
            SessionOpenResult::Status(status) => return status,
        };

        match session.get_json::<LcuSummoner>("/lol-summoner/v1/current-summoner") {
            Ok(_) => connected_status(),
            Err(error) => status_from_request_error(error),
        }
    }

    fn read_self_data(&self, match_limit: i64) -> LeagueSelfData {
        let session = match self.open_session() {
            SessionOpenResult::Ready(session) => session,
            SessionOpenResult::Status(status) => return empty_self_data(status),
        };

        let summoner = match session.get_json::<LcuSummoner>("/lol-summoner/v1/current-summoner") {
            Ok(value) => value,
            Err(error) => return empty_self_data(status_from_request_error(error)),
        };

        build_self_data(session, summoner, match_limit)
    }

    fn read_profile_icon(
        &self,
        profile_icon_id: i64,
    ) -> Result<LeagueImageAsset, LeagueClientReadError> {
        let session = match self.open_session() {
            SessionOpenResult::Ready(session) => session,
            SessionOpenResult::Status(status) => return Err(read_error_from_status(status)),
        };

        session
            .get_image_asset(
                profile_icon_path(profile_icon_id).as_str(),
                PROFILE_ICON_MIME,
            )
            .map_err(read_error_from_request)
    }

    fn read_champion_icon(
        &self,
        champion_id: i64,
    ) -> Result<LeagueImageAsset, LeagueClientReadError> {
        let session = match self.open_session() {
            SessionOpenResult::Ready(session) => session,
            SessionOpenResult::Status(status) => return Err(read_error_from_status(status)),
        };

        session
            .get_image_asset(champion_icon_path(champion_id).as_str(), CHAMPION_ICON_MIME)
            .map_err(read_error_from_request)
    }

    fn read_champion_details(
        &self,
        champion_id: i64,
    ) -> Result<LeagueChampionDetails, LeagueClientReadError> {
        let session = match self.open_session() {
            SessionOpenResult::Ready(session) => session,
            SessionOpenResult::Status(status) => return Err(read_error_from_status(status)),
        };
        let details = session
            .get_json::<LcuChampionDetails>(champion_details_path(champion_id).as_str())
            .map_err(read_error_from_request)?;

        let bin_data = details
            .name
            .as_deref()
            .and_then(|name| self.community_dragon.fetch_champion_bin(name));

        map_champion_details(&session, champion_id, details, bin_data.as_ref())
    }

    fn read_game_asset(
        &self,
        kind: LeagueGameAssetKind,
        asset_id: i64,
    ) -> Result<LeagueGameAsset, LeagueClientReadError> {
        let session = match self.open_session() {
            SessionOpenResult::Ready(session) => session,
            SessionOpenResult::Status(status) => return Err(read_error_from_status(status)),
        };

        let metadata = session
            .get_json::<Value>(game_asset_metadata_path(kind))
            .ok()
            .and_then(|value| find_game_asset_metadata(value, asset_id));
        let image_path =
            game_asset_image_path(kind, asset_id, metadata.as_ref()).ok_or_else(|| {
                LeagueClientReadError::Integration(
                    "League game asset icon is unavailable from local game data".to_string(),
                )
            })?;
        let image = session
            .get_image_asset(image_path.as_str(), GAME_ASSET_MIME)
            .map_err(read_error_from_request)?;

        Ok(LeagueGameAsset {
            kind,
            asset_id,
            name: game_asset_name(kind, asset_id, metadata.as_ref()),
            description: game_asset_description(metadata.as_ref()),
            image,
        })
    }

    fn read_completed_match(
        &self,
        game_id: i64,
    ) -> Result<application::LeagueCompletedMatch, LeagueClientReadError> {
        let session = match self.open_session() {
            SessionOpenResult::Ready(session) => session,
            SessionOpenResult::Status(status) => return Err(read_error_from_status(status)),
        };
        let summoner = session
            .get_json::<LcuSummoner>("/lol-summoner/v1/current-summoner")
            .map_err(read_error_from_request)?;
        let champion_names = session
            .get_json::<Vec<LcuChampionSummary>>("/lol-game-data/assets/v1/champion-summary.json")
            .map(champion_name_map)
            .unwrap_or_default();
        let history = session
            .get_json::<LcuMatchHistoryResponse>(
                current_matches_path(MAX_COMPLETED_MATCH_SCAN).as_str(),
            )
            .map_err(read_error_from_request)?;
        let summary_match = history
            .games
            .and_then(|games| find_completed_game(games.games, game_id))
            .and_then(|game| map_completed_match(game, &summoner, &champion_names))
            .ok_or_else(|| {
                LeagueClientReadError::Integration(
                    "Completed match was not found in current user's recent history".to_string(),
                )
            })?;

        let detail_match = session
            .get_json::<LcuGame>(completed_match_path(game_id).as_str())
            .ok()
            .and_then(|game| map_completed_match(game, &summoner, &champion_names));

        Ok(best_completed_match(summary_match, detail_match))
    }

    fn read_participant_recent_stats(
        &self,
        player_puuid: &str,
        limit: i64,
    ) -> Result<ParticipantRecentStats, LeagueClientReadError> {
        if !is_safe_lcu_path_id(player_puuid) {
            return Err(LeagueClientReadError::Integration(
                "Participant identity could not be used for local profile lookup".to_string(),
            ));
        }

        let session = match self.open_session() {
            SessionOpenResult::Ready(session) => session,
            SessionOpenResult::Status(status) => return Err(read_error_from_status(status)),
        };
        let champion_names = session
            .get_json::<Vec<LcuChampionSummary>>("/lol-game-data/assets/v1/champion-summary.json")
            .map(champion_name_map)
            .unwrap_or_default();

        read_participant_recent_stats_with_session(&session, player_puuid, limit, &champion_names)
    }

    fn read_participant_recent_stats_batch(
        &self,
        player_puuids: &[String],
        limit: i64,
    ) -> HashMap<String, Result<ParticipantRecentStats, LeagueClientReadError>> {
        let mut results = HashMap::new();
        let mut valid_puuids = Vec::new();

        for player_puuid in player_puuids {
            if !is_safe_lcu_path_id(player_puuid) {
                results.insert(
                    player_puuid.clone(),
                    Err(LeagueClientReadError::Integration(
                        "Participant identity could not be used for local profile lookup"
                            .to_string(),
                    )),
                );
                continue;
            }

            if !valid_puuids.contains(player_puuid) {
                valid_puuids.push(player_puuid.clone());
            }
        }

        if valid_puuids.is_empty() {
            return results;
        }

        let session = match self.open_session() {
            SessionOpenResult::Ready(session) => session,
            SessionOpenResult::Status(status) => {
                let error = read_error_from_status(status);
                for player_puuid in valid_puuids {
                    results.insert(player_puuid, Err(error.clone()));
                }
                return results;
            }
        };
        let champion_names = session
            .get_json::<Vec<LcuChampionSummary>>("/lol-game-data/assets/v1/champion-summary.json")
            .map(champion_name_map)
            .unwrap_or_default();

        results.extend(
            valid_puuids
                .par_iter()
                .map(|player_puuid| {
                    (
                        player_puuid.clone(),
                        read_participant_recent_stats_with_session(
                            &session,
                            player_puuid,
                            limit,
                            &champion_names,
                        ),
                    )
                })
                .collect::<HashMap<_, _>>(),
        );

        results
    }

    fn read_live_client_session(&self) -> Result<ChampSelectSessionData, LeagueClientReadError> {
        let http_client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(REQUEST_TIMEOUT)
            .no_proxy()
            .tls_danger_accept_invalid_certs(true)
            .build()
            .map_err(|_| {
                LeagueClientReadError::Integration(
                    "Live Client local connection could not be prepared".to_string(),
                )
            })?;
        let players = live_client_get_json::<Vec<GameClientPlayer>>(
            &http_client,
            "/liveclientdata/playerlist",
        )?;
        let players = enrich_live_overlay_players(&http_client, players);
        let active_player = live_client_get_json::<GameClientActivePlayer>(
            &http_client,
            "/liveclientdata/activeplayer",
        )
        .ok();
        let active_team = active_player
            .as_ref()
            .and_then(|active| {
                let active_name = normalize_player_name(active.summoner_name.as_str());
                players.iter().find_map(|player| {
                    if player
                        .summoner_name
                        .as_deref()
                        .is_some_and(|name| normalize_player_name(name) == active_name)
                    {
                        Some(player.team.clone())
                    } else {
                        None
                    }
                })
            })
            .or_else(|| players.first().map(|player| player.team.clone()))
            .unwrap_or_else(|| "ORDER".to_string());
        let mut ally_names = Vec::new();
        let mut enemy_names = Vec::new();

        for player in players {
            let Some(display_name) = player.display_name() else {
                continue;
            };

            if strings_match(Some(active_team.as_str()), Some(player.team.as_str())) {
                ally_names.push(display_name);
            } else {
                enemy_names.push(display_name);
            }
        }

        Ok(ChampSelectSessionData {
            ally_ids: Vec::new(),
            enemy_ids: Vec::new(),
            champion_selections: HashMap::new(),
            ally_names,
            enemy_names,
            champion_selections_by_name: HashMap::new(),
            source: ChampSelectSessionSource::LiveClient,
            players: Vec::new(),
        })
    }

    fn read_live_overlay_snapshot(&self) -> Result<LiveOverlaySnapshot, LeagueClientReadError> {
        let http_client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(REQUEST_TIMEOUT)
            .no_proxy()
            .tls_danger_accept_invalid_certs(true)
            .build()
            .map_err(|_| {
                LeagueClientReadError::Integration(
                    "Live Client local connection could not be prepared".to_string(),
                )
            })?;
        let players = live_client_get_json::<Vec<GameClientPlayer>>(
            &http_client,
            "/liveclientdata/playerlist",
        )?;
        let active_player = live_client_get_json::<GameClientActivePlayer>(
            &http_client,
            "/liveclientdata/activeplayer",
        )
        .ok();
        let events =
            live_client_get_json::<GameClientEventData>(&http_client, "/liveclientdata/eventdata")
                .map(|data| data.events)
                .unwrap_or_default();
        let game_stats =
            live_client_get_json::<GameClientStats>(&http_client, "/liveclientdata/gamestats").ok();

        Ok(map_live_overlay_snapshot(
            players,
            active_player,
            events,
            game_stats,
        ))
    }

    fn read_gameflow_session(
        &self,
        session: &LcuSession,
    ) -> Result<ChampSelectSessionData, LeagueClientReadError> {
        let gameflow = session
            .get_json::<LcuGameflowSession>("/lol-gameflow/v1/session")
            .map_err(read_error_from_request)?;

        map_gameflow_session(gameflow).ok_or_else(|| {
            LeagueClientReadError::Integration(
                "League gameflow session did not include player identities".to_string(),
            )
        })
    }

    fn open_session(&self) -> SessionOpenResult {
        // Try cached session first
        if let Some(session) = self.session_cache.lock().unwrap().as_ref() {
            let alive = session
                .get_json::<LcuSummoner>("/lol-summoner/v1/current-summoner")
                .is_ok();
            if alive {
                return SessionOpenResult::Ready(session.clone());
            }
        }

        let credentials = match self.read_lockfile_credentials() {
            Ok(credentials) => credentials,
            Err(status) => return SessionOpenResult::Status(status),
        };

        match LcuSession::new(credentials) {
            Ok(session) => {
                log_lcu_adapter_event("local session created");
                *self.session_cache.lock().unwrap() = Some(session.clone());
                SessionOpenResult::Ready(session)
            }
            Err(_) => {
                log_lcu_adapter_event("local session creation failed");
                *self.session_cache.lock().unwrap() = None;
                SessionOpenResult::Status(unavailable_status(
                    true,
                    true,
                    LeagueClientPhase::Unavailable,
                    "League Client local connection could not be prepared",
                ))
            }
        }
    }

    fn read_lockfile_credentials(&self) -> Result<LockfileCredentials, LeagueClientStatus> {
        // Use cached credentials for up to 2 seconds to avoid repeated process scans.
        {
            let cache = self.cached_credentials.lock().unwrap();
            if let Some((cached_at, cached)) = cache.as_ref()
                && cached_at.elapsed() < Duration::from_secs(2)
            {
                return Ok(cached.clone());
            }
        }

        let lockfile_path = match self.discover_lockfile_path() {
            LockfileDiscovery::Found(path) => {
                log_lcu_adapter_event("lockfile discovered");
                path
            }
            LockfileDiscovery::NotRunning => {
                log_lcu_adapter_event("league client process not detected");
                *self.cached_credentials.lock().unwrap() = None;
                return Err(unavailable_status(
                    false,
                    false,
                    LeagueClientPhase::NotRunning,
                    "League Client is not running",
                ));
            }
            LockfileDiscovery::LockfileMissing => {
                log_lcu_adapter_event("league client process detected without lockfile");
                *self.cached_credentials.lock().unwrap() = None;
                return Err(unavailable_status(
                    true,
                    false,
                    LeagueClientPhase::LockfileMissing,
                    "League Client is running, but its lockfile was not found",
                ));
            }
        };

        let lockfile_contents = match fs::read_to_string(&lockfile_path) {
            Ok(contents) => {
                log_lcu_adapter_event("lockfile read");
                contents
            }
            Err(_) => {
                log_lcu_adapter_event("lockfile read failed");
                return Err(unavailable_status(
                    true,
                    true,
                    LeagueClientPhase::Unavailable,
                    "League Client lockfile could not be read",
                ));
            }
        };

        match parse_lockfile(lockfile_contents.as_str()) {
            Ok(credentials) => {
                log_lcu_adapter_event("lockfile parsed");
                *self.cached_credentials.lock().unwrap() =
                    Some((Instant::now(), credentials.clone()));
                Ok(credentials)
            }
            Err(_) => {
                log_lcu_adapter_event("lockfile parse failed");
                Err(unavailable_status(
                    true,
                    true,
                    LeagueClientPhase::Unavailable,
                    "League Client lockfile could not be parsed",
                ))
            }
        }
    }

    fn discover_lockfile_path(&self) -> LockfileDiscovery {
        if let Some(path) = &self.lockfile_override {
            return if path.exists() {
                LockfileDiscovery::Found(path.clone())
            } else {
                LockfileDiscovery::LockfileMissing
            };
        }

        let mut system = System::new();
        system.refresh_processes(ProcessesToUpdate::All, true);

        let mut client_is_running = false;
        for process in system.processes().values() {
            let process_name = process.name().to_string_lossy();

            if !is_league_client_process(process_name.as_ref()) {
                continue;
            }

            client_is_running = true;
            let Some(executable_path) = process.exe() else {
                continue;
            };
            let Some(client_dir) = executable_path.parent() else {
                continue;
            };

            let lockfile_path = client_dir.join("lockfile");
            if lockfile_path.exists() {
                return LockfileDiscovery::Found(lockfile_path);
            }
        }

        if client_is_running {
            LockfileDiscovery::LockfileMissing
        } else {
            LockfileDiscovery::NotRunning
        }
    }
}

impl LeagueClientReader for LocalLeagueClient {
    fn status(&self) -> Result<LeagueClientStatus, LeagueClientReadError> {
        Ok(self.read_status())
    }

    fn gameflow_phase(&self) -> Result<String, LeagueClientReadError> {
        LocalLeagueClient::gameflow_phase(self)
    }

    fn live_overlay(&self) -> Result<LiveOverlaySnapshot, LeagueClientReadError> {
        self.read_live_overlay_snapshot()
    }

    fn self_data(&self, match_limit: i64) -> Result<LeagueSelfData, LeagueClientReadError> {
        Ok(self.read_self_data(match_limit))
    }

    fn profile_icon(
        &self,
        profile_icon_id: i64,
    ) -> Result<LeagueImageAsset, LeagueClientReadError> {
        self.read_profile_icon(profile_icon_id)
    }

    fn champion_icon(&self, champion_id: i64) -> Result<LeagueImageAsset, LeagueClientReadError> {
        self.read_champion_icon(champion_id)
    }

    fn game_asset(
        &self,
        kind: LeagueGameAssetKind,
        asset_id: i64,
    ) -> Result<LeagueGameAsset, LeagueClientReadError> {
        self.read_game_asset(kind, asset_id)
    }

    fn completed_match(
        &self,
        game_id: i64,
    ) -> Result<application::LeagueCompletedMatch, LeagueClientReadError> {
        self.read_completed_match(game_id)
    }

    fn participant_recent_stats(
        &self,
        player_puuid: &str,
        limit: i64,
    ) -> Result<ParticipantRecentStats, LeagueClientReadError> {
        self.read_participant_recent_stats(player_puuid, limit)
    }

    fn participant_recent_stats_batch(
        &self,
        player_puuids: &[String],
        limit: i64,
    ) -> HashMap<String, Result<ParticipantRecentStats, LeagueClientReadError>> {
        self.read_participant_recent_stats_batch(player_puuids, limit)
    }

    fn champ_select_session(&self) -> Result<ChampSelectSessionData, LeagueClientReadError> {
        let session = match self.open_session() {
            SessionOpenResult::Ready(session) => session,
            SessionOpenResult::Status(status) => return Err(read_error_from_status(status)),
        };
        let champ_select =
            match session.get_json::<LcuChampSelectSession>("/lol-champ-select/v1/session") {
                Ok(value) => value,
                Err(error) => {
                    if let Ok(session) = self.read_gameflow_session(&session) {
                        return Ok(session);
                    }
                    return self
                        .read_live_client_session()
                        .map_err(|_| read_error_from_request(error));
                }
            };

        let mut champion_selections = HashMap::new();
        let mut champion_selections_by_name = HashMap::new();
        for member in champ_select
            .my_team
            .iter()
            .chain(champ_select.their_team.iter())
        {
            if let (Some(sid), Some(cid)) = (member.summoner_id, member.champion_id)
                && cid > 0
            {
                champion_selections.insert(sid, cid);
            }
            if let (Some(name), Some(cid)) = (member.display_name(), member.champion_id)
                && cid > 0
            {
                champion_selections_by_name.insert(normalize_player_name(name.as_str()), cid);
            }
        }

        Ok(ChampSelectSessionData {
            ally_ids: champ_select
                .my_team
                .iter()
                .filter_map(|member| member.summoner_id)
                .filter(|id| *id > 0)
                .collect(),
            enemy_ids: champ_select
                .their_team
                .iter()
                .filter_map(|member| member.summoner_id)
                .filter(|id| *id > 0)
                .collect(),
            champion_selections,
            ally_names: champ_select
                .my_team
                .iter()
                .filter_map(LcuChampSelectMember::display_name)
                .collect(),
            enemy_names: champ_select
                .their_team
                .iter()
                .filter_map(LcuChampSelectMember::display_name)
                .collect(),
            champion_selections_by_name,
            source: ChampSelectSessionSource::ChampSelect,
            players: Vec::new(),
        })
    }

    fn summoners_by_ids(&self, ids: &[i64]) -> Vec<SummonerBatchEntry> {
        if ids.is_empty() {
            return Vec::new();
        }

        let session = match self.open_session() {
            SessionOpenResult::Ready(session) => session,
            SessionOpenResult::Status(_) => return Vec::new(),
        };
        let ids_str = ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let ids_json = format!("[{ids_str}]");

        for path in [
            format!("/lol-summoner/v2/summoners?ids={ids_json}"),
            format!("/lol-summoner/v1/summoners?ids={ids_json}"),
            format!("/lol-summoner/v1/summoners?ids={ids_str}"),
        ] {
            if let Ok(summoners) = session.get_json::<Vec<LcuSummonerBatch>>(path.as_str()) {
                let entries = map_summoner_batch_entries(summoners);
                if !entries.is_empty() {
                    return entries;
                }
            }
        }

        ids.iter()
            .filter_map(|id| {
                session
                    .get_json::<LcuSummonerBatch>(
                        format!("/lol-summoner/v1/summoners/{id}").as_str(),
                    )
                    .ok()
            })
            .filter_map(map_summoner_batch_entry)
            .collect()
    }

    fn summoners_by_names(&self, names: &[String]) -> Vec<SummonerBatchEntry> {
        if names.is_empty() {
            return Vec::new();
        }

        let session = match self.open_session() {
            SessionOpenResult::Ready(session) => session,
            SessionOpenResult::Status(_) => return Vec::new(),
        };
        let mut entries = Vec::new();
        let mut seen_ids = HashSet::new();

        for name in names {
            let trimmed = name.trim();
            if trimmed.is_empty() {
                continue;
            }
            let encoded_name = percent_encode_path_value(trimmed);
            let mut found = None;

            for path in [
                format!("/lol-summoner/v1/summoners?name={encoded_name}"),
                format!("/lol-summoner/v2/summoners?name={encoded_name}"),
            ] {
                if let Ok(summoner) = session.get_json::<LcuSummonerBatch>(path.as_str()) {
                    found = map_summoner_batch_entry(summoner);
                    if found.is_some() {
                        break;
                    }
                }
            }

            if let Some(entry) = found
                && seen_ids.insert(entry.summoner_id)
            {
                entries.push(entry);
            }
        }

        entries
    }

    fn champion_catalog(&self) -> Result<Vec<LeagueChampionSummary>, LeagueClientReadError> {
        let session = match self.open_session() {
            SessionOpenResult::Ready(session) => session,
            SessionOpenResult::Status(status) => return Err(read_error_from_status(status)),
        };

        session
            .get_json::<Vec<LcuChampionSummary>>("/lol-game-data/assets/v1/champion-summary.json")
            .map(map_champion_catalog)
            .map_err(read_error_from_request)
    }

    fn champion_details(
        &self,
        champion_id: i64,
    ) -> Result<LeagueChampionDetails, LeagueClientReadError> {
        self.read_champion_details(champion_id)
    }

    fn accept_ready_check(&self) -> Result<(), LeagueClientReadError> {
        let session = match self.open_session() {
            SessionOpenResult::Ready(session) => session,
            SessionOpenResult::Status(status) => return Err(read_error_from_status(status)),
        };

        session
            .post_empty("/lol-matchmaking/v1/ready-check/accept")
            .map_err(read_error_from_request)
    }

    fn apply_champ_select_preferences(
        &self,
        pick_champion_id: Option<i64>,
        ban_champion_id: Option<i64>,
    ) -> Result<(), LeagueClientReadError> {
        if pick_champion_id.is_none() && ban_champion_id.is_none() {
            return Ok(());
        }

        let session = match self.open_session() {
            SessionOpenResult::Ready(session) => session,
            SessionOpenResult::Status(status) => return Err(read_error_from_status(status)),
        };
        let champ_select = session
            .get_json::<LcuChampSelectSession>("/lol-champ-select/v1/session")
            .map_err(read_error_from_request)?;
        let Some(local_cell_id) = champ_select.local_player_cell_id else {
            return Ok(());
        };

        if let Some(champion_id) = ban_champion_id {
            apply_champ_select_action(&session, &champ_select, local_cell_id, "ban", champion_id)?;
        }

        if let Some(champion_id) = pick_champion_id {
            apply_champ_select_action(&session, &champ_select, local_cell_id, "pick", champion_id)?;
        }

        Ok(())
    }
}

enum SessionOpenResult {
    Ready(LcuSession),
    Status(LeagueClientStatus),
}

enum LockfileDiscovery {
    Found(PathBuf),
    NotRunning,
    LockfileMissing,
}

mod websocket;
pub use websocket::{LcuWebSocketEvent, LcuSubscription, LcuWebSocketError, LcuWebSocketClient, parse_lcu_websocket_event_text};
mod lockfile;
use lockfile::*;

mod session;
use session::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LcuAdapterError {
    InvalidLockfile,
    Http,
}

fn is_league_client_process(name: &str) -> bool {
    LEAGUE_CLIENT_PROCESSES
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(name))
}

fn connected_status() -> LeagueClientStatus {
    LeagueClientStatus {
        is_running: true,
        lockfile_found: true,
        connection: LeagueClientConnection::Connected,
        phase: LeagueClientPhase::Connected,
        message: None,
    }
}

fn unavailable_status(
    is_running: bool,
    lockfile_found: bool,
    phase: LeagueClientPhase,
    message: impl Into<String>,
) -> LeagueClientStatus {
    LeagueClientStatus {
        is_running,
        lockfile_found,
        connection: LeagueClientConnection::Unavailable,
        phase,
        message: Some(message.into()),
    }
}

fn status_from_request_error(error: LcuRequestError) -> LeagueClientStatus {
    match error {
        LcuRequestError::Unauthorized => unavailable_status(
            true,
            true,
            LeagueClientPhase::Unauthorized,
            "League Client rejected local authentication",
        ),
        LcuRequestError::NotLoggedIn => unavailable_status(
            true,
            true,
            LeagueClientPhase::NotLoggedIn,
            "League Client is not logged in yet",
        ),
        LcuRequestError::Patching => unavailable_status(
            true,
            true,
            LeagueClientPhase::Patching,
            "League Client local API is not ready",
        ),
        LcuRequestError::Unavailable => unavailable_status(
            true,
            true,
            LeagueClientPhase::Unavailable,
            "League Client local API is unavailable",
        ),
        LcuRequestError::Unexpected => unavailable_status(
            true,
            true,
            LeagueClientPhase::Unavailable,
            "League Client returned an unexpected response",
        ),
    }
}

fn read_error_from_status(status: LeagueClientStatus) -> LeagueClientReadError {
    let message = status
        .message
        .unwrap_or_else(|| "League Client data is unavailable".to_string());

    match status.phase {
        LeagueClientPhase::Unauthorized => LeagueClientReadError::ClientAccess(message),
        LeagueClientPhase::Connected | LeagueClientPhase::PartialData => {
            LeagueClientReadError::Integration(message)
        }
        LeagueClientPhase::NotRunning
        | LeagueClientPhase::LockfileMissing
        | LeagueClientPhase::Connecting
        | LeagueClientPhase::NotLoggedIn
        | LeagueClientPhase::Patching
        | LeagueClientPhase::Unavailable => LeagueClientReadError::ClientUnavailable(message),
    }
}

fn read_error_from_request(error: LcuRequestError) -> LeagueClientReadError {
    let status = status_from_request_error(error);

    read_error_from_status(status)
}

fn empty_self_data(status: LeagueClientStatus) -> LeagueSelfData {
    LeagueSelfData {
        status,
        summoner: None,
        ranked_queues: Vec::new(),
        recent_matches: Vec::new(),
        data_warnings: Vec::new(),
    }
}

fn build_self_data(session: LcuSession, summoner: LcuSummoner, match_limit: i64) -> LeagueSelfData {
    let champion_names_result = session
        .get_json::<Vec<LcuChampionSummary>>("/lol-game-data/assets/v1/champion-summary.json");
    let ranked_result = session.get_json::<LcuRankedStats>("/lol-ranked/v1/current-ranked-stats");
    let matches_path = current_matches_path(match_limit);
    let matches_result = session.get_json::<LcuMatchHistoryResponse>(matches_path.as_str());

    compose_self_data(
        summoner,
        champion_names_result,
        ranked_result,
        matches_result,
    )
}

fn compose_self_data(
    summoner: LcuSummoner,
    champion_names_result: Result<Vec<LcuChampionSummary>, LcuRequestError>,
    ranked_result: Result<LcuRankedStats, LcuRequestError>,
    matches_result: Result<LcuMatchHistoryResponse, LcuRequestError>,
) -> LeagueSelfData {
    let mut warnings = Vec::new();
    let champion_names = match champion_names_result {
        Ok(champions) => champion_name_map(champions),
        Err(error) => {
            warnings.push(data_warning(
                LeagueDataSection::Champions,
                section_error_message("Champion names", error),
            ));
            HashMap::new()
        }
    };
    let ranked_queues = match ranked_result {
        Ok(stats) => map_ranked_queues(stats),
        Err(error) => {
            warnings.push(data_warning(
                LeagueDataSection::Ranked,
                section_error_message("Ranked data", error),
            ));
            Vec::new()
        }
    };
    let recent_matches = match matches_result {
        Ok(history) => map_recent_matches(history, &summoner, &champion_names),
        Err(error) => {
            warnings.push(data_warning(
                LeagueDataSection::Matches,
                section_error_message("Recent matches", error),
            ));
            Vec::new()
        }
    };

    let status = if warnings.is_empty() {
        connected_status()
    } else {
        partial_data_status()
    };

    LeagueSelfData {
        status,
        summoner: Some(summoner.profile()),
        ranked_queues,
        recent_matches,
        data_warnings: warnings,
    }
}

fn partial_data_status() -> LeagueClientStatus {
    LeagueClientStatus {
        is_running: true,
        lockfile_found: true,
        connection: LeagueClientConnection::Connected,
        phase: LeagueClientPhase::PartialData,
        message: Some("League Client connected with partial data".to_string()),
    }
}

fn data_warning(section: LeagueDataSection, message: impl Into<String>) -> LeagueDataWarning {
    LeagueDataWarning {
        section,
        message: message.into(),
    }
}

fn section_error_message(section_name: &str, error: LcuRequestError) -> String {
    match error {
        LcuRequestError::Unauthorized => format!("{section_name} could not be read"),
        LcuRequestError::NotLoggedIn => format!("{section_name} is unavailable before login"),
        LcuRequestError::Patching => {
            format!("{section_name} is unavailable while the client is preparing")
        }
        LcuRequestError::Unavailable => format!("{section_name} is temporarily unavailable"),
        LcuRequestError::Unexpected => format!("{section_name} returned an unexpected response"),
    }
}



impl LcuChampSelectMember {
    fn display_name(&self) -> Option<String> {
        match (
            non_empty(self.game_name.as_deref()),
            non_empty(self.tag_line.as_deref()),
        ) {
            (Some(game_name), Some(tag_line)) => Some(format!("{game_name}#{tag_line}")),
            (Some(game_name), None) => Some(game_name.to_string()),
            _ => non_empty(self.display_name.as_deref())
                .or_else(|| non_empty(self.summoner_name.as_deref()))
                .map(str::to_string),
        }
    }
}

impl GameClientPlayer {
    fn display_name(&self) -> Option<String> {
        non_empty(self.riot_id.as_deref())
            .or_else(|| non_empty(self.summoner_name.as_deref()))
            .map(str::to_string)
    }
}

const QUERY_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'`')
    .add(b'?')
    .add(b'{')
    .add(b'}')
    .add(b'/')
    .add(b':')
    .add(b'@')
    .add(b'\\')
    .add(b'$')
    .add(b'&')
    .add(b'+')
    .add(b',')
    .add(b'=')
    .add(b'%');

fn enrich_live_overlay_players(
    http_client: &Client,
    mut players: Vec<GameClientPlayer>,
) -> Vec<GameClientPlayer> {
    players.par_iter_mut().for_each(|player| {
        let Some(display_name) = player.display_name() else {
            return;
        };
        let encoded_name = percent_encode(display_name.as_bytes(), QUERY_ENCODE_SET).to_string();

        match live_client_get_json::<Vec<GameClientItem>>(
            http_client,
            format!("/liveclientdata/playeritems?summonerName={encoded_name}").as_str(),
        ) {
            Ok(items) if !items.is_empty() => player.items = items,
            Err(e) => {
                log_lcu_adapter_event(&format!("failed to fetch items for {display_name}: {e}"))
            }
            _ => {}
        }

        match live_client_get_json::<GameClientScores>(
            http_client,
            format!("/liveclientdata/playerscores?summonerName={encoded_name}").as_str(),
        ) {
            Ok(scores) => player.scores = Some(scores),
            Err(e) => {
                log_lcu_adapter_event(&format!("failed to fetch scores for {display_name}: {e}"))
            }
        }

        match live_client_get_json::<GameClientSummonerSpells>(
            http_client,
            format!("/liveclientdata/playersummonerspells?summonerName={encoded_name}").as_str(),
        ) {
            Ok(spells) => player.summoner_spells = Some(spells),
            Err(e) => {
                log_lcu_adapter_event(&format!("failed to fetch spells for {display_name}: {e}"))
            }
        }
    });

    players
}

fn map_live_overlay_snapshot(
    players: Vec<GameClientPlayer>,
    active_player: Option<GameClientActivePlayer>,
    events: Vec<GameClientEvent>,
    game_stats: Option<GameClientStats>,
) -> LiveOverlaySnapshot {
    let active_team = active_player
        .as_ref()
        .and_then(|active| {
            let active_name = normalize_player_name(active.summoner_name.as_str());
            players.iter().find_map(|player| {
                let is_active = player
                    .display_name()
                    .is_some_and(|name| normalize_player_name(name.as_str()) == active_name)
                    || player
                        .summoner_name
                        .as_deref()
                        .is_some_and(|name| normalize_player_name(name) == active_name);
                if is_active {
                    Some(player.team.clone())
                } else {
                    None
                }
            })
        })
        .or_else(|| players.first().map(|player| player.team.clone()));

    let mut ally_item_value = 0;
    let mut enemy_item_value = 0;
    let mapped_players: Vec<LiveOverlayPlayer> = players
        .into_iter()
        .filter_map(|player| {
            let display_name = player.display_name()?;
            let items: Vec<LiveOverlayItem> = player
                .items
                .into_iter()
                .filter_map(map_live_overlay_item)
                .collect();
            let item_value: i64 = items
                .iter()
                .map(|item| item.price.saturating_mul(item.count.max(1)))
                .sum();
            if active_team
                .as_deref()
                .is_some_and(|team| strings_match(Some(team), Some(player.team.as_str())))
            {
                ally_item_value += item_value;
            } else {
                enemy_item_value += item_value;
            }
            let summoner_spells = player
                .summoner_spells
                .map(map_live_overlay_summoner_spells)
                .unwrap_or_default();
            Some(LiveOverlayPlayer {
                display_name,
                champion_name: player.champion_name.and_then(non_empty_owned),
                team: player.team,
                level: player.level,
                position: player.position.and_then(non_empty_owned),
                is_dead: player.is_dead,
                respawn_timer: player.respawn_timer,
                items,
                scores: player.scores.map(|scores| LiveOverlayScores {
                    kills: scores.kills.unwrap_or_default(),
                    deaths: scores.deaths.unwrap_or_default(),
                    assists: scores.assists.unwrap_or_default(),
                    creep_score: scores.creep_score.unwrap_or_default(),
                    ward_score: scores.ward_score.unwrap_or_default(),
                }),
                summoner_spells,
            })
        })
        .collect();
    LiveOverlaySnapshot {
        game_time_seconds: game_stats.as_ref().and_then(|stats| stats.game_time),
        game_mode: game_stats
            .as_ref()
            .and_then(|stats| stats.game_mode.clone().and_then(non_empty_owned)),
        map_name: game_stats
            .as_ref()
            .and_then(|stats| stats.map_name.clone().and_then(non_empty_owned)),
        active_player: active_player.map(|active| LiveOverlayActivePlayer {
            display_name: active.summoner_name,
            level: active.level,
            current_gold: active.current_gold,
            resource_type: active
                .champion_stats
                .as_ref()
                .and_then(|stats| stats.resource_type.clone()),
            resource_value: active
                .champion_stats
                .as_ref()
                .and_then(|stats| stats.resource_value),
            resource_max: active
                .champion_stats
                .as_ref()
                .and_then(|stats| stats.resource_max),
        }),
        players: mapped_players,
        events: events
            .into_iter()
            .filter_map(|event| {
                let event_name = event.event_name.and_then(non_empty_owned)?;
                Some(LiveOverlayEvent {
                    event_id: event.event_id.unwrap_or_default(),
                    event_name,
                    event_time: event.event_time.unwrap_or_default(),
                    actor: event.killer_name.and_then(non_empty_owned),
                    victim: event.victim_name.and_then(non_empty_owned),
                    assisting_participants: event
                        .assisters
                        .into_iter()
                        .filter_map(non_empty_owned)
                        .collect(),
                })
            })
            .collect(),
        gold: LiveOverlayGoldSummary {
            ally_item_value,
            enemy_item_value,
            item_value_diff: ally_item_value - enemy_item_value,
        },
        refreshed_at: unix_timestamp_seconds(),
    }
}

fn map_live_overlay_item(item: GameClientItem) -> Option<LiveOverlayItem> {
    let item_id = item.item_id?;
    if item_id <= 0 {
        return None;
    }

    Some(LiveOverlayItem {
        item_id,
        display_name: item
            .display_name
            .and_then(non_empty_owned)
            .unwrap_or_else(|| format!("Item {item_id}")),
        price: item.price.unwrap_or_default().max(0),
        count: item.count.unwrap_or(1).max(1),
        slot: item.slot,
    })
}

fn map_live_overlay_summoner_spells(spells: GameClientSummonerSpells) -> Vec<AdvisorNamedRef> {
    [spells.summoner_spell_one, spells.summoner_spell_two]
        .into_iter()
        .flatten()
        .filter_map(|spell| {
            let name = spell.display_name.and_then(non_empty_owned)?;
            Some(AdvisorNamedRef { id: spell.id, name })
        })
        .collect()
}

fn map_gameflow_session(session: LcuGameflowSession) -> Option<ChampSelectSessionData> {
    let data = session.game_data?;
    let mut ally_ids = Vec::new();
    let mut enemy_ids = Vec::new();
    let mut ally_names = Vec::new();
    let mut enemy_names = Vec::new();
    let mut champion_selections = HashMap::new();
    let mut champion_selections_by_name = HashMap::new();
    let mut players = Vec::new();

    collect_gameflow_team(
        &data.team_one,
        ChampSelectTeam::Ally,
        &data.player_champion_selections,
        &mut ally_ids,
        &mut ally_names,
        &mut champion_selections,
        &mut champion_selections_by_name,
        &mut players,
    );
    collect_gameflow_team(
        &data.team_two,
        ChampSelectTeam::Enemy,
        &data.player_champion_selections,
        &mut enemy_ids,
        &mut enemy_names,
        &mut champion_selections,
        &mut champion_selections_by_name,
        &mut players,
    );

    if players.is_empty() {
        for (index, selection) in data.player_champion_selections.iter().enumerate() {
            let team = match selection.team_id {
                Some(100) => ChampSelectTeam::Ally,
                Some(200) => ChampSelectTeam::Enemy,
                _ if index < 5 => ChampSelectTeam::Ally,
                _ => ChampSelectTeam::Enemy,
            };
            let display_name = selection
                .summoner_id
                .map(|id| format!("Summoner {id}"))
                .unwrap_or_else(|| format!("Player {}", index + 1));
            let champion_id = positive_id(selection.champion_id.or(selection.selected_champion_id));

            if let Some(summoner_id) = positive_id(selection.summoner_id) {
                match team {
                    ChampSelectTeam::Ally => ally_ids.push(summoner_id),
                    ChampSelectTeam::Enemy => enemy_ids.push(summoner_id),
                }
                if let Some(champion_id) = champion_id {
                    champion_selections.insert(summoner_id, champion_id);
                }
            }

            match team {
                ChampSelectTeam::Ally => ally_names.push(display_name.clone()),
                ChampSelectTeam::Enemy => enemy_names.push(display_name.clone()),
            }
            players.push(ChampSelectSessionPlayer {
                summoner_id: positive_id(selection.summoner_id),
                puuid: non_empty(selection.puuid.as_deref()).map(str::to_string),
                display_name,
                champion_id,
                team,
            });
        }
    }

    if players.is_empty() {
        return None;
    }

    Some(ChampSelectSessionData {
        ally_ids,
        enemy_ids,
        champion_selections,
        ally_names,
        enemy_names,
        champion_selections_by_name,
        source: ChampSelectSessionSource::GameflowSession,
        players,
    })
}

#[allow(clippy::too_many_arguments)]
fn collect_gameflow_team(
    team_players: &[LcuGameflowParticipant],
    team: ChampSelectTeam,
    selections: &[LcuGameflowChampionSelection],
    team_ids: &mut Vec<i64>,
    team_names: &mut Vec<String>,
    champion_selections: &mut HashMap<i64, i64>,
    champion_selections_by_name: &mut HashMap<String, i64>,
    players: &mut Vec<ChampSelectSessionPlayer>,
) {
    for (index, participant) in team_players.iter().enumerate() {
        let display_name = gameflow_participant_display_name(participant).unwrap_or_else(|| {
            participant
                .summoner_id
                .map(|id| format!("Summoner {id}"))
                .unwrap_or_else(|| format!("Player {}", index + 1))
        });
        let champion_id = gameflow_participant_champion_id(participant, selections);
        let summoner_id = positive_id(participant.summoner_id);

        if let Some(summoner_id) = summoner_id {
            team_ids.push(summoner_id);
            if let Some(champion_id) = champion_id {
                champion_selections.insert(summoner_id, champion_id);
            }
        }
        if let Some(champion_id) = champion_id {
            champion_selections_by_name
                .insert(normalize_player_name(display_name.as_str()), champion_id);
        }
        team_names.push(display_name.clone());
        players.push(ChampSelectSessionPlayer {
            summoner_id,
            puuid: non_empty(participant.puuid.as_deref()).map(str::to_string),
            display_name,
            champion_id,
            team: team.clone(),
        });
    }
}

fn gameflow_participant_display_name(participant: &LcuGameflowParticipant) -> Option<String> {
    match (
        non_empty(participant.game_name.as_deref()),
        non_empty(participant.tag_line.as_deref()),
    ) {
        (Some(game_name), Some(tag_line)) => Some(format!("{game_name}#{tag_line}")),
        (Some(game_name), None) => Some(game_name.to_string()),
        _ => non_empty(participant.display_name.as_deref())
            .or_else(|| non_empty(participant.summoner_name.as_deref()))
            .map(str::to_string),
    }
}

fn gameflow_participant_champion_id(
    participant: &LcuGameflowParticipant,
    selections: &[LcuGameflowChampionSelection],
) -> Option<i64> {
    positive_id(participant.champion_id.or(participant.selected_champion_id)).or_else(|| {
        selections.iter().find_map(|selection| {
            let same_puuid = non_empty(participant.puuid.as_deref())
                .is_some_and(|puuid| strings_match(Some(puuid), selection.puuid.as_deref()));
            let same_summoner = ids_match(participant.summoner_id, selection.summoner_id);

            if same_puuid || same_summoner {
                positive_id(selection.champion_id.or(selection.selected_champion_id))
            } else {
                None
            }
        })
    })
}

fn positive_id(value: Option<i64>) -> Option<i64> {
    value.filter(|inner| *inner > 0)
}

fn apply_champ_select_action(
    session: &LcuSession,
    champ_select: &LcuChampSelectSession,
    local_cell_id: i64,
    action_type: &str,
    champion_id: i64,
) -> Result<(), LeagueClientReadError> {
    let Some(action_id) = champ_select
        .actions
        .iter()
        .flatten()
        .find(|action| {
            action.actor_cell_id == Some(local_cell_id)
                && action.completed != Some(true)
                && action
                    .action_type
                    .as_deref()
                    .is_some_and(|value| value.eq_ignore_ascii_case(action_type))
                && action.is_ally_action != Some(false)
        })
        .and_then(|action| action.id)
    else {
        return Ok(());
    };

    session
        .patch_json(
            format!("/lol-champ-select/v1/session/actions/{action_id}").as_str(),
            &LcuChampSelectActionUpdate {
                champion_id,
                completed: true,
            },
        )
        .map_err(read_error_from_request)
}

fn map_summoner_batch_entries(summoners: Vec<LcuSummonerBatch>) -> Vec<SummonerBatchEntry> {
    summoners
        .into_iter()
        .filter_map(map_summoner_batch_entry)
        .collect()
}

fn map_summoner_batch_entry(summoner: LcuSummonerBatch) -> Option<SummonerBatchEntry> {
    let summoner_id = summoner.summoner_id?;
    let puuid = summoner.puuid.filter(|value| !value.is_empty())?;
    let display_name = match (
        non_empty(summoner.game_name.as_deref()),
        non_empty(summoner.tag_line.as_deref()),
    ) {
        (Some(name), Some(tag)) => format!("{name}#{tag}"),
        (Some(name), None) => name.to_string(),
        _ => non_empty(summoner.display_name.as_deref())?.to_string(),
    };

    Some(SummonerBatchEntry {
        summoner_id,
        puuid,
        display_name,
    })
}

impl LcuSummoner {
    fn profile(&self) -> CurrentSummonerProfile {
        CurrentSummonerProfile {
            display_name: self.display_name(),
            summoner_level: self.summoner_level.unwrap_or_default(),
            profile_icon_id: self.profile_icon_id,
        }
    }

    fn display_name(&self) -> String {
        if let Some(value) = non_empty(self.display_name.as_deref()) {
            return value.to_string();
        }

        match (
            non_empty(self.game_name.as_deref()),
            non_empty(self.tag_line.as_deref()),
        ) {
            (Some(game_name), Some(tag_line)) => format!("{game_name}#{tag_line}"),
            (Some(game_name), None) => game_name.to_string(),
            _ => "Current summoner".to_string(),
        }
    }

    fn matches_player(&self, player: &LcuPlayer) -> bool {
        ids_match(self.summoner_id, player.summoner_id)
            || ids_match(self.account_id, player.account_id)
            || ids_match(self.account_id, player.current_account_id)
            || strings_match(self.puuid.as_deref(), player.puuid.as_deref())
    }
}


fn map_champion_catalog(champions: Vec<LcuChampionSummary>) -> Vec<LeagueChampionSummary> {
    champions
        .into_iter()
        .filter(|champion| champion.id > 0 && !champion.name.trim().is_empty())
        .map(|champion| LeagueChampionSummary {
            champion_id: champion.id,
            champion_name: champion.name.trim().to_string(),
        })
        .collect()
}

fn map_champion_details(
    session: &LcuSession,
    champion_id: i64,
    details: LcuChampionDetails,
    bin_data: Option<&community_dragon::BinChampionData>,
) -> Result<LeagueChampionDetails, LeagueClientReadError> {
    let champion_name = non_empty(details.name.as_deref())
        .map(str::to_string)
        .unwrap_or_else(|| format!("Champion {champion_id}"));
    let square_portrait = details
        .square_portrait_path
        .as_deref()
        .and_then(normalize_lcu_asset_path)
        .and_then(|path| {
            session
                .get_image_asset(path.as_str(), CHAMPION_ICON_MIME)
                .ok()
        });
    let mut abilities = Vec::new();

    if let Some(passive) = details.passive {
        let passive_bin = bin_data.and_then(|bd| bd.get_spell("Passive"));
        abilities.push(map_champion_ability(
            session,
            champion_name.as_str(),
            "Passive",
            passive,
            passive_bin,
        ));
    }

    for (index, spell) in details.spells.into_iter().take(4).enumerate() {
        let slot = spell
            .spell_key
            .as_deref()
            .and_then(|value| non_empty(Some(value)))
            .map(str::to_string)
            .unwrap_or_else(|| ["Q", "W", "E", "R"][index].to_string());
        let slot_bin = bin_data.and_then(|bd| bd.get_spell(slot.as_str()));
        abilities.push(map_champion_ability(
            session,
            champion_name.as_str(),
            slot.as_str(),
            spell,
            slot_bin,
        ));
    }

    Ok(LeagueChampionDetails {
        champion_id,
        champion_name,
        title: details.title.and_then(non_empty_owned),
        square_portrait,
        abilities,
    })
}

fn map_champion_ability(
    session: &LcuSession,
    champion_name: &str,
    slot: &str,
    ability: LcuChampionAbility,
    bin_spell: Option<&community_dragon::BinSpellData>,
) -> LeagueChampionAbility {
    let icon = ability
        .ability_icon_path
        .as_deref()
        .and_then(normalize_lcu_asset_path)
        .and_then(|path| {
            session
                .get_image_asset(path.as_str(), CHAMPION_ICON_MIME)
                .ok()
        });
    let raw_description = ability
        .dynamic_description
        .or(ability.description)
        .map(clean_game_asset_text)
        .and_then(non_empty_owned)
        .unwrap_or_else(|| "No description available".to_string());
    let ability_name = ability
        .name
        .and_then(non_empty_owned)
        .unwrap_or_else(|| slot.to_string());

    // Resolve @Token@ placeholders using CommunityDragon bin data
    let summary_description = raw_description.clone();
    let effect_values = ability
        .effect_amounts
        .iter()
        .map(|(name, values)| community_dragon::DataValue {
            name: name.clone(),
            values: values.clone(),
        })
        .collect::<Vec<_>>();
    let description = bin_spell
        .map(|spell| {
            community_dragon::resolve_spell_tokens_with_fallbacks(
                &raw_description,
                spell,
                &effect_values,
            )
        })
        .map(|resolution| {
            if !resolution.unresolved_tokens.is_empty() {
                log_lcu_adapter_event(
                    format!(
                        "unresolved champion ability tokens champion={champion_name} slot={slot} ability={} tokens={:?}",
                        ability_name, resolution.unresolved_tokens
                    )
                    .as_str(),
                );
            }
            resolution.text
        })
        .unwrap_or(raw_description);

    let cooldown_values = coefficient_values(&ability.cooldown_coefficients, true);
    let cost_values = coefficient_values(&ability.cost_coefficients, true);
    let range_values = ability
        .range
        .as_ref()
        .map(value_as_display_values)
        .unwrap_or_default();

    // Extract structured ability stats from CommunityDragon bin data
    let stats: Vec<AbilityStat> = bin_spell
        .map(|spell| {
            spell
                .data_values
                .iter()
                .filter(|dv| community_dragon::is_interesting_stat(&dv.name))
                .filter(|dv| !community_dragon::is_noise_stat(&dv.name, &dv.values))
                .map(|dv| {
                    let (label, label_suffix) = community_dragon::clean_stat_label(&dv.name);
                    let (values, auto_suffix) = community_dragon::scale_percent_values(&dv.values);
                    let suffix = if !auto_suffix.is_empty() {
                        auto_suffix
                    } else {
                        label_suffix
                    };
                    AbilityStat {
                        label,
                        values,
                        suffix,
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    LeagueChampionAbility {
        slot: slot.to_string(),
        name: ability_name,
        description,
        icon,
        cooldown: ability.cooldown.as_ref().and_then(value_as_display_string),
        cost: ability.cost.as_ref().and_then(value_as_display_string),
        range: ability.range.as_ref().and_then(value_as_display_string),
        summary_description,
        cooldown_values,
        cost_values,
        range_values,
        stats,
    }
}

fn champion_name_map(champions: Vec<LcuChampionSummary>) -> HashMap<i64, String> {
    champions
        .into_iter()
        .filter(|champion| champion.id > 0 && !champion.name.trim().is_empty())
        .map(|champion| (champion.id, champion.name))
        .collect()
}


impl LcuParticipantStats {
    fn items(&self) -> Vec<i64> {
        [
            self.item0, self.item1, self.item2, self.item3, self.item4, self.item5, self.item6,
        ]
        .into_iter()
        .flatten()
        .filter(|value| *value > 0)
        .collect()
    }

    fn runes(&self) -> Vec<i64> {
        [
            self.perk0, self.perk1, self.perk2, self.perk3, self.perk4, self.perk5,
        ]
        .into_iter()
        .flatten()
        .filter(|value| *value > 0)
        .collect()
    }
}

fn map_recent_matches(
    history: LcuMatchHistoryResponse,
    summoner: &LcuSummoner,
    champion_names: &HashMap<i64, String>,
) -> Vec<RecentMatchSummary> {
    history
        .games
        .map(|games| {
            games
                .games
                .into_iter()
                .filter_map(|game| map_recent_match(game, summoner, champion_names))
                .collect()
        })
        .unwrap_or_default()
}

fn map_recent_matches_for_puuid(
    history: LcuMatchHistoryResponse,
    player_puuid: &str,
    champion_names: &HashMap<i64, String>,
) -> Vec<RecentMatchSummary> {
    history
        .games
        .map(|games| {
            games
                .games
                .into_iter()
                .filter_map(|game| map_recent_match_for_puuid(game, player_puuid, champion_names))
                .collect()
        })
        .unwrap_or_default()
}

fn map_recent_match(
    game: LcuGame,
    summoner: &LcuSummoner,
    champion_names: &HashMap<i64, String>,
) -> Option<RecentMatchSummary> {
    let participant_id = game
        .participant_identities
        .iter()
        .find_map(|identity| match &identity.player {
            Some(player) if summoner.matches_player(player) => identity.participant_id,
            _ => None,
        })?;
    let participant = game
        .participants
        .iter()
        .find(|participant| participant.participant_id == Some(participant_id))?;
    let stats = participant.stats.as_ref()?;
    let kills = stats.kills.unwrap_or_default();
    let deaths = stats.deaths.unwrap_or_default();
    let assists = stats.assists.unwrap_or_default();

    Some(RecentMatchSummary {
        game_id: game.game_id?,
        champion_id: participant.champion_id,
        champion_name: participant_champion_name(participant, champion_names),
        queue_name: game.queue_id.and_then(queue_name).map(str::to_string),
        result: match stats.win {
            Some(true) => MatchResult::Win,
            Some(false) => MatchResult::Loss,
            None => MatchResult::Unknown,
        },
        kills,
        deaths,
        assists,
        kda: Some(round_to_tenth(calculate_kda(kills, deaths, assists))),
        played_at: game
            .game_creation_date
            .or_else(|| value_to_string(game.game_creation)),
        game_duration_seconds: game.game_duration,
    })
}

fn map_recent_match_for_puuid(
    game: LcuGame,
    player_puuid: &str,
    champion_names: &HashMap<i64, String>,
) -> Option<RecentMatchSummary> {
    let participant_id = game
        .participant_identities
        .iter()
        .find_map(|identity| match &identity.player {
            Some(player) if strings_match(Some(player_puuid), player.puuid.as_deref()) => {
                identity.participant_id
            }
            _ => None,
        })?;
    let participant = game
        .participants
        .iter()
        .find(|participant| participant.participant_id == Some(participant_id))?;
    let stats = participant.stats.as_ref()?;

    Some(recent_match_from_participant(
        &game,
        participant,
        stats,
        champion_names,
    ))
}

fn recent_match_from_participant(
    game: &LcuGame,
    participant: &LcuParticipant,
    stats: &LcuParticipantStats,
    champion_names: &HashMap<i64, String>,
) -> RecentMatchSummary {
    let kills = stats.kills.unwrap_or_default();
    let deaths = stats.deaths.unwrap_or_default();
    let assists = stats.assists.unwrap_or_default();

    RecentMatchSummary {
        game_id: game.game_id.unwrap_or_default(),
        champion_id: participant.champion_id,
        champion_name: participant_champion_name(participant, champion_names),
        queue_name: game.queue_id.and_then(queue_name).map(str::to_string),
        result: match stats.win {
            Some(true) => MatchResult::Win,
            Some(false) => MatchResult::Loss,
            None => MatchResult::Unknown,
        },
        kills,
        deaths,
        assists,
        kda: Some(round_to_tenth(calculate_kda(kills, deaths, assists))),
        played_at: game
            .game_creation_date
            .clone()
            .or_else(|| value_to_string(game.game_creation.clone())),
        game_duration_seconds: game.game_duration,
    }
}

fn map_completed_match(
    game: LcuGame,
    summoner: &LcuSummoner,
    champion_names: &HashMap<i64, String>,
) -> Option<application::LeagueCompletedMatch> {
    let self_result = game
        .participant_identities
        .iter()
        .find_map(|identity| match &identity.player {
            Some(player) if summoner.matches_player(player) => identity.participant_id,
            _ => None,
        })
        .and_then(|participant_id| {
            game.participants
                .iter()
                .find(|participant| participant.participant_id == Some(participant_id))
        })
        .and_then(|participant| participant.stats.as_ref())
        .map(match_result_from_stats)
        .unwrap_or(MatchResult::Unknown);
    let participants = game
        .participants
        .iter()
        .filter_map(|participant| {
            let player = participant
                .participant_id
                .and_then(|participant_id| player_for_participant(&game, participant_id));
            map_completed_participant(participant, player, champion_names)
        })
        .collect();

    Some(application::LeagueCompletedMatch {
        game_id: game.game_id?,
        queue_name: game.queue_id.and_then(queue_name).map(str::to_string),
        played_at: game
            .game_creation_date
            .or_else(|| value_to_string(game.game_creation)),
        game_duration_seconds: game.game_duration,
        result: self_result,
        participants,
    })
}

fn find_completed_game(games: Vec<LcuGame>, game_id: i64) -> Option<LcuGame> {
    games.into_iter().find(|game| game.game_id == Some(game_id))
}

fn best_completed_match(
    summary_match: application::LeagueCompletedMatch,
    detail_match: Option<application::LeagueCompletedMatch>,
) -> application::LeagueCompletedMatch {
    match detail_match {
        Some(detail_match)
            if detail_match.participants.len() > summary_match.participants.len() =>
        {
            detail_match
        }
        _ => summary_match,
    }
}

fn player_for_participant(game: &LcuGame, participant_id: i64) -> Option<&LcuPlayer> {
    game.participant_identities.iter().find_map(|identity| {
        if identity.participant_id == Some(participant_id) {
            identity.player.as_ref()
        } else {
            None
        }
    })
}

fn map_completed_participant(
    participant: &LcuParticipant,
    player: Option<&LcuPlayer>,
    champion_names: &HashMap<i64, String>,
) -> Option<application::LeagueCompletedParticipant> {
    let stats = participant.stats.as_ref()?;
    let participant_id = participant.participant_id?;
    let kills = stats.kills.unwrap_or_default();
    let deaths = stats.deaths.unwrap_or_default();
    let assists = stats.assists.unwrap_or_default();

    Some(application::LeagueCompletedParticipant {
        participant_id,
        team_id: participant.team_id.unwrap_or_default(),
        display_name: player
            .and_then(player_display_name)
            .unwrap_or_else(|| format!("Participant {participant_id}")),
        player_puuid: player
            .and_then(|value| non_empty(value.puuid.as_deref()).map(str::to_string)),
        profile_icon_id: player.and_then(player_profile_icon_id),
        champion_id: participant.champion_id,
        champion_name: participant_champion_name(participant, champion_names),
        role: participant
            .timeline
            .as_ref()
            .and_then(|timeline| non_empty(timeline.role.as_deref()).map(str::to_string)),
        lane: participant
            .timeline
            .as_ref()
            .and_then(|timeline| non_empty(timeline.lane.as_deref()).map(str::to_string)),
        result: match_result_from_stats(stats),
        kills,
        deaths,
        assists,
        kda: Some(round_to_tenth(calculate_kda(kills, deaths, assists))),
        cs: stats.total_minions_killed.unwrap_or_default()
            + stats.neutral_minions_killed.unwrap_or_default(),
        gold_earned: stats.gold_earned.unwrap_or_default(),
        damage_to_champions: stats.total_damage_dealt_to_champions.unwrap_or_default(),
        vision_score: stats.vision_score.unwrap_or_default(),
        items: stats.items(),
        runes: stats.runes(),
        spells: [participant.spell1_id, participant.spell2_id]
            .into_iter()
            .flatten()
            .filter(|value| *value > 0)
            .collect(),
    })
}

fn participant_champion_name(
    participant: &LcuParticipant,
    champion_names: &HashMap<i64, String>,
) -> String {
    if let Some(name) = non_empty(participant.champion_name.as_deref()) {
        return name.to_string();
    }

    participant
        .champion_id
        .and_then(|id| champion_names.get(&id).cloned())
        .or_else(|| participant.champion_id.map(|id| format!("Champion {id}")))
        .unwrap_or_else(|| "Unknown champion".to_string())
}

fn read_participant_recent_stats_with_session(
    session: &LcuSession,
    player_puuid: &str,
    limit: i64,
    champion_names: &HashMap<i64, String>,
) -> Result<ParticipantRecentStats, LeagueClientReadError> {
    let history = session
        .get_json::<LcuMatchHistoryResponse>(puuid_matches_path(player_puuid, limit).as_str())
        .map_err(read_error_from_request)?;
    let recent_matches = map_recent_matches_for_puuid(history, player_puuid, champion_names);

    Ok(participant_recent_stats(recent_matches))
}

fn participant_recent_stats(matches: Vec<RecentMatchSummary>) -> ParticipantRecentStats {
    let mut total_kda = 0.0;
    let mut match_count = 0;
    let mut recent_champions = Vec::new();

    for match_summary in &matches {
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

    ParticipantRecentStats {
        match_count,
        average_kda,
        recent_champions,
        recent_matches: matches,
    }
}

fn find_game_asset_metadata(value: Value, asset_id: i64) -> Option<LcuGameAssetMetadata> {
    match value {
        Value::Array(values) => values
            .into_iter()
            .find_map(|value| metadata_from_value(value, None, asset_id)),
        Value::Object(mut object) => {
            if let Some(data) = object.remove("data") {
                return find_game_asset_metadata(data, asset_id);
            }

            object.into_iter().find_map(|(key, value)| {
                let key_id = key.parse::<i64>().ok();
                metadata_from_value(value, key_id, asset_id)
            })
        }
        _ => None,
    }
}

fn metadata_from_value(
    value: Value,
    key_id: Option<i64>,
    asset_id: i64,
) -> Option<LcuGameAssetMetadata> {
    let object = value.as_object()?;
    let value_id = object
        .get("id")
        .or_else(|| object.get("itemId"))
        .or_else(|| object.get("spellId"))
        .and_then(value_as_i64)
        .or(key_id)?;

    if value_id != asset_id {
        return None;
    }

    Some(LcuGameAssetMetadata {
        name: object.get("name").and_then(value_as_string),
        description: game_asset_description_from_object(object),
        icon_path: object.get("iconPath").and_then(value_as_string),
    })
}

fn game_asset_description_from_object(object: &serde_json::Map<String, Value>) -> Option<String> {
    [
        "description",
        "plaintext",
        "longDesc",
        "shortDesc",
        "tooltip",
    ]
    .into_iter()
    .find_map(|key| object.get(key).and_then(value_as_string))
    .and_then(|value| non_empty_owned(clean_game_asset_text(value)))
}

fn game_asset_name(
    kind: LeagueGameAssetKind,
    asset_id: i64,
    metadata: Option<&LcuGameAssetMetadata>,
) -> String {
    metadata
        .and_then(|metadata| non_empty(metadata.name.as_deref()).map(str::to_string))
        .unwrap_or_else(|| format!("{} {asset_id}", game_asset_label(kind)))
}

fn game_asset_description(metadata: Option<&LcuGameAssetMetadata>) -> Option<String> {
    metadata.and_then(|metadata| metadata.description.clone())
}

fn game_asset_image_path(
    kind: LeagueGameAssetKind,
    asset_id: i64,
    metadata: Option<&LcuGameAssetMetadata>,
) -> Option<String> {
    metadata
        .and_then(|metadata| metadata.icon_path.as_deref())
        .and_then(normalize_lcu_asset_path)
        .or_else(|| fallback_game_asset_image_path(kind, asset_id))
}

fn game_asset_metadata_path(kind: LeagueGameAssetKind) -> &'static str {
    match kind {
        LeagueGameAssetKind::Item => "/lol-game-data/assets/v1/items.json",
        LeagueGameAssetKind::Rune => "/lol-game-data/assets/v1/perks.json",
        LeagueGameAssetKind::Spell => "/lol-game-data/assets/v1/summoner-spells.json",
    }
}

fn fallback_game_asset_image_path(kind: LeagueGameAssetKind, asset_id: i64) -> Option<String> {
    match kind {
        LeagueGameAssetKind::Item => Some(format!("/lol-game-data/assets/v1/items/{asset_id}.png")),
        LeagueGameAssetKind::Rune => None,
        LeagueGameAssetKind::Spell => Some(format!(
            "/lol-game-data/assets/v1/summoner-spells/{asset_id}.png"
        )),
    }
}

fn game_asset_label(kind: LeagueGameAssetKind) -> &'static str {
    match kind {
        LeagueGameAssetKind::Item => "Item",
        LeagueGameAssetKind::Rune => "Rune",
        LeagueGameAssetKind::Spell => "Spell",
    }
}

fn normalize_lcu_asset_path(value: &str) -> Option<String> {
    let path = value.trim().replace('\\', "/").replace(' ', "%20");

    if path.is_empty() || path.contains("://") || path.contains("..") {
        return None;
    }

    if path.starts_with("/lol-game-data/assets/") {
        Some(path)
    } else if path.starts_with("lol-game-data/assets/") {
        Some(format!("/{path}"))
    } else if path.starts_with("ASSETS/") || path.starts_with("assets/") {
        Some(format!("/lol-game-data/assets/{path}"))
    } else {
        None
    }
}

fn clean_game_asset_text(value: String) -> String {
    let mut text = String::new();
    let mut inside_tag = false;

    for character in value.chars() {
        match character {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => text.push(character),
            _ => {}
        }
    }

    text.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("  ", " ")
        .trim()
        .to_string()
}

fn value_as_i64(value: &Value) -> Option<i64> {
    match value {
        Value::Number(number) => number.as_i64(),
        Value::String(value) => value.parse::<i64>().ok(),
        _ => None,
    }
}

fn value_as_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => non_empty(Some(value.as_str())).map(str::to_string),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn value_as_display_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => non_empty(Some(value.as_str())).map(str::to_string),
        Value::Number(value) => value.as_f64().map(format_display_number),
        Value::Array(values) => {
            let values = values
                .iter()
                .filter_map(value_as_display_string)
                .collect::<Vec<_>>();
            if values.is_empty() {
                None
            } else {
                Some(values.join(" / "))
            }
        }
        _ => None,
    }
}

fn value_as_display_values(value: &Value) -> Vec<String> {
    match value {
        Value::Array(values) => {
            let numbers = values.iter().filter_map(Value::as_f64).collect::<Vec<_>>();
            if !numbers.is_empty() {
                compact_display_numbers(numbers, true)
            } else {
                compact_display_strings(
                    values
                        .iter()
                        .filter_map(value_as_display_string)
                        .collect::<Vec<_>>(),
                )
            }
        }
        Value::Number(value) => value
            .as_f64()
            .map(format_display_number)
            .into_iter()
            .collect(),
        Value::String(value) => non_empty(Some(value.as_str()))
            .map(str::to_string)
            .into_iter()
            .collect(),
        _ => Vec::new(),
    }
}

fn coefficient_values(values: &[f64], keep_zero: bool) -> Vec<String> {
    if !keep_zero && values.iter().all(|value| value.abs() < f64::EPSILON) {
        return Vec::new();
    }

    compact_display_numbers(values.to_vec(), keep_zero)
}

fn compact_display_numbers(mut values: Vec<f64>, keep_zero: bool) -> Vec<String> {
    if !keep_zero {
        values.retain(|value| value.is_finite() && value.abs() >= f64::EPSILON);
    } else {
        values.retain(|value| value.is_finite());
    }

    while values.len() > 1 {
        let last = *values.last().unwrap_or(&0.0);
        let previous = values[values.len() - 2];
        if (last - previous).abs() < 0.001 {
            values.pop();
        } else {
            break;
        }
    }

    values.into_iter().map(format_display_number).collect()
}

fn compact_display_strings(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty());

    while values.len() > 1 && values.last() == values.get(values.len() - 2) {
        values.pop();
    }

    values
}

fn format_display_number(value: f64) -> String {
    if value.is_nan() || value.is_infinite() {
        return value.to_string();
    }

    let rounded = value.round();
    if (value - rounded).abs() < 0.001 {
        return (rounded as i64).to_string();
    }

    let formatted = format!("{value:.2}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

fn match_result_from_stats(stats: &LcuParticipantStats) -> MatchResult {
    match stats.win {
        Some(true) => MatchResult::Win,
        Some(false) => MatchResult::Loss,
        None => MatchResult::Unknown,
    }
}

fn player_display_name(player: &LcuPlayer) -> Option<String> {
    match (
        non_empty(player.game_name.as_deref()),
        non_empty(player.tag_line.as_deref()),
    ) {
        (Some(game_name), Some(tag_line)) => Some(format!("{game_name}#{tag_line}")),
        (Some(game_name), None) => Some(game_name.to_string()),
        _ => non_empty(player.summoner_name.as_deref()).map(str::to_string),
    }
}

fn player_profile_icon_id(player: &LcuPlayer) -> Option<i64> {
    player.profile_icon_id.or(player.profile_icon)
}

fn queue_name(queue_id: i64) -> Option<&'static str> {
    match queue_id {
        400 => Some("Normal Draft"),
        420 => Some("Ranked Solo/Duo"),
        430 => Some("Normal Blind"),
        440 => Some("Ranked Flex"),
        450 => Some("ARAM"),
        700 => Some("Clash"),
        1700 => Some("Arena"),
        _ => None,
    }
}

fn profile_icon_path(profile_icon_id: i64) -> String {
    format!("/lol-game-data/assets/v1/profile-icons/{profile_icon_id}.jpg")
}

fn champion_icon_path(champion_id: i64) -> String {
    format!("/lol-game-data/assets/v1/champion-icons/{champion_id}.png")
}

fn champion_details_path(champion_id: i64) -> String {
    format!("/lol-game-data/assets/v1/champions/{champion_id}.json")
}

fn current_matches_path(limit: i64) -> String {
    format!(
        "/lol-match-history/v1/products/lol/current-summoner/matches?begIndex=0&endIndex={limit}"
    )
}

fn completed_match_path(game_id: i64) -> String {
    format!("/lol-match-history/v1/games/{game_id}")
}

fn puuid_matches_path(player_puuid: &str, limit: i64) -> String {
    format!("/lol-match-history/v1/products/lol/{player_puuid}/matches?begIndex=0&endIndex={limit}")
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

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.and_then(|inner| {
        let trimmed = inner.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn normalize_player_name(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn percent_encode_path_value(value: &str) -> String {
    let mut encoded = String::new();

    for byte in value.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(*byte as char);
            }
            _ => encoded.push_str(format!("%{byte:02X}").as_str()),
        }
    }

    encoded
}

fn non_empty_owned(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn ids_match(left: Option<i64>, right: Option<i64>) -> bool {
    left.zip(right).is_some_and(|(a, b)| a == b)
}

fn strings_match(left: Option<&str>, right: Option<&str>) -> bool {
    left.zip(right).is_some_and(|(a, b)| a == b)
}

fn is_safe_lcu_path_id(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        })
}

fn value_to_string(value: Option<Value>) -> Option<String> {
    match value {
        Some(Value::String(value)) => Some(value),
        Some(Value::Number(value)) => Some(value.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use application::RankedChampionDataError;
    use domain::RankedChampionLane;
    use std::path::Path;

    const TEST_LOCKFILE_VALUE: &str = "not-a-real-test-value";

    #[test]
    fn parses_ranked_champion_json_snapshot() {
        let snapshot = parse_ranked_champion_snapshot_json(
            r#"{
                "formatVersion": 1,
                "source": "test-json",
                "patch": "26.08",
                "region": "KR",
                "queue": "RANKED_SOLO_5X5",
                "tier": "EMERALD_PLUS",
                "generatedAt": "2026-04-25T00:00:00Z",
                "champions": [
                    {
                        "championId": 103,
                        "championName": "Ahri",
                        "championAlias": "Ahri",
                        "lane": "mid",
                        "games": 1000,
                        "winRate": 51.4,
                        "pickRate": 10.2,
                        "banRate": 8.0
                    }
                ]
            }"#,
        )
        .expect("ranked champion json parses");

        assert_eq!(snapshot.source, "test-json");
        assert_eq!(snapshot.records.len(), 1);
        assert_eq!(snapshot.records[0].lane, RankedChampionLane::Middle);
        assert_eq!(snapshot.records[0].wins, 514);
        assert_eq!(snapshot.records[0].picks, 1000);
        assert_eq!(snapshot.records[0].bans, 80);
        assert!(snapshot.imported_at.parse::<u64>().is_ok());
    }

    #[test]
    fn rejects_invalid_ranked_champion_json_snapshot() {
        let error = parse_ranked_champion_snapshot_json(
            r#"{
                "formatVersion": 1,
                "champions": [
                    {
                        "championId": 103,
                        "championName": "Ahri",
                        "lane": "river",
                        "games": 1000,
                        "winRate": 51.4,
                        "pickRate": 10.2,
                        "banRate": 8.0
                    }
                ]
            }"#,
        )
        .expect_err("invalid ranked champion json is rejected");

        assert!(matches!(error, RankedChampionDataError::InvalidData(_)));
        assert!(!error.to_string().contains("Authorization"));
    }

    #[test]
    fn rejects_duplicate_ranked_champion_lane_entries() {
        let error = parse_ranked_champion_snapshot_json(
            r#"{
                "formatVersion": 1,
                "champions": [
                    {
                        "championId": 103,
                        "championName": "Ahri",
                        "lane": "middle",
                        "games": 1000,
                        "winRate": 51.4,
                        "pickRate": 10.2,
                        "banRate": 8.0
                    },
                    {
                        "championId": 103,
                        "championName": "Ahri",
                        "lane": "mid",
                        "games": 900,
                        "winRate": 50.0,
                        "pickRate": 9.0,
                        "banRate": 7.0
                    }
                ]
            }"#,
        )
        .expect_err("duplicate ranked champion lane is rejected");

        assert!(error.to_string().contains("duplicate"));
    }

    #[test]
    fn checked_in_ranked_champion_data_matches_adapter_contract() {
        let snapshot = parse_ranked_champion_snapshot_json(include_str!(
            "../../../data/ranked-champions/latest.json"
        ))
        .expect("checked-in ranked champion data parses");

        assert_eq!(snapshot.source, "lol-desktop-assistant-sample");
        assert_eq!(snapshot.records.len(), 10);
        assert!(
            snapshot
                .records
                .iter()
                .any(|record| record.lane == RankedChampionLane::Support)
        );
    }

    #[test]
    fn parses_advisor_json_snapshot() {
        let snapshot = parse_advisor_snapshot_json(
            advisor_json(&[advisor_json_entry(103, "Ahri", "mid")]).as_str(),
        )
        .expect("advisor json parses");

        assert_eq!(snapshot.source, "test-advisor-json");
        assert_eq!(snapshot.records.len(), 1);
        assert_eq!(snapshot.records[0].champion_name, "Ahri");
        assert_eq!(snapshot.records[0].lane, RankedChampionLane::Middle);
        assert_eq!(snapshot.records[0].summoner_spells[0].name, "Flash");
    }

    #[test]
    fn advisor_json_rejects_duplicate_champion_lane() {
        let error = parse_advisor_snapshot_json(
            advisor_json(&[
                advisor_json_entry(103, "Ahri", "middle"),
                advisor_json_entry(103, "Ahri", "mid"),
            ])
            .as_str(),
        )
        .expect_err("duplicate advisor entries are rejected");

        assert!(matches!(error, RankedChampionDataError::InvalidData(_)));
        assert!(error.to_string().contains("duplicate"));
    }

    #[test]
    fn advisor_json_rejects_missing_required_fields() {
        let error = parse_advisor_snapshot_json(
            r#"{
                "formatVersion": 1,
                "champions": [
                    {
                        "championId": 103,
                        "championName": "Ahri",
                        "lane": "middle",
                        "games": 1000,
                        "winRate": 51.4,
                        "pickRate": 10.2,
                        "banRate": 8.0
                    }
                ]
            }"#,
        )
        .expect_err("incomplete advisor entry is rejected");

        assert!(matches!(error, RankedChampionDataError::InvalidData(_)));
    }

    #[test]
    fn checked_in_advisor_data_matches_adapter_contract() {
        let snapshot =
            parse_advisor_snapshot_json(include_str!("../../../data/advisor/latest.json"))
                .expect("checked-in advisor data parses");

        assert_eq!(snapshot.source, "lol-desktop-assistant-advisor-sample");
        assert_eq!(snapshot.records.len(), 5);
        assert!(
            snapshot
                .records
                .iter()
                .any(|record| record.lane == RankedChampionLane::Support)
        );
    }

    #[test]
    fn live_overlay_maps_visible_item_values_scores_spells_and_events() {
        let snapshot = map_live_overlay_snapshot(
            vec![
                GameClientPlayer {
                    summoner_name: Some("Ally One".to_string()),
                    team: "ORDER".to_string(),
                    riot_id: Some("Ally One#NA1".to_string()),
                    champion_name: Some("Garen".to_string()),
                    level: Some(6),
                    position: Some("TOP".to_string()),
                    is_dead: false,
                    respawn_timer: None,
                    items: vec![GameClientItem {
                        item_id: Some(1055),
                        display_name: Some("Doran's Blade".to_string()),
                        price: Some(450),
                        count: Some(1),
                        slot: Some(0),
                    }],
                    scores: Some(GameClientScores {
                        kills: Some(1),
                        deaths: Some(0),
                        assists: Some(2),
                        creep_score: Some(42),
                        ward_score: Some(5.0),
                    }),
                    summoner_spells: Some(GameClientSummonerSpells {
                        summoner_spell_one: Some(GameClientNamedEntity {
                            display_name: Some("Flash".to_string()),
                            id: Some(4),
                        }),
                        summoner_spell_two: Some(GameClientNamedEntity {
                            display_name: Some("Ignite".to_string()),
                            id: Some(14),
                        }),
                    }),
                },
                GameClientPlayer {
                    summoner_name: Some("Enemy One".to_string()),
                    team: "CHAOS".to_string(),
                    riot_id: None,
                    champion_name: Some("Darius".to_string()),
                    level: Some(6),
                    position: Some("TOP".to_string()),
                    is_dead: true,
                    respawn_timer: Some(8.0),
                    items: vec![GameClientItem {
                        item_id: Some(1054),
                        display_name: Some("Doran's Shield".to_string()),
                        price: Some(450),
                        count: Some(1),
                        slot: Some(0),
                    }],
                    scores: None,
                    summoner_spells: None,
                },
            ],
            Some(GameClientActivePlayer {
                summoner_name: "Ally One".to_string(),
                level: Some(6),
                current_gold: Some(733.0),
                champion_stats: Some(GameClientChampionStats {
                    resource_type: Some("MANA".to_string()),
                    resource_value: Some(100.0),
                    resource_max: Some(300.0),
                }),
            }),
            vec![GameClientEvent {
                event_id: Some(1),
                event_name: Some("ChampionKill".to_string()),
                event_time: Some(120.0),
                killer_name: Some("Ally One".to_string()),
                victim_name: Some("Enemy One".to_string()),
                assisters: vec!["Ally Two".to_string()],
            }],
            Some(GameClientStats {
                game_time: Some(125.0),
                game_mode: Some("CLASSIC".to_string()),
                map_name: Some("Summoner's Rift".to_string()),
            }),
        );

        assert_eq!(snapshot.game_time_seconds, Some(125.0));
        assert_eq!(
            snapshot.active_player.as_ref().unwrap().current_gold,
            Some(733.0)
        );
        assert_eq!(snapshot.players.len(), 2);
        assert_eq!(snapshot.players[0].summoner_spells.len(), 2);
        assert_eq!(snapshot.gold.item_value_diff, 0);
        assert_eq!(snapshot.events[0].event_name, "ChampionKill");
    }

    #[test]
    fn parses_valid_lockfile() {
        let credentials =
            parse_lockfile(format!("LeagueClient:1234:2999:{TEST_LOCKFILE_VALUE}:https").as_str())
                .expect("lockfile parses");

        assert_eq!(credentials.port, 2999);
        assert_eq!(credentials.password, TEST_LOCKFILE_VALUE);
    }

    #[test]
    fn lockfile_credentials_debug_redacts_password() {
        let credentials =
            parse_lockfile(format!("LeagueClient:1234:2999:{TEST_LOCKFILE_VALUE}:https").as_str())
                .expect("lockfile parses");
        let message = format!("{credentials:?}");

        assert!(message.contains("<redacted>"));
        assert!(!message.contains(TEST_LOCKFILE_VALUE));
    }

    #[test]
    fn lcu_subscription_formats_json_api_events() {
        assert_eq!(
            LcuSubscription::JsonApiEvent("/lol-gameflow/v1/gameflow-phase").to_string(),
            "OnJsonApiEvent_lol-gameflow_v1_gameflow-phase"
        );
        assert_eq!(
            LcuSubscription::JsonApiEvent("/lol-champ-select/v1/session").to_string(),
            "OnJsonApiEvent_lol-champ-select_v1_session"
        );
    }

    #[test]
    fn parses_lcu_websocket_json_api_event() {
        let event = parse_lcu_websocket_event_text(
            r#"[8,"OnJsonApiEvent_lol-gameflow_v1_gameflow-phase",{"data":"ChampSelect","eventType":"Update","uri":"/lol-gameflow/v1/gameflow-phase"}]"#,
        )
        .expect("event parses");

        assert_eq!(event.uri, "/lol-gameflow/v1/gameflow-phase");
        assert_eq!(event.event_type, "Update");
        assert_eq!(event.data.as_str(), Some("ChampSelect"));
    }

    #[test]
    fn parses_lcu_websocket_event_uri_from_subscription() {
        let event = parse_lcu_websocket_event_text(
            r#"[8,"OnJsonApiEvent_lol-champ-select_v1_session",{"data":{"myTeam":[],"theirTeam":[]},"eventType":"Update"}]"#,
        )
        .expect("event parses");

        assert_eq!(event.uri, "/lol-champ-select/v1/session");
        assert_eq!(event.event_type, "Update");
        assert!(event.data.get("myTeam").is_some());
    }

    #[test]
    fn ignores_invalid_lcu_websocket_events() {
        assert!(parse_lcu_websocket_event_text("not-json").is_none());
        assert!(parse_lcu_websocket_event_text(r#"{"opcode":8}"#).is_none());
        assert!(parse_lcu_websocket_event_text(
            r#"[5,"OnJsonApiEvent_lol-gameflow_v1_gameflow-phase",{"data":"ChampSelect","eventType":"Update","uri":"/lol-gameflow/v1/gameflow-phase"}]"#
        )
        .is_none());
        assert!(
            parse_lcu_websocket_event_text(
                r#"[8,"OnJsonApiEvent",{"data":"ChampSelect","eventType":"Update"}]"#
            )
            .is_none()
        );
    }

    #[test]
    fn lcu_websocket_error_debug_does_not_expose_auth_details() {
        let message = format!("{:?}", LcuWebSocketError::Authentication);

        assert!(!message.contains("Authorization"));
        assert!(!message.contains(TEST_LOCKFILE_VALUE));
    }

    #[test]
    fn rejects_malformed_lockfile_without_exposing_password() {
        let error = parse_lockfile(
            format!("LeagueClient:1234:not-a-port:{TEST_LOCKFILE_VALUE}:https").as_str(),
        )
        .expect_err("lockfile is rejected");
        let message = format!("{error:?}");

        assert!(!message.contains(TEST_LOCKFILE_VALUE));
    }

    #[test]
    fn rejects_non_https_lockfile() {
        let result =
            parse_lockfile(format!("LeagueClient:1234:2999:{TEST_LOCKFILE_VALUE}:http").as_str());

        assert!(matches!(result, Err(LcuAdapterError::InvalidLockfile)));
    }

    #[test]
    fn request_errors_map_to_specific_safe_statuses() {
        let unauthorized = status_from_request_error(LcuRequestError::Unauthorized);
        let not_logged_in = status_from_request_error(LcuRequestError::NotLoggedIn);
        let patching = status_from_request_error(LcuRequestError::Patching);

        assert_eq!(unauthorized.phase, LeagueClientPhase::Unauthorized);
        assert_eq!(not_logged_in.phase, LeagueClientPhase::NotLoggedIn);
        assert_eq!(patching.phase, LeagueClientPhase::Patching);
        assert!(unauthorized.message.unwrap().contains("authentication"));
        assert!(!format!("{not_logged_in:?}").contains(TEST_LOCKFILE_VALUE));
    }

    #[test]
    fn gameflow_session_maps_direct_player_identities() {
        let session = map_gameflow_session(LcuGameflowSession {
            game_data: Some(LcuGameflowGameData {
                team_one: vec![LcuGameflowParticipant {
                    summoner_id: Some(10),
                    puuid: Some("puuid-10".to_string()),
                    summoner_name: Some("Ally".to_string()),
                    display_name: None,
                    game_name: Some("Ally".to_string()),
                    tag_line: Some("NA1".to_string()),
                    champion_id: Some(103),
                    selected_champion_id: None,
                }],
                team_two: vec![LcuGameflowParticipant {
                    summoner_id: Some(20),
                    puuid: Some("puuid-20".to_string()),
                    summoner_name: Some("Enemy".to_string()),
                    display_name: None,
                    game_name: Some("Enemy".to_string()),
                    tag_line: Some("NA1".to_string()),
                    champion_id: None,
                    selected_champion_id: Some(222),
                }],
                player_champion_selections: Vec::new(),
            }),
        })
        .expect("gameflow session maps");

        assert_eq!(session.source, ChampSelectSessionSource::GameflowSession);
        assert_eq!(session.players.len(), 2);
        assert_eq!(session.players[0].puuid.as_deref(), Some("puuid-10"));
        assert_eq!(session.players[0].display_name, "Ally#NA1");
        assert_eq!(session.players[0].champion_id, Some(103));
        assert_eq!(session.players[1].puuid.as_deref(), Some("puuid-20"));
        assert_eq!(session.players[1].champion_id, Some(222));
    }

    #[test]
    fn image_asset_paths_are_fixed_local_game_data_paths() {
        assert_eq!(
            profile_icon_path(29),
            "/lol-game-data/assets/v1/profile-icons/29.jpg"
        );
        assert_eq!(
            champion_icon_path(103),
            "/lol-game-data/assets/v1/champion-icons/103.png"
        );
    }

    #[test]
    fn game_asset_metadata_paths_are_fixed_local_game_data_paths() {
        assert_eq!(
            game_asset_metadata_path(LeagueGameAssetKind::Item),
            "/lol-game-data/assets/v1/items.json"
        );
        assert_eq!(
            game_asset_metadata_path(LeagueGameAssetKind::Rune),
            "/lol-game-data/assets/v1/perks.json"
        );
        assert_eq!(
            game_asset_metadata_path(LeagueGameAssetKind::Spell),
            "/lol-game-data/assets/v1/summoner-spells.json"
        );
    }

    #[test]
    fn champion_ability_maps_lcu_rank_values() {
        let session = LcuSession::new(LockfileCredentials {
            port: 1,
            password: "test".to_string(),
        })
        .expect("session builds");

        let ability = map_champion_ability(
            &session,
            "Ahri",
            "Q",
            LcuChampionAbility {
                name: Some("Orb of Deception".to_string()),
                description: Some("Deals magic damage.".to_string()),
                dynamic_description: None,
                ability_icon_path: None,
                spell_key: Some("q".to_string()),
                cooldown: Some(serde_json::json!("@Cooldown@s %i:cooldown%")),
                cost: Some(serde_json::json!("@Cost@ @AbilityResourceName@")),
                range: Some(serde_json::json!([
                    970.0, 970.0, 970.0, 970.0, 970.0, 970.0
                ])),
                cooldown_coefficients: vec![9.0, 8.0, 7.0, 6.0, 5.0, 5.0],
                cost_coefficients: vec![55.0, 65.0, 75.0, 85.0, 95.0, 95.0],
                effect_amounts: HashMap::new(),
            },
            None,
        );

        assert_eq!(ability.cooldown_values, vec!["9", "8", "7", "6", "5"]);
        assert_eq!(ability.cost_values, vec!["55", "65", "75", "85", "95"]);
        assert_eq!(ability.range_values, vec!["970"]);
    }

    #[test]
    fn lcu_champion_ability_deserializes_rank_coefficients() {
        let ability: LcuChampionAbility = serde_json::from_value(serde_json::json!({
            "spellKey": "q",
            "name": "Orb of Deception",
            "description": "Deals magic damage.",
            "cooldownCoefficients": [9, 8, 7, 6, 5],
            "costCoefficients": [55, 65, 75, 85, 95],
            "range": [970, 970, 970, 970, 970]
        }))
        .expect("ability deserializes");

        assert_eq!(ability.cooldown_coefficients, vec![9.0, 8.0, 7.0, 6.0, 5.0]);
        assert_eq!(
            ability.cost_coefficients,
            vec![55.0, 65.0, 75.0, 85.0, 95.0]
        );
    }

    #[test]
    fn champion_ability_keeps_zero_rank_values() {
        assert_eq!(coefficient_values(&[0.0, 0.0, 0.0], true), vec!["0"]);
        assert_eq!(
            value_as_display_values(&serde_json::json!([0, 0, 0])),
            vec!["0"]
        );
    }

    #[test]
    fn game_asset_metadata_accepts_array_and_cleans_description() {
        let metadata = find_game_asset_metadata(
            serde_json::json!([
                {
                    "id": 1054,
                    "name": "Doran's Shield",
                    "description": "<mainText>Blocks &amp; recovers health.</mainText>",
                    "iconPath": "ASSETS/Items/Icons2D/1054.png"
                }
            ]),
            1054,
        )
        .expect("metadata is found");

        assert_eq!(metadata.name.as_deref(), Some("Doran's Shield"));
        assert_eq!(
            metadata.description.as_deref(),
            Some("Blocks & recovers health.")
        );
        assert_eq!(
            game_asset_image_path(LeagueGameAssetKind::Item, 1054, Some(&metadata)).as_deref(),
            Some("/lol-game-data/assets/ASSETS/Items/Icons2D/1054.png")
        );
    }

    #[test]
    fn game_asset_metadata_accepts_object_data_shape() {
        let metadata = find_game_asset_metadata(
            serde_json::json!({
                "data": {
                    "4": {
                        "name": "Flash",
                        "tooltip": "Teleport a short distance.",
                        "iconPath": "/lol-game-data/assets/v1/summoner-spells/4.png"
                    }
                }
            }),
            4,
        )
        .expect("metadata is found");

        assert_eq!(metadata.name.as_deref(), Some("Flash"));
        assert_eq!(
            game_asset_description(Some(&metadata)).as_deref(),
            Some("Teleport a short distance.")
        );
    }

    #[test]
    fn maps_ranked_solo_and_flex_queues() {
        let queues = map_ranked_queues(LcuRankedStats {
            queues: vec![
                LcuRankedQueue {
                    queue_type: Some("RANKED_SOLO_5x5".to_string()),
                    tier: Some("GOLD".to_string()),
                    division: Some("II".to_string()),
                    league_points: Some(72),
                    wins: Some(20),
                    losses: Some(10),
                },
                LcuRankedQueue {
                    queue_type: Some("RANKED_FLEX_SR".to_string()),
                    tier: None,
                    division: None,
                    league_points: None,
                    wins: Some(0),
                    losses: Some(0),
                },
            ],
        });

        assert_eq!(queues.len(), 2);
        assert_eq!(queues[0].queue, RankedQueue::SoloDuo);
        assert!(queues[0].is_ranked);
        assert_eq!(queues[1].queue, RankedQueue::Flex);
        assert!(!queues[1].is_ranked);
    }

    #[test]
    fn maps_recent_match_for_current_summoner_only() {
        let summoner = sample_summoner();
        let mut champion_names = HashMap::new();
        champion_names.insert(103, "Ahri".to_string());

        let matches = map_recent_matches(
            LcuMatchHistoryResponse {
                games: Some(LcuGames {
                    games: vec![LcuGame {
                        game_id: Some(10),
                        game_creation_date: Some("2026-04-19T12:00:00Z".to_string()),
                        game_creation: None,
                        game_duration: Some(1880),
                        queue_id: Some(420),
                        participants: vec![
                            LcuParticipant {
                                participant_id: Some(1),
                                team_id: Some(100),
                                champion_id: Some(266),
                                champion_name: None,
                                spell1_id: Some(4),
                                spell2_id: Some(12),
                                stats: Some(LcuParticipantStats {
                                    kills: Some(0),
                                    deaths: Some(5),
                                    assists: Some(1),
                                    win: Some(false),
                                    total_minions_killed: Some(100),
                                    neutral_minions_killed: Some(10),
                                    gold_earned: Some(8_000),
                                    total_damage_dealt_to_champions: Some(9_000),
                                    vision_score: Some(11),
                                    item0: Some(1055),
                                    item1: None,
                                    item2: None,
                                    item3: None,
                                    item4: None,
                                    item5: None,
                                    item6: None,
                                    perk0: Some(8010),
                                    perk1: None,
                                    perk2: None,
                                    perk3: None,
                                    perk4: None,
                                    perk5: None,
                                }),
                                timeline: Some(LcuParticipantTimeline {
                                    role: Some("SOLO".to_string()),
                                    lane: Some("TOP".to_string()),
                                }),
                            },
                            LcuParticipant {
                                participant_id: Some(2),
                                team_id: Some(200),
                                champion_id: Some(103),
                                champion_name: None,
                                spell1_id: Some(4),
                                spell2_id: Some(14),
                                stats: Some(LcuParticipantStats {
                                    kills: Some(7),
                                    deaths: Some(1),
                                    assists: Some(8),
                                    win: Some(true),
                                    total_minions_killed: Some(200),
                                    neutral_minions_killed: Some(10),
                                    gold_earned: Some(12_000),
                                    total_damage_dealt_to_champions: Some(22_000),
                                    vision_score: Some(18),
                                    item0: Some(1056),
                                    item1: Some(3020),
                                    item2: None,
                                    item3: None,
                                    item4: None,
                                    item5: None,
                                    item6: None,
                                    perk0: Some(8112),
                                    perk1: None,
                                    perk2: None,
                                    perk3: None,
                                    perk4: None,
                                    perk5: None,
                                }),
                                timeline: Some(LcuParticipantTimeline {
                                    role: Some("SOLO".to_string()),
                                    lane: Some("MIDDLE".to_string()),
                                }),
                            },
                        ],
                        participant_identities: vec![LcuParticipantIdentity {
                            participant_id: Some(2),
                            player: Some(LcuPlayer {
                                summoner_name: Some("Player".to_string()),
                                game_name: None,
                                tag_line: None,
                                summoner_id: Some(99),
                                account_id: Some(55),
                                current_account_id: None,
                                profile_icon: Some(1),
                                profile_icon_id: None,
                                puuid: Some("self-puuid".to_string()),
                            }),
                        }],
                    }],
                }),
            },
            &summoner,
            &champion_names,
        );

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].champion_name, "Ahri");
        assert_eq!(matches[0].champion_id, Some(103));
        assert_eq!(matches[0].queue_name.as_deref(), Some("Ranked Solo/Duo"));
        assert_eq!(matches[0].game_duration_seconds, Some(1880));
        assert_eq!(matches[0].kda, Some(15.0));
        assert_eq!(matches[0].result, MatchResult::Win);
    }

    #[test]
    fn completed_match_path_uses_post_match_detail_endpoint() {
        assert_eq!(
            completed_match_path(698521151),
            "/lol-match-history/v1/games/698521151"
        );
    }

    #[test]
    fn post_match_detail_prefers_full_detail_over_summary() {
        let summary = completed_match_with_participant_count(1);
        let detail = completed_match_with_participant_count(10);

        let selected = best_completed_match(summary, Some(detail));

        assert_eq!(selected.participants.len(), 10);
    }

    #[test]
    fn post_match_detail_keeps_summary_when_detail_is_not_richer() {
        let summary = completed_match_with_participant_count(1);
        let detail = completed_match_with_participant_count(1);

        let selected = best_completed_match(summary, Some(detail));

        assert_eq!(selected.participants.len(), 1);
    }

    #[test]
    fn optional_ranked_failure_returns_partial_snapshot() {
        let data = compose_self_data(
            sample_summoner(),
            Ok(vec![LcuChampionSummary {
                id: 103,
                name: "Ahri".to_string(),
            }]),
            Err(LcuRequestError::Unavailable),
            Ok(empty_match_history()),
        );

        assert_eq!(data.status.phase, LeagueClientPhase::PartialData);
        assert_eq!(data.summoner.unwrap().display_name, "Player");
        assert_eq!(data.data_warnings.len(), 1);
        assert_eq!(data.data_warnings[0].section, LeagueDataSection::Ranked);
    }

    #[test]
    fn malformed_optional_payload_does_not_drop_summoner() {
        let data = compose_self_data(
            sample_summoner(),
            Err(LcuRequestError::Unexpected),
            Ok(LcuRankedStats { queues: Vec::new() }),
            Err(LcuRequestError::Unexpected),
        );

        assert_eq!(data.status.phase, LeagueClientPhase::PartialData);
        assert!(data.summoner.is_some());
        assert_eq!(data.data_warnings.len(), 2);
        assert!(
            data.data_warnings
                .iter()
                .all(|warning| !warning.message.contains(TEST_LOCKFILE_VALUE))
        );
    }

    #[test]
    fn missing_override_path_reports_lockfile_missing() {
        let client = LocalLeagueClient::with_lockfile_path(Path::new("missing-lockfile-for-test"));

        assert!(matches!(
            client.discover_lockfile_path(),
            LockfileDiscovery::LockfileMissing
        ));
    }

    fn advisor_json(entries: &[String]) -> String {
        format!(
            r#"{{
                "formatVersion": 1,
                "source": "test-advisor-json",
                "patch": "26.08",
                "region": "KR",
                "queue": "RANKED_SOLO_5X5",
                "tier": "EMERALD_PLUS",
                "generatedAt": "2026-04-25T00:00:00Z",
                "champions": [{}]
            }}"#,
            entries.join(",")
        )
    }

    fn advisor_json_entry(champion_id: i64, champion_name: &str, lane: &str) -> String {
        format!(
            r#"{{
                "championId": {champion_id},
                "championName": "{champion_name}",
                "championAlias": "{champion_name}",
                "lane": "{lane}",
                "games": 1000,
                "winRate": 51.4,
                "pickRate": 10.2,
                "banRate": 8.0,
                "runes": {{
                    "primaryStyle": "Domination",
                    "primaryRunes": [{{ "id": 8112, "name": "Electrocute" }}],
                    "secondaryStyle": "Sorcery",
                    "secondaryRunes": [{{ "id": 8210, "name": "Transcendence" }}],
                    "statShards": ["Adaptive Force"]
                }},
                "summonerSpells": [
                    {{ "id": 4, "name": "Flash" }},
                    {{ "id": 14, "name": "Ignite" }}
                ],
                "skillOrder": {{
                    "maxOrder": ["Q", "W", "E"],
                    "earlyOrder": ["Q", "W", "E", "Q", "Q", "R"]
                }},
                "itemBuild": {{
                    "starter": [{{ "id": 1056, "name": "Doran's Ring" }}],
                    "core": [{{ "id": 6655, "name": "Luden's Companion" }}],
                    "boots": [{{ "id": 3020, "name": "Sorcerer's Shoes" }}],
                    "late": [{{ "id": 3089, "name": "Rabadon's Deathcap" }}],
                    "situational": [{{ "id": 3157, "name": "Zhonya's Hourglass" }}]
                }},
                "strongAgainst": [
                    {{ "championId": 777, "championName": "Yone", "note": "Punish Q3.", "winRateDelta": 2.1 }}
                ],
                "weakAgainst": [
                    {{ "championId": 238, "championName": "Zed", "note": "Respect level 6.", "winRateDelta": -1.8 }}
                ],
                "powerSpikes": [
                    {{ "timing": "6", "label": "Ultimate", "description": "Look for all-in windows." }}
                ],
                "laneAdvice": "Push waves before roaming.",
                "teamfightAdvice": "Enter after key cooldowns are used."
            }}"#
        )
    }

    fn sample_summoner() -> LcuSummoner {
        LcuSummoner {
            display_name: Some("Player".to_string()),
            game_name: None,
            tag_line: None,
            summoner_level: Some(100),
            profile_icon_id: Some(1),
            account_id: Some(55),
            summoner_id: Some(99),
            puuid: Some("self-puuid".to_string()),
        }
    }

    fn empty_match_history() -> LcuMatchHistoryResponse {
        LcuMatchHistoryResponse {
            games: Some(LcuGames { games: Vec::new() }),
        }
    }

    fn completed_match_with_participant_count(count: i64) -> application::LeagueCompletedMatch {
        application::LeagueCompletedMatch {
            game_id: 698521151,
            queue_name: Some("Ranked Solo/Duo".to_string()),
            played_at: Some("2026-04-19T01:21:32Z".to_string()),
            game_duration_seconds: Some(1807),
            result: MatchResult::Win,
            participants: (1..=count)
                .map(|participant_id| application::LeagueCompletedParticipant {
                    participant_id,
                    team_id: if participant_id <= 5 { 100 } else { 200 },
                    display_name: format!("Participant {participant_id}"),
                    player_puuid: None,
                    profile_icon_id: None,
                    champion_id: Some(103),
                    champion_name: "Ahri".to_string(),
                    role: None,
                    lane: None,
                    result: MatchResult::Win,
                    kills: 1,
                    deaths: 1,
                    assists: 1,
                    kda: Some(2.0),
                    cs: 100,
                    gold_earned: 10_000,
                    damage_to_champions: 10_000,
                    vision_score: 10,
                    items: Vec::new(),
                    runes: Vec::new(),
                    spells: Vec::new(),
                })
                .collect(),
        }
    }
}
