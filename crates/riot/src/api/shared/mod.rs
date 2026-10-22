//! Endpoints on the shared host: content service and client config.

use std::collections::HashMap;

use domain::region::Region;
use serde::Deserialize;

use crate::http::{Host, Request};

/// `GET /content-service/v3/content`: seasons (episodes and acts) and events.
pub fn get_content() -> Request {
    Request::get(Host::Shared, "/content-service/v3/content")
}

/// `GET /v1/config/{region}`: feature flags and service URLs as a flat string map.
pub fn get_config(region: &Region) -> Request {
    Request::get(Host::Shared, format!("/v1/config/{}", region.region))
}

/// The content answer.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Content {
    #[serde(
        rename = "DisabledIDs",
        deserialize_with = "crate::api::pd::null_as_default"
    )]
    pub disabled_ids: Vec<String>,
    #[serde(deserialize_with = "crate::api::pd::null_as_default")]
    pub seasons: Vec<Season>,
    #[serde(deserialize_with = "crate::api::pd::null_as_default")]
    pub events: Vec<ContentEvent>,
}

impl Content {
    /// The id that keys MMR blocks: the active season of type `act`, else the last active one.
    ///
    /// The active *episode* is listed first and must not be used (it blanks every win rate).
    pub fn active_act_id(&self) -> Option<&str> {
        let mut active = self.seasons.iter().filter(|season| season.is_active);
        let fallback = self.seasons.iter().rev().find(|season| season.is_active);
        active
            .find(|season| season.kind.eq_ignore_ascii_case("act"))
            .or(fallback)
            .map(|season| season.id.as_str())
    }
}

/// An episode or act.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Season {
    #[serde(rename = "ID")]
    pub id: String,
    pub name: String,
    /// `episode` or `act`.
    #[serde(rename = "Type")]
    pub kind: String,
    pub start_time: String,
    pub end_time: String,
    pub is_active: bool,
}

/// A timed in-game event.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct ContentEvent {
    #[serde(rename = "ID")]
    pub id: String,
    pub name: String,
    pub start_time: String,
    pub end_time: String,
    pub is_active: bool,
}

/// The client config. Values are strings (`"true"`, numbers, URLs).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct ClientConfig {
    pub last_application: String,
    #[serde(deserialize_with = "crate::api::pd::null_as_default")]
    pub collapsed: HashMap<String, String>,
}

impl ClientConfig {
    /// A boolean flag such as `personalization.equipAnyLevel.enabled`; `None` when absent/unparsable.
    pub fn flag(&self, key: &str) -> Option<bool> {
        match self
            .collapsed
            .get(key)?
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTENT: &str = r#"{"DisabledIDs":["x"],"Seasons":[
        {"ID":"ep","Name":"EPISODE 9","Type":"episode","StartTime":"a","EndTime":"b","IsActive":true},
        {"ID":"act-old","Name":"ACT V","Type":"act","IsActive":false},
        {"ID":"act-now","Name":"ACT VI","Type":"act","IsActive":true}],
        "Events":[{"ID":"ev","Name":"Event","IsActive":false}]}"#;

    #[test]
    fn content_request_targets_the_shared_host() {
        let request = get_content();

        assert_eq!(request.host, Host::Shared);
        assert_eq!(request.path, "/content-service/v3/content");
    }

    #[test]
    fn config_request_uses_the_region_name() {
        let region = Region::resolve("eu").expect("eu");

        let request = get_config(&region);

        assert_eq!(request.host, Host::Shared);
        assert_eq!(request.path, "/v1/config/eu");
    }

    #[test]
    fn active_act_skips_the_active_episode() {
        let content: Content = serde_json::from_str(CONTENT).expect("decodes");

        assert_eq!(content.active_act_id(), Some("act-now"));
        assert_eq!(content.seasons[0].kind, "episode");
        assert_eq!(content.disabled_ids, ["x"]);
        assert_eq!(content.events[0].id, "ev");
    }

    #[test]
    fn active_act_falls_back_to_the_last_active_season() {
        let json = r#"{"Seasons":[
            {"ID":"ep1","Type":"episode","IsActive":true},
            {"ID":"ep2","Type":"episode","IsActive":true}]}"#;
        let content: Content = serde_json::from_str(json).expect("decodes");

        assert_eq!(content.active_act_id(), Some("ep2"));
    }

    #[test]
    fn active_act_is_none_without_active_seasons() {
        let content: Content = serde_json::from_str(r#"{"Seasons":null}"#).expect("decodes");

        assert_eq!(content.active_act_id(), None);
    }

    #[test]
    fn config_flags_parse_strings() {
        let json = r#"{"LastApplication":"t","Collapsed":{
            "personalization.equipAnyLevel.enabled":"true","friends.enabled":"false","n":"5"}}"#;
        let config: ClientConfig = serde_json::from_str(json).expect("decodes");

        assert_eq!(
            config.flag("personalization.equipAnyLevel.enabled"),
            Some(true)
        );
        assert_eq!(config.flag("friends.enabled"), Some(false));
        assert_eq!(config.flag("n"), None);
        assert_eq!(config.flag("missing"), None);
    }
}
