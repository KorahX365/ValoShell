//! Errors of the Riot connection layer, worded so the UI can show them as-is.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RiotError {
    #[error("The Riot Client is not running (no lockfile at {path}).")]
    LockfileMissing { path: String },

    #[error("The Riot Client lockfile could not be read: {reason}.")]
    LockfileMalformed { reason: String },

    #[error(
        "VALORANT is still starting or not signed in (HTTP {status} from the Riot Client). Retrying."
    )]
    Handshake { status: u16 },

    #[error("Region not detected yet. Start VALORANT once so it can be read from the game log.")]
    RegionUnknown,

    #[error("Could not reach {endpoint}: {reason}.")]
    Transport { endpoint: String, reason: String },

    #[error("Riot answered HTTP {status} for {endpoint}.")]
    Status { status: u16, endpoint: String },

    #[error("Unexpected answer from {endpoint}: {reason}.")]
    Decode { endpoint: String, reason: String },
}

pub type RiotResult<T> = Result<T, RiotError>;
