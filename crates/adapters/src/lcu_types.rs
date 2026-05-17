//! LCU API response types — champ select, gameflow, summoner.

use serde::{Deserialize, Serialize};

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
