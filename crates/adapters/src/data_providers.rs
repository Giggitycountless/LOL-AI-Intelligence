//! Remote data providers: ranked champion stats and advisor data.
//! Parses JSON from remote URLs, validates, and normalizes into domain types.

use std::collections::HashSet;

use application::{AdvisorDataProvider, AdvisorDataRefreshInput, RankedChampionDataError, RankedChampionDataProvider, RankedChampionRefreshInput};
use domain::{AdvisorDataSnapshot, AdvisorItemBuild, AdvisorMatchup, AdvisorNamedRef, AdvisorPowerSpike, AdvisorRecord, AdvisorRunePage, AdvisorSkillOrder, RankedChampionDataSnapshot, RankedChampionLane, RankedChampionStat};
use reqwest::blocking::Client;
use serde::Deserialize;

use super::constants::*;
use crate::round_to_tenth;

#[derive(Debug, Clone)]
pub struct RemoteRankedChampionJsonProvider {
    default_url: Option<String>,
    http_client: Client,
}

impl RemoteRankedChampionJsonProvider {
    pub fn new(default_url: impl Into<String>) -> Self {
        Self {
            default_url: Some(default_url.into()),
            http_client: ranked_champion_http_client(),
        }
    }

    pub fn without_default_url() -> Self {
        Self {
            default_url: None,
            http_client: ranked_champion_http_client(),
        }
    }
}

