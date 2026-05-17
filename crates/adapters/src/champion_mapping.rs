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

pub(crate) fn champion_name_map(champions: Vec<LcuChampionSummary>) -> HashMap<i64, String> {
    champions
        .into_iter()
        .filter(|champion| champion.id > 0 && !champion.name.trim().is_empty())
        .map(|champion| (champion.id, champion.name))
        .collect()
}
