//! Party lobby requests: lookup, accessibility, invites, join requests, codes, leave/kick.

use domain::ids::{PartyId, Puuid};
use serde_json::json;

use super::common::encode_path_segment;
use crate::http::{Host, Request};

/// Who may join the party without an invite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Accessibility {
    Open,
    Closed,
}

impl Accessibility {
    /// The wire value Riot expects.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "OPEN",
            Self::Closed => "CLOSED",
        }
    }
}

/// `GET /parties/v1/players/{puuid}`: the signed-in player's current party id (no ping query).
pub fn get_player_party(puuid: &Puuid) -> Request {
    Request::get(Host::Glz, format!("/parties/v1/players/{puuid}"))
}

/// `GET /parties/v1/parties/{party_id}`: the party with members, queue and eligibility.
pub fn get_party(party_id: &PartyId) -> Request {
    Request::get(Host::Glz, format!("/parties/v1/parties/{party_id}"))
}

/// `POST /parties/v1/parties/{party_id}/accessibility`: open or close the party to friends.
pub fn set_accessibility(party_id: &PartyId, accessibility: Accessibility) -> Request {
    Request::post(
        Host::Glz,
        format!("/parties/v1/parties/{party_id}/accessibility"),
    )
    .json(json!({ "accessibility": accessibility.as_str() }))
}

/// `POST /parties/v1/parties/{party_id}/invites/name/{name}/tag/{tag}`: invite by Riot ID.
pub fn invite_by_riot_id(party_id: &PartyId, name: &str, tag: &str) -> Request {
    let (name, tag) = (encode_path_segment(name), encode_path_segment(tag));
    Request::post(
        Host::Glz,
        format!("/parties/v1/parties/{party_id}/invites/name/{name}/tag/{tag}"),
    )
}

/// `POST /parties/v1/parties/{party_id}/invites/decline`: decline the pending invite (204).
///
/// UNVERIFIED on 13.08: the path is logged, the body is not known; none is sent.
pub fn decline_invite(party_id: &PartyId) -> Request {
    Request::post(
        Host::Glz,
        format!("/parties/v1/parties/{party_id}/invites/decline"),
    )
}

/// `POST /parties/v1/parties/{party_id}/request`: ask to join a closed party.
pub fn request_to_join(party_id: &PartyId, requester: &Puuid) -> Request {
    Request::post(Host::Glz, format!("/parties/v1/parties/{party_id}/request"))
        .json(json!({ "Subjects": [requester.as_str()] }))
}

/// `POST /parties/v1/parties/{party_id}/request/{request_id}/decline`: owner declines a request.
pub fn decline_join_request(party_id: &PartyId, request_id: &str) -> Request {
    let request_id = encode_path_segment(request_id);
    Request::post(
        Host::Glz,
        format!("/parties/v1/parties/{party_id}/request/{request_id}/decline"),
    )
}

/// `POST /parties/v1/parties/{party_id}/invitecode`: generate a shareable code.
pub fn generate_invite_code(party_id: &PartyId) -> Request {
    Request::post(
        Host::Glz,
        format!("/parties/v1/parties/{party_id}/invitecode"),
    )
}

/// `DELETE /parties/v1/parties/{party_id}/invitecode`: disable the code.
pub fn disable_invite_code(party_id: &PartyId) -> Request {
    Request::delete(
        Host::Glz,
        format!("/parties/v1/parties/{party_id}/invitecode"),
    )
}

/// `POST /parties/v1/players/joinbycode/{code}`: join the party that owns `code`.
pub fn join_by_code(code: &str) -> Request {
    let code = encode_path_segment(code);
    Request::post(Host::Glz, format!("/parties/v1/players/joinbycode/{code}")).json(json!({}))
}

/// `POST /parties/v1/players/{puuid}/joinparty/{party_id}`: accept an invite or join an open party.
pub fn join_party(puuid: &Puuid, party_id: &PartyId) -> Request {
    Request::post(
        Host::Glz,
        format!("/parties/v1/players/{puuid}/joinparty/{party_id}"),
    )
}

/// `DELETE /parties/v1/players/{puuid}`: leave the party (own puuid); starts a fresh solo party.
pub fn leave_party(puuid: &Puuid) -> Request {
    Request::delete(Host::Glz, format!("/parties/v1/players/{puuid}"))
}

/// `DELETE /parties/v1/players/{member}`: the owner removes another member.
///
/// UNVERIFIED on 13.08: kick semantics come from old docs; the same path is a plain leave
/// when `member` is the caller.
pub fn kick_member(member: &Puuid) -> Request {
    Request::delete(Host::Glz, format!("/parties/v1/players/{member}"))
}

