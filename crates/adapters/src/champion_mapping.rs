//! Champion data mapping: LCU API types → domain types.

use std::collections::HashMap;

use application::LeagueClientReadError;
use domain::{LeagueChampionAbility, LeagueChampionDetails, LeagueChampionSummary, AbilityStat};

use crate::community_dragon;
use crate::constants::CHAMPION_ICON_MIME;
use crate::lcu_types::{LcuChampionAbility, LcuChampionDetails, LcuChampionSummary};
use crate::session::LcuSession;
use crate::tencent_lol_api::{TencentChampionData, TencentSpell};
use crate::{clean_game_asset_text, coefficient_values, non_empty, non_empty_owned, normalize_lcu_asset_path, value_as_display_string, value_as_display_values};

/// Split a Tencent slash-separated burn string into per-rank display values.
/// `"8/7/6/5/4"` → `["8", "7", "6", "5", "4"]`. Empty input → empty vec.
fn split_burn(s: &str) -> Vec<String> {
    if s.is_empty() {
        return Vec::new();
    }
    s.split('/').map(|v| v.trim().to_string()).collect()
}

pub(crate) fn map_champion_catalog(champions: Vec<LcuChampionSummary>) -> Vec<LeagueChampionSummary> {
    champions
        .into_iter()
        .filter(|champion| champion.id > 0 && !champion.name.trim().is_empty())
        .map(|champion| LeagueChampionSummary {
            champion_id: champion.id,
            champion_name: champion.name.trim().to_string(),
            champion_alias: champion.alias.filter(|a| !a.trim().is_empty()),
        })
        .collect()
}

