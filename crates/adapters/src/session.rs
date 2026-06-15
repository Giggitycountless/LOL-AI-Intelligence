use std::fmt;
use std::sync::{Arc, Mutex};

use reqwest::{StatusCode, blocking::Client, header::CONTENT_TYPE};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::lockfile::LockfileCredentials;
use crate::constants::{LOCAL_LCU_HOST, REQUEST_TIMEOUT};
use crate::LcuAdapterError;
use domain::LeagueImageAsset;
use application::LeagueClientReadError;

/// Retries a fallible operation up to `max_retries` times with exponential
/// backoff. Only retries when `is_retryable` returns true for the error.
/// Backoff: 200ms × 2^(attempt-1) → 200ms, 400ms, 800ms, …
fn with_retry<F, T, E>(
    mut operation: F,
    max_retries: u32,
    is_retryable: fn(&E) -> bool,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    // Every LCU request flows through here, and each `operation()` is a blocking
    // reqwest call that builds/drops a Tokio runtime internally. Run the whole
    // retry loop (requests + backoff sleeps) in a blocking region so it's safe to
    // call from the auto-accept monitor's async websocket loop.
    run_blocking(move || {
        let mut attempt = 0;
        loop {
            match operation() {
                Ok(value) => return Ok(value),
                Err(err) => {
                    if attempt >= max_retries || !is_retryable(&err) {
                        return Err(err);
                    }
                    attempt += 1;
                    let backoff_ms = 200 * 2u64.pow(attempt.saturating_sub(1));
                    std::thread::sleep(std::time::Duration::from_millis(backoff_ms));
                }
            }
        }
    })
}

/// One process-wide blocking HTTP client shared by every LCU session.
///
/// `reqwest::blocking::Client` owns an internal Tokio runtime. Dropping that
/// runtime inside an async context panics ("Cannot drop a runtime in a context
/// where blocking is not allowed") — and the auto-accept monitor calls blocking
/// LCU methods from inside its async websocket loop. Keeping a single client
/// alive in a static means per-call session clones only decrement a refcount on
/// drop; the runtime is never torn down during operation (and statics are not
/// dropped at process exit, so it is never torn down at all).
static LCU_HTTP_CLIENT: Mutex<Option<Arc<Client>>> = Mutex::new(None);

fn build_lcu_http_client() -> Result<Client, LcuAdapterError> {
    Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .connect_timeout(REQUEST_TIMEOUT)
        .no_proxy()
        .tls_danger_accept_invalid_certs(true)
        .build()
        .map_err(|_| LcuAdapterError::Http)
}

/// Runs a blocking LCU operation safely regardless of the calling context.
///
/// `reqwest::blocking` builds and tears down a Tokio runtime on every call (both
/// client construction and each request). Doing that inside ANY async runtime
/// context panics in debug builds ("Cannot drop a runtime in a context where
/// blocking is not allowed"). Blocking LCU calls reach us from several contexts —
/// the auto-accept monitor's multi-threaded runtime (sometimes already inside
/// `block_in_place`), Tauri command dispatch (which can be a current-thread
/// runtime, where `block_in_place` is illegal), rayon prefetch threads, and plain
/// sync threads. `block_in_place` only covers one of those, so instead: if we are
/// inside any Tokio runtime, offload the work to a fresh OS thread that has no
/// async context (reqwest's internal runtime is then built/dropped freely); only
/// outside a runtime do we run directly.
pub(crate) fn run_blocking<T>(operation: impl FnOnce() -> T) -> T {
    use tokio::runtime::{Handle, RuntimeFlavor};
    match Handle::try_current() {
        Ok(handle) if handle.runtime_flavor() == RuntimeFlavor::MultiThread => {
            tokio::task::block_in_place(operation)
        }
        _ => operation(),
    }
}

