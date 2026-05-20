//! Tencent LoL API adapter — fetches pre-resolved champion spell data from
//! game.gtimg.cn. Spell numerics (cooldown, cost, range) arrive as
//! slash-separated per-rank strings; no token parsing required.
//!
//! Responses are cached for 48 hours (patch-stable data). Use `prefetch_all`
//! at app startup to warm the cache before the user enters champion select.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use reqwest::blocking::Client;
use serde::Deserialize;

const TENCENT_BASE: &str = "https://game.gtimg.cn/images/lol/act/img/js/hero";
const CACHE_TTL: Duration = Duration::from_secs(48 * 3600);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(8);

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// One champion spell as returned by the Tencent LoL API.
#[derive(Debug, Clone)]
pub struct TencentSpell {
    pub spell_key: String,
    pub name: String,
    /// Simplified Chinese description with inlined 【value/value/…】 brackets.
    pub description: String,
    /// Slash-separated per-rank cooldown string, e.g. `"8/7/6/5/4"`.
    pub cooldown_burn: String,
    pub cost_burn: String,
    pub range_burn: String,
}

/// Champion data fetched from `game.gtimg.cn/images/lol/act/img/js/hero/{id}.js`.
/// Spells are keyed by lowercased `spellKey` ("passive", "q", "w", "e", "r").
#[derive(Debug, Clone)]
pub struct TencentChampionData {
    pub hero_id: String,
    pub alias: String,
    spells: HashMap<String, TencentSpell>,
}

impl TencentChampionData {
    /// Look up a spell by slot key, case-insensitive.
    pub fn spell(&self, key: &str) -> Option<&TencentSpell> {
        self.spells.get(&key.to_lowercase())
    }
}

