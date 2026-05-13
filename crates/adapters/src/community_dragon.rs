use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use reqwest::blocking::Client;
use serde::Deserialize;

const COMMUNITY_DRAGON_BASE: &str = "https://raw.communitydragon.org/latest/game/data/characters";
const CACHE_TTL: Duration = Duration::from_secs(3600);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

const NON_DISPLAY_TOKENS: &[&str] = &["SpellModifierDescriptionAppend"];

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub(crate) struct DataValue {
    pub name: String,
    pub values: Vec<f64>,
}

#[derive(Debug, Clone)]
pub(crate) struct BinSpellData {
    pub data_values: Vec<DataValue>,
    calculations: HashMap<String, serde_json::Value>,
    rank_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TokenResolution {
    pub text: String,
    pub unresolved_tokens: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct BinChampionData {
    spells: HashMap<String, BinSpellData>,
}

impl BinChampionData {
    pub fn get_spell(&self, path: &str) -> Option<&BinSpellData> {
        self.spells.get(path)
    }
}

// ---------------------------------------------------------------------------
// Deserialization helpers (raw JSON shapes from CommunityDragon)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct RawSpellContainer {
    #[serde(rename = "mSpell")]
    m_spell: Option<RawSpell>,
}

#[derive(Debug, Deserialize)]
struct RawSpell {
    #[serde(rename = "DataValues", default)]
    data_values: Vec<RawDataValue>,
    #[serde(rename = "mDataValues", default)]
    m_data_values: Vec<RawDataValue>,
    #[serde(rename = "mSpellCalculations", default)]
    spell_calculations: HashMap<String, serde_json::Value>,
    #[serde(rename = "mClientData")]
    client_data: Option<RawClientData>,
}

#[derive(Debug, Deserialize)]
struct RawDataValue {
    name: Option<String>,
    values: Option<Vec<f64>>,
}

#[derive(Debug, Deserialize)]
struct RawClientData {
    #[serde(rename = "mTooltipData")]
    tooltip_data: Option<RawTooltipData>,
}

#[derive(Debug, Deserialize)]
struct RawTooltipData {
    #[serde(rename = "mLists")]
    lists: Option<RawTooltipLists>,
}

#[derive(Debug, Deserialize)]
struct RawTooltipLists {
    #[serde(rename = "LevelUp")]
    level_up: Option<RawLevelUpList>,
}

#[derive(Debug, Deserialize)]
struct RawLevelUpList {
    #[serde(rename = "levelCount")]
    level_count: Option<usize>,
}

// ---------------------------------------------------------------------------
// CommunityDragon client
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub(crate) struct CommunityDragonClient {
    http: Client,
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
}

impl Default for CommunityDragonClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
struct CacheEntry {
    data: BinChampionData,
    fetched_at: Instant,
}

impl CommunityDragonClient {
    pub fn new() -> Self {
        let http = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("CommunityDragon HTTP client builds");
        Self {
            http,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Fetch bin data for a champion.  The name should be the human-readable
    /// champion name (e.g. "Sett", "Nunu & Willump").  Returns `None` on any
    /// failure so callers can silently fall back.
    pub fn fetch_champion_bin(&self, champion_name: &str) -> Option<BinChampionData> {
        let cd_name = community_dragon_name(champion_name);
        {
            let cache = self.cache.lock().ok()?;
            if let Some(entry) = cache.get(&cd_name)
                && entry.fetched_at.elapsed() < CACHE_TTL
            {
                return Some(entry.data.clone());
            }
        }

        let url = format!("{COMMUNITY_DRAGON_BASE}/{cd_name}/{cd_name}.bin.json");
        let response = self.http.get(&url).send().ok()?;
        if !response.status().is_success() {
            return None;
        }
        let raw: HashMap<String, serde_json::Value> = response.json().ok()?;
        let parsed = parse_bin_json(&raw, &cd_name)?;

        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(
                cd_name,
                CacheEntry {
                    data: parsed.clone(),
                    fetched_at: Instant::now(),
                },
            );
        }

        Some(parsed)
    }
}

// ---------------------------------------------------------------------------
// Bin JSON parsing
// ---------------------------------------------------------------------------

fn parse_bin_json(
    raw: &HashMap<String, serde_json::Value>,
    cd_name: &str,
) -> Option<BinChampionData> {
    let name_camel = capitalize_first(cd_name);
    let mut spells = HashMap::new();

    // Abilities we care about: Passive, Q, W, E, R
    let slots: &[(&str, &str)] = &[
        ("Passive", "Passive"),
        ("Q", "Q"),
        ("W", "W"),
        ("E", "E"),
        ("R", "R"),
    ];

    for (slot, suffix) in slots {
        let path = if *slot == "Passive" {
            format!(
                "Characters/{}/Spells/{}PassiveAbility/{}Passive",
                name_camel, name_camel, name_camel
            )
        } else {
            format!(
                "Characters/{}/Spells/{}{}Ability/{}{}",
                name_camel, name_camel, suffix, name_camel, suffix
            )
        };

        if let Some(spell_data) = extract_spell_data(raw, &path, default_rank_count(slot)) {
            spells.insert(slot.to_string(), spell_data);
        }
    }

    if spells.is_empty() {
        return None;
    }

    Some(BinChampionData { spells })
}

fn extract_spell_data(
    raw: &HashMap<String, serde_json::Value>,
    path: &str,
    default_rank_count: usize,
) -> Option<BinSpellData> {
    let value = raw.get(path)?;
    let container: RawSpellContainer = serde_json::from_value(value.clone()).ok()?;
    let spell = container.m_spell?;
    let calculations = spell.spell_calculations;
    let rank_count = spell
        .client_data
        .and_then(|client_data| client_data.tooltip_data)
        .and_then(|tooltip_data| tooltip_data.lists)
        .and_then(|lists| lists.level_up)
        .and_then(|level_up| level_up.level_count)
        .filter(|count| *count > 0)
        .unwrap_or(default_rank_count);
    let data_values: Vec<DataValue> = spell
        .data_values
        .into_iter()
        .chain(spell.m_data_values)
        .filter_map(|dv| {
            let name = dv.name?;
            let values = dv.values?;
            Some(DataValue { name, values })
        })
        .collect();

    if data_values.is_empty() && calculations.is_empty() {
        return None;
    }

    Some(BinSpellData {
        data_values,
        calculations,
        rank_count,
    })
}

fn default_rank_count(slot: &str) -> usize {
    match slot {
        "Passive" => 1,
        "R" => 3,
        _ => 5,
    }
}

// ---------------------------------------------------------------------------
// Token resolver
// ---------------------------------------------------------------------------

/// Resolve `@Token@` placeholders in an ability description using bin data
/// values. Unresolved tokens are replaced with bracketed names and returned as
/// diagnostics so callers can log them without leaking raw template markup.
#[cfg(test)]
pub(crate) fn resolve_tokens(description: &str, data_values: &[DataValue]) -> TokenResolution {
    resolve_tokens_with_context(description, data_values, &[], None, 5)
}

#[cfg(test)]
pub(crate) fn resolve_spell_tokens(description: &str, spell: &BinSpellData) -> TokenResolution {
    resolve_spell_tokens_with_fallbacks(description, spell, &[])
}

pub(crate) fn resolve_spell_tokens_with_fallbacks(
    description: &str,
    spell: &BinSpellData,
    fallback_values: &[DataValue],
) -> TokenResolution {
    resolve_tokens_with_context(
        description,
        &spell.data_values,
        fallback_values,
        Some(&spell.calculations),
        spell.rank_count,
    )
}

fn resolve_tokens_with_context(
    description: &str,
    data_values: &[DataValue],
    fallback_values: &[DataValue],
    calculations: Option<&HashMap<String, serde_json::Value>>,
    rank_count: usize,
) -> TokenResolution {
    let mut result = String::with_capacity(description.len());
    let mut unresolved_tokens = Vec::new();
    let mut rest = description;

    while let Some(start) = rest.find('@') {
        result.push_str(&rest[..start]);
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('@') else {
            result.push('@');
            result.push_str(after_start);
            rest = "";
            break;
        };

        let token_body = &after_start[..end];
        rest = &after_start[end + 1..];
        if token_body.is_empty() {
            result.push_str("@@");
            continue;
        }

        if NON_DISPLAY_TOKENS
            .iter()
            .any(|skip| token_body.contains(skip))
        {
            continue;
        }

        // Parse optional multiplier: TokenName*factor
        let (token_name, multiplier) = if let Some(star_pos) = token_body.find('*') {
            let name = &token_body[..star_pos];
            let factor: f64 = token_body[star_pos + 1..].parse().unwrap_or(1.0);
            (name, factor)
        } else {
            (token_body, 1.0)
        };

        let replacement = resolve_token_text(
            token_name,
            multiplier,
            data_values,
            fallback_values,
            calculations,
            rank_count,
        );

        if let Some(text) = replacement {
            result.push_str(&text);
        } else {
            // Token not found – leave it as-is
            unresolved_tokens.push(token_body.to_string());
            result.push('[');
            result.push_str(token_body);
            result.push(']');
        }
    }

    result.push_str(rest);

    TokenResolution {
        text: strip_icon_tokens(&result),
        unresolved_tokens,
    }
}

fn resolve_token_text(
    token_name: &str,
    multiplier: f64,
    data_values: &[DataValue],
    fallback_values: &[DataValue],
    calculations: Option<&HashMap<String, serde_json::Value>>,
    rank_count: usize,
) -> Option<String> {
    data_value_numbers(token_name, data_values, fallback_values, rank_count)
        .and_then(|values| display_values(&multiply_values(values, multiplier), false))
        .or_else(|| {
            calculations.and_then(|items| {
                find_calculation(token_name, items).and_then(|calculation| {
                    calculation_text(calculation, data_values, fallback_values, items, rank_count)
                })
            })
        })
}

fn find_calculation<'a>(
    token_name: &str,
    calculations: &'a HashMap<String, serde_json::Value>,
) -> Option<&'a serde_json::Value> {
    calculations
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(token_name))
        .map(|(_, value)| value)
}

