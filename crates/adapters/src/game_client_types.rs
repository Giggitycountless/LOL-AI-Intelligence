//! Game Client API types (Live Client Data).
//! JSON shapes for the in-game overlay / live client data endpoints.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub(crate) struct GameClientPlayer {
    pub(crate) summoner_name: Option<String>,
    pub(crate) team: String,
    pub(crate) riot_id: Option<String>,
    pub(crate) champion_name: Option<String>,
    pub(crate) level: Option<i64>,
    pub(crate) position: Option<String>,
    #[serde(default)]
    pub(crate) is_dead: bool,
    pub(crate) respawn_timer: Option<f64>,
    #[serde(default)]
    pub(crate) items: Vec<GameClientItem>,
    pub(crate) scores: Option<GameClientScores>,
    pub(crate) summoner_spells: Option<GameClientSummonerSpells>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GameClientActivePlayer {
    pub(crate) summoner_name: String,
    pub(crate) level: Option<i64>,
    pub(crate) current_gold: Option<f64>,
    pub(crate) champion_stats: Option<GameClientChampionStats>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GameClientChampionStats {
    pub(crate) resource_type: Option<String>,
    pub(crate) resource_value: Option<f64>,
    pub(crate) resource_max: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GameClientItem {
    #[serde(rename = "itemID")]
    pub(crate) item_id: Option<i64>,
    pub(crate) display_name: Option<String>,
    pub(crate) price: Option<i64>,
    pub(crate) count: Option<i64>,
    pub(crate) slot: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GameClientScores {
    pub(crate) kills: Option<i64>,
    pub(crate) deaths: Option<i64>,
    pub(crate) assists: Option<i64>,
    pub(crate) creep_score: Option<i64>,
    pub(crate) ward_score: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GameClientSummonerSpells {
    pub(crate) summoner_spell_one: Option<GameClientNamedEntity>,
    pub(crate) summoner_spell_two: Option<GameClientNamedEntity>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GameClientNamedEntity {
    pub(crate) display_name: Option<String>,
    pub(crate) id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct GameClientEventData {
    #[serde(default)]
    pub(crate) events: Vec<GameClientEvent>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct GameClientEvent {
    #[serde(rename = "EventID")]
    pub(crate) event_id: Option<i64>,
    pub(crate) event_name: Option<String>,
    pub(crate) event_time: Option<f64>,
    pub(crate) killer_name: Option<String>,
    pub(crate) victim_name: Option<String>,
    #[serde(default)]
    pub(crate) assisters: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GameClientStats {
    pub(crate) game_time: Option<f64>,
    pub(crate) game_mode: Option<String>,
    pub(crate) map_name: Option<String>,
}

use crate::non_empty;

impl GameClientPlayer {
    pub(crate) fn display_name(&self) -> Option<String> {
        non_empty(self.riot_id.as_deref())
            .or_else(|| non_empty(self.summoner_name.as_deref()))
            .map(str::to_string)
    }
}

// ── tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn player(riot_id: Option<&str>, summoner_name: Option<&str>) -> GameClientPlayer {
        GameClientPlayer {
            summoner_name: summoner_name.map(str::to_string),
            team: "ORDER".into(),
            riot_id: riot_id.map(str::to_string),
            champion_name: Some("Ahri".into()),
            level: Some(18),
            position: Some("MIDDLE".into()),
            is_dead: false,
            respawn_timer: None,
            items: vec![],
            scores: None,
            summoner_spells: None,
        }
    }

    #[test]
    fn display_name_prefers_riot_id() {
        let p = player(Some("Faker#KR1"), Some("OldSummoner"));
        assert_eq!(p.display_name().as_deref(), Some("Faker#KR1"));
    }

    #[test]
    fn display_name_falls_back_to_summoner_name() {
        let p = player(None, Some("OldSummoner"));
        assert_eq!(p.display_name().as_deref(), Some("OldSummoner"));
    }

    #[test]
    fn display_name_returns_none_when_all_empty() {
        let p = player(None, None);
        assert_eq!(p.display_name(), None);
    }

    #[test]
    fn display_name_ignores_empty_strings() {
        let p = player(Some(""), Some(""));
        assert_eq!(p.display_name(), None);
    }

    #[test]
    fn display_name_riot_id_empty_falls_back() {
        let p = player(Some(""), Some("RealName"));
        assert_eq!(p.display_name().as_deref(), Some("RealName"));
    }
}
