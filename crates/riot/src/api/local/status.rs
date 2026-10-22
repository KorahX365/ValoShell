//! Riot service status and the player behaviour token.

use serde::{Deserialize, Deserializer};
use serde_json::Value;

use crate::error::{RiotError, RiotResult};
use crate::http::{Host, Request};

const STATUS_ENDPOINT: &str = "riot-status";
const PREFERRED_LOCALE: &str = "en_US";
/// Incident severities worth showing; `info` incidents can stay open for months.
const SHOWN_SEVERITIES: [&str; 2] = ["critical", "warning"];
/// Maintenance statuses that mean "affects players now or soon".
const ACTIVE_MAINTENANCE_STATUSES: [&str; 2] = ["scheduled", "in_progress"];

/// `GET /riot-status/v1/products/valorant/patchlines/live/deployments/{region}`.
/// `region` is the short region (`eu`, `na`...). The body may be a JSON string holding JSON:
/// decode it with `parse_status_body`.
pub fn get_service_status(region: &str) -> Request {
    Request::get(
        Host::Local,
        format!("/riot-status/v1/products/valorant/patchlines/live/deployments/{region}"),
    )
}

/// `GET /player-behavior-token/v1/jwt/payload`: the claims of the behaviour token.
pub fn get_behavior_token_payload() -> Request {
    Request::get(Host::Local, "/player-behavior-token/v1/jwt/payload")
}

/// A text in one locale.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Translation {
    pub locale: String,
    pub content: String,
}

/// One progress update of an incident or maintenance.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct StatusUpdate {
    pub translations: Vec<Translation>,
}

/// A planned or running maintenance.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Maintenance {
    #[serde(deserialize_with = "id_as_text")]
    pub id: String,
    pub maintenance_status: String,
    pub titles: Vec<Translation>,
    /// Oldest first.
    pub updates: Vec<StatusUpdate>,
}

impl Maintenance {
    /// Scheduled or in progress (finished ones stay in the list for a while).
    pub fn is_active(&self) -> bool {
        ACTIVE_MAINTENANCE_STATUSES.contains(&self.maintenance_status.as_str())
    }
}

/// A service incident.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Incident {
    #[serde(deserialize_with = "id_as_text")]
    pub id: String,
    /// `info|warning|critical`.
    pub incident_severity: String,
    pub titles: Vec<Translation>,
    /// Oldest first.
    pub updates: Vec<StatusUpdate>,
}

impl Incident {
    /// Critical or warning; `info` incidents are noise.
    pub fn is_shown(&self) -> bool {
        SHOWN_SEVERITIES.contains(&self.incident_severity.as_str())
    }
}

/// Maintenances and incidents of one deployment.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ServiceStatus {
    pub maintenances: Vec<Maintenance>,
    pub incidents: Vec<Incident>,
}

impl ServiceStatus {
    /// Maintenances that are scheduled or running.
    pub fn active_maintenances(&self) -> impl Iterator<Item = &Maintenance> {
        self.maintenances
            .iter()
            .filter(|maintenance| maintenance.is_active())
    }

    /// Incidents severe enough to show.
    pub fn shown_incidents(&self) -> impl Iterator<Item = &Incident> {
        self.incidents.iter().filter(|incident| incident.is_shown())
    }
}

/// Decode the status body, unwrapping the JSON string that Riot sometimes wraps around it.
pub fn parse_status_body(body: &str) -> RiotResult<ServiceStatus> {
    let decode_error = |reason: String| RiotError::Decode {
        endpoint: STATUS_ENDPOINT.to_owned(),
        reason,
    };
    let outer: Value = serde_json::from_str(body).map_err(|e| decode_error(e.to_string()))?;
    let inner = match outer {
        Value::String(text) => {
            serde_json::from_str(&text).map_err(|e| decode_error(e.to_string()))?
        }
        other => other,
    };
    serde_json::from_value(inner).map_err(|e| decode_error(e.to_string()))
}

/// The `en_US` text, else the first one; `None` for an empty list.
pub fn localized(translations: &[Translation]) -> Option<&str> {
    translations
        .iter()
        .find(|translation| translation.locale == PREFERRED_LOCALE)
        .or_else(|| translations.first())
        .map(|translation| translation.content.as_str())
}

/// The newest update's text (updates are oldest first), if any update has a translation.
pub fn latest_update_text(updates: &[StatusUpdate]) -> Option<&str> {
    updates
        .last()
        .and_then(|update| localized(&update.translations))
}

