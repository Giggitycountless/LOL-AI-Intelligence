use std::fmt;

use crate::LcuAdapterError;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct LockfileCredentials {
    pub(crate) port: u16,
    pub(crate) password: String,
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
    })
}