fn calculation_text(
    calculation: &serde_json::Value,
    data_values: &[DataValue],
    fallback_values: &[DataValue],
    calculations: &HashMap<String, serde_json::Value>,
    rank_count: usize,
) -> Option<String> {
    if let Some(modified_name) = calculation
        .get("mModifiedGameCalculation")
        .and_then(serde_json::Value::as_str)
    {
        return find_calculation(modified_name, calculations).and_then(|value| {
            calculation_text(
                value,
                data_values,
                fallback_values,
                calculations,
                rank_count,
            )
        });
    }

    let parts = calculation
        .get("mFormulaParts")
        .and_then(serde_json::Value::as_array)?;
    let display_as_percent = calculation
        .get("mDisplayAsPercent")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let mut flat_parts = Vec::new();
    let mut percent_parts = Vec::new();

    for part in parts {
        if part.get("mStat").is_some() {
            if let Some(values) =
                formula_part_values(part, data_values, fallback_values, rank_count).or_else(|| {
                    part.get("mCoefficient")
                        .and_then(serde_json::Value::as_f64)
                        .map(|value| vec![value])
                })
                && let Some(text) = display_percent_values(values)
            {
                percent_parts.push(format!("+{text}"));
            }
            continue;
        }

        if let Some(value) = part.get("mCoefficient").and_then(serde_json::Value::as_f64) {
            if let Some(text) = display_percent_values(vec![value]) {
                percent_parts.push(format!("+{text}"));
            }
            continue;
        }

        if let Some(values) = formula_part_values(part, data_values, fallback_values, rank_count)
            && let Some(text) = display_values(&values, display_as_percent)
        {
            flat_parts.push(text);
        }
    }

    match (flat_parts.is_empty(), percent_parts.is_empty()) {
        (true, true) => None,
        (false, true) => Some(flat_parts.join(" + ")),
        (true, false) => Some(percent_parts.join(" ")),
        (false, false) => Some(format!(
            "{} ({})",
            flat_parts.join(" + "),
            percent_parts.join(" ")
        )),
    }
}

