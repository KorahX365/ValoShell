//! `customgameconfigs`: queue rules, custom-game maps/modes and pod ids.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

use super::common::null_as_default;
use crate::http::{Host, Request};

/// `GET /parties/v1/parties/customgameconfigs`: fetch once per session, not every poll.
pub fn get_custom_game_configs() -> Request {
    Request::get(Host::Glz, "/parties/v1/parties/customgameconfigs")
}

/// Response of [`get_custom_game_configs`].
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct CustomGameConfigs {
    pub enabled: bool,
    #[serde(deserialize_with = "null_as_default")]
    pub enabled_maps: Vec<String>,
    #[serde(deserialize_with = "null_as_default")]
    pub enabled_modes: Vec<String>,
    #[serde(deserialize_with = "null_as_default")]
    pub queues: Vec<QueueConfig>,
    /// Pod id -> ping proxy info; the keys are the live pod list.
    #[serde(deserialize_with = "null_as_default")]
    pub game_pod_ping_service_info: BTreeMap<String, Value>,
}

/// One queue's rules; `Priority` orders the mode picker.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct QueueConfig {
    #[serde(rename = "QueueID")]
    pub queue_id: String,
    pub enabled: bool,
    pub team_size: u32,
    pub num_teams: u32,
    pub max_party_size: u32,
    pub min_party_size: u32,
    #[serde(deserialize_with = "null_as_default")]
    pub invalid_party_sizes: Vec<u32>,
    pub max_party_size_high_skill: u32,
    pub high_skill_tier: u32,
    pub max_skill_tier: u32,
    pub mode: String,
    pub is_ranked: bool,
    pub is_tournament: bool,
    pub require_roster: bool,
    pub priority: i32,
    pub party_max_competitive_tier_range: u32,
    pub full_party_max_competitive_tier_range: u32,
    pub use_account_level_requirement: bool,
    pub minimum_account_level_required: u32,
    /// Rule values are the strings `"true"` / `"false"`.
    #[serde(deserialize_with = "null_as_default")]
    pub game_rules: BTreeMap<String, String>,
    #[serde(deserialize_with = "null_as_default")]
    pub supported_platform_types: Vec<String>,
    #[serde(deserialize_with = "null_as_default")]
    pub disabled_content: Vec<Value>,
    /// `"map:weight"` strings, e.g. `"Ascent:1"`.
    #[serde(deserialize_with = "null_as_default")]
    pub map_weights: Vec<String>,
    pub next_schedule_change_seconds: i64,
    pub time_until_next_schedule_change_seconds: i64,
}

#[cfg(test)]
mod tests {
    use reqwest::Method;

    use super::*;

    const FIXTURE: &str = include_str!("fixtures/custom_game_configs.json");

    #[test]
    fn builder_targets_configs_path() {
        let request = get_custom_game_configs();
        assert_eq!(request.host, Host::Glz);
        assert_eq!(request.method, Method::GET);
        assert_eq!(request.path, "/parties/v1/parties/customgameconfigs");
        assert!(request.body.is_none());
    }

    #[test]
    fn fixture_decodes_maps_modes_and_pods() {
        let configs: CustomGameConfigs = serde_json::from_str(FIXTURE).unwrap();
        assert!(configs.enabled);
        assert_eq!(
            configs.enabled_maps,
            ["/Game/Maps/Ascent/Ascent", "/Game/Maps/Triad/Triad"]
        );
        assert_eq!(configs.enabled_modes.len(), 1);
        assert!(
            configs
                .game_pod_ping_service_info
                .contains_key("aresriot.aws-eus2-prod.eu-gp-madrid-1")
        );
    }

    #[test]
    fn fixture_decodes_queue_rules() {
        let configs: CustomGameConfigs = serde_json::from_str(FIXTURE).unwrap();
        let queue = &configs.queues[0];
        assert_eq!(queue.queue_id, "competitive");
        assert_eq!(queue.invalid_party_sizes, [4]);
        assert_eq!(queue.max_skill_tier, 27);
        assert!(queue.is_ranked);
        assert_eq!(queue.game_rules["IsOvertimeWinByTwo"], "true");
        assert_eq!(queue.map_weights, ["Ascent:1", "Bind:1"]);
        assert_eq!(queue.priority, 3);
    }

    #[test]
    fn empty_object_decodes_to_defaults() {
        let configs: CustomGameConfigs = serde_json::from_str("{}").unwrap();
        assert!(!configs.enabled && configs.queues.is_empty());
    }
}
