//! Party response DTOs.
//!
//! Riot omits zero values and sends `null` for empty lists, so every field is optional on the
//! wire; lists accept `null`.

use domain::ids::{PartyId, Puuid};
use serde::Deserialize;
use serde_json::Value;

use super::common::{PlayerIdentity, SeasonalBadgeInfo, null_as_default};
use super::custom_game::CustomGameSettings;

/// Smallest timestamp Riot sends for "never"; `QueueEntryTime` carries it while not searching.
const UNSET_TIME_PREFIX: &str = "0001-01-01";

/// `GET /parties/v1/players/{puuid}`: where the player's party id comes from.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct PartyPlayer {
    pub subject: Option<Puuid>,
    pub version: u64,
    #[serde(rename = "CurrentPartyID")]
    pub current_party_id: Option<PartyId>,
    #[serde(deserialize_with = "null_as_default")]
    pub requests: Vec<JoinRequest>,
    pub platform_info: Option<Value>,
}

/// A pending "request to join" on a closed party.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct JoinRequest {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "PartyID")]
    pub party_id: Option<PartyId>,
    pub requested_by_subject: Option<Puuid>,
    #[serde(deserialize_with = "null_as_default")]
    pub subjects: Vec<Puuid>,
    pub created_at: String,
    pub refreshed_at: String,
    /// Unit unknown (nanoseconds in the official fixtures); kept raw.
    pub expires_in: Value,
}

/// Latency to one game pod.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct PodPingSample {
    pub ping: u32,
    #[serde(rename = "GamePodID")]
    pub game_pod_id: String,
}

/// One person in the party lobby.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct PartyMember {
    pub subject: Option<Puuid>,
    pub competitive_tier: u32,
    #[serde(deserialize_with = "null_as_default")]
    pub player_identity: PlayerIdentity,
    pub seasonal_badge_info: Option<SeasonalBadgeInfo>,
    pub is_owner: bool,
    pub queue_eligible_remaining_account_levels: u32,
    #[serde(deserialize_with = "null_as_default")]
    pub pings: Vec<PodPingSample>,
    pub is_ready: bool,
    pub is_moderator: bool,
    #[serde(rename = "UseBroadcastHUD")]
    pub use_broadcast_hud: bool,
    pub platform_type: String,
}

/// Queue the party selected plus its server preferences.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct MatchmakingData {
    #[serde(rename = "QueueID")]
    pub queue_id: String,
    #[serde(deserialize_with = "null_as_default")]
    pub preferred_game_pods: Vec<String>,
    #[serde(rename = "SkillDisparityRRPenalty")]
    pub skill_disparity_rr_penalty: i32,
}

/// Custom lobby state; only meaningful while the party is `CUSTOM_GAME_SETUP`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct CustomGameData {
    #[serde(deserialize_with = "null_as_default")]
    pub settings: CustomGameSettings,
    /// Team slots (`teamOne`, `teamTwo`, ...); element shape is unverified so it stays raw.
    pub membership: Value,
    pub max_party_size: u32,
    pub autobalance_enabled: bool,
    pub autobalance_min_players: u32,
    pub has_recovery_data: bool,
}

/// Last error toast the server raised for the party (e.g. a member lacks the rank).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct PartyError {
    pub error_type: String,
    #[serde(deserialize_with = "null_as_default")]
    pub errored_players: Vec<Value>,
}

/// A party-wide XP bonus such as `SOCIAL_XP`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct XpBonus {
    #[serde(rename = "ID")]
    pub id: String,
    pub applied: bool,
}

/// The full party object returned by lookups and every party mutation.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Party {
    #[serde(rename = "ID")]
    pub id: Option<PartyId>,
    #[serde(rename = "MUCName")]
    pub muc_name: String,
    #[serde(rename = "VoiceRoomID")]
    pub voice_room_id: String,
    /// Epoch milliseconds; a monotonic change token.
    pub version: u64,
    pub client_version: String,
    #[serde(deserialize_with = "null_as_default")]
    pub members: Vec<PartyMember>,
    /// `DEFAULT`, `CUSTOM_GAME_SETUP`, `MATCHMAKING` (searching) or `MATCHMADE_GAME_STARTING`.
    pub state: String,
    pub previous_state: String,
    pub state_transition_reason: String,
    /// `OPEN` or `CLOSED`.
    pub accessibility: String,
    #[serde(deserialize_with = "null_as_default")]
    pub custom_game_data: CustomGameData,
    #[serde(deserialize_with = "null_as_default")]
    pub matchmaking_data: MatchmakingData,
    pub invites: Option<Value>,
    #[serde(deserialize_with = "null_as_default")]
    pub requests: Vec<JoinRequest>,
    /// ISO-8601; `0001-01-01T00:00:00Z` while not searching.
    pub queue_entry_time: String,
    pub estimated_queue_time_seconds: Option<f64>,
    #[serde(deserialize_with = "null_as_default")]
    pub error_notification: PartyError,
    pub restricted_seconds: u32,
    #[serde(deserialize_with = "null_as_default")]
    pub eligible_queues: Vec<String>,
    /// Reasons a queue is blocked; element shape is unverified so it stays raw.
    #[serde(deserialize_with = "null_as_default")]
    pub queue_ineligibilities: Vec<Value>,
    #[serde(rename = "XPBonuses", deserialize_with = "null_as_default")]
    pub xp_bonuses: Vec<XpBonus>,
    /// Empty when no code was generated.
    pub invite_code: String,
}