fn formula_part_values(
    part: &serde_json::Value,
    data_values: &[DataValue],
    fallback_values: &[DataValue],
    rank_count: usize,
) -> Option<Vec<f64>> {
    if let Some(name) = part.get("mDataValue").and_then(serde_json::Value::as_str) {
        return data_value_numbers(name, data_values, fallback_values, rank_count);
    }

    part.get("mNumber")
        .and_then(serde_json::Value::as_f64)
        .or_else(|| part.get("mLevel1Value").and_then(serde_json::Value::as_f64))
        .map(|value| vec![value])
}

fn data_value_numbers(
    name: &str,
    data_values: &[DataValue],
    fallback_values: &[DataValue],
    rank_count: usize,
) -> Option<Vec<f64>> {
    data_values
        .iter()
        .chain(fallback_values.iter())
        .find(|dv| dv.name.eq_ignore_ascii_case(name))
        .map(|dv| rank_values(&dv.values, rank_count))
        .filter(|values| !values.is_empty())
}

fn rank_values(values: &[f64], rank_count: usize) -> Vec<f64> {
    let values = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .collect::<Vec<_>>();

    if rank_count > 0 && values.len() > rank_count {
        values.into_iter().skip(1).take(rank_count).collect()
    } else if rank_count > 0 {
        values.into_iter().take(rank_count).collect()
    } else {
        values
    }
}

