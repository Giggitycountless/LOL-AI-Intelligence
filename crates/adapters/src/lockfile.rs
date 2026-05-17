use std::fmt;

use crate::LcuAdapterError;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct LockfileCredentials {
    pub(crate) port: u16,
    pub(crate) password: String,
    pub(crate) pid: u32,
}

impl fmt::Debug for LockfileCredentials {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LockfileCredentials")
            .field("port", &self.port)
            .field("password", &"<redacted>")
            .finish()
    }
}

pub(crate) fn parse_lockfile(contents: &str) -> Result<LockfileCredentials, LcuAdapterError> {
    let parts: Vec<&str> = contents.trim().split(':').collect();

    if parts.len() != 5 {
        return Err(LcuAdapterError::InvalidLockfile);
    }

    let name = parts[0].trim();
    let pid = parts[1].trim();
    let port = parts[2].trim();
    let password = parts[3].trim();
    let protocol = parts[4].trim();

    if name.is_empty() || password.is_empty() {
        return Err(LcuAdapterError::InvalidLockfile);
    }

    let parsed_pid = pid
        .parse::<u32>()
        .map_err(|_| LcuAdapterError::InvalidLockfile)?;
    if parsed_pid == 0 {
        return Err(LcuAdapterError::InvalidLockfile);
    }

    let port = port
        .parse::<u16>()
        .map_err(|_| LcuAdapterError::InvalidLockfile)?;
    if port == 0 {
        return Err(LcuAdapterError::InvalidLockfile);
    }

    if protocol != "https" {
        return Err(LcuAdapterError::InvalidLockfile);
    }

    Ok(LockfileCredentials {
        port,
        password: password.to_string(),
        pid: parsed_pid,
    })
}

// ── tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_lockfile_with_pid() {
        let creds = parse_lockfile("LeagueClient:12345:56789:secret_pass:https").unwrap();
        assert_eq!(creds.port, 56789);
        assert_eq!(creds.password, "secret_pass");
        assert_eq!(creds.pid, 12345);
    }

    #[test]
    fn rejects_zero_pid() {
        let result = parse_lockfile("LeagueClient:0:12345:pass:https");
        assert!(result.is_err());
    }

    #[test]
    fn rejects_empty_name() {
        let result = parse_lockfile(":1234:5678:pass:https");
        assert!(result.is_err());
    }

    #[test]
    fn rejects_non_https() {
        let result = parse_lockfile("LeagueClient:1234:5678:pass:http");
        assert!(result.is_err());
    }

    #[test]
    fn rejects_wrong_field_count() {
        let result = parse_lockfile("a:b:c:d");
        assert!(result.is_err());
    }
}
