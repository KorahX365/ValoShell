//! Career extras: per-map region index and post-match gold stars.

use std::collections::HashMap;

use domain::ids::Puuid;
use serde::Deserialize;
use serde_json::Value;

use crate::http::{Host, Request};

/// `GET /region-index/v1/players/{puuid}?seasonID=`: per-map rating; the season is required.
pub fn get_region_index(puuid: &Puuid, season_id: &str) -> Request {
    Request::get(
        Host::Pd,
        format!("/region-index/v1/players/{puuid}?seasonID={season_id}"),
    )
}

/// `GET /goldstars/v1/goldstars`: medal definitions. Keys are obfuscated, so the body stays raw.
pub fn get_goldstars() -> Request {
    Request::get(Host::Pd, "/goldstars/v1/goldstars")
}

/// `GET /goldstars/v1/players/{puuid}`: medals earned by the player (raw, obfuscated keys).
pub fn get_player_goldstars(puuid: &Puuid) -> Request {
    Request::get(Host::Pd, format!("/goldstars/v1/players/{puuid}"))
}

/// Gold-star bodies are decoded as raw JSON.
pub type GoldStars = Value;

/// Region index of one player (shape from a single source).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RegionIndex {
    pub subject: Option<Puuid>,
    pub platform_group: String,
    pub season_id: String,
    /// Keyed by map uuid (`maps[].uuid` in valorant-api.com).
    #[serde(deserialize_with = "super::null_as_default")]
    pub region_data: HashMap<String, MapRegionData>,
}

/// Rating of the player on one map.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MapRegionData {
    pub region_id: String,
    pub values: RegionValues,
    #[serde(deserialize_with = "super::null_as_default")]
    pub top_agents: Vec<TopAgent>,
}

/// Win rates and the 0..500 region index.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RegionValues {
    pub win_rate: f64,
    pub attack_win_rate: f64,
    pub defense_win_rate: f64,
    pub region_index: f64,
    pub total_games: u32,
}

/// One of the most-played agents on a map.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TopAgent {
    pub agent_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_index_request_carries_the_season() {
        let request = get_region_index(&Puuid::new("abc"), "season-1");

        assert_eq!(
            request.path,
            "/region-index/v1/players/abc?seasonID=season-1"
        );
    }

    #[test]
    fn goldstar_requests_target_definitions_and_player() {
        assert_eq!(get_goldstars().path, "/goldstars/v1/goldstars");
        assert_eq!(
            get_player_goldstars(&Puuid::new("abc")).path,
            "/goldstars/v1/players/abc"
        );
    }

    #[test]
    fn region_index_decodes_per_map_values() {
        let json = r#"{"subject":"abc","platformGroup":"PC","seasonId":"s","regionData":{
            "map-1":{"regionId":"eu","values":{"winRate":0.55,"attackWinRate":0.5,
            "defenseWinRate":0.6,"regionIndex":312.5,"totalGames":20},
            "topAgents":[{"agentId":"agent-1"}]}}}"#;

        let index: RegionIndex = serde_json::from_str(json).expect("decodes");

        let map = &index.region_data["map-1"];
        assert_eq!(map.values.total_games, 20);
        assert!((map.values.win_rate - 0.55).abs() < f64::EPSILON);
        assert_eq!(map.top_agents[0].agent_id, "agent-1");
    }
}