/// `POST /parties/v1/parties/{party_id}/members/{member}/owner`: hand the party to `member`.
///
/// UNVERIFIED on 13.08: seen only in a 2022 endpoint list; body unknown, none is sent.
pub fn transfer_owner(party_id: &PartyId, member: &Puuid) -> Request {
    Request::post(
        Host::Glz,
        format!("/parties/v1/parties/{party_id}/members/{member}/owner"),
    )
}

/// `GET /parties/v2/parties/{party_id}/muctoken`: party chat (XMPP MUC) token.
pub fn get_muc_token(party_id: &PartyId) -> Request {
    Request::get(
        Host::Glz,
        format!("/parties/v2/parties/{party_id}/muctoken"),
    )
}

#[cfg(test)]
mod tests {
    use reqwest::Method;
    use serde_json::Value;

    use super::*;

    const ME: &str = "d7ff5397-caf5-4451-9057-0db61e73ab33";

    fn party() -> PartyId {
        PartyId::new("fe5ea6f4-a974-412a-8422-e0631171756b")
    }

    fn assert_request(request: &Request, method: Method, path: &str, body: Option<Value>) {
        assert_eq!(request.host, Host::Glz);
        assert_eq!(request.method, method);
        assert_eq!(request.path, path);
        assert_eq!(request.body, body);
    }

    #[test]
    fn reads_target_player_and_party() {
        let player = get_player_party(&Puuid::new(ME));
        assert_request(
            &player,
            Method::GET,
            &format!("/parties/v1/players/{ME}"),
            None,
        );
        let path = format!("/parties/v1/parties/{}", party());
        assert_request(&get_party(&party()), Method::GET, &path, None);
    }

    #[test]
    fn accessibility_sends_wire_value() {
        let request = set_accessibility(&party(), Accessibility::Open);
        let path = format!("/parties/v1/parties/{}/accessibility", party());
        let body = json!({ "accessibility": "OPEN" });
        assert_request(&request, Method::POST, &path, Some(body));
        assert_eq!(Accessibility::Closed.as_str(), "CLOSED");
    }

    #[test]
    fn invite_by_riot_id_encodes_name_and_tag() {
        let request = invite_by_riot_id(&party(), "Mr Fox", "EU 1");
        let path = format!(
            "/parties/v1/parties/{}/invites/name/Mr%20Fox/tag/EU%201",
            party()
        );
        assert_request(&request, Method::POST, &path, None);
    }

    #[test]
    fn decline_invite_has_no_body() {
        let path = format!("/parties/v1/parties/{}/invites/decline", party());
        assert_request(&decline_invite(&party()), Method::POST, &path, None);
    }

    #[test]
    fn join_requests_carry_requester_subjects() {
        let request = request_to_join(&party(), &Puuid::new(ME));
        let path = format!("/parties/v1/parties/{}/request", party());
        assert_request(
            &request,
            Method::POST,
            &path,
            Some(json!({ "Subjects": [ME] })),
        );
        let decline = decline_join_request(&party(), "req-1");
        let path = format!("/parties/v1/parties/{}/request/req-1/decline", party());
        assert_request(&decline, Method::POST, &path, None);
    }

    #[test]
    fn invite_code_is_created_and_disabled_on_same_path() {
        let path = format!("/parties/v1/parties/{}/invitecode", party());
        assert_request(&generate_invite_code(&party()), Method::POST, &path, None);
        assert_request(&disable_invite_code(&party()), Method::DELETE, &path, None);
    }

    #[test]
    fn join_by_code_sends_empty_object() {
        let request = join_by_code("ABC 12");
        let path = "/parties/v1/players/joinbycode/ABC%2012";
        assert_request(&request, Method::POST, path, Some(json!({})));
    }

    #[test]
    fn join_party_names_player_and_party() {
        let request = join_party(&Puuid::new(ME), &party());
        let path = format!("/parties/v1/players/{ME}/joinparty/{}", party());
        assert_request(&request, Method::POST, &path, None);
    }

    #[test]
    fn leave_and_kick_use_delete_on_player() {
        let path = format!("/parties/v1/players/{ME}");
        assert_request(&leave_party(&Puuid::new(ME)), Method::DELETE, &path, None);
        assert_request(&kick_member(&Puuid::new(ME)), Method::DELETE, &path, None);
    }

    #[test]
    fn transfer_owner_targets_member_owner_path() {
        let request = transfer_owner(&party(), &Puuid::new(ME));
        let path = format!("/parties/v1/parties/{}/members/{ME}/owner", party());
        assert_request(&request, Method::POST, &path, None);
    }

    #[test]
    fn muc_token_uses_v2_path() {
        let path = format!("/parties/v2/parties/{}/muctoken", party());
        assert_request(&get_muc_token(&party()), Method::GET, &path, None);
    }
}