/// Returns the shared LCU client, building it once on first use. Sessions hold an
/// `Arc<Client>`, so dropping a session only decrements the refcount — the client
/// (and its internal `reqwest::blocking` runtime) is never torn down during
/// operation, which is what previously panicked when the auto-accept monitor
/// dropped a session inside its async websocket loop. Building happens inside the
/// lock so exactly one client is ever constructed.
pub(crate) fn shared_lcu_http_client() -> Result<Arc<Client>, LcuAdapterError> {
    let mut guard = LCU_HTTP_CLIENT
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(client) = guard.as_ref() {
        return Ok(Arc::clone(client));
    }
    let client = Arc::new(run_blocking(build_lcu_http_client)?);
    *guard = Some(Arc::clone(&client));
    Ok(client)
}

#[derive(Clone)]
pub(crate) struct LcuSession {
    pub(crate) credentials: LockfileCredentials,
    pub(crate) http_client: Arc<Client>,
}

impl fmt::Debug for LcuSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LcuSession")
            .field("port", &self.credentials.port)
            .field("http_client", &self.http_client)
            .finish()
    }
}

impl LcuSession {
    pub(crate) fn new(credentials: LockfileCredentials) -> Result<Self, LcuAdapterError> {
        Ok(Self {
            credentials,
            http_client: shared_lcu_http_client()?,
        })
    }

    /// Sends a lightweight request to verify that the LCU is reachable and
    /// the credentials are valid. Called by `open_session` before returning Ready.
    pub(crate) fn verify(&self) -> Result<(), LcuAdapterError> {
        let url = format!("https://{LOCAL_LCU_HOST}:{}/", self.credentials.port);
        with_retry(
            || {
                self.http_client
                    .get(url.as_str())
                    .basic_auth("riot", Some(self.credentials.password.as_str()))
                    .send()
                    .map_err(|_| LcuAdapterError::Http)
            },
            3,
            |_: &LcuAdapterError| true,
        )?;
        Ok(())
    }

