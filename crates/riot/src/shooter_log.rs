//! Client build and region from the head of `ShooterGame.log`.
//!
//! Riot gates newer services on `X-Riot-ClientVersion` and answers a stale version with empty
//! data rather than an error, so the header must be the RUNNING build, read from this log.
//! Both markers are written once near the top (version ~6 KB in, region ~40 KB in); a tail
//! scan stops working minutes into a session.

use std::path::PathBuf;

use tokio::io::AsyncReadExt;

const LOG_HEAD_BYTES: u64 = 262_144;
const CLIENT_VERSION_MARKER: &str = "CI server version: ";
const REGION_MARKER: &str = "regions/";

/// What the current session's log says, each part empty when absent.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LogFacts {
    pub client_version: String,
    pub region: String,
}

/// `%LOCALAPPDATA%\VALORANT\Saved\Logs\ShooterGame.log`.
pub fn shooter_log_path() -> PathBuf {
    let local_app_data = std::env::var_os("LOCALAPPDATA").unwrap_or_default();
    PathBuf::from(local_app_data).join(r"VALORANT\Saved\Logs\ShooterGame.log")
}

/// Read the log head; an unreadable log yields empty facts (callers have fallbacks).
pub async fn read_log_facts() -> LogFacts {
    match read_head().await {
        Ok(head) => parse_log_head(&head),
        Err(error) => {
            tracing::debug!(%error, "shooter_log_unreadable");
            LogFacts::default()
        }
    }
}

async fn read_head() -> std::io::Result<String> {
    let file = tokio::fs::File::open(shooter_log_path()).await?;
    let mut bytes = Vec::new();
    file.take(LOG_HEAD_BYTES).read_to_end(&mut bytes).await?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// Extract the facts from log text.
pub fn parse_log_head(head: &str) -> LogFacts {
    let client_version = head
        .split_once(CLIENT_VERSION_MARKER)
        .and_then(|(_, rest)| rest.split_whitespace().next())
        .unwrap_or_default();
    let region = head
        .split_once(REGION_MARKER)
        .and_then(|(_, rest)| rest.split_once(']'))
        .map(|(region, _)| region.trim())
        .unwrap_or_default();
    LogFacts {
        client_version: client_version.to_owned(),
        region: region.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_version_and_region_markers() {
        let head = concat!(
            "[x] LogInit: CI server version: release-13.08-shipping-19-5698904\n",
            "[y] GET https://euc1.pp.sgp.pvp.net/login-queue/v2/login/products/valorant/regions/eu]\n",
        );
        let facts = parse_log_head(head);
        assert_eq!(facts.client_version, "release-13.08-shipping-19-5698904");
        assert_eq!(facts.region, "eu");
    }

    #[test]
    fn missing_markers_give_empty_facts() {
        assert_eq!(parse_log_head("nothing here"), LogFacts::default());
    }
}