pub(crate) fn map_champion_details(
    session: &LcuSession,
    champion_id: i64,
    details: LcuChampionDetails,
    bin_data: Option<&community_dragon::BinChampionData>,
    tencent_data: Option<&TencentChampionData>,
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
        let passive_tencent = tencent_data.and_then(|td| td.spell("passive"));
        abilities.push(map_champion_ability(
            session,
            "Passive",
            passive,
            passive_bin,
            passive_tencent,
        ));
    }

    for (index, spell) in details.spells.into_iter().take(4).enumerate() {
        let slot = spell
            .spell_key
            .as_deref()
            .and_then(|value| non_empty(Some(value)))
            .map(str::to_string)
            .unwrap_or_else(|| ["Q", "W", "E", "R"][index].to_string());
        let slot_bin = bin_data.and_then(|bd| bd.get_spell(slot.to_uppercase().as_str()));
        let slot_tencent = tencent_data.and_then(|td| td.spell(slot.as_str()));
        abilities.push(map_champion_ability(
            session,
            slot.as_str(),
            spell,
            slot_bin,
            slot_tencent,
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

pub(crate) fn map_champion_ability(
    session: &LcuSession,
    slot: &str,
    ability: LcuChampionAbility,
    bin_spell: Option<&community_dragon::BinSpellData>,
    tencent_spell: Option<&TencentSpell>,
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
    // Raw HTML description from the LCU — HTML tags preserved intentionally so
    // that <magicDamage>, <passive>, <active> etc. survive into the frontend.
    let raw_html = ability
        .dynamic_description
        .or(ability.description)
        .unwrap_or_default();

    // HTML-stripped fallback used for the summary tooltip and as last resort.
    let raw_clean = non_empty_owned(clean_game_asset_text(raw_html.clone()))
        .unwrap_or_else(|| "No description available".to_string());

    let ability_name = ability
        .name
        .and_then(non_empty_owned)
        .unwrap_or_else(|| slot.to_string());

    // summary_description: plain text used for the hover tooltip (no HTML).
    let summary_description = raw_clean.clone();

    let description = if let Some(ts) = tencent_spell {
        if ts.description.is_empty() { raw_clean } else { ts.description.clone() }
    } else {
        String::new()
    };

    // Spell numeric priority: Tencent burn strings (primary) → LCU coefficients (fallback).
    // Rank cap is hardcoded per slot so the LCU array is always truncated correctly
    // regardless of whether CDragon bin data is available.
    let rank_cap = match slot { "Passive" => 1, "R" => 3, _ => 5 };
    let cooldown_values = tencent_spell
        .map(|ts| split_burn(&ts.cooldown_burn))
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| coefficient_values(&truncate_to_rank(ability.cooldown_coefficients, Some(rank_cap)), true));
    let cost_values = tencent_spell
        .map(|ts| split_burn(&ts.cost_burn))
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| coefficient_values(&truncate_to_rank(ability.cost_coefficients, Some(rank_cap)), true));
    let range_values = tencent_spell
        .map(|ts| split_burn(&ts.range_burn))
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| ability.range.as_ref().map(value_as_display_values).unwrap_or_default());

    // Extract structured ability stats from CommunityDragon bin data.
    // DataValues include an index-0 "unranked" sentinel; slice to rank_count
    // so only the meaningful per-rank values reach the frontend.
    let stats: Vec<AbilityStat> = bin_spell
        .map(|spell| {
            spell
                .data_values
                .iter()
                .filter(|dv| community_dragon::is_interesting_stat(&dv.name))
                .map(|dv| {
                    let sliced = rank_slice(&dv.values, spell.rank_count);
                    (dv, sliced)
                })
                .filter(|(dv, sliced)| !community_dragon::is_noise_stat(&dv.name, sliced))
                .map(|(dv, sliced)| {
                    let (label, label_suffix) = community_dragon::clean_stat_label(&dv.name);
                    let (values, auto_suffix) = community_dragon::scale_percent_values(&sliced);
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

/// Slice a DataValue's raw values array to the per-rank window used by CDragon:
/// when there are more values than rank_count, skip index 0 (the unranked
/// sentinel) and take `rank_count` entries; otherwise take all.
fn rank_slice(values: &[f64], rank_count: usize) -> Vec<f64> {
    let finite: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if rank_count > 0 && finite.len() > rank_count {
        finite.into_iter().skip(1).take(rank_count).collect()
    } else if rank_count > 0 {
        finite.into_iter().take(rank_count).collect()
    } else {
        finite
    }
}

/// Truncate LCU coefficient arrays (cooldown, cost) to `rank_count` entries
/// when CDragon data is available.  The LCU sometimes appends a trailing
/// base/reset value beyond the actual number of ranks.
fn truncate_to_rank(values: Vec<f64>, rank_count: Option<usize>) -> Vec<f64> {
    match rank_count {
        Some(n) if n > 0 && values.len() > n => values.into_iter().take(n).collect(),
        _ => values,
    }
}

pub(crate) fn champion_name_map(champions: Vec<LcuChampionSummary>) -> HashMap<i64, String> {
    champions
        .into_iter()
        .filter(|champion| champion.id > 0 && !champion.name.trim().is_empty())
        .map(|champion| (champion.id, champion.name))
        .collect()
}

// ── tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(id: i64, name: &str) -> LcuChampionSummary {
        LcuChampionSummary {
            id,
            name: name.to_string(),
            alias: None,
        }
    }

    // ── map_champion_catalog ──────────────────────────────────────────────

    #[test]
    fn catalog_maps_valid_champions() {
        let input = vec![summary(103, "Ahri"), summary(1, "Annie")];
        let result = map_champion_catalog(input);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].champion_id, 103);
        assert_eq!(result[0].champion_name, "Ahri");
    }

    #[test]
    fn catalog_filters_non_positive_id() {
        let input = vec![summary(0, "Zero"), summary(-1, "Negative"), summary(1, "Annie")];
        let result = map_champion_catalog(input);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].champion_id, 1);
    }

    #[test]
    fn catalog_filters_empty_name() {
        let input = vec![summary(1, ""), summary(2, "  "), summary(3, "Ahri")];
        let result = map_champion_catalog(input);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].champion_name, "Ahri");
    }

    #[test]
    fn catalog_trims_name_whitespace() {
        let input = vec![summary(103, "  Ahri  ")];
        let result = map_champion_catalog(input);
        assert_eq!(result[0].champion_name, "Ahri");
    }

    #[test]
    fn catalog_empty_input() {
        let result = map_champion_catalog(vec![]);
        assert!(result.is_empty());
    }

    // ── map_champion_ability ──────────────────────────────────────────────

    fn test_session() -> LcuSession {
        use crate::lockfile::LockfileCredentials;
        LcuSession::new(LockfileCredentials {
            port: 1,
            password: "test".to_string(),
            pid: 1,
        })
        .expect("dummy session builds")
    }

    fn minimal_ability() -> LcuChampionAbility {
        LcuChampionAbility {
            name: Some("Decisive Strike".to_string()),
            description: Some("<passive>Garen Q clears debuffs</passive>".to_string()),
            dynamic_description: None,
            ability_icon_path: None,
            spell_key: Some("q".to_string()),
            cooldown: None,
            cost: None,
            range: None,
            cooldown_coefficients: Vec::new(),
            cost_coefficients: Vec::new(),
        }
    }

    #[test]
    fn tencent_primary_path_maps_garen_q_correctly() {
        let tencent = TencentSpell {
            spell_key: "Q".to_string(),
            name: "斩钢闪".to_string(),
            description: "每【8/7/6/5/4】秒冷却".to_string(),
            cooldown_burn: "8/7/6/5/4".to_string(),
            cost_burn: "0".to_string(),
            range_burn: "300".to_string(),
        };
        let ability = map_champion_ability(
            &test_session(),
            "Q",
            minimal_ability(),
            None,
            Some(&tencent),
        );
        // Burn string split correctly
        assert_eq!(ability.cooldown_values, vec!["8", "7", "6", "5", "4"]);
        // Chinese description from Tencent
        assert!(
            ability.description.contains("【8/7/6/5/4】"),
            "description must contain Tencent 【】 brackets, got: {:?}",
            ability.description
        );
        // summary_description is LCU HTML-stripped, independent of Tencent
        assert_eq!(
            ability.summary_description,
            "Garen Q clears debuffs",
            "summary_description must be the HTML-stripped LCU fallback"
        );
    }

    #[test]
    fn lcu_fallback_truncates_cooldown_to_hardcoded_rank_cap() {
        // LCU returns 6 values for a Q spell; the 6th (99) is a trailing reset sentinel.
        // With no Tencent data and no CDragon, rank_cap = 5 must drop the 99.
        let mut ability = minimal_ability();
        ability.cooldown_coefficients = vec![8.0, 7.0, 6.0, 5.0, 4.0, 99.0];

        let result = map_champion_ability(&test_session(), "Q", ability, None, None);

        assert_eq!(
            result.cooldown_values,
            vec!["8", "7", "6", "5", "4"],
            "trailing reset sentinel 99 must be truncated by hardcoded rank cap"
        );
    }

    // ── champion_name_map ─────────────────────────────────────────────────

    #[test]
    fn name_map_builds_from_valid_champions() {
        let input = vec![summary(103, "Ahri"), summary(1, "Annie")];
        let map = champion_name_map(input);
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&103), Some(&"Ahri".to_string()));
        assert_eq!(map.get(&1), Some(&"Annie".to_string()));
    }

    #[test]
    fn name_map_filters_invalid_entries() {
        let input = vec![summary(0, "Bad"), summary(-1, "Bad"), summary(1, "")];
        let map = champion_name_map(input);
        assert!(map.is_empty());
    }

    #[test]
    fn name_map_empty_input() {
        let map = champion_name_map(vec![]);
        assert!(map.is_empty());
    }
}