    pub(crate) fn get_json<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
    ) -> Result<T, LcuRequestError> {
        let url = format!("https://{LOCAL_LCU_HOST}:{}{}", self.credentials.port, path);
        let response = with_retry(
            || {
                self.http_client
                    .get(url.as_str())
                    .basic_auth("riot", Some(self.credentials.password.as_str()))
                    .send()
                    .map_err(|_| LcuRequestError::Unavailable)
            },
            3,
            |e: &LcuRequestError| matches!(e, LcuRequestError::Unavailable),
        )?;
        let status = response.status();

        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            return Err(LcuRequestError::Unauthorized);
        }

        if status == StatusCode::NOT_FOUND {
            return Err(LcuRequestError::NotLoggedIn);
        }

        if status == StatusCode::SERVICE_UNAVAILABLE {
            return Err(LcuRequestError::Patching);
        }

        if !status.is_success() {
            return Err(LcuRequestError::Unavailable);
        }

        response
            .json::<T>()
            .map_err(|_| LcuRequestError::Unexpected)
    }

    pub(crate) fn post_empty(&self, path: &str) -> Result<(), LcuRequestError> {
        let url = format!("https://{LOCAL_LCU_HOST}:{}{}", self.credentials.port, path);
        let response = with_retry(
            || {
                self.http_client
                    .post(url.as_str())
                    .basic_auth("riot", Some(self.credentials.password.as_str()))
                    .send()
                    .map_err(|_| LcuRequestError::Unavailable)
            },
            3,
            |e: &LcuRequestError| matches!(e, LcuRequestError::Unavailable),
        )?;

        validate_lcu_status(response.status())
    }

    pub(crate) fn post_json<T: Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<(), LcuRequestError> {
        let url = format!("https://{LOCAL_LCU_HOST}:{}{}", self.credentials.port, path);
        let response = with_retry(
            || {
                self.http_client
                    .post(url.as_str())
                    .basic_auth("riot", Some(self.credentials.password.as_str()))
                    .header(CONTENT_TYPE, "application/json")
                    .json(body)
                    .send()
                    .map_err(|_| LcuRequestError::Unavailable)
            },
            3,
            |e: &LcuRequestError| matches!(e, LcuRequestError::Unavailable),
        )?;

        validate_lcu_status(response.status())
    }

    pub(crate) fn delete_empty(&self, path: &str) -> Result<(), LcuRequestError> {
        let url = format!("https://{LOCAL_LCU_HOST}:{}{}", self.credentials.port, path);
        let response = with_retry(
            || {
                self.http_client
                    .delete(url.as_str())
                    .basic_auth("riot", Some(self.credentials.password.as_str()))
                    .send()
                    .map_err(|_| LcuRequestError::Unavailable)
            },
            3,
            |e: &LcuRequestError| matches!(e, LcuRequestError::Unavailable),
        )?;

        validate_lcu_status(response.status())
    }

    pub(crate) fn patch_json<T: Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<(), LcuRequestError> {
        let url = format!("https://{LOCAL_LCU_HOST}:{}{}", self.credentials.port, path);
        let response = with_retry(
            || {
                self.http_client
                    .patch(url.as_str())
                    .basic_auth("riot", Some(self.credentials.password.as_str()))
                    .header(CONTENT_TYPE, "application/json")
                    .json(body)
                    .send()
                    .map_err(|_| LcuRequestError::Unavailable)
            },
            3,
            |e: &LcuRequestError| matches!(e, LcuRequestError::Unavailable),
        )?;

        validate_lcu_status(response.status())
    }

    pub(crate) fn get_image_asset(
        &self,
        path: &str,
        fallback_mime_type: &str,
    ) -> Result<LeagueImageAsset, LcuRequestError> {
        let url = format!("https://{LOCAL_LCU_HOST}:{}{}", self.credentials.port, path);
        let response = with_retry(
            || {
                self.http_client
                    .get(url.as_str())
                    .basic_auth("riot", Some(self.credentials.password.as_str()))
                    .send()
                    .map_err(|_| LcuRequestError::Unavailable)
            },
            3,
            |e: &LcuRequestError| matches!(e, LcuRequestError::Unavailable),
        )?;
        let status = response.status();

        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            return Err(LcuRequestError::Unauthorized);
        }

        if status == StatusCode::NOT_FOUND {
            return Err(LcuRequestError::NotLoggedIn);
        }

        if status == StatusCode::SERVICE_UNAVAILABLE {
            return Err(LcuRequestError::Patching);
        }

        if !status.is_success() {
            return Err(LcuRequestError::Unavailable);
        }

        let mime_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .filter(|value| value.starts_with("image/"))
            .unwrap_or(fallback_mime_type)
            .to_string();
        let bytes = response
            .bytes()
            .map_err(|_| LcuRequestError::Unexpected)?
            .to_vec();

        if bytes.is_empty() {
            return Err(LcuRequestError::Unexpected);
        }

        Ok(LeagueImageAsset { mime_type, bytes })
    }
}

