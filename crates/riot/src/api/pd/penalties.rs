//! Restrictions: penalties, future interventions and the avoid list.

use domain::ids::Puuid;
use serde::Deserialize;
use serde_json::Value;

use crate::http::{Host, Request};

/// `GET /restrictions/v3/penalties`: active penalties (queue bans, AFK warnings).
pub fn get_penalties() -> Request {
    Request::get(Host::Pd, "/restrictions/v3/penalties")
}

/// `GET /restrictions/v1/activeFutureInterventions`: behaviour ratings and coming interventions.
pub fn get_active_future_interventions() -> Request {
    Request::get(Host::Pd, "/restrictions/v1/activeFutureInterventions")
}

/// `GET /restrictions/v1/avoidList`: players the account avoids.
pub fn get_avoid_list() -> Request {
    Request::get(Host::Pd, "/restrictions/v1/avoidList")
}

/// Penalty list; empty `penalties` for clean accounts.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Penalties {
    pub subject: Option<Puuid>,
    pub version: i64,
    /// Effect fields vary per penalty type, so each entry stays a raw tree.
    #[serde(deserialize_with = "super::null_as_default")]
    pub penalties: Vec<Value>,
    #[serde(deserialize_with = "super::null_as_default")]
    pub infractions: Vec<Infraction>,
}

impl Penalties {
    /// `true` when the account has at least one active penalty.
    pub fn has_active_penalty(&self) -> bool {
        !self.penalties.is_empty()
    }
}

/// A named behaviour infraction.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Infraction {
    #[serde(rename = "ID")]
    pub id: String,
    pub name: String,
    pub rating_name: String,
}

/// Future-interventions response.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct FutureInterventions {
    pub subject: Option<Puuid>,
    #[serde(deserialize_with = "super::null_as_default")]
    pub interventions_by_category: Vec<InterventionCategory>,
}

/// Interventions of one behaviour category (shape from a single source).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct InterventionCategory {
    pub behavior_category: String,
    pub behavior_rating_name: String,
    pub last_rating_reduction: Value,
    #[serde(deserialize_with = "super::null_as_default")]
    pub active_interventions: Vec<Value>,
    #[serde(deserialize_with = "super::null_as_default")]
    pub next_intervention_names: Vec<String>,
    pub applied_infractions: Value,
}

/// Avoid-list response.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct AvoidList {
    pub subject: Option<Puuid>,
    #[serde(deserialize_with = "super::null_as_default")]
    pub avoid_list: Vec<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restriction_requests_use_the_documented_paths() {
        assert_eq!(get_penalties().path, "/restrictions/v3/penalties");
        assert_eq!(
            get_active_future_interventions().path,
            "/restrictions/v1/activeFutureInterventions"
        );
        assert_eq!(get_avoid_list().path, "/restrictions/v1/avoidList");
        assert_eq!(get_penalties().host, Host::Pd);
    }

    #[test]
    fn clean_account_has_no_active_penalty() {
        let json = r#"{"Subject":"abc","Penalties":[],"Version":3}"#;

        let penalties: Penalties = serde_json::from_str(json).expect("decodes");

        assert!(!penalties.has_active_penalty());
        assert_eq!(penalties.version, 3);
    }

    #[test]
    fn penalty_entries_stay_raw_trees() {
        let json = r#"{"Penalties":[{"ID":"p","IssuingMatchID":"m","GamesRemaining":2}],
            "Infractions":[{"ID":"i","Name":"AFK","RatingName":"afk"}]}"#;

        let penalties: Penalties = serde_json::from_str(json).expect("decodes");

        assert!(penalties.has_active_penalty());
        assert_eq!(penalties.penalties[0]["GamesRemaining"], 2);
        assert_eq!(penalties.infractions[0].rating_name, "afk");
    }

    #[test]
    fn interventions_decode_categories() {
        let json = r#"{"Subject":"abc","InterventionsByCategory":[{"BehaviorCategory":"PARTICIPATION",
            "BehaviorRatingName":"afk","LastRatingReduction":null,"ActiveInterventions":null,
            "NextInterventionNames":["warning"],"AppliedInfractions":{}}]}"#;

        let interventions: FutureInterventions = serde_json::from_str(json).expect("decodes");

        let category = &interventions.interventions_by_category[0];
        assert_eq!(category.behavior_rating_name, "afk");
        assert_eq!(category.next_intervention_names, ["warning"]);
        assert!(category.active_interventions.is_empty());
    }

    #[test]
    fn avoid_list_decodes_empty() {
        let list: AvoidList =
            serde_json::from_str(r#"{"Subject":"abc","AvoidList":[]}"#).expect("decodes");

        assert!(list.avoid_list.is_empty());
    }
}