impl RankedChampionDataProvider for RemoteRankedChampionJsonProvider {
    fn fetch_ranked_champion_snapshot(
        &self,
        input: RankedChampionRefreshInput,
    ) -> Result<RankedChampionDataSnapshot, RankedChampionDataError> {
        let url = input
            .url
            .or_else(|| self.default_url.clone())
            .ok_or_else(|| {
                RankedChampionDataError::InvalidData(
                    "Ranked champion data URL is required".to_string(),
                )
            })?;

        if !url.starts_with("https://") {
            return Err(RankedChampionDataError::InvalidData(
                "Ranked champion data URL must use HTTPS".to_string(),
            ));
        }

        let response = self.http_client.get(url).send().map_err(|error| {
            RankedChampionDataError::Unavailable(format!(
                "Ranked champion data could not be downloaded: {error}"
            ))
        })?;

        if !response.status().is_success() {
            return Err(RankedChampionDataError::Unavailable(format!(
                "Ranked champion data returned HTTP {}",
                response.status()
            )));
        }

        let body = response.text().map_err(|error| {
            RankedChampionDataError::Unavailable(format!(
                "Ranked champion data response could not be read: {error}"
            ))
        })?;

        parse_ranked_champion_snapshot_json(body.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct RemoteAdvisorJsonProvider {
    default_url: Option<String>,
    http_client: Client,
}

impl RemoteAdvisorJsonProvider {
    pub fn new(default_url: impl Into<String>) -> Self {
        Self {
            default_url: Some(default_url.into()),
            http_client: ranked_champion_http_client(),
        }
    }

    pub fn without_default_url() -> Self {
        Self {
            default_url: None,
            http_client: ranked_champion_http_client(),
        }
    }
}

impl AdvisorDataProvider for RemoteAdvisorJsonProvider {
    fn fetch_advisor_snapshot(
        &self,
        input: AdvisorDataRefreshInput,
    ) -> Result<AdvisorDataSnapshot, RankedChampionDataError> {
        let url = input
            .url
            .or_else(|| self.default_url.clone())
            .ok_or_else(|| {
                RankedChampionDataError::InvalidData("Advisor data URL is required".to_string())
            })?;

        if !url.starts_with("https://") {
            return Err(RankedChampionDataError::InvalidData(
                "Advisor data URL must use HTTPS".to_string(),
            ));
        }

        let response = self.http_client.get(url).send().map_err(|error| {
            RankedChampionDataError::Unavailable(format!(
                "Advisor data could not be downloaded: {error}"
            ))
        })?;

        if !response.status().is_success() {
            return Err(RankedChampionDataError::Unavailable(format!(
                "Advisor data returned HTTP {}",
                response.status()
            )));
        }

        let body = response.text().map_err(|error| {
            RankedChampionDataError::Unavailable(format!(
                "Advisor data response could not be read: {error}"
            ))
        })?;

        parse_advisor_snapshot_json(body.as_str())
    }
}

pub fn parse_advisor_snapshot_json(
    json: &str,
) -> Result<AdvisorDataSnapshot, RankedChampionDataError> {
    let document: AdvisorJsonDocument = serde_json::from_str(json).map_err(|error| {
        RankedChampionDataError::InvalidData(format!("Advisor data JSON is invalid: {error}"))
    })?;

    if document.format_version != ADVISOR_DATA_FORMAT_VERSION {
        return Err(RankedChampionDataError::InvalidData(format!(
            "Unsupported advisor data format version {}",
            document.format_version
        )));
    }

    if document.champions.is_empty() {
        return Err(RankedChampionDataError::InvalidData(
            "Advisor data must contain at least one champion".to_string(),
        ));
    }

    let mut seen_records = HashSet::new();
    let mut records = Vec::with_capacity(document.champions.len());
    for champion in document.champions {
        let record = normalize_advisor_entry(champion)?;
        let record_key = format!("{}:{}", record.champion_id, record.lane.as_str());
        if !seen_records.insert(record_key) {
            return Err(RankedChampionDataError::InvalidData(
                "Advisor data contains duplicate champion/lane entries".to_string(),
            ));
        }
        records.push(record);
    }

    Ok(AdvisorDataSnapshot {
        source: optional_non_empty(document.source)
            .unwrap_or_else(|| "remoteAdvisorJson".to_string()),
        patch: optional_non_empty(document.patch),
        region: optional_non_empty(document.region),
        queue: optional_non_empty(document.queue),
        tier: optional_non_empty(document.tier),
        generated_at: optional_non_empty(document.generated_at),
        imported_at: unix_timestamp_seconds(),
        records,
    })
}

pub fn parse_ranked_champion_snapshot_json(
    json: &str,
) -> Result<RankedChampionDataSnapshot, RankedChampionDataError> {
    let document: RankedChampionJsonDocument = serde_json::from_str(json).map_err(|error| {
        RankedChampionDataError::InvalidData(format!(
            "Ranked champion data JSON is invalid: {error}"
        ))
    })?;

    if document.format_version != RANKED_CHAMPION_FORMAT_VERSION {
        return Err(RankedChampionDataError::InvalidData(format!(
            "Unsupported ranked champion data format version {}",
            document.format_version
        )));
    }

    if document.champions.is_empty() {
        return Err(RankedChampionDataError::InvalidData(
            "Ranked champion data must contain at least one champion".to_string(),
        ));
    }

    let mut records = Vec::with_capacity(document.champions.len());
    let mut seen_records = HashSet::new();
    for champion in document.champions {
        let record = normalize_ranked_champion_entry(champion)?;
        let record_key = format!("{}:{}", record.champion_id, record.lane.as_str());

        if !seen_records.insert(record_key) {
            return Err(RankedChampionDataError::InvalidData(
                "Ranked champion data contains duplicate champion/lane entries".to_string(),
            ));
        }

        records.push(record);
    }

    Ok(RankedChampionDataSnapshot {
        source: optional_non_empty(document.source).unwrap_or_else(|| "remoteJson".to_string()),
        patch: optional_non_empty(document.patch),
        region: optional_non_empty(document.region),
        queue: optional_non_empty(document.queue),
        tier: optional_non_empty(document.tier),
        generated_at: optional_non_empty(document.generated_at),
        imported_at: unix_timestamp_seconds(),
        records,
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RankedChampionJsonDocument {
    format_version: i64,
    source: Option<String>,
    patch: Option<String>,
    region: Option<String>,
    queue: Option<String>,
    tier: Option<String>,
    generated_at: Option<String>,
    champions: Vec<RankedChampionJsonEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RankedChampionJsonEntry {
    champion_id: i64,
    champion_name: String,
    champion_alias: Option<String>,
    lane: String,
    games: i64,
    wins: Option<i64>,
    picks: Option<i64>,
    bans: Option<i64>,
    win_rate: f64,
    pick_rate: f64,
    ban_rate: f64,
    overall_score: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdvisorJsonDocument {
    format_version: i64,
    source: Option<String>,
    patch: Option<String>,
    region: Option<String>,
    queue: Option<String>,
    tier: Option<String>,
    generated_at: Option<String>,
    champions: Vec<AdvisorJsonEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdvisorJsonEntry {
    champion_id: i64,
    champion_name: String,
    champion_alias: Option<String>,
    lane: String,
    games: i64,
    win_rate: f64,
    pick_rate: f64,
    ban_rate: f64,
    overall_score: Option<f64>,
    runes: AdvisorRunePage,
    summoner_spells: Vec<AdvisorNamedRef>,
    skill_order: AdvisorSkillOrder,
    item_build: AdvisorItemBuild,
    strong_against: Vec<AdvisorMatchup>,
    weak_against: Vec<AdvisorMatchup>,
    power_spikes: Vec<AdvisorPowerSpike>,
    lane_advice: String,
    teamfight_advice: String,
}

fn normalize_advisor_entry(
    entry: AdvisorJsonEntry,
) -> Result<AdvisorRecord, RankedChampionDataError> {
    if entry.champion_id <= 0 {
        return Err(RankedChampionDataError::InvalidData(
            "Advisor champion id must be positive".to_string(),
        ));
    }
    if entry.games < 0 {
        return Err(RankedChampionDataError::InvalidData(
            "Advisor champion games must not be negative".to_string(),
        ));
    }

    let champion_name = optional_non_empty(Some(entry.champion_name)).ok_or_else(|| {
        RankedChampionDataError::InvalidData("Advisor champion name is required".to_string())
    })?;
    let lane = ranked_lane_from_remote(entry.lane.as_str()).ok_or_else(|| {
        RankedChampionDataError::InvalidData(format!("Advisor lane is invalid: {}", entry.lane))
    })?;
    validate_rate(entry.win_rate, "winRate")?;
    validate_rate(entry.pick_rate, "pickRate")?;
    validate_rate(entry.ban_rate, "banRate")?;
    let overall_score = entry
        .overall_score
        .unwrap_or_else(|| ranked_overall_score(entry.win_rate, entry.pick_rate, entry.ban_rate));
    validate_rate(overall_score, "overallScore")?;
    validate_advisor_named_refs(&entry.summoner_spells, "summonerSpells")?;
    validate_advisor_named_refs(&entry.item_build.starter, "itemBuild.starter")?;
    validate_advisor_named_refs(&entry.item_build.core, "itemBuild.core")?;
    validate_advisor_named_refs(&entry.item_build.boots, "itemBuild.boots")?;
    validate_advisor_named_refs(&entry.item_build.late, "itemBuild.late")?;
    validate_advisor_named_refs(&entry.item_build.situational, "itemBuild.situational")?;
    validate_string_list(&entry.skill_order.max_order, "skillOrder.maxOrder")?;
    validate_string_list(&entry.skill_order.early_order, "skillOrder.earlyOrder")?;
    if optional_non_empty(Some(entry.runes.primary_style.clone())).is_none()
        || optional_non_empty(Some(entry.runes.secondary_style.clone())).is_none()
    {
        return Err(RankedChampionDataError::InvalidData(
            "Advisor rune styles are required".to_string(),
        ));
    }
    validate_advisor_named_refs(&entry.runes.primary_runes, "runes.primaryRunes")?;
    validate_advisor_named_refs(&entry.runes.secondary_runes, "runes.secondaryRunes")?;
    validate_string_list(&entry.runes.stat_shards, "runes.statShards")?;
    validate_matchups(&entry.strong_against, "strongAgainst")?;
    validate_matchups(&entry.weak_against, "weakAgainst")?;
    validate_power_spikes(&entry.power_spikes)?;
    let lane_advice = optional_non_empty(Some(entry.lane_advice)).ok_or_else(|| {
        RankedChampionDataError::InvalidData("Advisor laneAdvice is required".to_string())
    })?;
    let teamfight_advice = optional_non_empty(Some(entry.teamfight_advice)).ok_or_else(|| {
        RankedChampionDataError::InvalidData("Advisor teamfightAdvice is required".to_string())
    })?;

    Ok(AdvisorRecord {
        champion_id: entry.champion_id,
        champion_name,
        champion_alias: optional_non_empty(entry.champion_alias),
        lane,
        win_rate: round_to_tenth(entry.win_rate),
        pick_rate: round_to_tenth(entry.pick_rate),
        ban_rate: round_to_tenth(entry.ban_rate),
        overall_score: round_to_tenth(overall_score),
        games: entry.games,
        runes: entry.runes,
        summoner_spells: entry.summoner_spells,
        skill_order: entry.skill_order,
        item_build: entry.item_build,
        strong_against: entry.strong_against,
        weak_against: entry.weak_against,
        power_spikes: entry.power_spikes,
        lane_advice,
        teamfight_advice,
    })
}

fn validate_advisor_named_refs(
    refs: &[AdvisorNamedRef],
    label: &str,
) -> Result<(), RankedChampionDataError> {
    if refs.is_empty() {
        return Err(RankedChampionDataError::InvalidData(format!(
            "Advisor {label} must not be empty"
        )));
    }
    for value in refs {
        if value.id.is_some_and(|id| id <= 0) || value.name.trim().is_empty() {
            return Err(RankedChampionDataError::InvalidData(format!(
                "Advisor {label} contains an invalid entry"
            )));
        }
    }
    Ok(())
}

fn validate_string_list(values: &[String], label: &str) -> Result<(), RankedChampionDataError> {
    if values.is_empty() || values.iter().any(|value| value.trim().is_empty()) {
        return Err(RankedChampionDataError::InvalidData(format!(
            "Advisor {label} must contain non-empty values"
        )));
    }
    Ok(())
}

fn validate_matchups(
    matchups: &[AdvisorMatchup],
    label: &str,
) -> Result<(), RankedChampionDataError> {
    for matchup in matchups {
        if matchup.champion_id <= 0
            || matchup.champion_name.trim().is_empty()
            || matchup.note.trim().is_empty()
        {
            return Err(RankedChampionDataError::InvalidData(format!(
                "Advisor {label} contains an invalid matchup"
            )));
        }
    }
    Ok(())
}

fn validate_power_spikes(spikes: &[AdvisorPowerSpike]) -> Result<(), RankedChampionDataError> {
    if spikes.is_empty()
        || spikes.iter().any(|spike| {
            spike.timing.trim().is_empty()
                || spike.label.trim().is_empty()
                || spike.description.trim().is_empty()
        })
    {
        return Err(RankedChampionDataError::InvalidData(
            "Advisor powerSpikes must contain non-empty values".to_string(),
        ));
    }
    Ok(())
}

fn normalize_ranked_champion_entry(
    entry: RankedChampionJsonEntry,
) -> Result<RankedChampionStat, RankedChampionDataError> {
    if entry.champion_id <= 0 {
        return Err(RankedChampionDataError::InvalidData(
            "Ranked champion id must be positive".to_string(),
        ));
    }

    let champion_name = optional_non_empty(Some(entry.champion_name)).ok_or_else(|| {
        RankedChampionDataError::InvalidData("Ranked champion name is required".to_string())
    })?;
    let lane = ranked_lane_from_remote(entry.lane.as_str()).ok_or_else(|| {
        RankedChampionDataError::InvalidData(format!(
            "Ranked champion lane is invalid: {}",
            entry.lane
        ))
    })?;

    validate_rate(entry.win_rate, "winRate")?;
    validate_rate(entry.pick_rate, "pickRate")?;
    validate_rate(entry.ban_rate, "banRate")?;

    if entry.games < 0 {
        return Err(RankedChampionDataError::InvalidData(
            "Ranked champion games must not be negative".to_string(),
        ));
    }

    let wins = entry
        .wins
        .unwrap_or_else(|| ((entry.games as f64) * (entry.win_rate / 100.0)).round() as i64);
    let picks = entry.picks.unwrap_or(entry.games);
    let bans = entry
        .bans
        .unwrap_or_else(|| ((entry.games as f64) * (entry.ban_rate / 100.0)).round() as i64);
    for (label, value) in [("wins", wins), ("picks", picks), ("bans", bans)] {
        if value < 0 {
            return Err(RankedChampionDataError::InvalidData(format!(
                "Ranked champion {label} must not be negative"
            )));
        }
    }

    let overall_score = entry
        .overall_score
        .unwrap_or_else(|| ranked_overall_score(entry.win_rate, entry.pick_rate, entry.ban_rate));
    validate_rate(overall_score, "overallScore")?;

    if wins > entry.games {
        return Err(RankedChampionDataError::InvalidData(
            "Ranked champion wins must not exceed games".to_string(),
        ));
    }

    Ok(RankedChampionStat {
        champion_id: entry.champion_id,
        champion_name,
        champion_alias: optional_non_empty(entry.champion_alias),
        lane,
        win_rate: round_to_tenth(entry.win_rate),
        pick_rate: round_to_tenth(entry.pick_rate),
        ban_rate: round_to_tenth(entry.ban_rate),
        overall_score: round_to_tenth(overall_score),
        games: entry.games,
        wins,
        picks,
        bans,
    })
}

fn ranked_lane_from_remote(value: &str) -> Option<RankedChampionLane> {
    match value.trim().to_ascii_lowercase().as_str() {
        "top" => Some(RankedChampionLane::Top),
        "jungle" | "jug" => Some(RankedChampionLane::Jungle),
        "middle" | "mid" => Some(RankedChampionLane::Middle),
        "bottom" | "bot" | "adc" => Some(RankedChampionLane::Bottom),
        "support" | "sup" => Some(RankedChampionLane::Support),
        _ => None,
    }
}

fn validate_rate(value: f64, label: &str) -> Result<(), RankedChampionDataError> {
    if !(0.0..=100.0).contains(&value) || !value.is_finite() {
        return Err(RankedChampionDataError::InvalidData(format!(
            "Ranked champion {label} must be between 0 and 100"
        )));
    }

    Ok(())
}

fn ranked_overall_score(win_rate: f64, pick_rate: f64, ban_rate: f64) -> f64 {
    round_to_tenth((win_rate * 0.55) + (pick_rate * 0.25) + (ban_rate * 0.20))
}

fn ranked_champion_http_client() -> Client {
    match Client::builder()
        .timeout(RANKED_CHAMPION_REMOTE_TIMEOUT)
        .connect_timeout(RANKED_CHAMPION_REMOTE_TIMEOUT)
        .build()
    {
        Ok(client) => client,
        Err(_) => Client::new(),
    }
}

fn optional_non_empty(value: Option<String>) -> Option<String> {
    let value = value?;
    let trimmed = value.trim();

    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub(crate) fn unix_timestamp_seconds() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

// ── tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use application::RankedChampionDataError;
    use domain::RankedChampionLane;

    // ── validate_rate ────────────────────────────────────────────────────

    #[test]
    fn validate_rate_accepts_zero() {
        assert!(validate_rate(0.0, "winRate").is_ok());
    }

    #[test]
    fn validate_rate_accepts_one_hundred() {
        assert!(validate_rate(100.0, "pickRate").is_ok());
    }

    #[test]
    fn validate_rate_accepts_mid_range() {
        assert!(validate_rate(52.3, "banRate").is_ok());
    }

    #[test]
    fn validate_rate_rejects_negative() {
        let err = validate_rate(-0.1, "winRate").unwrap_err();
        assert!(matches!(err, RankedChampionDataError::InvalidData(_)));
        assert!(err.to_string().contains("winRate"));
    }

    #[test]
    fn validate_rate_rejects_over_one_hundred() {
        let err = validate_rate(100.1, "pickRate").unwrap_err();
        assert!(matches!(err, RankedChampionDataError::InvalidData(_)));
    }

    #[test]
    fn validate_rate_rejects_nan() {
        let err = validate_rate(f64::NAN, "banRate").unwrap_err();
        assert!(matches!(err, RankedChampionDataError::InvalidData(_)));
    }

    #[test]
    fn validate_rate_rejects_infinity() {
        let err = validate_rate(f64::INFINITY, "overallScore").unwrap_err();
        assert!(matches!(err, RankedChampionDataError::InvalidData(_)));
    }

    // ── optional_non_empty ────────────────────────────────────────────────

    #[test]
    fn optional_non_empty_none_is_none() {
        assert_eq!(optional_non_empty(None), None);
    }

    #[test]
    fn optional_non_empty_empty_string_is_none() {
        assert_eq!(optional_non_empty(Some("".to_string())), None);
    }

    #[test]
    fn optional_non_empty_whitespace_only_is_none() {
        assert_eq!(optional_non_empty(Some("   ".to_string())), None);
        assert_eq!(optional_non_empty(Some("\n\t".to_string())), None);
    }

    #[test]
    fn optional_non_empty_keeps_value() {
        assert_eq!(
            optional_non_empty(Some("hello".to_string())),
            Some("hello".to_string())
        );
    }

    #[test]
    fn optional_non_empty_trims_whitespace() {
        assert_eq!(
            optional_non_empty(Some("  world  ".to_string())),
            Some("world".to_string())
        );
    }

    #[test]
    fn optional_non_empty_whitespace_around_text() {
        assert_eq!(
            optional_non_empty(Some("\t  hello\n".to_string())),
            Some("hello".to_string())
        );
    }

    // ── ranked_lane_from_remote ───────────────────────────────────────────

    #[test]
    fn ranked_lane_parses_top() {
        assert_eq!(ranked_lane_from_remote("top"), Some(RankedChampionLane::Top));
    }

    #[test]
    fn ranked_lane_parses_jungle_and_jug() {
        assert_eq!(ranked_lane_from_remote("jungle"), Some(RankedChampionLane::Jungle));
        assert_eq!(ranked_lane_from_remote("jug"), Some(RankedChampionLane::Jungle));
    }

    #[test]
    fn ranked_lane_parses_middle_and_mid() {
        assert_eq!(ranked_lane_from_remote("middle"), Some(RankedChampionLane::Middle));
        assert_eq!(ranked_lane_from_remote("mid"), Some(RankedChampionLane::Middle));
    }

    #[test]
    fn ranked_lane_parses_bottom_bot_and_adc() {
        assert_eq!(ranked_lane_from_remote("bottom"), Some(RankedChampionLane::Bottom));
        assert_eq!(ranked_lane_from_remote("bot"), Some(RankedChampionLane::Bottom));
        assert_eq!(ranked_lane_from_remote("adc"), Some(RankedChampionLane::Bottom));
    }

    #[test]
    fn ranked_lane_parses_support_and_sup() {
        assert_eq!(ranked_lane_from_remote("support"), Some(RankedChampionLane::Support));
        assert_eq!(ranked_lane_from_remote("sup"), Some(RankedChampionLane::Support));
    }

    #[test]
    fn ranked_lane_is_case_insensitive() {
        assert_eq!(ranked_lane_from_remote("TOP"), Some(RankedChampionLane::Top));
        assert_eq!(ranked_lane_from_remote("JuNgLe"), Some(RankedChampionLane::Jungle));
        assert_eq!(ranked_lane_from_remote("MiD"), Some(RankedChampionLane::Middle));
    }

    #[test]
    fn ranked_lane_trims_whitespace() {
        assert_eq!(ranked_lane_from_remote("  top  "), Some(RankedChampionLane::Top));
        assert_eq!(ranked_lane_from_remote("\tmid\n"), Some(RankedChampionLane::Middle));
    }

    #[test]
    fn ranked_lane_rejects_unknown() {
        assert_eq!(ranked_lane_from_remote("aram"), None);
        assert_eq!(ranked_lane_from_remote("carry"), None);
    }

    #[test]
    fn ranked_lane_rejects_empty() {
        assert_eq!(ranked_lane_from_remote(""), None);
        assert_eq!(ranked_lane_from_remote("   "), None);
    }

    // ── parse_ranked_champion_snapshot_json ──────────────────────────────

    fn ranked_champion_json(champions: &str) -> String {
        format!(
            r#"{{"formatVersion":1,"source":"test","patch":"26.08","region":"KR","queue":"RANKED_SOLO_5X5","tier":"EMERALD_PLUS","generatedAt":"2026-05-16T00:00:00Z","champions":[{champions}]}}"#
        )
    }

    fn ranked_champion_entry_json(champion_id: i64, champion_name: &str, lane: &str) -> String {
        format!(
            r#"{{"championId":{champion_id},"championName":"{champion_name}","lane":"{lane}","games":1000,"winRate":51.4,"pickRate":10.2,"banRate":8.0}}"#
        )
    }

    #[test]
    fn parse_ranked_valid_json() {
        let json = ranked_champion_json(&ranked_champion_entry_json(103, "Ahri", "mid"));
        let result = parse_ranked_champion_snapshot_json(&json);
        assert!(result.is_ok(), "expected ok, got {result:?}");
        let snapshot = result.unwrap();
        assert_eq!(snapshot.records.len(), 1);
        assert_eq!(snapshot.records[0].champion_id, 103);
        assert_eq!(snapshot.records[0].champion_name, "Ahri");
        assert_eq!(snapshot.records[0].lane, RankedChampionLane::Middle);
    }

    #[test]
    fn parse_ranked_rejects_malformed_json() {
        let result = parse_ranked_champion_snapshot_json("not json at all");
        assert!(matches!(result, Err(RankedChampionDataError::InvalidData(_))));
    }

    #[test]
    fn parse_ranked_rejects_wrong_format_version() {
        let json = r#"{"formatVersion":999,"source":"t","champions":[]}"#;
        let result = parse_ranked_champion_snapshot_json(json);
        assert!(matches!(result, Err(RankedChampionDataError::InvalidData(_))));
    }

    #[test]
    fn parse_ranked_rejects_empty_champions() {
        let json = r#"{"formatVersion":1,"source":"t","champions":[]}"#;
        let result = parse_ranked_champion_snapshot_json(json);
        assert!(matches!(result, Err(RankedChampionDataError::InvalidData(_))));
    }

    #[test]
    fn parse_ranked_rejects_duplicate_champion_lane() {
        let entry = ranked_champion_entry_json(103, "Ahri", "mid");
        let json = ranked_champion_json(&format!("{entry},{entry}"));
        let result = parse_ranked_champion_snapshot_json(&json);
        assert!(matches!(result, Err(RankedChampionDataError::InvalidData(_))));
    }

    #[test]
    fn parse_ranked_rejects_negative_games() {
        let json = ranked_champion_json(
            r#"{"championId":1,"championName":"Ahri","lane":"mid","games":-1,"winRate":50.0,"pickRate":10.0,"banRate":5.0}"#,
        );
        let result = parse_ranked_champion_snapshot_json(&json);
        assert!(matches!(result, Err(RankedChampionDataError::InvalidData(_))));
    }

    #[test]
    fn parse_ranked_rejects_rate_out_of_range() {
        let json = ranked_champion_json(
            r#"{"championId":1,"championName":"Ahri","lane":"mid","games":100,"winRate":101.0,"pickRate":10.0,"banRate":5.0}"#,
        );
        let result = parse_ranked_champion_snapshot_json(&json);
        assert!(matches!(result, Err(RankedChampionDataError::InvalidData(_))));
    }

    #[test]
    fn parse_ranked_rejects_missing_champion_name() {
        let json = ranked_champion_json(
            r#"{"championId":1,"championName":"","lane":"mid","games":100,"winRate":50.0,"pickRate":10.0,"banRate":5.0}"#,
        );
        let result = parse_ranked_champion_snapshot_json(&json);
        assert!(matches!(result, Err(RankedChampionDataError::InvalidData(_))));
    }

    // ── parse_advisor_snapshot_json ───────────────────────────────────────

    fn advisor_json(champions: &str) -> String {
        format!(
            r#"{{"formatVersion":1,"source":"test","patch":"26.08","region":"KR","queue":"RANKED_SOLO_5X5","tier":"EMERALD_PLUS","generatedAt":"2026-05-16T00:00:00Z","champions":[{champions}]}}"#
        )
    }

    fn advisor_entry_json(champion_id: i64, champion_name: &str, lane: &str) -> String {
        format!(
            r#"{{"championId":{champion_id},"championName":"{champion_name}","lane":"{lane}","games":1000,"winRate":51.4,"pickRate":10.2,"banRate":8.0,"runes":{{"primaryStyle":"Precision","secondaryStyle":"Domination","primaryRunes":[{{"id":8008,"name":"Lethal Tempo"}}],"secondaryRunes":[{{"id":9101,"name":"Triumph"}}],"statShards":["Attack Speed","Adaptive","Health"]}},"summonerSpells":[{{"id":6,"name":"Ghost"}}],"skillOrder":{{"maxOrder":["Q","E","W"],"earlyOrder":["Q","E","Q"]}},"itemBuild":{{"starter":[{{"id":1054,"name":"Doran's Blade"}}],"core":[{{"id":3031,"name":"Infinity Edge"}}],"boots":[{{"id":3006,"name":"Berserker's Greaves"}}],"late":[{{"id":3094,"name":"Rapid Firecannon"}}],"situational":[{{"id":3026,"name":"Guardian Angel"}}]}},"strongAgainst":[{{"championId":1,"championName":"Annie","note":"Easy"}}],"weakAgainst":[{{"championId":2,"championName":"Olaf","note":"Hard"}}],"powerSpikes":[{{"timing":"Level 6","label":"Spike","description":"Unlocks ultimate"}}],"laneAdvice":"Play safe early","teamfightAdvice":"Focus carries"}}"#
        )
    }

    #[test]
    fn parse_advisor_valid_json() {
        let json = advisor_json(&advisor_entry_json(103, "Ahri", "mid"));
        let result = parse_advisor_snapshot_json(&json);
        assert!(result.is_ok(), "expected ok, got {result:?}");
        let snapshot = result.unwrap();
        assert_eq!(snapshot.records.len(), 1);
        assert_eq!(snapshot.records[0].champion_id, 103);
        assert_eq!(snapshot.records[0].champion_name, "Ahri");
    }

    #[test]
    fn parse_advisor_rejects_malformed_json() {
        let result = parse_advisor_snapshot_json("garbage");
        assert!(matches!(result, Err(RankedChampionDataError::InvalidData(_))));
    }

    #[test]
    fn parse_advisor_rejects_wrong_format_version() {
        let json = r#"{"formatVersion":999,"source":"t","champions":[]}"#;
        let result = parse_advisor_snapshot_json(json);
        assert!(matches!(result, Err(RankedChampionDataError::InvalidData(_))));
    }

    #[test]
    fn parse_advisor_rejects_empty_champions() {
        let json = r#"{"formatVersion":1,"source":"t","champions":[]}"#;
        let result = parse_advisor_snapshot_json(json);
        assert!(matches!(result, Err(RankedChampionDataError::InvalidData(_))));
    }
}