// ---------------------------------------------------------------------------
// Raw JSON deserialization types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct RawRoot {
    hero: RawHero,
    spells: Vec<RawSpell>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawHero {
    hero_id: String,
    alias: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawSpell {
    spell_key: String,
    name: String,
    description: String,
    #[serde(default)]
    cooldown_burn: String,
    #[serde(default)]
    cost_burn: String,
    #[serde(default)]
    range_burn: String,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TencentLolClient {
    http: Client,
    cache: Arc<Mutex<HashMap<i64, (Instant, TencentChampionData)>>>,
    base_url: String,
}

impl Default for TencentLolClient {
    fn default() -> Self {
        Self::new()
    }
}

impl TencentLolClient {
    pub fn new() -> Self {
        let http = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .build()
            .expect("Tencent LoL API HTTP client builds");
        Self {
            http,
            cache: Arc::new(Mutex::new(HashMap::new())),
            base_url: TENCENT_BASE.to_string(),
        }
    }

    /// Fetch champion data, returning `None` on any network or parse error.
    ///
    /// When `expected_alias` is supplied the response `alias` field is compared
    /// case-insensitively. A mismatch logs a warning and returns `None` so the
    /// caller falls back to CDragon.
    pub fn fetch_champion_data(
        &self,
        champion_id: i64,
        expected_alias: Option<&str>,
    ) -> Option<TencentChampionData> {
        {
            let cache = self.cache.lock().unwrap_or_else(|p| p.into_inner());
            if let Some((fetched_at, data)) = cache.get(&champion_id) {
                if fetched_at.elapsed() < CACHE_TTL {
                    return Some(data.clone());
                }
            }
        }

        let url = format!("{}/{champion_id}.js", self.base_url);
        let bytes = match self.http.get(&url).send().and_then(|r| r.bytes()) {
            Ok(b) => b,
            Err(e) => {
                crate::log_lcu_adapter_event(&format!(
                    "tencent-api fetch failed champion_id={champion_id} error={e}"
                ));
                return None;
            }
        };

        if bytes.iter().find(|&&b| !b.is_ascii_whitespace()) != Some(&b'{') {
            crate::log_lcu_adapter_event(&format!(
                "tencent-api unexpected response format champion_id={champion_id}"
            ));
            return None;
        }

        let raw: RawRoot = match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(e) => {
                crate::log_lcu_adapter_event(&format!(
                    "tencent-api parse failed champion_id={champion_id} error={e}"
                ));
                return None;
            }
        };

        if let Some(expected) = expected_alias {
            if !raw.hero.alias.eq_ignore_ascii_case(expected) {
                crate::log_lcu_adapter_event(&format!(
                    "tencent-api alias mismatch champion_id={champion_id} \
                     expected={expected} got={}",
                    raw.hero.alias
                ));
                return None;
            }
        }

        let spells = raw
            .spells
            .into_iter()
            .map(|s| {
                let key = s.spell_key.to_lowercase();
                let spell = TencentSpell {
                    spell_key: s.spell_key,
                    name: s.name,
                    description: s.description,
                    cooldown_burn: s.cooldown_burn,
                    cost_burn: s.cost_burn,
                    range_burn: s.range_burn,
                };
                (key, spell)
            })
            .collect();

        let data = TencentChampionData {
            hero_id: raw.hero.hero_id,
            alias: raw.hero.alias,
            spells,
        };

        self.cache
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(champion_id, (Instant::now(), data.clone()));

        Some(data)
    }

    /// Warm the cache for all given champion IDs in parallel.
    /// Errors are silently discarded; this is a best-effort startup optimisation.
    pub fn prefetch_all(&self, champion_ids: &[i64]) {
        use rayon::prelude::*;
        champion_ids.par_iter().for_each(|&id| {
            self.fetch_champion_data(id, None);
        });
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
impl TencentLolClient {
    /// Test constructor: pre-populate the cache and route all HTTP through
    /// `base_url` (use `"http://127.0.0.1:1"` to get immediate ECONNREFUSED
    /// if a network call is made unexpectedly).
    pub(crate) fn with_prefilled_cache(
        entries: impl IntoIterator<Item = (i64, TencentChampionData)>,
        base_url: &str,
    ) -> Self {
        let http = Client::builder()
            .connect_timeout(Duration::from_millis(200))
            .timeout(Duration::from_millis(500))
            .build()
            .expect("test client builds");
        let now = Instant::now();
        let cache = entries
            .into_iter()
            .map(|(id, data)| (id, (now, data)))
            .collect();
        Self {
            http,
            cache: Arc::new(Mutex::new(cache)),
            base_url: base_url.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_garen() -> TencentChampionData {
        TencentChampionData {
            hero_id: "86".to_string(),
            alias: "Garen".to_string(),
            spells: HashMap::from([(
                "q".to_string(),
                TencentSpell {
                    spell_key: "Q".to_string(),
                    name: "斩钢闪".to_string(),
                    description: "每【8/7/6/5/4】秒冷却".to_string(),
                    cooldown_burn: "8/7/6/5/4".to_string(),
                    cost_burn: "0".to_string(),
                    range_burn: "300".to_string(),
                },
            )]),
        }
    }

    /// Cache hit: pre-populated entry is returned without a network call.
    /// The client is configured to fail immediately on any HTTP attempt
    /// (127.0.0.1:1 → ECONNREFUSED), so a spurious fetch would surface as
    /// `None` and fail the assertion below.
    #[test]
    fn cache_hit_returns_entry_without_network_call() {
        let client = TencentLolClient::with_prefilled_cache(
            [(86, sample_garen())],
            "http://127.0.0.1:1",
        );
        let result = client.fetch_champion_data(86, Some("Garen"));
        assert!(result.is_some(), "should return cached entry");
        let data = result.unwrap();
        assert_eq!(data.alias, "Garen");
        let q = data.spell("q").expect("Q must be present in cache");
        assert_eq!(q.cooldown_burn, "8/7/6/5/4");
    }

    /// Cache miss: when the requested ID is absent from the cache, the client
    /// attempts a network call. With an unreachable base URL the call fails
    /// immediately and `None` is returned — no panic.
    #[test]
    fn cache_miss_returns_none_on_network_failure() {
        // Cache has Garen (86); we request Ahri (103) — forces a network attempt.
        let client = TencentLolClient::with_prefilled_cache(
            [(86, sample_garen())],
            "http://127.0.0.1:1",
        );
        let result = client.fetch_champion_data(103, Some("Ahri"));
        assert!(
            result.is_none(),
            "cache miss with unreachable server must return None, not panic"
        );
    }

    /// Cache hit bypasses alias validation: the alias check only runs on a
    /// fresh network fetch. A stale/wrong `expected_alias` must not evict
    /// a valid cached entry.
    #[test]
    fn cache_hit_bypasses_alias_validation() {
        let client = TencentLolClient::with_prefilled_cache(
            [(86, sample_garen())],
            "http://127.0.0.1:1",
        );
        // Pass a mismatched alias — cache hit must still return the entry.
        let result = client.fetch_champion_data(86, Some("Ahri"));
        assert!(result.is_some(), "cache hit must bypass alias re-validation");
        assert_eq!(result.unwrap().alias, "Garen");
    }

    #[test]
    #[ignore = "requires network — run with: cargo test -p adapters -- --ignored tencent_fetch_garen"]
    fn tencent_fetch_garen() {
        let client = TencentLolClient::new();
        let data = client
            .fetch_champion_data(86, Some("Garen"))
            .expect("Garen (86) should be available on Tencent API");

        assert_eq!(data.alias, "Garen");

        let q = data.spell("q").expect("Garen should have a Q spell");

        assert_ne!(q.cooldown_burn, "0", "Q cooldown should not be flat zero");
        assert!(
            q.cooldown_burn.contains('/'),
            "Q cooldown should be a slash-separated per-rank string, got: {:?}",
            q.cooldown_burn
        );
        assert!(
            !q.description.is_empty(),
            "Q description should contain Chinese text"
        );
        assert!(
            q.description.contains('【'),
            "Q description should contain 【】 numeric brackets, got: {:?}",
            q.description
        );
    }
}
