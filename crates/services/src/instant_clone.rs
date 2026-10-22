//! InstantClone stream delay control: read its state and send delay commands through the HTTP
//! API it serves on this machine. Requests only ever go to the loopback address, so a widget
//! can not be pointed at another host. Without a dashboard password InstantClone accepts them
//! as they are; with one, its dock token has to ride along as `?token=`.

use std::sync::OnceLock;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::action::ActionError;

/// InstantClone's default control port.
pub const DEFAULT_PORT: u16 = 7799;
/// Loopback answers in milliseconds; anything slower means InstantClone is not there.
const TIMEOUT: Duration = Duration::from_millis(1500);
/// InstantClone's own ceiling (one hour).
const MAX_DELAY_MS: u32 = 3_600_000;

/// InstantClone's `/state`, reduced to what the widget shows.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct InstantCloneState {
    /// `idle`, `preparing` (buffer filling), `ready` (armed, waiting) or `active`.
    pub phase: String,
    pub armed_delay_ms: u32,
    pub current_delay_ms: u32,
    pub buffer_fill_ms: u32,
    pub ingest_alive: bool,
    pub destinations_alive: u32,
    pub destinations_total: u32,
}

/// A delay command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum InstantCloneAction {
    /// Arm `delay_ms` and go live with it as soon as the buffer holds it.
    Delay,
    /// Cut back to live now.
    Cut,
    /// Cut back to live once the current moment has aired everywhere.
    CutAfter,
    /// Drop a delay that is still filling, without going live with it.
    Disarm,
}

impl InstantCloneAction {
    fn path(self) -> &'static str {
        match self {
            Self::Delay => "/delay",
            Self::Cut => "/stop",
            Self::CutAfter => "/cut-after",
            Self::Disarm => "/disarm",
        }
    }
}

/// Where InstantClone listens and the dock token it wants (empty when it has no password).
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct InstantCloneTarget {
    pub port: u16,
    pub token: String,
}

impl InstantCloneTarget {
    fn url(&self, path: &str) -> Result<String, ActionError> {
        let token = self.token.trim();
        if !token.chars().all(|char| char.is_ascii_alphanumeric()) {
            return Err(ActionError::could_not(
                "reach InstantClone",
                "the token looks wrong",
            ));
        }
        let query = if token.is_empty() {
            String::new()
        } else {
            format!("?token={token}")
        };
        Ok(format!("http://127.0.0.1:{}{path}{query}", self.port))
    }
}

/// InstantClone's state, or `None` when it is not running or refuses the token.
pub async fn read_state(target: &InstantCloneTarget) -> Option<InstantCloneState> {
    let response = client().get(target.url("/state").ok()?).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    response.json().await.ok()
}

/// Send `action`; `delay_ms` only matters for [`InstantCloneAction::Delay`].
pub async fn send(
    target: &InstantCloneTarget,
    action: InstantCloneAction,
    delay_ms: u32,
) -> Result<(), ActionError> {
    let what = "control the stream delay";
    let request = client()
        .post(target.url(action.path())?)
        .header("Content-Type", "application/x-www-form-urlencoded");
    let request = match action {
        InstantCloneAction::Delay => request.body(format!("ms={}", delay_ms.min(MAX_DELAY_MS))),
        _ => request,
    };
    let response = request
        .send()
        .await
        .map_err(|_| ActionError::could_not(what, "InstantClone is not running"))?;
    match response.status().as_u16() {
        200..=299 => Ok(()),
        401 => Err(ActionError::could_not(
            what,
            "InstantClone wants its dock token",
        )),
        409 => Err(ActionError::could_not(
            what,
            "OBS is not streaming through InstantClone",
        )),
        status => Err(ActionError::could_not(
            what,
            format!("InstantClone answered {status}"),
        )),
    }
}

fn client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(TIMEOUT)
            .no_proxy()
            .build()
            .unwrap_or_default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(token: &str) -> InstantCloneTarget {
        InstantCloneTarget {
            port: DEFAULT_PORT,
            token: token.to_owned(),
        }
    }

    #[test]
    fn url_stays_on_loopback_and_carries_the_token() {
        assert_eq!(
            target("").url("/state").unwrap(),
            "http://127.0.0.1:7799/state"
        );
        assert_eq!(
            target(" abc123 ").url("/stop").unwrap(),
            "http://127.0.0.1:7799/stop?token=abc123"
        );
    }

    #[test]
    fn token_with_url_characters_is_refused() {
        assert!(target("a&b=c").url("/state").is_err());
        assert!(target("../x").url("/state").is_err());
    }

    #[test]
    fn state_decodes_the_documented_sample() {
        let body = r#"{"phase":"active","armed_delay_ms":15000,"current_delay_ms":15040,
            "buffer_fill_ms":15000,"ingest_alive":true,"egress_alive":true,
            "destinations_alive":2,"destinations_total":3,"stats":{"cuts":1}}"#;
        let state: InstantCloneState = serde_json::from_str(body).unwrap();
        assert_eq!(state.phase, "active");
        assert_eq!(state.current_delay_ms, 15040);
        assert_eq!((state.destinations_alive, state.destinations_total), (2, 3));
    }
}
