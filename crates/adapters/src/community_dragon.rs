use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use reqwest::blocking::Client;
use serde::Deserialize;

const COMMUNITY_DRAGON_BASE: &str =
    "https://raw.communitydragon.org/latest/game/data/characters";
const CACHE_TTL: Duration = Duration::from_secs(3600);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

const SKIP_TOKENS: &[&str] = &[
    "DamageCalc",
    "SpellModifierDescriptionAppend",
];

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
struct RawBinRoot(HashMap<String, serde_json::Value>);

#[derive(Debug, Deserialize)]
struct RawSpellContainer {
    #[serde(rename = "mSpell")]
    m_spell: Option<RawSpell>,
}

#[derive(Debug, Deserialize)]
struct RawSpell {
    #[serde(rename = "DataValues")]
    data_values: Option<Vec<RawDataValue>>,
}

#[derive(Debug, Deserialize)]
struct RawDataValue {
    name: Option<String>,
    values: Option<Vec<f64>>,
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
            if let Some(entry) = cache.get(&cd_name) {
                if entry.fetched_at.elapsed() < CACHE_TTL {
                    return Some(entry.data.clone());
                }
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

        if let Some(spell_data) = extract_spell_data(raw, &path) {
            spells.insert(slot.to_string(), spell_data);
        }
    }

    if spells.is_empty() {
        return None;
    }

    Some(BinChampionData { spells })
}

fn extract_spell_data(raw: &HashMap<String, serde_json::Value>, path: &str) -> Option<BinSpellData> {
    let value = raw.get(path)?;
    let container: RawSpellContainer = serde_json::from_value(value.clone()).ok()?;
    let spell = container.m_spell?;
    let raw_dvs = spell.data_values?;
    let data_values: Vec<DataValue> = raw_dvs
        .into_iter()
        .filter_map(|dv| {
            let name = dv.name?;
            let values = dv.values?;
            Some(DataValue { name, values })
        })
        .collect();

    if data_values.is_empty() {
        return None;
    }

    Some(BinSpellData { data_values })
}

// ---------------------------------------------------------------------------
// Token resolver
// ---------------------------------------------------------------------------

/// Resolve `@Token@` placeholders in an ability description using bin data
/// values.  Returns `None` if no substitutions were made (so callers know the
/// original text is unchanged).
pub(crate) fn resolve_tokens(
    description: &str,
    data_values: &[DataValue],
) -> Option<String> {
    let mut result = String::with_capacity(description.len());
    let mut chars = description.char_indices().peekable();
    let mut changed = false;

    while let Some((i, ch)) = chars.next() {
        if ch != '@' {
            result.push(ch);
            continue;
        }

        // Find the closing '@'
        let rest = &description[i + 1..];
        let Some(end) = rest.find('@') else {
            result.push(ch);
            continue;
        };

        let token_body = &rest[..end];
        if token_body.is_empty() {
            result.push(ch);
            continue;
        }

        // Skip tokens we can't resolve
        if SKIP_TOKENS.iter().any(|skip| token_body.contains(skip)) {
            // Consume through the closing '@'
            for _ in 0..=end {
                chars.next();
            }
            changed = true;
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

        // Look up the value
        let replacement = data_values
            .iter()
            .find(|dv| dv.name.eq_ignore_ascii_case(token_name))
            .and_then(|dv| best_value(&dv.values))
            .map(|v| format_f64(v * multiplier));

        if let Some(text) = replacement {
            result.push_str(&text);
            changed = true;
        } else {
            // Token not found – leave it as-is
            result.push('@');
            result.push_str(token_body);
            result.push('@');
        }

        // Advance the iterator past the closing '@'
        for _ in 0..=end {
            chars.next();
        }
    }

    if changed { Some(result) } else { None }
}

/// Pick the best value from the per-level array.
/// Index 0 is "unranked"; prefer index 1 (level 1).  Fall back to first
/// non-zero value, then index 0.
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
        assert_eq!(result.as_deref(), Some("10 damage"));
    }

    #[test]
    fn resolves_multiplied_token() {
        let dvs = sample_data_values();
        let result = resolve_tokens("Moves @MSAmount*100@% faster", &dvs);
        assert_eq!(result.as_deref(), Some("Moves 30% faster"));
    }

    #[test]
    fn resolves_multiple_tokens() {
        let dvs = sample_data_values();
        let result = resolve_tokens(
            "Deals @BaseDamage@ (+@ADRatio*100@% AD) damage",
            &dvs,
        );
        assert_eq!(result.as_deref(), Some("Deals 10 (+120% AD) damage"));
    }

    #[test]
    fn leaves_unknown_tokens_as_is() {
        let dvs = sample_data_values();
        let result = resolve_tokens("Does @UnknownToken@ stuff", &dvs);
        // No tokens resolved → returns None (caller falls back to original text)
        assert_eq!(result, None);
    }

    #[test]
    fn skips_damage_calc_token() {
        let dvs = sample_data_values();
        let result = resolve_tokens("Does @DamageCalc@ damage", &dvs);
        // DamageCalc is skipped entirely (removed)
        assert_eq!(result.as_deref(), Some("Does  damage"));
    }

    #[test]
    fn skips_spell_modifier_token() {
        let dvs = sample_data_values();
        let result = resolve_tokens("Text @SpellModifierDescriptionAppend@ end", &dvs);
        assert_eq!(result.as_deref(), Some("Text  end"));
    }

    #[test]
    fn returns_none_when_no_tokens_match() {
        let dvs = sample_data_values();
        let result = resolve_tokens("No tokens here", &dvs);
        assert_eq!(result, None);
    }

    #[test]
    fn returns_none_for_empty_description() {
        let dvs = sample_data_values();
        let result = resolve_tokens("", &dvs);
        assert_eq!(result, None);
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
            format!(
                "Characters/{}/Spells/{}QAbility/{}Q",
                name, name, name
            ),
            "Characters/Sett/Spells/SettQAbility/SettQ"
        );
    }
}
