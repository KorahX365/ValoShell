//! Presences, friends, friend requests and blocks.

use domain::ids::Puuid;
use serde::Deserialize;
use serde_json::json;

use super::with_query;
use crate::http::{Host, Request};

/// `GET /chat/v4/presences`: self plus friends (never the lobby). All filters optional.
pub fn get_presences(puuid: Option<&Puuid>, product: Option<&str>) -> Request {
    let puuid = puuid.map(Puuid::as_str);
    let path = with_query(
        "/chat/v4/presences",
        &[("puuid", puuid), ("product", product)],
    );
    Request::get(Host::Local, path)
}

/// `GET /chat/v4/friends`: the friend list.
pub fn get_friends() -> Request {
    Request::get(Host::Local, "/chat/v4/friends")
}

/// `DELETE /chat/v4/friends`: removes a friend. Destructive; only on an explicit click.
pub fn remove_friend(puuid: &Puuid) -> Request {
    Request::delete(Host::Local, "/chat/v4/friends").json(json!({ "puuid": puuid }))
}

/// `GET /chat/v4/friendrequests`: pending incoming and outgoing requests.
pub fn get_friend_requests() -> Request {
    Request::get(Host::Local, "/chat/v4/friendrequests")
}

/// `POST /chat/v4/friendrequests` by Riot ID. Sends a request, or accepts one the target
/// already sent. Never use it to look a name up: the request is a real side effect.
pub fn send_friend_request(game_name: &str, game_tag: &str) -> Request {
    Request::post(Host::Local, "/chat/v4/friendrequests")
        .json(json!({ "game_name": game_name, "game_tag": game_tag }))
}

/// `POST /chat/v4/friendrequests` by puuid: accepts an incoming request.
pub fn accept_friend_request(puuid: &Puuid) -> Request {
    Request::post(Host::Local, "/chat/v4/friendrequests").json(json!({ "puuid": puuid }))
}

/// `DELETE /chat/v4/friendrequests`: declines an incoming or cancels an outgoing request.
pub fn decline_friend_request(puuid: &Puuid) -> Request {
    Request::delete(Host::Local, "/chat/v4/friendrequests").json(json!({ "puuid": puuid }))
}

/// `GET /chat/v4/blocked`: blocked players.
pub fn get_blocked() -> Request {
    Request::get(Host::Local, "/chat/v4/blocked")
}

/// One entry of `presences`; `private` stays the raw base64 blob (see `domain::presence`).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Presence {
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub actor: String,
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub basic: String,
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub details: String,
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub game_name: String,
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub game_tag: String,
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub location: String,
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub msg: String,
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub name: String,
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub patchline: String,
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub pid: String,
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub platform: String,
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub private: String,
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub product: String,
    pub puuid: Option<Puuid>,
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub region: String,
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub resource: String,
    /// `mobile|dnd|away|chat`.
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub state: String,
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub summary: String,
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub time: String,
}

impl Presence {
    /// The VALORANT presence of a player (League friends share the same feed).
    pub fn is_valorant(&self) -> bool {
        self.product == "valorant"
    }
}

/// Body of `GET /chat/v4/presences`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct PresencesResponse {
    #[serde(deserialize_with = "crate::api::codec::skip_bad_entries")]
    pub presences: Vec<Presence>,
}

/// One friend.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Friend {
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub game_name: String,
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub game_tag: String,
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub group: String,
    #[serde(
        rename = "displayGroup",
        deserialize_with = "crate::api::codec::loose_string"
    )]
    pub display_group: String,
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub name: String,
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub note: String,
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub pid: String,
    pub puuid: Option<Puuid>,
    #[serde(deserialize_with = "crate::api::codec::loose_string")]
    pub region: String,
    #[serde(rename = "activePlatform")]
    pub active_platform: Option<String>,
    /// Milliseconds since the epoch; `null` while online or unknown.
    pub last_online_ts: Option<i64>,
}

/// Body of `GET /chat/v4/friends`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct FriendsResponse {
    #[serde(deserialize_with = "crate::api::codec::skip_bad_entries")]
    pub friends: Vec<Friend>,
}

/// One friend request or blocked player (the two lists share their shape).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct FriendRequest {
    pub game_name: String,
    pub game_tag: String,
    pub name: String,
    pub note: String,
    pub pid: String,
    pub puuid: Option<Puuid>,
    pub region: String,
    /// `pending_in` (incoming) or `pending_out` (outgoing); empty for blocked players.
    pub subscription: String,
    pub platform: String,
}

impl FriendRequest {
    /// The other player asked us.
    pub fn is_incoming(&self) -> bool {
        self.subscription == "pending_in"
    }
}

/// Body of `GET /chat/v4/friendrequests`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct FriendRequestsResponse {
    pub requests: Vec<FriendRequest>,
}