fn multiply_values(values: Vec<f64>, multiplier: f64) -> Vec<f64> {
    values
        .into_iter()
        .map(|value| value * multiplier)
        .collect::<Vec<_>>()
}

fn display_percent_values(values: Vec<f64>) -> Option<String> {
    display_values(&multiply_values(values, 100.0), true)
}

fn display_values(values: &[f64], append_percent: bool) -> Option<String> {
    if values.is_empty() {
        return None;
    }

    let values = if values
        .iter()
        .all(|value| (*value - values[0]).abs() < 0.001)
    {
        vec![values[0]]
    } else {
        values.to_vec()
    };

    Some(
        values
            .into_iter()
            .map(|value| {
                let formatted = format_f64(value);
                if append_percent {
                    format!("{formatted}%")
                } else {
                    formatted
                }
            })
            .collect::<Vec<_>>()
            .join("/"),
    )
}

fn strip_icon_tokens(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut rest = value;

    while let Some(start) = rest.find("%i:") {
        result.push_str(&rest[..start]);
        let after_start = &rest[start + 3..];
        if let Some(end) = after_start.find('%') {
            rest = &after_start[end + 1..];
        } else {
            rest = "";
        }
    }

    result.push_str(rest);
    result
}

/// Pick the best value from the per-level array.
/// Index 0 is "unranked"; prefer index 1 (level 1).  Fall back to first
/// non-zero value, then index 0.
#[cfg(test)]
fn best_value(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    // Prefer index 1 (level 1)
    if values.len() > 1 && values[1] != 0.0 {
        return Some(values[1]);
    }
    // First non-zero
    if let Some(v) = values.iter().copied().find(|v| *v != 0.0) {
        return Some(v);
    }
    // Fall back to index 0
    values.first().copied()
}

/// Format an f64 for display.
/// - If it's close to an integer, show without decimals.
/// - Otherwise show up to 1 decimal place, trimming trailing zeros.
fn format_f64(value: f64) -> String {
    if value.is_nan() || value.is_infinite() {
        return value.to_string();
    }
    let rounded = value.round();
    if (value - rounded).abs() < 0.01 {
        return format!("{}", rounded as i64);
    }
    // Show 1 decimal
    let s = format!("{:.1}", value);
    // Should already be trimmed by {:.1}
    s
}

