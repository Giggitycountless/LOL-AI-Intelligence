use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use reqwest::blocking::Client;
use serde::Deserialize;

const COMMUNITY_DRAGON_BASE: &str = "https://raw.communitydragon.org/latest/game/data/characters";
const CACHE_TTL: Duration = Duration::from_secs(3600);
/// Total read timeout for a CDragon bin.json fetch.  The files are several MB,
/// so connections to distant CDN nodes (common in some regions) need more time
/// than a localhost LCU call.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
/// Separate connect timeout: if the host is unreachable we want to fail fast.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(8);

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
    pub rank_count: usize,
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
    #[serde(rename = "mClientData")]
    client_data: Option<RawClientData>,
}

#[derive(Debug, Deserialize)]
struct RawDataValue {
    #[serde(alias = "mDataValue")]
    name: Option<String>,
    #[serde(alias = "mValues")]
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
            .connect_timeout(CONNECT_TIMEOUT)
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
        let response = match crate::session::run_blocking(|| self.http.get(&url).send()) {
            Ok(r) => r,
            Err(err) => {
                eprintln!("[community-dragon] fetch failed champion={cd_name} url={url} err={err}");
                return None;
            }
        };
        if !response.status().is_success() {
            eprintln!(
                "[community-dragon] fetch failed champion={cd_name} url={url} status={}",
                response.status()
            );
            return None;
        }
        let raw: HashMap<String, serde_json::Value> = match crate::session::run_blocking(move || response.json()) {
            Ok(r) => r,
            Err(err) => {
                eprintln!("[community-dragon] json parse failed champion={cd_name} err={err}");
                return None;
            }
        };
        let parsed = match parse_bin_json(&raw, &cd_name) {
            Some(p) => p,
            None => {
                eprintln!("[community-dragon] bin parse produced no spells champion={cd_name}");
                return None;
            }
        };

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
    let rank_count = spell
        .client_data
        .and_then(|client_data| client_data.tooltip_data)
        .and_then(|tooltip_data| tooltip_data.lists)
        .and_then(|lists| lists.level_up)
        .and_then(|level_up| level_up.level_count)
        .filter(|count| *count > 0)
        .unwrap_or(default_rank_count);
    let mut data_values: Vec<DataValue> = spell
        .data_values
        .into_iter()
        .chain(spell.m_data_values)
        .filter_map(|dv| {
            let name = dv.name?;
            let values = dv.values?;
            Some(DataValue { name, values })
        })
        .collect();

    // CDragon also stores some spell properties as direct `mXxx` numeric arrays on
    // `mSpell` (e.g. `mAmmoRechargeTime`), which are distinct from the `DataValues`
    // list.  Extract them as additional DataValues so `@AmmoRechargeTime@` tokens
    // (and similar) resolve correctly.
    if let Some(raw_spell) = value.get("mSpell").and_then(|v| v.as_object()) {
        for (key, val) in raw_spell {
            // Only `mXxx` fields where the second character is uppercase
            let Some(prop_name) = key.strip_prefix('m') else { continue };
            if !prop_name.starts_with(|c: char| c.is_ascii_uppercase()) { continue }
            // Only flat homogeneous numeric arrays
            let Some(arr) = val.as_array() else { continue };
            let nums: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();
            if nums.len() != arr.len() || nums.is_empty() { continue }
            // Don't duplicate anything already captured as a DataValue
            if !data_values.iter().any(|dv| dv.name.eq_ignore_ascii_case(prop_name)) {
                data_values.push(DataValue { name: prop_name.to_string(), values: nums });
            }
        }
    }

    if data_values.is_empty() {
        return None;
    }

