//! Pregame (agent select) requests.
//!
//! Select and lock are irreversible-ish and policy-gated: callers must only invoke them from a
//! direct user click. Select, lock and quit use the short pregame timeout so a retry still
//! lands inside the select timer.

use domain::ids::{AgentId, MatchId, Puuid};
use serde_json::json;

use crate::http::{Host, PREGAME_TIMEOUT, Request};

/// `GET /pregame/v1/players/{puuid}`: the player's pregame match id (404 = not in agent select).
pub fn get_player(puuid: &Puuid) -> Request {
    Request::get(Host::Glz, format!("/pregame/v1/players/{puuid}"))
}

/// `GET /pregame/v1/matches/{match_id}`: teams, selection states, timers.
pub fn get_match(match_id: &MatchId) -> Request {
    Request::get(Host::Glz, format!("/pregame/v1/matches/{match_id}"))
}

/// `POST /pregame/v1/matches/{match_id}/select/{agent_id}`: hover an agent; teammates see it.
pub fn select_agent(match_id: &MatchId, agent_id: &AgentId) -> Request {
    Request::post(
        Host::Glz,
        format!("/pregame/v1/matches/{match_id}/select/{agent_id}"),
    )
    .json(json!({}))
    .timeout(PREGAME_TIMEOUT)
}

/// `POST /pregame/v1/matches/{match_id}/lock/{agent_id}`: irreversible lock-in.
pub fn lock_agent(match_id: &MatchId, agent_id: &AgentId) -> Request {
    Request::post(
        Host::Glz,
        format!("/pregame/v1/matches/{match_id}/lock/{agent_id}"),
    )
    .json(json!({}))
    .timeout(PREGAME_TIMEOUT)
}

/// `POST /pregame/v1/matches/{match_id}/quit`: dodge; carries a queue-dodge penalty (204).
pub fn quit_match(match_id: &MatchId) -> Request {
    Request::post(Host::Glz, format!("/pregame/v1/matches/{match_id}/quit"))
        .json(json!({}))
        .timeout(PREGAME_TIMEOUT)
}

/// `GET /pregame/v1/matches/{match_id}/loadouts`: all players' loadouts.
pub fn get_loadouts(match_id: &MatchId) -> Request {
    Request::get(
        Host::Glz,
        format!("/pregame/v1/matches/{match_id}/loadouts"),
    )
}

#[cfg(test)]
mod tests {
    use reqwest::Method;

    use super::*;
    use crate::http::REQUEST_TIMEOUT;

    fn match_id() -> MatchId {
        MatchId::new("6e69661e-a009-4e89-b697-6fd442cd7277")
    }

    fn agent() -> AgentId {
        AgentId::new("add6443a-41bd-e414-f6ad-e58d267f4e95")
    }

    #[test]
    fn reads_use_default_timeout_and_no_body() {
        let player = get_player(&Puuid::new("abc"));
        assert_eq!(
            (player.method.clone(), player.path.as_str()),
            (Method::GET, "/pregame/v1/players/abc")
        );
        for request in [player, get_match(&match_id()), get_loadouts(&match_id())] {
            assert_eq!(request.host, Host::Glz);
            assert_eq!(request.timeout, REQUEST_TIMEOUT);
            assert!(request.body.is_none());
        }
        assert!(get_loadouts(&match_id()).path.ends_with("/loadouts"));
    }

    #[test]
    fn select_and_lock_send_empty_object_with_pregame_timeout() {
        let select = select_agent(&match_id(), &agent());
        let lock = lock_agent(&match_id(), &agent());
        assert_eq!(
            select.path,
            format!("/pregame/v1/matches/{}/select/{}", match_id(), agent())
        );
        assert_eq!(
            lock.path,
            format!("/pregame/v1/matches/{}/lock/{}", match_id(), agent())
        );
        for request in [select, lock] {
            assert_eq!(request.method, Method::POST);
            assert_eq!(request.body, Some(json!({})));
            assert_eq!(request.timeout, PREGAME_TIMEOUT);
        }
    }

    #[test]
    fn quit_sends_empty_object_with_pregame_timeout() {
        let request = quit_match(&match_id());
        assert_eq!(request.method, Method::POST);
        assert_eq!(
            request.path,
            format!("/pregame/v1/matches/{}/quit", match_id())
        );
        assert_eq!(request.body, Some(json!({})));
        assert_eq!(request.timeout, PREGAME_TIMEOUT);
    }
}