// ---------------------------------------------------------------------------
// Champion name mapping
// ---------------------------------------------------------------------------

/// Map a human-readable champion name to the CommunityDragon folder name
/// (lowercase, special cases handled).
fn community_dragon_name(name: &str) -> String {
    let lower = name.to_lowercase();
    match lower.as_str() {
        "nunu & willump" => "nunu".to_string(),
        "wukong" => "monkeyking".to_string(),
        "renata glasc" => "renata".to_string(),
        "kog'maw" => "kogmaw".to_string(),
        "cho'gath" => "chogath".to_string(),
        "vel'koz" => "velkoz".to_string(),
        "kha'zix" => "khazix".to_string(),
        "rek'sai" => "reksai".to_string(),
        "kai'sa" => "kaisa".to_string(),
        _ => lower,
    }
}

/// Capitalize the first character of a string.
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => {
            let upper: String = first.to_uppercase().collect();
            upper + chars.as_str()
        }
        None => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data_values() -> Vec<DataValue> {
        vec![
            DataValue {
                name: "BaseDamage".to_string(),
                values: vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0],
            },
            DataValue {
                name: "MSAmount".to_string(),
                values: vec![0.0, 0.3, 0.35, 0.4, 0.45, 0.5, 0.55],
            },
            DataValue {
                name: "ADRatio".to_string(),
                values: vec![1.0, 1.2, 1.2, 1.2, 1.2, 1.2, 1.2],
            },
        ]
    }

    #[test]
    fn resolves_simple_token() {
        let dvs = sample_data_values();
        let result = resolve_tokens("@BaseDamage@ damage", &dvs);
        assert_eq!(result.text, "10/20/30/40/50 damage");
        assert!(result.unresolved_tokens.is_empty());
    }

    #[test]
    fn resolves_multiplied_token() {
        let dvs = sample_data_values();
        let result = resolve_tokens("Moves @MSAmount*100@% faster", &dvs);
        assert_eq!(result.text, "Moves 30/35/40/45/50% faster");
        assert!(result.unresolved_tokens.is_empty());
    }

    #[test]
    fn resolves_multiple_tokens() {
        let dvs = sample_data_values();
        let result = resolve_tokens("Deals @BaseDamage@ (+@ADRatio*100@% AD) damage", &dvs);
        assert_eq!(result.text, "Deals 10/20/30/40/50 (+120% AD) damage");
        assert!(result.unresolved_tokens.is_empty());
    }

    #[test]
    fn resolves_simple_spell_calculation_token() {
        let spell = BinSpellData {
            data_values: sample_data_values(),
            rank_count: 5,
            calculations: HashMap::from([(
                "TotalDamage".to_string(),
                serde_json::json!({
                    "mFormulaParts": [
                        { "mDataValue": "BaseDamage" },
                        { "mCoefficient": 0.5 }
                    ]
                }),
            )]),
        };

        let result = resolve_spell_tokens("Deals @TotalDamage@ magic damage", &spell);

        assert_eq!(result.text, "Deals 10/20/30/40/50 (+50%) magic damage");
        assert!(result.unresolved_tokens.is_empty());
    }

    #[test]
    fn resolves_stat_ratio_spell_calculation_token() {
        let spell = BinSpellData {
            data_values: sample_data_values(),
            rank_count: 5,
            calculations: HashMap::from([(
                "AttackDamage".to_string(),
                serde_json::json!({
                    "mFormulaParts": [
                        { "mDataValue": "BaseDamage" },
                        { "mStat": 2, "mDataValue": "ADRatio" }
                    ]
                }),
            )]),
        };

        let result = resolve_spell_tokens("Deals @AttackDamage@ physical damage", &spell);

        assert_eq!(result.text, "Deals 10/20/30/40/50 (+120%) physical damage");
        assert!(result.unresolved_tokens.is_empty());
    }

    #[test]
    fn resolves_stat_coefficient_spell_calculation_token() {
        let spell = BinSpellData {
            data_values: sample_data_values(),
            rank_count: 5,
            calculations: HashMap::from([(
                "MagicDamage".to_string(),
                serde_json::json!({
                    "mFormulaParts": [
                        { "mDataValue": "BaseDamage" },
                        { "mStat": 3, "mCoefficient": 0.65 }
                    ]
                }),
            )]),
        };

        let result = resolve_spell_tokens("Deals @MagicDamage@ magic damage", &spell);

        assert_eq!(result.text, "Deals 10/20/30/40/50 (+65%) magic damage");
        assert!(result.unresolved_tokens.is_empty());
    }

    #[test]
    fn real_ahri_and_sett_fixture_descriptions_resolve_without_raw_tokens() {
        // Fixture excerpts from:
        // - /plugins/rcp-be-lol-game-data/global/default/v1/champions/103.json
        // - /game/data/characters/ahri/ahri.bin.json
        // - /plugins/rcp-be-lol-game-data/global/default/v1/champions/875.json
        // - /game/data/characters/sett/sett.bin.json
        let ahri_raw =
            serde_json::from_value::<HashMap<String, serde_json::Value>>(serde_json::json!({
                "Characters/Ahri/Spells/AhriQAbility/AhriQ": {
                    "mSpell": {
                        "DataValues": [
                            {
                                "name": "BaseDamage",
                                "values": [10.0, 35.0, 60.0, 85.0, 110.0, 135.0, 160.0]
                            }
                        ],
                        "mSpellCalculations": {
                            "TotalDamage": {
                                "mSimpleTooltipCalculationDisplay": 6,
                                "mFormulaParts": [
                                    { "mDataValue": "BaseDamage" },
                                    { "mCoefficient": 0.5 }
                                ]
                            }
                        }
                    }
                }
            }))
            .expect("Ahri fixture is valid JSON");
        let sett_raw = serde_json::from_value::<HashMap<String, serde_json::Value>>(
            serde_json::json!({
                "Characters/Sett/Spells/SettQAbility/SettQ": {
                    "mSpell": {
                        "DataValues": [
                            {
                                "name": "BaseDamage",
                                "values": [0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0]
                            },
                            {
                                "name": "MSAmount",
                                "values": [0.30000001192092896, 0.30000001192092896, 0.30000001192092896]
                            },
                            {
                                "name": "MSDuration",
                                "values": [1.5, 1.5, 1.5]
                            },
                            {
                                "name": "EnemyMaxHealthDamage",
                                "values": [0.009999999776482582, 0.009999999776482582]
                            },
                            {
                                "name": "MaxHealthTADRatio",
                                "values": [0.00004999999873689376, 0.00009999999747378752]
                            }
                        ],
                        "mSpellCalculations": {
                            "MaxHealthDamageCalc": {
                                "mFormulaParts": [
                                    { "mDataValue": "EnemyMaxHealthDamage" },
                                    { "mStat": 2, "mDataValue": "MaxHealthTADRatio" }
                                ],
                                "mDisplayAsPercent": true,
                                "mPrecision": 1
                            }
                        },
                        "mClientData": {
                            "mTooltipData": {
                                "mLists": {
                                    "LevelUp": {
                                        "levelCount": 5
                                    }
                                }
                            }
                        }
                    }
                }
            }),
        )
        .expect("Sett fixture is valid JSON");

        let ahri_data = parse_bin_json(&ahri_raw, "ahri").expect("Ahri fixture parses");
        let sett_data = parse_bin_json(&sett_raw, "sett").expect("Sett fixture parses");
        let ahri_q = ahri_data.get_spell("Q").expect("Ahri Q is present");
        let sett_q = sett_data.get_spell("Q").expect("Sett Q is present");

        assert_eq!(resolve_spell_tokens("@MSAmount*100@", sett_q).text, "30");
        assert_eq!(resolve_spell_tokens("@MSDuration@", sett_q).text, "1.5");
        assert_eq!(
            resolve_spell_tokens("@BaseDamage@", sett_q).text,
            "10/20/30/40/50"
        );

        let ahri_description = r#"Ahri throws then pulls back her orb, dealing <magicDamage>@TotalDamage@ magic damage</magicDamage> on the way out and <trueDamage>@TotalDamage@ true damage</trueDamage> on the way back.@SpellModifierDescriptionAppend@"#;
        let sett_description = r#"Sett itches for a fight, gaining <speed>@MSAmount*100@% Move Speed</speed> towards enemy champions for @MSDuration@ seconds.<br><br>Additionally Sett's next two Attacks deal an additional <physicalDamage>@BaseDamage@ plus @MaxHealthDamageCalc@ max Health physical damage</physicalDamage>.@SpellModifierDescriptionAppend@"#;

        let resolved_ahri = resolve_spell_tokens(ahri_description, ahri_q);
        let resolved_sett = resolve_spell_tokens(sett_description, sett_q);

        assert_no_raw_template_tokens("Ahri Q", &resolved_ahri.text);
        assert_no_raw_template_tokens("Sett Q", &resolved_sett.text);
        assert!(resolved_ahri.unresolved_tokens.is_empty());
        assert!(resolved_sett.unresolved_tokens.is_empty());
        assert!(
            resolved_ahri
                .text
                .contains("35/60/85/110/135 (+50%) magic damage")
        );
        assert!(
            resolved_ahri
                .text
                .contains("35/60/85/110/135 (+50%) true damage")
        );
        assert!(resolved_sett.text.contains("30% Move Speed"));
        assert!(resolved_sett.text.contains("1.5 seconds"));
        assert!(resolved_sett.text.contains("10/20/30/40/50 plus"));
    }

    #[test]
    fn resolves_m_data_values_alias_and_fallback_effect_amounts() {
        let raw = serde_json::from_value::<HashMap<String, serde_json::Value>>(serde_json::json!({
            "Characters/Sett/Spells/SettQAbility/SettQ": {
                "mSpell": {
                    "mDataValues": [
                        {
                            "name": "AliasDamage",
                            "values": [0.0, 11.0, 22.0, 33.0, 44.0, 55.0]
                        }
                    ]
                }
            }
        }))
        .expect("fixture is valid JSON");
        let data = parse_bin_json(&raw, "sett").expect("fixture parses");
        let spell = data.get_spell("Q").expect("Sett Q is present");
        let fallback_values = vec![DataValue {
            name: "Effect1Amount".to_string(),
            values: vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0],
        }];

        let alias_result = resolve_spell_tokens("@AliasDamage@", spell);
        let fallback_result =
            resolve_spell_tokens_with_fallbacks("@Effect1Amount@", spell, &fallback_values);

        assert_eq!(alias_result.text, "11/22/33/44/55");
        assert_eq!(fallback_result.text, "10/20/30/40/50");
        assert!(alias_result.unresolved_tokens.is_empty());
        assert!(fallback_result.unresolved_tokens.is_empty());
    }

    #[test]
    fn leaves_unknown_tokens_as_is() {
        let dvs = sample_data_values();
        let result = resolve_tokens("Does @UnknownToken@ stuff", &dvs);
        // No tokens resolved → returns None (caller falls back to original text)
        assert_eq!(result.text, "Does [UnknownToken] stuff");
        assert_eq!(result.unresolved_tokens, vec!["UnknownToken"]);
    }

    #[test]
    fn records_unresolved_damage_calc_token() {
        let dvs = sample_data_values();
        let result = resolve_tokens("Does @DamageCalc@ damage", &dvs);
        assert_eq!(result.text, "Does [DamageCalc] damage");
        assert_eq!(result.unresolved_tokens, vec!["DamageCalc"]);
    }

    #[test]
    fn skips_spell_modifier_token() {
        let dvs = sample_data_values();
        let result = resolve_tokens("Text @SpellModifierDescriptionAppend@ end", &dvs);
        assert_eq!(result.text, "Text  end");
        assert!(result.unresolved_tokens.is_empty());
    }

    #[test]
    fn returns_none_when_no_tokens_match() {
        let dvs = sample_data_values();
        let result = resolve_tokens("No tokens here", &dvs);
        assert_eq!(result.text, "No tokens here");
        assert!(result.unresolved_tokens.is_empty());
    }

    #[test]
    fn returns_none_for_empty_description() {
        let dvs = sample_data_values();
        let result = resolve_tokens("", &dvs);
        assert_eq!(result.text, "");
        assert!(result.unresolved_tokens.is_empty());
    }

    #[test]
    fn best_value_prefers_index_one() {
        let values = vec![0.0, 10.0, 20.0, 30.0];
        assert_eq!(best_value(&values), Some(10.0));
    }

    #[test]
    fn best_value_falls_back_to_first_nonzero() {
        let values = vec![0.0, 0.0, 5.0, 10.0];
        assert_eq!(best_value(&values), Some(5.0));
    }

    #[test]
    fn best_value_falls_back_to_index_zero() {
        let values = vec![7.0, 0.0, 0.0];
        assert_eq!(best_value(&values), Some(7.0));
    }

    #[test]
    fn format_f64_integer_like() {
        assert_eq!(format_f64(10.0), "10");
        assert_eq!(format_f64(0.0), "0");
        assert_eq!(format_f64(100.0), "100");
    }

    #[test]
    fn format_f64_fractional() {
        assert_eq!(format_f64(0.5), "0.5");
        assert_eq!(format_f64(33.3), "33.3");
    }

    #[test]
    fn format_f64_percentage_after_multiply() {
        // 0.3 * 100 = 30.0 → "30"
        assert_eq!(format_f64(0.3 * 100.0), "30");
        // 0.333 * 100 = 33.3 → "33.3"
        assert_eq!(format_f64(0.333 * 100.0), "33.3");
    }

    #[test]
    fn community_dragon_name_standard() {
        assert_eq!(community_dragon_name("Sett"), "sett");
        assert_eq!(community_dragon_name("Ahri"), "ahri");
    }

    #[test]
    fn community_dragon_name_special_cases() {
        assert_eq!(community_dragon_name("Nunu & Willump"), "nunu");
        assert_eq!(community_dragon_name("Wukong"), "monkeyking");
        assert_eq!(community_dragon_name("Kog'Maw"), "kogmaw");
        assert_eq!(community_dragon_name("Cho'Gath"), "chogath");
    }

    #[test]
    fn capitalize_first_works() {
        assert_eq!(capitalize_first("sett"), "Sett");
        assert_eq!(capitalize_first("monkeyking"), "Monkeyking");
        assert_eq!(capitalize_first(""), "");
    }

    fn assert_no_raw_template_tokens(label: &str, value: &str) {
        assert!(
            !value.contains('@'),
            "{label} left a raw template token in: {value}"
        );
        assert!(
            !value.contains("%i:"),
            "{label} left a raw icon token in: {value}"
        );
    }

    #[test]
    fn build_spell_path_passive() {
        let name = capitalize_first("sett");
        assert_eq!(
            format!(
                "Characters/{}/Spells/{}PassiveAbility/{}Passive",
                name, name, name
            ),
            "Characters/Sett/Spells/SettPassiveAbility/SettPassive"
        );
    }

    #[test]
    fn build_spell_path_q() {
        let name = capitalize_first("sett");
        assert_eq!(
            format!("Characters/{}/Spells/{}QAbility/{}Q", name, name, name),
            "Characters/Sett/Spells/SettQAbility/SettQ"
        );
    }
}
