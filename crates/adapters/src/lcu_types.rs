//! LCU API response types — champ select, gameflow, summoner.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuSummoner {
    pub(crate) display_name: Option<String>,
    pub(crate) game_name: Option<String>,
    pub(crate) tag_line: Option<String>,
    pub(crate) summoner_level: Option<i64>,
    pub(crate) profile_icon_id: Option<i64>,
    pub(crate) account_id: Option<i64>,
    pub(crate) summoner_id: Option<i64>,
    pub(crate) puuid: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuChampSelectSession {
    pub(crate) local_player_cell_id: Option<i64>,
    #[serde(default)]
    pub(crate) actions: Vec<Vec<LcuChampSelectAction>>,
    #[serde(default)]
    pub(crate) my_team: Vec<LcuChampSelectMember>,
    #[serde(default)]
    pub(crate) their_team: Vec<LcuChampSelectMember>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuGameflowSession {
    pub(crate) game_data: Option<LcuGameflowGameData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuGameflowGameData {
    #[serde(default)]
    pub(crate) team_one: Vec<LcuGameflowParticipant>,
    #[serde(default)]
    pub(crate) team_two: Vec<LcuGameflowParticipant>,
    #[serde(default)]
    pub(crate) player_champion_selections: Vec<LcuGameflowChampionSelection>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuGameflowParticipant {
    pub(crate) summoner_id: Option<i64>,
    pub(crate) puuid: Option<String>,
    pub(crate) summoner_name: Option<String>,
    pub(crate) display_name: Option<String>,
    pub(crate) game_name: Option<String>,
    pub(crate) tag_line: Option<String>,
    pub(crate) champion_id: Option<i64>,
    pub(crate) selected_champion_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuGameflowChampionSelection {
    pub(crate) summoner_id: Option<i64>,
    pub(crate) puuid: Option<String>,
    pub(crate) champion_id: Option<i64>,
    pub(crate) selected_champion_id: Option<i64>,
    pub(crate) team_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuChampSelectAction {
    pub(crate) id: Option<i64>,
    pub(crate) actor_cell_id: Option<i64>,
    pub(crate) champion_id: Option<i64>,
    pub(crate) completed: Option<bool>,
    pub(crate) is_ally_action: Option<bool>,
    #[serde(rename = "type")]
    pub(crate) action_type: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuChampSelectActionUpdate {
    pub(crate) champion_id: i64,
    pub(crate) completed: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuChampSelectMember {
    pub(crate) summoner_id: Option<i64>,
    pub(crate) champion_id: Option<i64>,
    pub(crate) puuid: Option<String>,
    pub(crate) summoner_name: Option<String>,
    pub(crate) display_name: Option<String>,
    pub(crate) game_name: Option<String>,
    pub(crate) tag_line: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuSummonerBatch {
    pub(crate) puuid: Option<String>,
    pub(crate) summoner_id: Option<i64>,
    pub(crate) display_name: Option<String>,
    pub(crate) game_name: Option<String>,
    pub(crate) tag_line: Option<String>,
    pub(crate) summoner_level: Option<i64>,
}

/// Response from `/lol-ranked/v2/ranked-stats/{puuid}`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuPlayerRankedStats {
    #[serde(default)]
    pub(crate) queues: Vec<LcuRankedQueue>,
}

/// Response from `/lol-champion-mastery/v1/{puuid}/champion-mastery/{championId}`
/// (single-champion mastery for one player).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuChampionMasteryEntry {
    pub(crate) champion_level: Option<i64>,
}

// ── Match history / participant types ──

#[derive(Debug, Deserialize)]
pub(crate) struct LcuMatchHistoryResponse {
    pub(crate) games: Option<LcuGames>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LcuGames {
    #[serde(default)]
    pub(crate) games: Vec<LcuGame>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuGame {
    pub(crate) game_id: Option<i64>,
    pub(crate) game_creation_date: Option<String>,
    #[serde(rename = "gameCreation")]
    pub(crate) game_creation: Option<Value>,
    pub(crate) game_duration: Option<i64>,
    pub(crate) queue_id: Option<i64>,
    #[serde(default)]
    pub(crate) participants: Vec<LcuParticipant>,
    #[serde(default)]
    pub(crate) participant_identities: Vec<LcuParticipantIdentity>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuParticipant {
    pub(crate) participant_id: Option<i64>,
    pub(crate) team_id: Option<i64>,
    pub(crate) champion_id: Option<i64>,
    pub(crate) champion_name: Option<String>,
    pub(crate) spell1_id: Option<i64>,
    pub(crate) spell2_id: Option<i64>,
    pub(crate) stats: Option<LcuParticipantStats>,
    pub(crate) timeline: Option<LcuParticipantTimeline>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuParticipantStats {
    pub(crate) kills: Option<i64>,
    pub(crate) deaths: Option<i64>,
    pub(crate) assists: Option<i64>,
    pub(crate) win: Option<bool>,
    pub(crate) total_minions_killed: Option<i64>,
    pub(crate) neutral_minions_killed: Option<i64>,
    pub(crate) gold_earned: Option<i64>,
    pub(crate) total_damage_dealt_to_champions: Option<i64>,
    pub(crate) physical_damage_dealt_to_champions: Option<i64>,
    pub(crate) magic_damage_dealt_to_champions: Option<i64>,
    pub(crate) true_damage_dealt_to_champions: Option<i64>,
    pub(crate) damage_dealt_to_objectives: Option<i64>,
    pub(crate) damage_dealt_to_turrets: Option<i64>,
    pub(crate) total_damage_taken: Option<i64>,
    pub(crate) vision_score: Option<i64>,
    pub(crate) wards_placed: Option<i64>,
    pub(crate) wards_killed: Option<i64>,
    pub(crate) vision_wards_bought_in_game: Option<i64>,
    pub(crate) total_time_spent_dead: Option<i64>,
    pub(crate) largest_killing_spree: Option<i64>,
    pub(crate) largest_multi_kill: Option<i64>,
    pub(crate) double_kills: Option<i64>,
    pub(crate) triple_kills: Option<i64>,
    pub(crate) quadra_kills: Option<i64>,
    pub(crate) penta_kills: Option<i64>,
    pub(crate) first_blood_kill: Option<bool>,
    pub(crate) first_blood_assist: Option<bool>,
    pub(crate) first_tower_kill: Option<bool>,
    pub(crate) first_tower_assist: Option<bool>,
    pub(crate) item0: Option<i64>,
    pub(crate) item1: Option<i64>,
    pub(crate) item2: Option<i64>,
    pub(crate) item3: Option<i64>,
    pub(crate) item4: Option<i64>,
    pub(crate) item5: Option<i64>,
    pub(crate) item6: Option<i64>,
    pub(crate) perk0: Option<i64>,
    pub(crate) perk1: Option<i64>,
    pub(crate) perk2: Option<i64>,
    pub(crate) perk3: Option<i64>,
    pub(crate) perk4: Option<i64>,
    pub(crate) perk5: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuParticipantTimeline {
    pub(crate) role: Option<String>,
    pub(crate) lane: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuParticipantIdentity {
    pub(crate) participant_id: Option<i64>,
    pub(crate) player: Option<LcuPlayer>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuPlayer {
    pub(crate) summoner_name: Option<String>,
    pub(crate) game_name: Option<String>,
    pub(crate) tag_line: Option<String>,
    pub(crate) summoner_id: Option<i64>,
    pub(crate) account_id: Option<i64>,
    pub(crate) current_account_id: Option<i64>,
    pub(crate) profile_icon: Option<i64>,
    pub(crate) profile_icon_id: Option<i64>,
    pub(crate) puuid: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LcuGameAssetMetadata {
    pub(crate) name: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) icon_path: Option<String>,
}

// ── Champion data types ──

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuRankedStats {
    #[serde(default)]
    pub(crate) queues: Vec<LcuRankedQueue>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuRankedQueue {
    pub(crate) queue_type: Option<String>,
    pub(crate) tier: Option<String>,
    pub(crate) division: Option<String>,
    pub(crate) league_points: Option<i64>,
    pub(crate) wins: Option<i64>,
    pub(crate) losses: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LcuChampionSummary {
    pub(crate) id: i64,
    pub(crate) name: String,
    /// Language-independent champion key used by CDragon (e.g. "Sett", "Ahri").
    /// Always English regardless of client language setting.
    pub(crate) alias: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuChampionDetails {
    pub(crate) name: Option<String>,
    pub(crate) title: Option<String>,
    pub(crate) square_portrait_path: Option<String>,
    pub(crate) passive: Option<LcuChampionAbility>,
    #[serde(default)]
    pub(crate) spells: Vec<LcuChampionAbility>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuChampionAbility {
    pub(crate) name: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) dynamic_description: Option<String>,
    pub(crate) ability_icon_path: Option<String>,
    pub(crate) spell_key: Option<String>,
    pub(crate) cooldown: Option<Value>,
    pub(crate) cost: Option<Value>,
    pub(crate) range: Option<Value>,
    #[serde(default)]
    pub(crate) cooldown_coefficients: Vec<f64>,
    #[serde(default)]
    pub(crate) cost_coefficients: Vec<f64>,
}

use crate::non_empty;
use crate::{ids_match, strings_match};

impl LcuChampSelectMember {
    pub(crate) fn display_name(&self) -> Option<String> {
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

use domain::CurrentSummonerProfile;

impl LcuSummoner {
    pub(crate) fn profile(
        &self,
        honor_level: Option<i64>,
        top_mastery: Vec<domain::ChampionMasteryEntry>,
    ) -> CurrentSummonerProfile {
        CurrentSummonerProfile {
            display_name: self.display_name(),
            summoner_level: self.summoner_level.unwrap_or_default(),
            profile_icon_id: self.profile_icon_id,
            honor_level,
            top_mastery,
        }
    }

    pub(crate) fn display_name(&self) -> String {
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

    pub(crate) fn matches_player(&self, player: &LcuPlayer) -> bool {
        ids_match(self.summoner_id, player.summoner_id)
            || ids_match(self.account_id, player.account_id)
            || ids_match(self.account_id, player.current_account_id)
            || strings_match(self.puuid.as_deref(), player.puuid.as_deref())
    }
}

impl LcuParticipantStats {
    pub(crate) fn items(&self) -> Vec<i64> {
        [
            self.item0, self.item1, self.item2, self.item3, self.item4, self.item5, self.item6,
        ]
        .into_iter()
        .flatten()
        .filter(|value| *value > 0)
        .collect()
    }

    pub(crate) fn runes(&self) -> Vec<i64> {
        [
            self.perk0, self.perk1, self.perk2, self.perk3, self.perk4, self.perk5,
        ]
            .into_iter()
            .flatten()
            .filter(|value| *value > 0)
            .collect()
    }
}

// ── LCU Honor / Mastery types ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuHonorProfile {
    pub(crate) honor_level: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuChampionMastery {
    pub(crate) champion_id: Option<i64>,
    pub(crate) champion_level: Option<i64>,
    pub(crate) champion_points: Option<i64>,
}

// ── LCU Perks types ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuRunePage {
    pub(crate) id: Option<i64>,
    #[serde(default)]
    pub(crate) is_deletable: bool,
    #[serde(default)]
    pub(crate) is_temporary: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LcuRunePageCreate {
    pub(crate) name: String,
    pub(crate) primary_style_id: i64,
    pub(crate) sub_style_id: i64,
    pub(crate) selected_perk_ids: Vec<i64>,
}

// ── tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── LcuChampSelectMember::display_name ────────────────────────────────

    fn member_with_game_tag(game_name: &str, tag_line: &str) -> LcuChampSelectMember {
        LcuChampSelectMember {
            summoner_id: None,
            champion_id: None,
            puuid: None,
            summoner_name: Some("Old Summoner Name".into()),
            display_name: Some("Riot Display".into()),
            game_name: Some(game_name.into()),
            tag_line: Some(tag_line.into()),
        }
    }

    #[test]
    fn member_display_name_prefers_riot_id() {
        let member = member_with_game_tag("Faker", "KR1");
        assert_eq!(member.display_name().as_deref(), Some("Faker#KR1"));
    }

    #[test]
    fn member_display_name_game_only_no_tag() {
        let mut member = member_with_game_tag("HideOnBush", "KR1");
        member.tag_line = None;
        assert_eq!(member.display_name().as_deref(), Some("HideOnBush"));
    }

    #[test]
    fn member_display_name_falls_back_to_display_name() {
        let mut member = member_with_game_tag("", "");
        member.summoner_name = None;
        assert_eq!(member.display_name().as_deref(), Some("Riot Display"));
    }

    #[test]
    fn member_display_name_falls_back_to_summoner_name() {
        let mut member = member_with_game_tag("", "");
        member.display_name = None;
        assert_eq!(
            member.display_name().as_deref(),
            Some("Old Summoner Name")
        );
    }

    #[test]
    fn member_display_name_all_empty_returns_none() {
        let member = LcuChampSelectMember {
            summoner_id: None,
            champion_id: None,
            puuid: None,
            summoner_name: None,
            display_name: None,
            game_name: None,
            tag_line: None,
        };
        assert_eq!(member.display_name(), None);
    }

    // ── LcuSummoner::display_name ─────────────────────────────────────────

    fn summoner(display_name: Option<&str>, game_name: Option<&str>, tag_line: Option<&str>) -> LcuSummoner {
        LcuSummoner {
            display_name: display_name.map(str::to_string),
            game_name: game_name.map(str::to_string),
            tag_line: tag_line.map(str::to_string),
            summoner_level: Some(100),
            profile_icon_id: Some(42),
            account_id: Some(1),
            summoner_id: Some(2),
            puuid: Some("puuid-abc".into()),
        }
    }

    #[test]
    fn summoner_display_name_uses_display_name_first() {
        let s = summoner(Some("MyDisplay"), Some("Game"), Some("TAG"));
        assert_eq!(s.display_name(), "MyDisplay");
    }

    #[test]
    fn summoner_display_name_falls_back_to_riot_id() {
        let s = summoner(None, Some("Faker"), Some("KR1"));
        assert_eq!(s.display_name(), "Faker#KR1");
    }

    #[test]
    fn summoner_display_name_game_only() {
        let s = summoner(None, Some("HideOnBush"), None);
        assert_eq!(s.display_name(), "HideOnBush");
    }

    #[test]
    fn summoner_display_name_ultimate_fallback() {
        let s = summoner(None, None, None);
        assert_eq!(s.display_name(), "Current summoner");
    }

    #[test]
    fn summoner_display_name_empty_strings_are_ignored() {
        let s = summoner(Some(""), Some("  "), Some("\t"));
        assert_eq!(s.display_name(), "Current summoner");
    }

    // ── LcuSummoner::profile ──────────────────────────────────────────────

    #[test]
    fn summoner_profile_uses_display_name() {
        let s = summoner(Some("Test"), None, None);
        let profile = s.profile(None, Vec::new());
        assert_eq!(profile.display_name, "Test");
        assert_eq!(profile.summoner_level, 100);
        assert_eq!(profile.profile_icon_id, Some(42));
    }

    #[test]
    fn summoner_profile_default_level_zero() {
        let mut s = summoner(None, None, None);
        s.summoner_level = None;
        let profile = s.profile(None, Vec::new());
        assert_eq!(profile.summoner_level, 0);
    }

    // ── LcuSummoner::matches_player ───────────────────────────────────────

    #[test]
    fn summoner_matches_by_summoner_id() {
        let s = summoner(Some("A"), None, None);
        let player = LcuPlayer {
            summoner_name: None, game_name: None, tag_line: None,
            summoner_id: Some(2), account_id: None, current_account_id: None,
            profile_icon: None, profile_icon_id: None, puuid: None,
        };
        assert!(s.matches_player(&player));
    }

    #[test]
    fn summoner_matches_by_account_id() {
        let s = summoner(Some("A"), None, None);
        let player = LcuPlayer {
            summoner_name: None, game_name: None, tag_line: None,
            summoner_id: None, account_id: Some(1), current_account_id: None,
            profile_icon: None, profile_icon_id: None, puuid: None,
        };
        assert!(s.matches_player(&player));
    }

    #[test]
    fn summoner_matches_by_current_account_id() {
        let s = summoner(Some("A"), None, None);
        let player = LcuPlayer {
            summoner_name: None, game_name: None, tag_line: None,
            summoner_id: None, account_id: None, current_account_id: Some(1),
            profile_icon: None, profile_icon_id: None, puuid: None,
        };
        assert!(s.matches_player(&player));
    }

    #[test]
    fn summoner_matches_by_puuid() {
        let s = summoner(Some("A"), None, None);
        let player = LcuPlayer {
            summoner_name: None, game_name: None, tag_line: None,
            summoner_id: None, account_id: None, current_account_id: None,
            profile_icon: None, profile_icon_id: None, puuid: Some("puuid-abc".into()),
        };
        assert!(s.matches_player(&player));
    }

    #[test]
    fn summoner_does_not_match_unrelated_player() {
        let s = summoner(Some("A"), None, None);
        let player = LcuPlayer {
            summoner_name: None, game_name: None, tag_line: None,
            summoner_id: Some(999), account_id: Some(999), current_account_id: Some(999),
            profile_icon: None, profile_icon_id: None, puuid: Some("different".into()),
        };
        assert!(!s.matches_player(&player));
    }

    #[test]
    fn summoner_does_not_match_when_none() {
        let mut s = summoner(Some("A"), None, None);
        s.summoner_id = None;
        s.account_id = None;
        s.puuid = None;
        let player = LcuPlayer {
            summoner_name: None, game_name: None, tag_line: None,
            summoner_id: Some(2), account_id: None, current_account_id: None,
            profile_icon: None, profile_icon_id: None, puuid: None,
        };
        assert!(!s.matches_player(&player));
    }

    // ── LcuParticipantStats::items ────────────────────────────────────────

    fn stats_with_items(items: &[i64]) -> LcuParticipantStats {
        LcuParticipantStats {
            item0: items.first().copied(), item1: items.get(1).copied(),
            item2: items.get(2).copied(), item3: items.get(3).copied(),
            item4: items.get(4).copied(), item5: items.get(5).copied(),
            item6: items.get(6).copied(),
            ..Default::default()
        }
    }

    #[test]
    fn items_filters_out_zero_and_none() {
        let stats = stats_with_items(&[3031, 0, 3006, 0, 0, 0, 0]);
        assert_eq!(stats.items(), vec![3031, 3006]);
    }

    #[test]
    fn items_returns_empty_when_all_zero() {
        let stats = stats_with_items(&[0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(stats.items(), Vec::<i64>::new());
    }

    #[test]
    fn items_all_nonzero() {
        let stats = stats_with_items(&[1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(stats.items(), vec![1, 2, 3, 4, 5, 6, 7]);
    }

    // ── LcuParticipantStats::runes ────────────────────────────────────────

    fn stats_with_runes(runes: &[i64]) -> LcuParticipantStats {
        LcuParticipantStats {
            perk0: runes.first().copied(), perk1: runes.get(1).copied(),
            perk2: runes.get(2).copied(), perk3: runes.get(3).copied(),
            perk4: runes.get(4).copied(), perk5: runes.get(5).copied(),
            ..Default::default()
        }
    }

    #[test]
    fn runes_filters_out_zero_and_none() {
        let stats = stats_with_runes(&[8008, 0, 9101, 0, 0, 8299]);
        assert_eq!(stats.runes(), vec![8008, 9101, 8299]);
    }

    #[test]
    fn runes_returns_empty_when_all_zero() {
        let stats = stats_with_runes(&[0, 0, 0, 0, 0, 0]);
        assert_eq!(stats.runes(), Vec::<i64>::new());
    }
}
