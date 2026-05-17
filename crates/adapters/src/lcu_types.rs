//! LCU API response types — champ select, gameflow, summoner.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
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
    pub(crate) vision_score: Option<i64>,
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
    #[serde(default)]
    pub(crate) effect_amounts: HashMap<String, Vec<f64>>,
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
    pub(crate) fn profile(&self) -> CurrentSummonerProfile {
        CurrentSummonerProfile {
            display_name: self.display_name(),
            summoner_level: self.summoner_level.unwrap_or_default(),
            profile_icon_id: self.profile_icon_id,
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
