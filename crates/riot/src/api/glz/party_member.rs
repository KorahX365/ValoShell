//! Requests scoped to one party member or player: ready flag, refreshes, solo experiences.

use domain::ids::{PartyId, Puuid};
use serde_json::json;

use super::common::encode_path_segment;
use crate::http::{Host, Request};

/// One game-pod latency sample for `refresh_pings`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PodPing {
    pub game_pod_id: String,
    pub milliseconds: u32,
}

/// What `start_solo_experience` launches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoloGameType {
    ShootingRange,
    BotTrainingMatch,
    NewPlayerExperience,
    WatchReplay,
    ReplayNewPlayerExperience,
}

impl SoloGameType {
    /// The wire value Riot expects.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ShootingRange => "ShootingRange",
            Self::BotTrainingMatch => "BotTrainingMatch",
            Self::NewPlayerExperience => "NewPlayerExperience",
            Self::WatchReplay => "WatchReplay",
            Self::ReplayNewPlayerExperience => "ReplayNewPlayerExperience",
        }
    }
}

fn member_path(party_id: &PartyId, member: &Puuid, action: &str) -> String {
    format!("/parties/v1/parties/{party_id}/members/{member}/{action}")
}

/// `POST /parties/v1/parties/{party_id}/members/{member}/setReady`: own ready flag only.
pub fn set_ready(party_id: &PartyId, member: &Puuid, is_ready: bool) -> Request {
    Request::post(Host::Glz, member_path(party_id, member, "setReady"))
        .json(json!({ "ready": is_ready }))
}

/// `POST .../members/{member}/refreshCompetitiveTier`: resync the member's displayed rank.
pub fn refresh_competitive_tier(party_id: &PartyId, member: &Puuid) -> Request {
    Request::post(
        Host::Glz,
        member_path(party_id, member, "refreshCompetitiveTier"),
    )
}

/// `POST .../members/{member}/refreshPlayerIdentity`: resync card, title and level border.
pub fn refresh_player_identity(party_id: &PartyId, member: &Puuid) -> Request {
    Request::post(
        Host::Glz,
        member_path(party_id, member, "refreshPlayerIdentity"),
    )
}

/// `POST .../members/{member}/refreshPings?{pod}={ms}&...`: publish latency samples.
pub fn refresh_pings(party_id: &PartyId, member: &Puuid, pings: &[PodPing]) -> Request {
    let query: Vec<String> = pings
        .iter()
        .map(|ping| {
            let pod = encode_path_segment(&ping.game_pod_id);
            format!("{pod}={}", ping.milliseconds)
        })
        .collect();
    let mut path = member_path(party_id, member, "refreshPings");
    if !query.is_empty() {
        path.push('?');
        path.push_str(&query.join("&"));
    }
    Request::post(Host::Glz, path)
}

/// `POST /parties/v1/players/{puuid}/startsoloexperience`: launch the Range, bots or a replay.
///
/// `module` is `""` for most game types (e.g. `"Aimbots_spawn"` for the Range).
pub fn start_solo_experience(puuid: &Puuid, game_type: SoloGameType, module: &str) -> Request {
    Request::post(
        Host::Glz,
        format!("/parties/v1/players/{puuid}/startsoloexperience"),
    )
    .json(json!({ "gameType": game_type.as_str(), "module": module }))
}

#[cfg(test)]
mod tests {
    use reqwest::Method;

    use super::*;

    const ME: &str = "d7ff5397-caf5-4451-9057-0db61e73ab33";

    fn party() -> PartyId {
        PartyId::new("fe5ea6f4-a974-412a-8422-e0631171756b")
    }

    #[test]
    fn set_ready_sends_flag() {
        let request = set_ready(&party(), &Puuid::new(ME), true);
        assert_eq!(request.method, Method::POST);
        assert_eq!(request.host, Host::Glz);
        assert_eq!(
            request.path,
            format!("/parties/v1/parties/{}/members/{ME}/setReady", party())
        );
        assert_eq!(request.body, Some(json!({ "ready": true })));
    }

    #[test]
    fn refreshes_post_without_body() {
        let member = Puuid::new(ME);
        let tier = refresh_competitive_tier(&party(), &member);
        let identity = refresh_player_identity(&party(), &member);
        assert!(tier.path.ends_with("/refreshCompetitiveTier"));
        assert!(identity.path.ends_with("/refreshPlayerIdentity"));
        assert!(tier.body.is_none() && identity.body.is_none());
        assert_eq!(tier.method, Method::POST);
    }

    #[test]
    fn refresh_pings_builds_query_per_pod() {
        let pings = [
            PodPing {
                game_pod_id: "aresriot.aws-eus2-prod.eu-gp-madrid-1".into(),
                milliseconds: 19,
            },
            PodPing {
                game_pod_id: "aresriot.aws-euw3-prod.eu-gp-paris-1".into(),
                milliseconds: 31,
            },
        ];
        let request = refresh_pings(&party(), &Puuid::new(ME), &pings);
        assert!(request.path.ends_with(
            "/refreshPings?aresriot.aws-eus2-prod.eu-gp-madrid-1=19&aresriot.aws-euw3-prod.eu-gp-paris-1=31"
        ));
    }

    #[test]
    fn refresh_pings_without_samples_has_no_query() {
        let request = refresh_pings(&party(), &Puuid::new(ME), &[]);
        assert!(request.path.ends_with("/refreshPings"));
    }

    #[test]
    fn solo_experience_sends_game_type_and_module() {
        let request = start_solo_experience(
            &Puuid::new(ME),
            SoloGameType::ShootingRange,
            "Aimbots_spawn",
        );
        assert_eq!(
            request.path,
            format!("/parties/v1/players/{ME}/startsoloexperience")
        );
        let expected = json!({ "gameType": "ShootingRange", "module": "Aimbots_spawn" });
        assert_eq!(request.body, Some(expected));
        assert_eq!(SoloGameType::WatchReplay.as_str(), "WatchReplay");
    }
}