    Some(BinSpellData {
        data_values,
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
// Stat name cleaning for display
// ---------------------------------------------------------------------------

/// Stat names that are internal implementation details, not player-facing.
const SKIP_STAT_NAMES: &[&str] = &[
    "Duration",
    "Tooltip",
];

/// Stat name suffixes that indicate the value is a percentage.
/// NOTE: "Mod" deliberately excluded — too broad (catches MinionMod, RepeatDamageMod).
const PERCENT_SUFFIXES: &[&str] = &["Percent", "Amp", "Mult", "Conversion"];

/// Stat names that are always noise — displayed elsewhere or too internal.
const NOISE_NAMES: &[&str] = &[
    "ADRatio",
    "APRatio",
    "MaxHealthTADRatio",
    "MaxHealthTADRatioTOOLTIP",
    "MonsterCap",
    "LockTime",
    "ResourceDecayRate",
    "DamageConversionBase",
    "PassiveADRatioTT",
    "RAPCoefficient",
];

/// Stat names where the value represents damage (physical or magical).
const DAMAGE_LABELS: &[&str] = &[
    "BaseDamage",
    "BonusDamage",
    "TotalDamage",
    "MaxHealthDamage",
    "MissingHealthDamage",
    "CurrentHealthDamage",
    "MinionMod",
    "MonsterDamage",
    "SingleBaseDamage",
    "RepeatDamageMod",
    "ReducedDamagePercent",
    "DamageAmp",
    "DamageStored",
    "EnemyMaxHealthDamage",
    "MinionBonusDamageMultiplier",
    "MinionBonusDamageThreshold",
    "QMinimumDamage",
    "FourthShotMultiplier",
    "PercentMissingAmp",
    "CraterEdgeDamageReduction",
];

/// Stat names where the value represents a duration in seconds.
const DURATION_LABELS: &[&str] = &[
    "StunDuration",
    "CharmDuration",
    "RootDuration",
    "SlowDuration",
    "TauntLength",
    "FearDuration",
    "KnockupDuration",
    "SuppressionDuration",
    "BlindDuration",
    "SilenceDuration",
    "GrabDuration",
    "LanternDuration",
    "ShieldDuration",
    "ShieldMaxDuration",
    "MarkerDuration",
    "SpottingDuration",
    "RevealDuration",
    "LockoutTimer",
    "EnrageDuration",
    "CraterDuration",
    "FlameDuration",
    "HasteDuration",
    "ResistDuration",
    "AdrenalineStorageWindow",
    "AdrenalineDecayLockTimer",
    "GreyHealthDuration",
    "TakedownWindow",
    "RecastWindow",
    "DelayBeforeDecay",
    "OutOfCombatTimeBeforeReload",
    "InitialDelay",
    "SecondaryDelay",
    "TrapArmTime",
];

/// Stat names where the value represents movement speed.
const MOVE_SPEED_LABELS: &[&str] = &[
    "MSAmount",
    "MSDuration",
    "MovementSpeed",
    "MovementSpeedDuration",
    "MoveSpeedBonus",
    "SelfSlowPercent",
];

/// Stat names where the value represents a shield.
const SHIELD_LABELS: &[&str] = &[
    "BaseShieldValue",
    "ShieldPerSoul",
    "ShieldConversion",
    "ShieldAmount",
];

/// Stat names where the value represents healing.
const HEAL_LABELS: &[&str] = &["HealAmount", "EnrageHealingMult"];

/// Stat names where the value represents a ratio/scaling.
const RATIO_LABELS: &[&str] = &[
    "APRatio",
    "ADRatio",
    "PassiveADRatio",
    "PassiveADRatioTT",
    "RAPCoefficient",
    "MaxHealthTADRatio",
    "MaxHealthTADRatioTOOLTIP",
    "PercentAttackSpeedPerLevel",
    "CritMoveSpeedPercentASRatio",
    "JabDamagePercent",
    "HookDamagePercentFinal",
    "PassiveHealthThresholdAmp",
    "EnrageADMult",
    "EnrageArmorPen",
    "CritReductionPercent",
    "FourthShotDamageMult",
    "DamageConversionBase",
    "PassiveHealthThreshold",
];

/// Stat names where the value represents a range or distance.
const RANGE_LABELS: &[&str] = &[
    "LeapDistance",
    "DashDistance",
    "BounceRange",
    "ThrowRange",
    "AcquisitionRange",
    "BonusAcquisitionRange",
    "OrbitRadius",
    "CraterRadius",
    "CraterSweetSpotRadius",
    "TrapAoERadius",
    "TrapTriggerRadius",
    "DistanceToFirstHit",
    "DistancePastFirstHit",
    "SpellRange",
    "RDashDistance",
    "RBaseDashSpeed",
    "RAcquisitionRange",
    "CarryDistancePerStack",
];

/// Stat names where the value is a count.
const COUNT_LABELS: &[&str] = &[
    "NumberOfBounces",
    "NumHits",
    "StackCount",
    "MaxStacks",
    "MaxAmmo",
    "MaxPassiveStacks",
    "SoulsToGainOnPickUp",
    "RMaxTargetsPerCast",
    "RMaxCasts",
    "RResetCasts",
];

/// Stat names where the value represents self-cost.
const COST_LABELS: &[&str] = &["HealthCost", "BaseHealthCost"];

/// Decide whether a DataValue is interesting enough to surface to the player.
pub(crate) fn is_interesting_stat(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let upper = name.to_uppercase();
    if SKIP_STAT_NAMES
        .iter()
        .any(|skip| upper.contains(&skip.to_uppercase()))
    {
        return false;
    }
    // Explicit noise names (ADRatio, MonsterCap, etc.)
    if NOISE_NAMES.iter().any(|noise| name.eq_ignore_ascii_case(noise)) {
        return false;
    }
    if name.len() < 5 {
        return false;
    }
    true
}

/// Check whether stat values are noise — all identical, or all near-zero.
pub(crate) fn is_noise_stat(_name: &str, values: &[f64]) -> bool {
    if values.is_empty() {
        return true;
    }
    // All values identical (e.g. StackCount always 1)
    if values.len() > 1 && values.iter().all(|v| (v - values[0]).abs() < f64::EPSILON) {
        return true;
    }
    // All values near zero (e.g. MaxHealthTADRatio 0.00005)
    if values.iter().all(|v| v.abs() < 0.001) {
        return true;
    }
    false
}

/// Turn a raw DataValue name into a human-readable label and suffix.
///
/// Returns `(label, suffix)` where suffix is e.g. `"%"` or `"s"` or `""`.
/// Specific categories checked first (DAMAGE, SHIELD, HEAL, MOVE, RANGE, COUNT, COST, RATIO);
/// PERCENT fallback runs last.
pub(crate) fn clean_stat_label(name: &str) -> (String, String) {
    // DAMAGE — physical/magical/true damage
    for keyword in DAMAGE_LABELS {
        if name.eq_ignore_ascii_case(keyword) {
            let label = split_camel_case(name);
            return (label, String::new());
        }
    }

    // SHIELD
    for keyword in SHIELD_LABELS {
        if name.eq_ignore_ascii_case(keyword) {
            let label = split_camel_case(name);
            return (label, String::new());
        }
    }

    // HEAL
    for keyword in HEAL_LABELS {
        if name.eq_ignore_ascii_case(keyword) {
            let label = split_camel_case(name);
            return (label, String::new());
        }
    }

    // MOVE SPEED
    for keyword in MOVE_SPEED_LABELS {
        if name.eq_ignore_ascii_case(keyword) {
            let label = split_camel_case(name)
                .replace("MS ", "Move Speed ")
                .replace("MovementSpeed", "Move Speed")
                .replace("Duration", "")
                .trim()
                .to_string();
            if name.contains("Percent") || name.contains("Slow") {
                return (label, "%".to_string());
            }
            // MovementSpeedDuration / MSDuration — duration of speed buff
            if name.eq_ignore_ascii_case("MSDuration")
                || name.eq_ignore_ascii_case("MovementSpeedDuration")
                || name.contains("Bonus")
            {
                return (label, "s".to_string());
            }
            return (label, String::new());
        }
    }

    // RANGE / DISTANCE
    for keyword in RANGE_LABELS {
        if name.eq_ignore_ascii_case(keyword) {
            let label = split_camel_case(name);
            return (label, String::new());
        }
    }

    // COUNT
    for keyword in COUNT_LABELS {
        if name.eq_ignore_ascii_case(keyword) {
            let label = split_camel_case(name);
            return (label, String::new());
        }
    }

    // COST
    for keyword in COST_LABELS {
        if name.eq_ignore_ascii_case(keyword) {
            let label = split_camel_case(name);
            return (label, String::new());
        }
    }

    // RATIO — scaling ratios (displayed in description, low priority)
    for keyword in RATIO_LABELS {
        if name.eq_ignore_ascii_case(keyword) {
            let label = split_camel_case(name);
            return (label, String::new());
        }
    }

    // DURATION — values in seconds (check after specific categories)
    for keyword in DURATION_LABELS {
        if name.eq_ignore_ascii_case(keyword) {
            let label = split_camel_case(name)
                .replace("Duration", "")
                .replace("Length", "")
                .replace("Timer", "")
                .replace("Window", "")
                .replace("Delay", "")
                .trim()
                .to_string();
            return (label, "s".to_string());
        }
    }

    // PERCENT — fallback for names with explicit percentage indicators
    for keyword in PERCENT_SUFFIXES {
        if name.contains(keyword) {
            let label = split_camel_case(name);
            return (label, "%".to_string());
        }
    }

    // Fallback: generic CamelCase split
    (split_camel_case(name), String::new())
}

/// Auto-detect whether values should be scaled to percentage display.
/// Returns scaled values, abs() applied, and suffix override.
pub(crate) fn scale_percent_values(values: &[f64]) -> (Vec<f64>, String) {
    if values.is_empty() {
        return (vec![], String::new());
    }
    // Scale if all values in [0, 1] or [-1, 0] range AND any non-integer
    let all_in_unit_range = values.iter().all(|v| v.abs() <= 1.0);
    let any_fractional = values.iter().any(|v| (v.abs() % 1.0) > f64::EPSILON);
    if all_in_unit_range && any_fractional {
        let scaled: Vec<f64> = values.iter().map(|v| v.abs() * 100.0).collect();
        return (scaled, "%".to_string());
    }
    (values.to_vec(), String::new())
}

/// Split a CamelCase identifier into words, preserving acronyms.
///
/// "BaseDamage" → "Base Damage", "ADRatioPerTick" → "AD Ratio Per Tick".
fn split_camel_case(name: &str) -> String {
    let clean = name.split('*').next().unwrap_or(name);
    let chars: Vec<char> = clean.chars().collect();
    let mut result = String::with_capacity(clean.len() + 4);

    for (i, &c) in chars.iter().enumerate() {
        if i == 0 {
            result.push(c);
            continue;
        }
        if c.is_uppercase() {
            let prev_lower = chars[i - 1].is_lowercase();
            // End of an acronym run: prev is uppercase but next is lowercase → "ADR" → "AD R"
            let acronym_end = chars[i - 1].is_uppercase()
                && chars.get(i + 1).map(|n| n.is_lowercase()).unwrap_or(false);
            if prev_lower || acronym_end {
                result.push(' ');
            }
        }
        result.push(c);
    }
    result
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

    fn garen_q_raw() -> HashMap<String, serde_json::Value> {
        serde_json::from_value(serde_json::json!({
            "Characters/Garen/Spells/GarenQAbility/GarenQ": {
                "ObjectName": "GarenQ",
                "mSpell": {
                    "DataValues": [
                        {"name": "BaseDamage",           "values": [0.0, 30.0, 60.0, 90.0, 120.0, 150.0, 180.0], "__type": "SpellDataValue"},
                        {"name": "SilenceDuration",      "values": [1.5, 1.5, 1.5, 1.5, 1.5, 1.5, 1.5],          "__type": "SpellDataValue"},
                        {"name": "MovementSpeedDuration","values": [0.85, 1.4, 1.95, 2.5, 3.05, 3.6, 4.15],       "__type": "SpellDataValue"},
                        {"name": "MovementSpeedAmount",  "values": [0.35, 0.35, 0.35, 0.35, 0.35, 0.35, 0.35],   "__type": "SpellDataValue"},
                        {"name": "AttackWindow",         "values": [4.5, 4.5, 4.5, 4.5, 4.5, 4.5, 4.5],          "__type": "SpellDataValue"},
                        {"name": "tADRatio",             "values": [1.5, 1.5, 1.5, 1.5, 1.5, 1.5, 1.5],          "__type": "SpellDataValue"}
                    ],
                    "mSpellCalculations": {
                        "TotalDamage": {
                            "mFormulaParts": [
                                {"mDataValue": "BaseDamage", "__type": "NamedDataValueCalculationPart"},
                                {"mStat": 2, "mDataValue": "tADRatio", "__type": "StatByNamedDataValueCalculationPart"}
                            ],
                            "__type": "GameCalculation"
                        }
                    },
                    "mClientData": {
                        "mTooltipData": { "mLists": { "LevelUp": {"levelCount": 5} } }
                    }
                }
            }
        }))
        .expect("Garen Q fixture is valid JSON")
    }

    fn garen_r_raw() -> HashMap<String, serde_json::Value> {
        serde_json::from_value(serde_json::json!({
            "Characters/Garen/Spells/GarenRAbility/GarenR": {
                "ObjectName": "GarenR",
                "mSpell": {
                    "DataValues": [
                        {"name": "BaseDamage",    "values": [50.0, 150.0, 250.0, 350.0, 450.0, 550.0, 650.0], "__type": "SpellDataValue"},
                        {"name": "ExecuteDamage", "values": [0.20, 0.25, 0.30, 0.35, 0.40, 0.45, 0.50],       "__type": "SpellDataValue"},
                        {"name": "RevealDuration","values": [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],              "__type": "SpellDataValue"}
                    ],
                    "mClientData": {
                        "mTooltipData": { "mLists": { "LevelUp": {"levelCount": 3} } }
                    }
                }
            }
        }))
        .expect("Garen R fixture is valid JSON")
    }

    #[test]
    fn bin_champion_data_spell_keys_are_uppercase_callers_must_normalize() {
        // parse_bin_json stores spells under "Q", "W", "E", "R", "Passive" (uppercase).
        // Callers MUST call .to_uppercase() before get_spell.
        let data = parse_bin_json(&garen_q_raw(), "garen").expect("fixture parses");
        assert!(data.get_spell("Q").is_some(), "uppercase Q must be present");
        assert!(data.get_spell("q").is_none(), "lowercase q must NOT match");
    }

    #[test]
    fn garen_q_extra_type_field_does_not_break_deserialization() {
        // Real CDragon data includes "__type" fields — serde must ignore them.
        let data = parse_bin_json(&garen_q_raw(), "garen");
        assert!(data.is_some(), "parse_bin_json must succeed when __type fields are present");
        let q = data.unwrap().get_spell("Q").cloned();
        assert!(q.is_some(), "Q spell must be extracted");
        assert!(!q.unwrap().data_values.is_empty(), "DataValues must be populated despite __type fields");
    }

    #[test]
    fn garen_r_stats_use_rank_count_3() {
        // R has levelCount=3; BaseDamage has 7 values.
        // After rank_slice: skip(1).take(3) = [150, 250, 350].
        let data = parse_bin_json(&garen_r_raw(), "garen").expect("Garen R fixture parses");
        let r = data.get_spell("R").expect("Garen R spell present");
        assert_eq!(r.rank_count, 3, "R rank_count must be 3 (from levelCount)");
        let base_damage = r.data_values.iter().find(|dv| dv.name == "BaseDamage").unwrap();
        let sliced: Vec<f64> = base_damage.values.iter().copied().skip(1).take(3).collect();
        assert_eq!(sliced, vec![150.0, 250.0, 350.0]);
        assert!(!sliced.contains(&50.0), "rank-0 sentinel 50 must be excluded");
        assert!(!sliced.contains(&450.0), "values beyond rank 3 must be excluded");
    }

    #[test]
    fn split_camel_case_preserves_acronyms() {
        assert_eq!(split_camel_case("BaseDamage"), "Base Damage");
        assert_eq!(split_camel_case("BaseDamagePerTick"), "Base Damage Per Tick");
        assert_eq!(split_camel_case("ADRatioPerTick"), "AD Ratio Per Tick");
        assert_eq!(split_camel_case("APRatio"), "AP Ratio");
        assert_eq!(split_camel_case("tADRatio"), "t AD Ratio");
        assert_eq!(split_camel_case("Duration"), "Duration");
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
            format!("Characters/{}/Spells/{}PassiveAbility/{}Passive", name, name, name),
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