//! Queue selection and matchmaking start/stop.

use domain::ids::PartyId;
use serde_json::json;

use crate::http::{Host, Request};

/// `POST /parties/v1/parties/{party_id}/queue`: pick a queue (owner only, must be eligible).
///
/// The key is `queueID`; it is the only spelling backed by working code. Rejected with 400
/// while the party is in `CUSTOM_GAME_SETUP`.
pub fn change_queue(party_id: &PartyId, queue_id: &str) -> Request {
    Request::post(Host::Glz, format!("/parties/v1/parties/{party_id}/queue"))
        .json(json!({ "queueID": queue_id }))
}

/// `POST /parties/v1/parties/{party_id}/matchmaking/join`: start searching for the whole party.
pub fn join_matchmaking(party_id: &PartyId) -> Request {
    Request::post(
        Host::Glz,
        format!("/parties/v1/parties/{party_id}/matchmaking/join"),
    )
    .json(json!({}))
}

/// `POST /parties/v1/parties/{party_id}/matchmaking/leave`: cancel the search.
pub fn leave_matchmaking(party_id: &PartyId) -> Request {
    Request::post(
        Host::Glz,
        format!("/parties/v1/parties/{party_id}/matchmaking/leave"),
    )
}

#[cfg(test)]
mod tests {
    use reqwest::Method;

    use super::*;

    fn party() -> PartyId {
        PartyId::new("fe5ea6f4-a974-412a-8422-e0631171756b")
    }

    #[test]
    fn change_queue_sends_queue_id_key() {
        let request = change_queue(&party(), "competitive");
        assert_eq!(request.host, Host::Glz);
        assert_eq!(request.method, Method::POST);
        assert_eq!(
            request.path,
            format!("/parties/v1/parties/{}/queue", party())
        );
        assert_eq!(request.body, Some(json!({ "queueID": "competitive" })));
    }

    #[test]
    fn join_matchmaking_sends_empty_object() {
        let request = join_matchmaking(&party());
        assert_eq!(request.method, Method::POST);
        assert_eq!(
            request.path,
            format!("/parties/v1/parties/{}/matchmaking/join", party())
        );
        assert_eq!(request.body, Some(json!({})));
    }

    #[test]
    fn leave_matchmaking_has_no_body() {
        let request = leave_matchmaking(&party());
        assert_eq!(request.method, Method::POST);
        assert_eq!(
            request.path,
            format!("/parties/v1/parties/{}/matchmaking/leave", party())
        );
        assert!(request.body.is_none());
    }
}
