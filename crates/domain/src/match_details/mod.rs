//! Tolerant parsing of Riot match-details JSON into typed rows. Input is `serde_json::Value`;
//! both camelCase and PascalCase keys are accepted, and malformed input costs one field, never
//! a panic.

mod context;
pub(crate) mod json;
mod own_match;
pub mod player_row;
mod rounds;
mod scoreboard;
mod sides;

pub use own_match::{ExtractedOwnMatch, extract_own_match};
pub use scoreboard::{ScoreboardRow, extract_scoreboard};
pub use sides::{ATTACKING_TEAM_FIRST_HALF, StartingSide};
