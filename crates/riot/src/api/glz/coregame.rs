//! Coregame (in-match) requests and DTOs.

use domain::ids::{AgentId, MatchId, Puuid};
use serde::Deserialize;
use serde_json::Value;

use super::common::{PlayerIdentity, SeasonalBadgeInfo, SecretString, null_as_default};
use super::party_dto::MatchmakingData;
use crate::http::{Host, Request};

/// `GET /core-game/v1/players/{puuid}`: the player's match id (404 = not in a match).
pub fn get_player(puuid: &Puuid) -> Request {
    Request::get(Host::Glz, format!("/core-game/v1/players/{puuid}"))
}

/// `GET /core-game/v1/matches/{match_id}`: players, map, mode and connection details.
pub fn get_match(match_id: &MatchId) -> Request {
    Request::get(Host::Glz, format!("/core-game/v1/matches/{match_id}"))
}

/// `GET /core-game/v1/matches/{match_id}/loadouts`: skins and sprays of all players.
pub fn get_loadouts(match_id: &MatchId) -> Request {
    Request::get(
        Host::Glz,
        format!("/core-game/v1/matches/{match_id}/loadouts"),
    )
}

/// `POST /core-game/v1/players/{puuid}/disassociate/{match_id}`: leave the match (204).
///
/// Abandoning applies leaver penalties in ranked. No body is sent.
pub fn disassociate(puuid: &Puuid, match_id: &MatchId) -> Request {
    Request::post(
        Host::Glz,
        format!("/core-game/v1/players/{puuid}/disassociate/{match_id}"),
    )
}

/// One player in a running match.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct CoregamePlayer {
    pub subject: Option<Puuid>,
    #[serde(rename = "TeamID")]
    pub team_id: String,
    #[serde(rename = "CharacterID")]
    pub character_id: Option<AgentId>,
    #[serde(deserialize_with = "null_as_default")]
    pub player_identity: PlayerIdentity,
    pub seasonal_badge_info: Option<SeasonalBadgeInfo>,
    pub is_coach: bool,
    pub is_associated: bool,
}

/// Where the game client connects. `PlayerKey` is a secret join key and is never printed.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct ConnectionDetails {
    #[serde(deserialize_with = "null_as_default")]
    pub game_server_hosts: Vec<String>,
    pub game_server_host: String,
    pub game_server_port: u16,
    pub player_key: SecretString,
}

/// The in-progress match.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct CoregameMatch {
    #[serde(rename = "MatchID")]
    pub match_id: Option<MatchId>,
    pub version: u64,
    /// `IN_PROGRESS` while playing.
    pub state: String,
    #[serde(rename = "MapID")]
    pub map_id: String,
    #[serde(rename = "ModeID")]
    pub mode_id: String,
    pub provisioning_flow: String,
    #[serde(rename = "GamePodID")]
    pub game_pod_id: String,
    #[serde(rename = "AllMUCName")]
    pub all_muc_name: String,
    #[serde(rename = "TeamMUCName")]
    pub team_muc_name: String,
    #[serde(rename = "TeamVoiceID")]
    pub team_voice_id: String,
    pub team_match_token: SecretString,
    pub is_reconnectable: bool,
    pub connection_details: Option<ConnectionDetails>,
    pub post_game_details: Option<Value>,
    #[serde(deserialize_with = "null_as_default")]
    pub players: Vec<CoregamePlayer>,
    /// Not always present; its `QueueID` is the queue the match came from.
    pub matchmaking_data: Option<MatchmakingData>,
}

impl CoregameMatch {
    /// The queue id when the server included matchmaking data.
    pub fn queue_id(&self) -> Option<&str> {
        self.matchmaking_data
            .as_ref()
            .map(|data| data.queue_id.as_str())
            .filter(|queue_id| !queue_id.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use reqwest::Method;

    use super::*;

    const FIXTURE: &str = include_str!("fixtures/coregame_match.json");

    fn match_id() -> MatchId {
        MatchId::new("0f6c5b0e-3b4c-4f0e-9c1e-aaaaaaaaaaaa")
    }

    #[test]
    fn reads_are_gets_on_glz_without_body() {
        let player = get_player(&Puuid::new("abc"));
        assert_eq!(player.path, "/core-game/v1/players/abc");
        let loadouts = get_loadouts(&match_id());
        assert_eq!(
            loadouts.path,
            format!("/core-game/v1/matches/{}/loadouts", match_id())
        );
        let matched = get_match(&match_id());
        assert_eq!(
            matched.path,
            format!("/core-game/v1/matches/{}", match_id())
        );
        for request in [player, loadouts, matched] {
            assert_eq!((request.host, request.method), (Host::Glz, Method::GET));
            assert!(request.body.is_none());
        }
    }

    #[test]
    fn disassociate_is_a_post_not_a_get() {
        let request = disassociate(&Puuid::new("abc"), &match_id());
        assert_eq!(request.method, Method::POST);
        assert_eq!(
            request.path,
            format!("/core-game/v1/players/abc/disassociate/{}", match_id())
        );
        assert!(request.body.is_none());
    }

    #[test]
    fn fixture_decodes_match_and_players() {
        let coregame: CoregameMatch = serde_json::from_str(FIXTURE).unwrap();
        assert_eq!(
            coregame.match_id.as_ref().unwrap().as_str(),
            match_id().as_str()
        );
        assert_eq!(coregame.state, "IN_PROGRESS");
        assert_eq!(coregame.map_id, "/Game/Maps/Triad/Triad");
        assert_eq!(
            coregame.mode_id,
            "/Game/GameModes/Bomb/BombGameMode.BombGameMode_C"
        );
        assert_eq!(coregame.queue_id(), Some("competitive"));
        assert_eq!(coregame.players.len(), 2);
        let player = &coregame.players[0];
        assert_eq!(player.team_id, "Red");
        assert!(player.is_associated && !player.is_coach);
        assert_eq!(player.player_identity.account_level, 120);
        assert_eq!(
            player
                .seasonal_badge_info
                .as_ref()
                .unwrap()
                .leaderboard_rank,
            42
        );
    }

    #[test]
    fn fixture_hides_secrets_in_debug_output() {
        let coregame: CoregameMatch = serde_json::from_str(FIXTURE).unwrap();
        let details = coregame.connection_details.as_ref().unwrap();
        assert_eq!(details.game_server_port, 7777);
        assert_eq!(details.player_key.expose(), "join-key-secret");
        let printed = format!("{coregame:?}");
        assert!(!printed.contains("join-key-secret") && !printed.contains("match-token-secret"));
    }

    #[test]
    fn missing_matchmaking_data_has_no_queue_id() {
        let coregame: CoregameMatch = serde_json::from_str(r#"{"MatchmakingData":null}"#).unwrap();
        assert_eq!(coregame.queue_id(), None);
        assert!(coregame.players.is_empty());
    }
}