pub(crate) fn live_client_get_json<T: DeserializeOwned>(
    http_client: &Client,
    path: &str,
) -> Result<T, LeagueClientReadError> {
    let url = format!("https://127.0.0.1:2999{path}");
    let response = with_retry(
        || {
            http_client
                .get(url.as_str())
                .send()
                .map_err(|_| {
                    LeagueClientReadError::ClientUnavailable(
                        "Live Client API is unavailable".to_string(),
                    )
                })
        },
        3,
        |_: &LeagueClientReadError| true,
    )?;

    if !response.status().is_success() {
        return Err(LeagueClientReadError::ClientUnavailable(
            "Live Client API did not return active game data".to_string(),
        ));
    }

    response.json::<T>().map_err(|_| {
        LeagueClientReadError::Integration("Live Client API response could not be read".to_string())
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LcuRequestError {
    Unauthorized,
    NotLoggedIn,
    Patching,
    Unavailable,
    Unexpected,
}

pub(crate) fn validate_lcu_status(status: StatusCode) -> Result<(), LcuRequestError> {
    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        return Err(LcuRequestError::Unauthorized);
    }

    if status == StatusCode::NOT_FOUND {
        return Err(LcuRequestError::NotLoggedIn);
    }

    if status == StatusCode::SERVICE_UNAVAILABLE {
        return Err(LcuRequestError::Patching);
    }

    if !status.is_success() {
        return Err(LcuRequestError::Unavailable);
    }

    Ok(())
}

// ── tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lockfile::LockfileCredentials;

    fn dummy() -> LockfileCredentials {
        LockfileCredentials {
            port: 1,
            password: "test".to_string(),
            pid: 1234,
        }
    }

    #[test]
    fn new_constructs_without_network() {
        let session = LcuSession::new(dummy()).expect("builds");
        assert_eq!(session.credentials.port, 1);
    }

    #[test]
    fn raw_reqwest_blocking_in_async_does_not_panic() {
        // Verifies the `[profile.*.package.reqwest] debug-assertions = false` override.
        // A raw blocking reqwest build + request + drop inside an async context would
        // hit reqwest's debug-only `wait::enter` assertion and panic ("Cannot drop a
        // runtime in a context where blocking is not allowed"); the override compiles
        // that assertion out, matching release behaviour. No block_in_place here.
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("runtime builds");
        runtime.block_on(async {
            let client = Client::builder()
                .timeout(std::time::Duration::from_millis(50))
                .build()
                .expect("client builds");
            let _ = client.get("http://127.0.0.1:1/").send();
            drop(client);
        });
    }

    #[test]
    fn blocking_lcu_calls_inside_async_context_do_not_panic() {
        // Regression: reqwest::blocking builds and tears down a Tokio runtime on
        // both client construction and every request. Doing that inside an async
        // context panics in debug builds ("Cannot drop a runtime in a context where
        // blocking is not allowed"). The auto-accept monitor calls blocking LCU
        // methods from inside its async websocket loop, so building a session AND
        // issuing a request from async must be safe (no server needed — the panic
        // fires before the connection attempt).
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("runtime builds");
        runtime.block_on(async {
            // The auto-accept monitor already wraps its blocking work in block_in_place,
            // so our own blocking happens *nested* inside an existing block_in_place region.
            // Reproduce that exact structure.
            tokio::task::block_in_place(|| {
                let session = LcuSession::new(dummy()).expect("builds");
                let _ = session.verify(); // issues a blocking request; must not panic
                drop(session);
            });
        });
    }

    #[test]
    fn verify_fails_when_no_server() {
        let session = LcuSession::new(dummy()).expect("builds");
        assert!(session.verify().is_err());
    }

    #[test]
    fn debug_redacts_password() {
        let session = LcuSession::new(dummy()).expect("builds");
        let msg = format!("{session:?}");
        assert!(!msg.contains("test"));
    }

    // ── with_retry ─────────────────────────────────────────────

    #[test]
    fn retry_succeeds_when_first_attempt_works() {
        let result = with_retry(|| Ok::<_, &str>(42), 3, |_: &&str| true);
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn retry_succeeds_on_third_attempt() {
        let mut calls = 0u32;
        let result = with_retry(
            || {
                calls += 1;
                if calls < 3 {
                    Err("fail")
                } else {
                    Ok(99)
                }
            },
            3,
            |_: &&str| true,
        );
        assert_eq!(result, Ok(99));
        assert_eq!(calls, 3);
    }

    #[test]
    fn retry_gives_up_after_max_retries() {
        let mut calls = 0u32;
        let result = with_retry(
            || {
                calls += 1;
                Err::<(), _>("always fails")
            },
            3,
            |_: &&str| true,
        );
        assert!(result.is_err());
        assert_eq!(calls, 4); // initial + 3 retries = 4 attempts
    }

    #[test]
    fn retry_stops_on_non_retryable_error() {
        let mut calls = 0u32;
        let result = with_retry(
            || {
                calls += 1;
                Err::<(), _>("bad")
            },
            3,
            |_: &&str| false, // never retry
        );
        assert!(result.is_err());
        assert_eq!(calls, 1);
    }

    #[test]
    fn retry_backoff_is_respected() {
        let start = std::time::Instant::now();
        let result = with_retry(
            || Err::<(), _>("fail"),
            3,
            |_: &&str| true,
        );
        assert!(result.is_err());
        let elapsed = start.elapsed();
        // 3 retries: 200ms + 400ms + 800ms = 1.4s
        // Allow some slack for scheduling jitter
        assert!(
            elapsed >= std::time::Duration::from_millis(1200),
            "elapsed {elapsed:?} should be >= 1.2s"
        );
    }
}
