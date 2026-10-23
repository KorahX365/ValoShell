//! Custom-game lobby requests.
//!
//! UNVERIFIED on 13.08: none of these appear in the 13.06-13.08 client logs; bodies come from
//! 2025 working code (VCL, vti). `customgamesettings` answers 404 unless the party is already
//! in `CUSTOM_GAME_SETUP`, and `/queue` must not be called in that state.

use std::collections::BTreeMap;

use domain::ids::{PartyId, Puuid};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::common::{encode_path_segment, null_as_default};
use crate::http::{Host, Request};

/// Map, mode, server and rules of a custom game; also the shape inside `CustomGameData`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct CustomGameSettings {
    /// Map asset path, e.g. `/Game/Maps/Ascent/Ascent` (from `customgameconfigs.EnabledMaps`).
    pub map: String,
    /// Mode asset path (from `customgameconfigs.EnabledModes`).
    pub mode: String,
    /// Ignored by the server today but still sent.
    pub use_bots: bool,
    /// Game pod id; read it from the live pod list, never hard-code one.
    pub game_pod: String,
    /// Rule values are the strings `"true"` / `"false"`; any rule may be omitted.
    #[serde(deserialize_with = "null_as_default")]
    pub game_rules: BTreeMap<String, String>,
}

/// A team slot in the custom lobby (`TeamOne` is the first team).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomTeam {
    TeamOne,
    TeamTwo,
    TeamSpectate,
    TeamOneCoaches,
    TeamTwoCoaches,
}

impl CustomTeam {
    /// The path segment Riot expects.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TeamOne => "TeamOne",
            Self::TeamTwo => "TeamTwo",
            Self::TeamSpectate => "TeamSpectate",
            Self::TeamOneCoaches => "TeamOneCoaches",
            Self::TeamTwoCoaches => "TeamTwoCoaches",
        }
    }
}

/// `POST /parties/v1/parties/{party_id}/makecustomgame`: switch the party to custom setup.
pub fn make_custom_game(party_id: &PartyId) -> Request {
    Request::post(
        Host::Glz,
        format!("/parties/v1/parties/{party_id}/makecustomgame"),
    )
    .json(json!({}))
}

/// `POST /parties/v1/parties/{party_id}/customgamesettings`: set map, mode, pod and rules.
pub fn set_custom_game_settings(party_id: &PartyId, settings: &CustomGameSettings) -> Request {
    // Serializing plain strings, bools and a string map cannot fail; `Null` is unreachable.
    let body = serde_json::to_value(settings).unwrap_or_default();
    Request::post(
        Host::Glz,
        format!("/parties/v1/parties/{party_id}/customgamesettings"),
    )
    .json(body)
}

/// `POST /parties/v1/parties/{party_id}/customgamemembership/{team}`: move a player to a slot.
pub fn change_team(party_id: &PartyId, team: CustomTeam, player: &Puuid) -> Request {
    Request::post(
        Host::Glz,
        format!(
            "/parties/v1/parties/{party_id}/customgamemembership/{}",
            team.as_str()
        ),
    )
    .json(json!({ "playerToPutOnTeam": player.as_str() }))
}

/// `POST /parties/v1/parties/{party_id}/startcustomgame`: provision the match.
pub fn start_custom_game(party_id: &PartyId) -> Request {
    Request::post(
        Host::Glz,
        format!("/parties/v1/parties/{party_id}/startcustomgame"),
    )
    .json(json!({}))
}

/// `POST /parties/v1/parties/{party_id}/converttodefault?queueID={queue_id}`: leave custom
/// mode for a matchmaking queue.
///
/// The client logs this as `Party_ConvertToDefault`; the older `makedefault` name is gone and
/// left the party stuck in custom setup.
pub fn convert_to_default(party_id: &PartyId, queue_id: &str) -> Request {
    let queue_id = encode_path_segment(queue_id);
    Request::post(
        Host::Glz,
        format!("/parties/v1/parties/{party_id}/converttodefault?queueID={queue_id}"),
    )
    .json(json!({}))
}

#[cfg(test)]
mod tests {
    use reqwest::Method;

    use super::*;

    const ME: &str = "d7ff5397-caf5-4451-9057-0db61e73ab33";

    fn party() -> PartyId {
        PartyId::new("fe5ea6f4-a974-412a-8422-e0631171756b")
    }

    fn ascent_settings() -> CustomGameSettings {
        CustomGameSettings {
            map: "/Game/Maps/Ascent/Ascent".into(),
            mode: "/Game/GameModes/Bomb/BombGameMode.BombGameMode_C".into(),
            use_bots: false,
            game_pod: "aresriot.aws-eus2-prod.eu-gp-madrid-1".into(),
            game_rules: BTreeMap::from([("IsOvertimeWinByTwo".into(), "true".into())]),
        }
    }

    #[test]
    fn make_custom_game_and_start_send_empty_object() {
        for request in [make_custom_game(&party()), start_custom_game(&party())] {
            assert_eq!(request.host, Host::Glz);
            assert_eq!(request.method, Method::POST);
            assert_eq!(request.body, Some(json!({})));
        }
        assert!(make_custom_game(&party()).path.ends_with("/makecustomgame"));
        assert!(
            start_custom_game(&party())
                .path
                .ends_with("/startcustomgame")
        );
    }

    #[test]
    fn settings_body_uses_pascal_case_keys_and_string_rules() {
        let request = set_custom_game_settings(&party(), &ascent_settings());
        assert_eq!(request.method, Method::POST);
        assert_eq!(
            request.path,
            format!("/parties/v1/parties/{}/customgamesettings", party())
        );
        let expected = json!({
            "Map": "/Game/Maps/Ascent/Ascent",
            "Mode": "/Game/GameModes/Bomb/BombGameMode.BombGameMode_C",
            "UseBots": false,
            "GamePod": "aresriot.aws-eus2-prod.eu-gp-madrid-1",
            "GameRules": { "IsOvertimeWinByTwo": "true" }
        });
        assert_eq!(request.body, Some(expected));
    }

    #[test]
    fn settings_decode_null_rules_as_empty() {
        let body = r#"{"Map":"","Mode":"","UseBots":false,"GamePod":"","GameRules":null}"#;
        let settings: CustomGameSettings = serde_json::from_str(body).unwrap();
        assert!(settings.game_rules.is_empty());
    }

    #[test]
    fn change_team_names_slot_in_path_and_player_in_body() {
        let request = change_team(&party(), CustomTeam::TeamTwoCoaches, &Puuid::new(ME));
        let path = format!(
            "/parties/v1/parties/{}/customgamemembership/TeamTwoCoaches",
            party()
        );
        assert_eq!(request.path, path);
        assert_eq!(request.body, Some(json!({ "playerToPutOnTeam": ME })));
        assert_eq!(CustomTeam::TeamSpectate.as_str(), "TeamSpectate");
    }

    #[test]
    fn convert_to_default_passes_queue_as_query() {
        let request = convert_to_default(&party(), "competitive");
        assert_eq!(request.method, Method::POST);
        assert_eq!(
            request.path,
            format!(
                "/parties/v1/parties/{}/converttodefault?queueID=competitive",
                party()
            )
        );
        assert_eq!(request.body, Some(json!({})));
    }
}
