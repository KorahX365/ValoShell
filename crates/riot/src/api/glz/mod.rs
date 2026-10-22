//! Endpoints on the glz host: party, custom games, pregame (agent select) and coregame.
//! Request builders return `crate::http::Request`; DTOs mirror Riot's PascalCase bodies.

pub mod common;
pub mod coregame;
pub mod custom_game;
pub mod custom_game_configs;
pub mod loadouts;
pub mod matchmaking;
pub mod party;
pub mod party_dto;
pub mod party_member;
pub mod pregame;
pub mod pregame_match;
