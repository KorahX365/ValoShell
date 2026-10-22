//! Facts about a match that are the same for every player: start, queue and team outcome.

use serde_json::Value;

use super::json::{first_truthy, is_truthy, lenient_float, match_info, text};
use crate::game_modes::resolve_queue;

/// When a match started, in unix seconds; 0 when the response does not say.
pub fn played_at(detail: &Value) -> f64 {
    let info = match_info(detail);
    lenient_float(first_truthy(info, &["gameStartMillis"])) / 1000.0
}

/// The queue a match was played in, resolved through the game-mode path for custom games.
/// Empty when unrecognised.
pub fn queue_of(detail: &Value) -> &'static str {
    let info = match_info(detail);
    let queue_id = text(info, &["queueID", "queueId", "QueueID"]);
    let mode_path = text(info, &["gameMode", "GameMode"]);
    resolve_queue(&queue_id, &mode_path)
}

/// Whether a team entry says it won (`won`, falling back to `Won`).
pub(crate) fn won_flag(team: &Value) -> bool {
    super::json::first_present(team, &["won", "Won"]).is_some_and(is_truthy)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn played_at_converts_millis_and_defaults_to_zero() {
        let detail = json!({"matchInfo": {"gameStartMillis": 1_700_000_000_500_i64}});
        assert_eq!(played_at(&detail), 1_700_000_000.5);
        assert_eq!(played_at(&json!({})), 0.0);
    }

    #[test]
    fn queue_falls_back_to_the_game_mode_path_for_customs() {
        let detail =
            json!({"matchInfo": {"queueID": "", "gameMode": "/Game/GameModes/QuickBomb/X"}});
        assert_eq!(queue_of(&detail), "spikerush");
        let ranked = json!({"matchInfo": {"queueId": "Competitive"}});
        assert_eq!(queue_of(&ranked), "competitive");
    }

    #[test]
    fn won_flag_reads_either_casing() {
        assert!(won_flag(&json!({"won": true})));
        assert!(won_flag(&json!({"Won": true})));
        assert!(!won_flag(&json!({"won": false})));
    }
}
