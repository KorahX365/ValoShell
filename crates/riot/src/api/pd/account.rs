//! Account level and XP history.

use domain::ids::{MatchId, Puuid};
use serde::Deserialize;
use serde_json::Value;

use crate::http::{Host, Request};

/// `GET /account-xp/v1/players/{puuid}`: account level, XP inside the level and recent XP grants.
pub fn get_account_xp(puuid: &Puuid) -> Request {
    Request::get(Host::Pd, format!("/account-xp/v1/players/{puuid}"))
}

/// The account-XP profile.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct AccountXp {
    pub version: i64,
    pub subject: Option<Puuid>,
    pub progress: LevelProgress,
    #[serde(deserialize_with = "super::null_as_default")]
    pub history: Vec<XpHistoryEntry>,
    pub last_time_granted_first_win: String,
    pub next_time_first_win_available: String,
}

/// Level and the XP earned inside it (5,000 XP per level).
#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct LevelProgress {
    pub level: u32,
    #[serde(rename = "XP")]
    pub xp: u32,
}

/// XP earned by one match.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct XpHistoryEntry {
    #[serde(rename = "ID")]
    pub id: Option<MatchId>,
    pub match_start: String,
    pub start_progress: LevelProgress,
    pub end_progress: LevelProgress,
    #[serde(rename = "XPDelta")]
    pub xp_delta: i64,
    #[serde(rename = "XPSources", deserialize_with = "super::null_as_default")]
    pub xp_sources: Vec<XpSource>,
    /// Multiplier shapes are not documented.
    #[serde(rename = "XPMultipliers", deserialize_with = "super::null_as_default")]
    pub xp_multipliers: Vec<Value>,
}

/// One XP source: `time-played`, `match-win` or `first-win-of-the-day`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct XpSource {
    #[serde(rename = "ID")]
    pub id: String,
    pub amount: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_xp_request_targets_the_player() {
        let request = get_account_xp(&Puuid::new("abc"));

        assert_eq!(request.host, Host::Pd);
        assert_eq!(request.path, "/account-xp/v1/players/abc");
    }

    #[test]
    fn account_xp_decodes_level_and_history() {
        let json = r#"{"Version":1,"Subject":"abc","Progress":{"Level":150,"XP":3941},
            "History":[{"ID":"m-1","MatchStart":"2026-10-14T01:28:56Z",
              "StartProgress":{"Level":150,"XP":3000},"EndProgress":{"Level":150,"XP":3941},
              "XPDelta":941,"XPSources":[{"ID":"time-played","Amount":600},
              {"ID":"match-win","Amount":341}],"XPMultipliers":[]}],
            "LastTimeGrantedFirstWin":"2026-10-13T10:00:00Z",
            "NextTimeFirstWinAvailable":"2026-10-14T10:00:00Z"}"#;

        let xp: AccountXp = serde_json::from_str(json).expect("decodes");

        assert_eq!(xp.progress.level, 150);
        assert_eq!(xp.progress.xp, 3941);
        assert_eq!(xp.history[0].xp_delta, 941);
        assert_eq!(xp.history[0].xp_sources[1].id, "match-win");
        assert_eq!(xp.history[0].end_progress.xp, 3941);
    }
}
