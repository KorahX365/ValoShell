//! The Riot Client lockfile: `name:pid:port:password:protocol`, present only while the client
//! runs. Port and password change on every launch, so everything built from it is per launch.

use std::path::PathBuf;
use std::time::SystemTime;

use crate::error::{RiotError, RiotResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lockfile {
    pub pid: u32,
    pub port: u16,
    pub password: String,
}

/// `%LOCALAPPDATA%\Riot Games\Riot Client\Config\lockfile`.
pub fn lockfile_path() -> PathBuf {
    let local_app_data = std::env::var_os("LOCALAPPDATA").unwrap_or_default();
    PathBuf::from(local_app_data).join(r"Riot Games\Riot Client\Config\lockfile")
}

/// Read and parse the lockfile.
pub async fn read_lockfile() -> RiotResult<Lockfile> {
    let path = lockfile_path();
    let text = tokio::fs::read_to_string(&path)
        .await
        .map_err(|_| RiotError::LockfileMissing {
            path: path.display().to_string(),
        })?;
    parse_lockfile(&text)
}

/// Modification time of the lockfile; `None` while the Riot Client is closed.
/// A new stamp means a new Riot Client session.
pub async fn lockfile_stamp() -> Option<SystemTime> {
    tokio::fs::metadata(lockfile_path())
        .await
        .ok()?
        .modified()
        .ok()
}

/// Parse `name:pid:port:password:protocol`.
pub fn parse_lockfile(text: &str) -> RiotResult<Lockfile> {
    let malformed = |reason: &str| RiotError::LockfileMalformed {
        reason: reason.to_owned(),
    };
    let fields: Vec<&str> = text.trim().split(':').collect();
    let [_name, pid, port, password, _protocol] = fields.as_slice() else {
        return Err(malformed("expected name:pid:port:password:protocol"));
    };
    if password.is_empty() {
        return Err(malformed("empty password"));
    }
    Ok(Lockfile {
        pid: pid.parse().map_err(|_| malformed("pid is not a number"))?,
        port: port
            .parse()
            .map_err(|_| malformed("port is not a number"))?,
        password: (*password).to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_five_fields() {
        let lockfile = parse_lockfile("Riot Client:12345:54321:s3cr3t:https\n").expect("valid");
        assert_eq!(
            lockfile,
            Lockfile {
                pid: 12345,
                port: 54321,
                password: "s3cr3t".to_owned()
            }
        );
    }

    #[test]
    fn rejects_truncated_or_non_numeric_content() {
        assert!(parse_lockfile("Riot Client:12345").is_err());
        assert!(parse_lockfile("Riot Client:abc:54321:pw:https").is_err());
        assert!(parse_lockfile("").is_err());
    }
}
