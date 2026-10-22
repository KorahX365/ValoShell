//! Name service: batch puuid -> Riot ID lookup.

use domain::ids::Puuid;
use serde::Deserialize;
use serde_json::Value;

use crate::http::{Host, Request};

/// `PUT /name-service/v2/players`: resolve many puuids in one call (body is a bare JSON array).
///
/// The official client sometimes gets 403 here (cause UNVERIFIED); callers treat it as "no names".
pub fn put_names(puuids: &[Puuid]) -> Request {
    let body = puuids
        .iter()
        .map(|puuid| Value::String(puuid.to_string()))
        .collect();
    Request::put(Host::Pd, "/name-service/v2/players").json(Value::Array(body))
}

/// One name-service entry. Answers are unordered, may omit or repeat subjects: map by `subject`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct PlayerName {
    pub subject: Option<Puuid>,
    pub display_name: String,
    pub game_name: String,
    pub tag_line: String,
}

impl PlayerName {
    /// A blank `GameName` means the player is hidden (streamer mode / incognito).
    pub fn is_redacted(&self) -> bool {
        self.game_name.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Method;

    #[test]
    fn put_names_sends_a_bare_array_of_puuids() {
        let puuids = [Puuid::new("AAA"), Puuid::new("bbb")];

        let request = put_names(&puuids);

        assert_eq!(request.method, Method::PUT);
        assert_eq!(request.host, Host::Pd);
        assert_eq!(request.path, "/name-service/v2/players");
        assert_eq!(request.body, Some(serde_json::json!(["aaa", "bbb"])));
    }

    #[test]
    fn names_decode_and_flag_redacted_players() {
        let json = r#"[
            {"DisplayName":"Neo#EUW","Subject":"aaa-1","GameName":"Neo","TagLine":"EUW"},
            {"DisplayName":"","Subject":"bbb-2","GameName":"","TagLine":""}
        ]"#;

        let names: Vec<PlayerName> = serde_json::from_str(json).expect("decodes");

        assert_eq!(names[0].game_name, "Neo");
        assert_eq!(names[0].tag_line, "EUW");
        assert!(!names[0].is_redacted());
        assert!(names[1].is_redacted());
    }
}