/// Body of `GET /chat/v4/blocked`. UNVERIFIED on 13.08: the list key is assumed to be
/// `blocked`, like the sibling endpoints; `players` is accepted as a fallback.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct BlockedResponse {
    #[serde(alias = "players")]
    pub blocked: Vec<FriendRequest>,
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn puuid() -> Puuid {
        Puuid::new("AAAA-1111")
    }

    fn assert_local(request: &Request, method: reqwest::Method, path: &str, body: Option<Value>) {
        assert_eq!(request.host, Host::Local);
        assert_eq!(request.method, method);
        assert_eq!(request.path, path);
        assert_eq!(request.body, body);
    }

    #[test]
    fn read_builders_have_no_body() {
        let get = reqwest::Method::GET;

        assert_local(&get_friends(), get.clone(), "/chat/v4/friends", None);
        assert_local(
            &get_friend_requests(),
            get.clone(),
            "/chat/v4/friendrequests",
            None,
        );
        assert_local(&get_blocked(), get.clone(), "/chat/v4/blocked", None);
        assert_local(&get_presences(None, None), get, "/chat/v4/presences", None);
    }

    #[test]
    fn presences_filters_become_query() {
        let request = get_presences(Some(&puuid()), Some("valorant"));

        assert_eq!(
            request.path,
            "/chat/v4/presences?puuid=aaaa-1111&product=valorant"
        );
    }

    #[test]
    fn friend_request_by_name_sends_riot_id() {
        let request = send_friend_request("Name", "TAG");

        let body = json!({ "game_name": "Name", "game_tag": "TAG" });
        let post = reqwest::Method::POST;
        assert_local(&request, post, "/chat/v4/friendrequests", Some(body));
    }

    #[test]
    fn accept_decline_and_remove_carry_puuid() {
        let body = Some(json!({ "puuid": "aaaa-1111" }));
        let requests = "/chat/v4/friendrequests";

        let accept = accept_friend_request(&puuid());
        let decline = decline_friend_request(&puuid());
        let remove = remove_friend(&puuid());

        assert_local(&accept, reqwest::Method::POST, requests, body.clone());
        assert_local(&decline, reqwest::Method::DELETE, requests, body.clone());
        assert_local(&remove, reqwest::Method::DELETE, "/chat/v4/friends", body);
    }

    #[test]
    fn presences_decode_with_raw_private_blob() {
        let json = r#"{"presences":[{"game_name":"A","game_tag":"B","product":"valorant",
            "private":"eyJ9","puuid":"P-1","state":"chat","pid":"x@eu1.pvp.net","time":"1"},
            {"product":"league_of_legends","private":"not base64 {"}]}"#;

        let response: PresencesResponse = serde_json::from_str(json).expect("decodes");

        assert_eq!(response.presences.len(), 2);
        assert!(response.presences[0].is_valorant());
        assert_eq!(response.presences[0].private, "eyJ9");
        assert!(!response.presences[1].is_valorant());
    }

    #[test]
    fn presences_tolerate_numeric_time_and_null_fields() {
        let json = r#"{"presences":[{"actor":null,"game_name":"A","game_tag":"B",
            "patchline":null,"platform":null,"private":"eyJ9","privateJwt":null,
            "product":"valorant","puuid":"P-1","state":"chat","time":1700000000000},
            {"product":"valorant","puuid":5,"state":"chat"}]}"#;

        let response: PresencesResponse = serde_json::from_str(json).expect("decodes");

        assert_eq!(response.presences.len(), 1);
        assert_eq!(response.presences[0].time, "1700000000000");
        assert_eq!(response.presences[0].patchline, "");
    }

    #[test]
    fn friends_decode_with_nullable_timestamp() {
        let json = r#"{"friends":[{"activePlatform":null,"displayGroup":"","game_name":"A",
            "game_tag":"B","group":"","last_online_ts":1700000000000,"name":"A","note":"",
            "pid":"p","puuid":"abc","region":"eu1"},{"game_name":"C","last_online_ts":null}]}"#;

        let response: FriendsResponse = serde_json::from_str(json).expect("decodes");

        assert_eq!(response.friends[0].last_online_ts, Some(1_700_000_000_000));
        assert_eq!(response.friends[1].last_online_ts, None);
        let puuid = response.friends[0].puuid.as_ref().expect("puuid");
        assert_eq!(puuid.as_str(), "abc");
    }

    #[test]
    fn friend_requests_tell_direction_apart() {
        let json = r#"{"requests":[{"game_name":"A","subscription":"pending_in","puuid":"1"},
            {"game_name":"B","subscription":"pending_out","puuid":"2"}]}"#;

        let response: FriendRequestsResponse = serde_json::from_str(json).expect("decodes");

        assert!(response.requests[0].is_incoming());
        assert!(!response.requests[1].is_incoming());
    }

    #[test]
    fn blocked_list_accepts_either_key() {
        let primary: BlockedResponse =
            serde_json::from_str(r#"{"blocked":[{"puuid":"1"}]}"#).expect("decodes");
        let fallback: BlockedResponse =
            serde_json::from_str(r#"{"players":[{"puuid":"2"}]}"#).expect("decodes");

        assert_eq!(primary.blocked.len(), 1);
        assert_eq!(fallback.blocked.len(), 1);
    }
}
