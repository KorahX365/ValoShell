//! Types and helpers shared by every glz module.

use std::fmt;

use domain::ids::{MatchId, Puuid};
use serde::Deserialize;
use serde_json::Value;

/// Deserializes `null` as the type's default; `#[serde(default)]` alone only covers missing keys.
pub(crate) use crate::api::codec::null_as_default;

pub(crate) use crate::api::codec::encode_path_segment;

/// A credential-like string (chat token, match token, game server key). `Debug` never prints it.
#[derive(Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct SecretString(String);

impl SecretString {
    /// The raw value; call only where it must be used, never to log.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretString(<redacted>)")
    }
}

/// Shared by party, pregame and coregame player rows.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct PlayerIdentity {
    pub subject: Option<Puuid>,
    #[serde(rename = "PlayerCardID")]
    pub player_card_id: String,
    #[serde(rename = "PlayerTitleID")]
    pub player_title_id: String,
    pub account_level: u32,
    #[serde(rename = "PreferredLevelBorderID")]
    pub preferred_level_border_id: String,
    pub incognito: bool,
    pub hide_account_level: bool,
}

/// Season badge shown next to a player; `LeaderboardRank` is 0 when unranked.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct SeasonalBadgeInfo {
    #[serde(rename = "SeasonID")]
    pub season_id: String,
    pub number_of_wins: u32,
    pub wins_by_tier: Option<Value>,
    pub rank: u32,
    pub leaderboard_rank: u32,
}

/// `GET .../players/{puuid}` on pregame and coregame: which match the player is in.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct PhaseLink {
    pub subject: Option<Puuid>,
    #[serde(rename = "MatchID")]
    pub match_id: Option<MatchId>,
    pub version: u64,
}

/// Chat or voice token reply: `{Token, Room}`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct ChatToken {
    pub token: SecretString,
    pub room: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_path_segment_escapes_spaces_and_unicode() {
        assert_eq!(encode_path_segment("Mr Ñ#1"), "Mr%20%C3%91%231");
    }

    #[test]
    fn encode_path_segment_keeps_unreserved_bytes() {
        assert_eq!(encode_path_segment("a-b_c.d~E9"), "a-b_c.d~E9");
    }

    #[test]
    fn secret_string_debug_hides_value() {
        let body = r#"{"Token":"jwt.secret","Room":"r@x"}"#;
        let token: ChatToken = serde_json::from_str(body).unwrap();
        assert!(!format!("{token:?}").contains("jwt.secret"));
        assert_eq!(token.token.expose(), "jwt.secret");
    }

    #[test]
    fn phase_link_decodes_match_id() {
        let body = r#"{"Subject":"ab-1","MatchID":"cd-2","Version":7}"#;
        let link: PhaseLink = serde_json::from_str(body).unwrap();
        assert_eq!(link.match_id.unwrap().as_str(), "cd-2");
        assert_eq!(link.version, 7);
    }

    #[test]
    fn null_as_default_turns_null_into_empty_vec() {
        #[derive(Deserialize)]
        struct Holder {
            #[serde(default, deserialize_with = "null_as_default")]
            values: Vec<u8>,
        }
        let holder: Holder = serde_json::from_str(r#"{"values":null}"#).unwrap();
        assert!(holder.values.is_empty());
    }
}