/// Riot ids here are numbers in some deployments and strings in others.
fn id_as_text<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(text) => text,
        Value::Number(number) => number.to_string(),
        _ => String::new(),
    })
}

/// Claims of the player behaviour token.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct BehaviorTokenPayload {
    pub restrictions_claim: RestrictionsClaim,
}

/// The restrictions part of the behaviour token.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RestrictionsClaim {
    /// Entry fields are not interpreted: only their count matters.
    pub active_restrictions: Vec<Value>,
}

impl BehaviorTokenPayload {
    /// How many restrictions Riot lists on the account.
    pub fn restriction_count(&self) -> usize {
        self.restrictions_claim.active_restrictions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATUS: &str = r#"{
        "maintenances":[
          {"id":1,"maintenance_status":"in_progress",
           "titles":[{"locale":"es_ES","content":"Mantenimiento"},{"locale":"en_US","content":"Maintenance"}],
           "updates":[]},
          {"id":"2","maintenance_status":"archived","titles":[],"updates":[]}],
        "incidents":[
          {"id":10,"incident_severity":"critical",
           "titles":[{"locale":"fr_FR","content":"Panne"}],
           "updates":[{"translations":[{"locale":"en_US","content":"old"}]},
                      {"translations":[{"locale":"en_US","content":"new"}]}]},
          {"id":11,"incident_severity":"info","titles":[],"updates":[]}]}"#;

    #[test]
    fn builders_target_local_host() {
        let status = get_service_status("eu");
        let token = get_behavior_token_payload();

        assert_eq!(status.host, Host::Local);
        assert_eq!(
            status.path,
            "/riot-status/v1/products/valorant/patchlines/live/deployments/eu"
        );
        assert_eq!(token.path, "/player-behavior-token/v1/jwt/payload");
        assert_eq!(token.method, reqwest::Method::GET);
    }

    #[test]
    fn plain_json_body_is_decoded() {
        let status = parse_status_body(STATUS).expect("decodes");

        assert_eq!(status.maintenances.len(), 2);
        assert_eq!(status.incidents[0].id, "10");
    }

    #[test]
    fn json_string_body_is_decoded_twice() {
        let wrapped = serde_json::to_string(STATUS).expect("encodes");

        let status = parse_status_body(&wrapped).expect("decodes");

        assert_eq!(status.maintenances.len(), 2);
    }

    #[test]
    fn invalid_body_is_a_decode_error() {
        let error = parse_status_body("<html>").expect_err("fails");

        assert!(matches!(error, RiotError::Decode { .. }));
        assert!(parse_status_body("\"not json\"").is_err());
    }

    #[test]
    fn only_active_maintenances_and_severe_incidents_are_kept() {
        let status = parse_status_body(STATUS).expect("decodes");

        let maintenance_ids: Vec<_> = status
            .active_maintenances()
            .map(|m| m.id.as_str())
            .collect();
        let incident_ids: Vec<_> = status.shown_incidents().map(|i| i.id.as_str()).collect();

        assert_eq!(maintenance_ids, ["1"]);
        assert_eq!(incident_ids, ["10"]);
    }

    #[test]
    fn warning_is_shown_and_info_is_not() {
        let warning = Incident {
            incident_severity: "warning".to_owned(),
            ..Incident::default()
        };

        assert!(warning.is_shown());
        assert!(!Incident::default().is_shown());
    }

    #[test]
    fn localized_prefers_english_then_first() {
        let status = parse_status_body(STATUS).expect("decodes");

        assert_eq!(
            localized(&status.maintenances[0].titles),
            Some("Maintenance")
        );
        assert_eq!(localized(&status.incidents[0].titles), Some("Panne"));
        assert_eq!(localized(&[]), None);
    }

    #[test]
    fn latest_update_is_the_last_entry() {
        let status = parse_status_body(STATUS).expect("decodes");

        assert_eq!(
            latest_update_text(&status.incidents[0].updates),
            Some("new")
        );
        assert_eq!(latest_update_text(&[]), None);
    }

    #[test]
    fn behavior_token_counts_restrictions() {
        let json = r#"{"restrictionsClaim":{"activeRestrictions":[{"type":"X"},{"type":"Y"}]},
            "other":1}"#;

        let payload: BehaviorTokenPayload = serde_json::from_str(json).expect("decodes");

        assert_eq!(payload.restriction_count(), 2);
    }

    #[test]
    fn behavior_token_without_claim_has_no_restrictions() {
        let payload: BehaviorTokenPayload = serde_json::from_str("{}").expect("decodes");

        assert_eq!(payload.restriction_count(), 0);
    }
}
