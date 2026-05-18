//! Champion data mapping: LCU API types → domain types.

use std::collections::HashMap;

use application::LeagueClientReadError;
use domain::{LeagueChampionAbility, LeagueChampionDetails, LeagueChampionSummary, AbilityStat};

use crate::community_dragon;
use crate::constants::CHAMPION_ICON_MIME;
use crate::lcu_types::{LcuChampionAbility, LcuChampionDetails, LcuChampionSummary};
use crate::session::LcuSession;
use crate::{clean_game_asset_text, coefficient_values, log_lcu_adapter_event, non_empty, non_empty_owned, normalize_lcu_asset_path, value_as_display_string, value_as_display_values};

pub(crate) fn map_champion_catalog(champions: Vec<LcuChampionSummary>) -> Vec<LeagueChampionSummary> {
    champions
        .into_iter()
        .filter(|champion| champion.id > 0 && !champion.name.trim().is_empty())
        .map(|champion| LeagueChampionSummary {
            champion_id: champion.id,
            champion_name: champion.name.trim().to_string(),
        })
        .collect()
}

pub(crate) fn map_champion_details(
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

pub(crate) fn map_champion_ability(
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

    let effect_values = ability
        .effect_amounts
        .iter()
        .map(|(name, values)| community_dragon::DataValue {
            name: name.clone(),
            values: values.clone(),
        })
        .collect::<Vec<_>>();

    // summary_description: plain text used for the hover tooltip (no HTML).
    let summary_description = raw_clean.clone();

    // Resolve @Token@ placeholders.  We pass raw_html so that color markup
    // (<magicDamage>, <physicalDamage>, etc.) is preserved for the frontend.
    let (description, unresolved_tokens) = if let Some(spell) = bin_spell {
        // CommunityDragon available: full resolution with calculations.
        let resolution = community_dragon::resolve_spell_tokens_with_fallbacks(
            &raw_html,
            spell,
            &effect_values,
        );
        if !resolution.unresolved_tokens.is_empty() {
            log_lcu_adapter_event(
                format!(
                    "unresolved champion ability tokens champion={champion_name} slot={slot} ability={} tokens={:?}",
                    ability_name, resolution.unresolved_tokens
                )
                .as_str(),
            );
        }
        let text = if resolution.text.is_empty() { raw_clean } else { resolution.text };
        (text, resolution.unresolved_tokens)
    } else if !effect_values.is_empty() {
        // CDragon unavailable. LCU effectAmounts keys are positional
        // (Effect1Amount, Effect2Amount, …) and don't match the semantic token
        // names in the template (@MSAmount*100@, @BaseDamage@, …).
        // Re-key by position so the n-th template token resolves against
        // Effect(n)Amount.
        let rank_count = match slot {
            "Passive" => 1,
            "R" => 3,
            _ => 5,
        };
        let positional = community_dragon::positional_fallback_values(&raw_html, &effect_values);
        let resolution = community_dragon::resolve_with_lcu_values(&raw_html, &positional, rank_count);
        if !resolution.unresolved_tokens.is_empty() {
            log_lcu_adapter_event(
                format!(
                    "unresolved lcu-positional tokens champion={champion_name} slot={slot} ability={} tokens={:?}",
                    ability_name, resolution.unresolved_tokens
                )
                .as_str(),
            );
        }
        let text = if resolution.text.is_empty() { raw_clean } else { resolution.text };
        (text, resolution.unresolved_tokens)
    } else {
        (raw_clean, Vec::new())
    };

    let cdragon_available = bin_spell.is_some();

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
        unresolved_tokens,
        cdragon_available,
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
