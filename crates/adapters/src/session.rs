use std::fmt;

use reqwest::{StatusCode, blocking::Client, header::CONTENT_TYPE};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::lockfile::LockfileCredentials;
use crate::constants::{LOCAL_LCU_HOST, REQUEST_TIMEOUT};
use crate::LcuAdapterError;
use domain::LeagueImageAsset;
use application::LeagueClientReadError;

#[derive(Clone)]
pub(crate) struct LcuSession {
    pub(crate) credentials: LockfileCredentials,
    pub(crate) http_client: Client,
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
        let http_client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(REQUEST_TIMEOUT)
            .no_proxy()
            .tls_danger_accept_invalid_certs(true)
            .build()
            .map_err(|_| LcuAdapterError::Http)?;

        Ok(Self {
            credentials,
            http_client,
        })
    }

    /// Sends a lightweight request to verify that the LCU is reachable and
    /// the credentials are valid. Called by `open_session` before returning Ready.
    pub(crate) fn verify(&self) -> Result<(), LcuAdapterError> {
        let url = format!(
            "https://{LOCAL_LCU_HOST}:{}/",
            self.credentials.port
        );
        self.http_client
            .get(&url)
            .basic_auth("riot", Some(self.credentials.password.as_str()))
            .send()
            .map_err(|_| LcuAdapterError::Http)?;
        Ok(())
    }

    pub(crate) fn get_json<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T, LcuRequestError> {
        let url = format!("https://{LOCAL_LCU_HOST}:{}{}", self.credentials.port, path);
        let response = self
            .http_client
            .get(url)
            .basic_auth("riot", Some(self.credentials.password.as_str()))
            .send()
            .map_err(|_| LcuRequestError::Unavailable)?;
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
        let response = self
            .http_client
            .post(url)
            .basic_auth("riot", Some(self.credentials.password.as_str()))
            .send()
            .map_err(|_| LcuRequestError::Unavailable)?;

        validate_lcu_status(response.status())
    }

    pub(crate) fn patch_json<T: Serialize>(&self, path: &str, body: &T) -> Result<(), LcuRequestError> {
        let url = format!("https://{LOCAL_LCU_HOST}:{}{}", self.credentials.port, path);
        let response = self
            .http_client
            .patch(url)
            .basic_auth("riot", Some(self.credentials.password.as_str()))
            .header(CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .map_err(|_| LcuRequestError::Unavailable)?;

        validate_lcu_status(response.status())
    }

    pub(crate) fn get_image_asset(
        &self,
        path: &str,
        fallback_mime_type: &str,
    ) -> Result<LeagueImageAsset, LcuRequestError> {
        let url = format!("https://{LOCAL_LCU_HOST}:{}{}", self.credentials.port, path);
        let response = self
            .http_client
            .get(url)
            .basic_auth("riot", Some(self.credentials.password.as_str()))
            .send()
            .map_err(|_| LcuRequestError::Unavailable)?;
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
    let response = http_client.get(url).send().map_err(|_| {
        LeagueClientReadError::ClientUnavailable("Live Client API is unavailable".to_string())
    })?;

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
}
