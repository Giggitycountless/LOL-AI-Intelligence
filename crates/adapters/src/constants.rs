use std::time::Duration;

pub(crate) const LOCAL_LCU_HOST: &str = "127.0.0.1";
pub(crate) const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);
pub(crate) const LEAGUE_CLIENT_PROCESSES: [&str; 2] = ["LeagueClientUx.exe", "LeagueClient.exe"];
pub(crate) const PROFILE_ICON_MIME: &str = "image/jpeg";
pub(crate) const CHAMPION_ICON_MIME: &str = "image/png";
pub(crate) const GAME_ASSET_MIME: &str = "image/png";
pub(crate) const MAX_COMPLETED_MATCH_SCAN: i64 = 20;
/// Upper bound on concurrent per-player LCU lookups in one batch. The LCU
/// proxies match-history/ranked/mastery reads to remote platform services
/// that rate-limit bursts; a full ten-player lobby fired through the global
/// rayon pool in one wave is exactly the shape that trips them into 429/500.
/// 5 stays well under that ~10-in-flight failure shape while halving the
/// number of sequential chunks (and thus overlay load latency) for a full
/// 5v5 lobby versus the original cap of 3.
pub(crate) const LCU_BATCH_CONCURRENCY: usize = 5;
pub(crate) const RANKED_CHAMPION_REMOTE_TIMEOUT: Duration = Duration::from_secs(5);
pub(crate) const RANKED_CHAMPION_FORMAT_VERSION: i64 = 1;
pub(crate) const ADVISOR_DATA_FORMAT_VERSION: i64 = 1;