impl Party {
    /// True while the party is searching. `State` decides: Riot keeps the last
    /// `QueueEntryTime` after a match is found, through the match and back to the menus, so the
    /// timestamp alone reads as "still searching". It is only the fallback when no state is sent.
    pub fn is_queueing(&self) -> bool {
        match self.state.as_str() {
            "MATCHMAKING" => true,
            "" => {
                !self.queue_entry_time.is_empty()
                    && !self.queue_entry_time.starts_with(UNSET_TIME_PREFIX)
            }
            _ => false,
        }
    }

    /// True when the party is in custom-game setup (lobby of up to 12).
    pub fn is_custom_game_setup(&self) -> bool {
        self.state == "CUSTOM_GAME_SETUP"
    }

    /// The member with this puuid, if present.
    pub fn member(&self, puuid: &Puuid) -> Option<&PartyMember> {
        self.members
            .iter()
            .find(|member| member.subject.as_ref() == Some(puuid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PARTY: &str = include_str!("fixtures/party.json");
    const ME: &str = "d7ff5397-caf5-4451-9057-0db61e73ab33";

    fn party() -> Party {
        serde_json::from_str(PARTY).unwrap()
    }

    #[test]
    fn party_fixture_decodes_header_and_queue_fields() {
        let party = party();
        assert_eq!(
            party.id.unwrap().as_str(),
            "fe5ea6f4-a974-412a-8422-e0631171756b"
        );
        assert_eq!(party.version, 1_702_241_006_830);
        assert_eq!(party.state, "DEFAULT");
        assert_eq!(party.accessibility, "CLOSED");
        assert_eq!(party.matchmaking_data.queue_id, "unrated");
        assert_eq!(party.eligible_queues.len(), 7);
        assert_eq!(party.restricted_seconds, 0);
        assert_eq!(party.invite_code, "");
    }

    #[test]
    fn party_fixture_decodes_member_details() {
        let party = party();
        let member = party.member(&Puuid::new(ME)).unwrap();
        assert!(member.is_owner && member.is_ready);
        assert_eq!(member.competitive_tier, 14);
        assert_eq!(member.player_identity.account_level, 239);
        assert!(member.player_identity.hide_account_level);
        assert_eq!(member.pings[0].ping, 19);
        assert_eq!(
            member.pings[0].game_pod_id,
            "aresriot.aws-eus2-prod.eu-gp-madrid-1"
        );
        assert!(member.seasonal_badge_info.is_none());
    }

    #[test]
    fn party_fixture_decodes_custom_data_and_requests() {
        let party = party();
        assert_eq!(party.custom_game_data.max_party_size, 5);
        assert!(party.custom_game_data.settings.game_rules.is_empty());
        assert_eq!(party.requests[0].id, "req-1");
        assert_eq!(party.requests[0].subjects.len(), 1);
        assert!(!party.xp_bonuses[0].applied);
    }

    #[test]
    fn idle_party_is_not_queueing_and_searching_party_is() {
        let mut party = party();
        assert!(!party.is_queueing());
        party.state = "MATCHMAKING".into();
        party.queue_entry_time = "2026-10-19T10:00:00Z".into();
        assert!(party.is_queueing());
    }

    #[test]
    fn a_kept_queue_time_after_the_match_is_not_queueing() {
        let mut party = party();
        party.queue_entry_time = "2026-10-19T10:00:00Z".into();
        for state in ["DEFAULT", "MATCHMADE_GAME_STARTING"] {
            party.state = state.into();
            assert!(!party.is_queueing(), "{state}");
        }
        party.state = String::new();
        assert!(party.is_queueing(), "no state: the timestamp decides");
    }

    #[test]
    fn null_lists_and_missing_fields_decode_to_defaults() {
        let body =
            r#"{"ID":"p","Members":null,"Requests":null,"Invites":null,"EligibleQueues":null}"#;
        let party: Party = serde_json::from_str(body).unwrap();
        assert!(party.members.is_empty() && party.requests.is_empty());
        assert!(party.invites.is_none());
        assert!(!party.is_custom_game_setup());
    }

    #[test]
    fn custom_setup_state_is_detected() {
        let party: Party = serde_json::from_str(r#"{"State":"CUSTOM_GAME_SETUP"}"#).unwrap();
        assert!(party.is_custom_game_setup());
    }

    #[test]
    fn party_player_decodes_current_party_id() {
        let body = format!(
            r#"{{"Subject":"{ME}","Version":5,"CurrentPartyID":"fe5ea6f4","Invites":null,
            "Requests":[],"PlatformInfo":{{"platformType":"PC"}}}}"#
        );
        let player: PartyPlayer = serde_json::from_str(&body).unwrap();
        assert_eq!(player.current_party_id.unwrap().as_str(), "fe5ea6f4");
        assert_eq!(player.version, 5);
        assert!(player.platform_info.is_some());
    }
}
