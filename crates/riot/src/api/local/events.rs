//! Local websocket (WAMP-1 style JSON arrays)
//! building and parsing; the socket itself lives in the connection supervisor.

use serde_json::Value;

/// WAMP opcode for a client subscription.
const WAMP_SUBSCRIBE: u64 = 5;
/// WAMP opcode for a server event.
const WAMP_EVENT: u64 = 8;
/// The single topic carrying every event the Riot Client emits.
pub const EVENT_TOPIC: &str = "OnJsonApiEvent";

const PREGAME_URI_PART: &str =
    "/riot-messaging-service/v1/message/ares-pregame/pregame/v1/matches/";
const COREGAME_URI_PART: &str =
    "/riot-messaging-service/v1/message/ares-core-game/core-game/v1/matches/";
const PARTY_URI_PART: &str = "/riot-messaging-service/v1/message/ares-parties/parties/v1/parties/";
const OUT_OF_SYNC_URI_PART: &str = "/riot-messaging-service/v1/out-of-sync";

/// The text frame that subscribes to all events (no acknowledgement is awaited).
pub fn subscribe_frame() -> String {
    Value::Array(vec![WAMP_SUBSCRIBE.into(), EVENT_TOPIC.into()]).to_string()
}

/// One decoded `[8, "OnJsonApiEvent", {uri, eventType, data}]` frame.
#[derive(Debug, Clone, PartialEq)]
pub struct JsonApiEvent {
    pub uri: String,
    /// `Create|Update|Delete`; empty when absent.
    pub event_type: String,
    pub data: Value,
}

/// Decode an event frame. `None` for anything else: invalid JSON, other opcodes, fewer than
/// three elements, or a non-object payload (skipped, never a crash).
pub fn parse_event_frame(text: &str) -> Option<JsonApiEvent> {
    let frame: Value = serde_json::from_str(text).ok()?;
    let elements = frame.as_array().filter(|elements| elements.len() >= 3)?;
    if elements[0].as_u64() != Some(WAMP_EVENT) {
        return None;
    }
    let payload = elements[2].as_object()?;
    Some(JsonApiEvent {
        uri: string_field(payload, "uri"),
        event_type: string_field(payload, "eventType"),
        data: payload.get("data").cloned().unwrap_or(Value::Null),
    })
}

fn string_field(payload: &serde_json::Map<String, Value>, key: &str) -> String {
    payload
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

/// What an event URI tells the app to do. Match and party ids in the URI are never read:
/// callers re-fetch over REST and use the event only as a trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UriKind {
    /// Agent select started, changed or locked.
    Pregame,
    /// Match loaded or ended.
    Coregame,
    /// Party members, queue or settings changed.
    Party,
    /// Own or friend presence update.
    Presence,
    /// Friend list changed. UNVERIFIED on 13.08.
    Friends,
    /// Friend requests changed. UNVERIFIED on 13.08.
    FriendRequests,
    /// Chat messages or conversations changed. UNVERIFIED on 13.08.
    Chat,
    /// The messaging service lost sync: refetch everything.
    OutOfSync,
}

/// Ordered substring rules; first match wins. Specific RMS paths come before generic ones.
const URI_RULES: [(&str, UriKind); 10] = [
    (PREGAME_URI_PART, UriKind::Pregame),
    (COREGAME_URI_PART, UriKind::Coregame),
    (PARTY_URI_PART, UriKind::Party),
    (OUT_OF_SYNC_URI_PART, UriKind::OutOfSync),
    ("/chat/v4/presences", UriKind::Presence),
    ("/chat/v4/friendrequests", UriKind::FriendRequests),
    ("/chat/v4/friends", UriKind::Friends),
    ("/chat/v6/messages", UriKind::Chat),
    ("/chat/v6/conversations", UriKind::Chat),
    ("/chat/v5/participants", UriKind::Chat),
];

/// Classify an event URI by substring; `None` for events we ignore.
pub fn classify_uri(uri: &str) -> Option<UriKind> {
    URI_RULES
        .iter()
        .find(|(part, _)| uri.contains(part))
        .map(|(_, kind)| *kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rms_uri(service: &str, path: &str) -> String {
        format!("/riot-messaging-service/v1/message/{service}/{path}abc")
    }

    #[test]
    fn subscribe_frame_is_the_wamp_subscribe_array() {
        assert_eq!(subscribe_frame(), r#"[5,"OnJsonApiEvent"]"#);
    }

    #[test]
    fn event_frame_is_parsed_into_typed_event() {
        let text = r#"[8,"OnJsonApiEvent",{"eventType":"Update","uri":"/chat/v4/presences",
            "data":{"presences":[]}}]"#;

        let event = parse_event_frame(text).expect("event");

        assert_eq!(event.uri, "/chat/v4/presences");
        assert_eq!(event.event_type, "Update");
        assert_eq!(event.data, serde_json::json!({ "presences": [] }));
    }

    #[test]
    fn missing_fields_default_to_empty() {
        let event = parse_event_frame(r#"[8,"OnJsonApiEvent",{}]"#).expect("event");

        assert_eq!(event.uri, "");
        assert_eq!(event.event_type, "");
        assert_eq!(event.data, Value::Null);
    }

    #[test]
    fn unusable_frames_are_skipped_without_panicking() {
        let frames = [
            "",
            "not json",
            "{}",
            "[]",
            r#"[8,"OnJsonApiEvent"]"#,
            r#"[8,"OnJsonApiEvent","text"]"#,
            r#"[8,"OnJsonApiEvent",[1,2]]"#,
            r#"[8,"OnJsonApiEvent",null]"#,
            r#"[0,"id",{"uri":"/x"}]"#,
            r#"["8","OnJsonApiEvent",{"uri":"/x"}]"#,
        ];

        for frame in frames {
            assert_eq!(parse_event_frame(frame), None, "frame: {frame}");
        }
    }

    #[test]
    fn riot_messaging_uris_map_to_game_events() {
        let pregame = rms_uri("ares-pregame", "pregame/v1/matches/");
        let coregame = rms_uri("ares-core-game", "core-game/v1/matches/");
        let party = rms_uri("ares-parties", "parties/v1/parties/");

        assert_eq!(classify_uri(&pregame), Some(UriKind::Pregame));
        assert_eq!(classify_uri(&coregame), Some(UriKind::Coregame));
        assert_eq!(classify_uri(&party), Some(UriKind::Party));
    }

    #[test]
    fn social_uris_are_classified() {
        let cases = [
            ("/chat/v4/presences", UriKind::Presence),
            ("/chat/v4/friends", UriKind::Friends),
            ("/chat/v4/friendrequests", UriKind::FriendRequests),
            ("/chat/v6/messages/abc", UriKind::Chat),
            ("/chat/v6/conversations", UriKind::Chat),
            ("/riot-messaging-service/v1/out-of-sync", UriKind::OutOfSync),
        ];

        for (uri, kind) in cases {
            assert_eq!(classify_uri(uri), Some(kind), "uri: {uri}");
        }
    }

    #[test]
    fn unrelated_uris_are_ignored() {
        assert_eq!(classify_uri("/product-session/v1/sessions"), None);
        assert_eq!(classify_uri(""), None);
    }
}
