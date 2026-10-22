//! Match history and match details.

use domain::ids::{MatchId, Puuid};
use serde::Deserialize;
use serde_json::Value;

use crate::http::{Host, Request};

/// Riot rejects wider history/update windows (20 at most).
pub const MAX_WINDOW_SPAN: u32 = 20;

/// A `startIndex..endIndex` slice of a paged list; the span is capped at [`MAX_WINDOW_SPAN`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexWindow {
    pub start_index: u32,
    pub end_index: u32,
}

impl IndexWindow {
    /// A window starting at `start_index` and covering at most `count` entries.
    pub fn new(start_index: u32, count: u32) -> Self {
        Self {
            start_index,
            end_index: start_index.saturating_add(count.min(MAX_WINDOW_SPAN)),
        }
    }

    /// Query string for this window plus an optional queue filter (`competitive`, `unrated`, ...).
    pub(crate) fn query(&self, queue: Option<&str>) -> String {
        let mut query = format!(
            "startIndex={}&endIndex={}",
            self.start_index, self.end_index
        );
        if let Some(queue) = queue {
            query.push_str("&queue=");
            query.push_str(queue);
        }
        query
    }
}

/// `GET /match-history/v1/history/{puuid}`: recent match ids; no queue filter = all modes.
///
/// 404 means an empty account or queue, not a failure.
pub fn get_match_history(puuid: &Puuid, window: IndexWindow, queue: Option<&str>) -> Request {
    let query = window.query(queue);
    Request::get(
        Host::Pd,
        format!("/match-history/v1/history/{puuid}?{query}"),
    )
}

/// `GET /match-details/v1/matches/{match_id}`: 50-150 KB, immutable once complete (cache forever).
///
/// 404 for expired matches and for custom games that skip history.
pub fn get_match_details(match_id: &MatchId) -> Request {
    Request::get(Host::Pd, format!("/match-details/v1/matches/{match_id}"))
}

/// Match details stay a raw tree: the domain crate parses it tolerantly (camel/Pascal variants).
pub type MatchDetails = Value;

/// The match-history page.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct MatchHistory {
    pub subject: Option<Puuid>,
    pub begin_index: u32,
    pub end_index: u32,
    pub total: u32,
    #[serde(alias = "history", deserialize_with = "super::null_as_default")]
    pub history: Vec<HistoryEntry>,
}

/// One match in the history list.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct HistoryEntry {
    #[serde(rename = "MatchID", alias = "matchId")]
    pub match_id: Option<MatchId>,
    /// Epoch milliseconds.
    pub game_start_time: i64,
    #[serde(rename = "QueueID")]
    pub queue_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_caps_the_span_at_twenty() {
        let window = IndexWindow::new(10, 100);

        assert_eq!(window.start_index, 10);
        assert_eq!(window.end_index, 30);
    }

    #[test]
    fn history_request_carries_window_and_queue() {
        let puuid = Puuid::new("abc");

        let request = get_match_history(&puuid, IndexWindow::new(0, 15), Some("competitive"));

        assert_eq!(request.host, Host::Pd);
        assert_eq!(request.method, reqwest::Method::GET);
        assert_eq!(
            request.path,
            "/match-history/v1/history/abc?startIndex=0&endIndex=15&queue=competitive"
        );
    }

    #[test]
    fn history_request_without_queue_has_no_filter() {
        let request = get_match_history(&Puuid::new("abc"), IndexWindow::new(20, 20), None);

        assert!(request.path.ends_with("?startIndex=20&endIndex=40"));
    }

    #[test]
    fn match_details_request_targets_the_match() {
        let request = get_match_details(&MatchId::new("M-1"));

        assert_eq!(request.path, "/match-details/v1/matches/m-1");
        assert!(request.body.is_none());
    }

    #[test]
    fn history_decodes_pascal_case() {
        let json = r#"{"Subject":"abc","BeginIndex":0,"EndIndex":2,"Total":40,"History":[
            {"MatchID":"m-1","GameStartTime":1700000000000,"QueueID":"competitive"},
            {"MatchID":"m-2","GameStartTime":1700000100000,"QueueID":"unrated"}]}"#;

        let history: MatchHistory = serde_json::from_str(json).expect("decodes");

        assert_eq!(history.total, 40);
        assert_eq!(history.history.len(), 2);
        assert_eq!(history.history[1].queue_id, "unrated");
        assert_eq!(history.history[0].game_start_time, 1_700_000_000_000);
    }

    #[test]
    fn history_decodes_lowercase_variants_and_null() {
        let json = r#"{"history":[{"matchId":"m-9"}]}"#;
        let null_json = r#"{"History":null}"#;

        let history: MatchHistory = serde_json::from_str(json).expect("decodes");
        let empty: MatchHistory = serde_json::from_str(null_json).expect("decodes");

        assert_eq!(history.history[0].match_id, Some(MatchId::new("m-9")));
        assert!(empty.history.is_empty());
    }
}
