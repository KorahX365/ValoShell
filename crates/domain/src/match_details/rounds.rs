//! Round-level readers shared by the scoreboard and the own-match extractor.

use serde_json::Value;

use super::json::{items, number, subject, text_ref};

/// The first kill of a round: who got it and who died.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OpeningKill<'a> {
    pub killer: &'a str,
    pub victim: &'a str,
}

/// Round entries of a match-details response; non-objects are dropped.
pub(crate) fn round_results(detail: &Value) -> impl Iterator<Item = &Value> {
    items(detail, &["roundResults", "RoundResults"])
        .iter()
        .filter(|round| round.is_object())
}

/// Per-player entries of one round; non-objects are dropped.
pub(crate) fn player_stats(round: &Value) -> impl Iterator<Item = &Value> {
    items(round, &["playerStats", "PlayerStats"])
        .iter()
        .filter(|stat| stat.is_object())
}

/// Kill entries of one player's round stats; non-objects are dropped.
pub(crate) fn kills(stat: &Value) -> impl Iterator<Item = &Value> {
    items(stat, &["kills", "Kills"])
        .iter()
        .filter(|kill| kill.is_object())
}

/// The earliest kill of a round by `roundTime`, counting only kills made by players for whom
/// `is_tracked` holds. Ties go to the first kill listed.
///
/// Both the scoreboard and the own-match extractor use this one rule: it is the only reliable
/// opener, because `firstBloodPlayer` is not guaranteed to be present.
pub(crate) fn opening_kill<'a>(
    round: &'a Value,
    is_tracked: impl Fn(&str) -> bool,
) -> Option<OpeningKill<'a>> {
    let mut best: Option<(f64, &'a Value)> = None;
    for stat in player_stats(round).filter(|stat| is_tracked(&subject(stat))) {
        for kill in kills(stat) {
            let Some(at) = number(kill.get("roundTime").or_else(|| kill.get("RoundTime"))) else {
                continue;
            };
            if best.is_none_or(|(earliest, _)| at < earliest) {
                best = Some((at, kill));
            }
        }
    }
    best.map(|(_, kill)| OpeningKill {
        killer: text_ref(kill, &["killer", "Killer"]),
        victim: text_ref(kill, &["victim", "Victim"]),
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn round_with_kills() -> Value {
        json!({"playerStats": [
            {"subject": "a", "kills": [{"roundTime": 9000, "killer": "a", "victim": "c"}]},
            {"subject": "b", "kills": [{"roundTime": 4000, "killer": "b", "victim": "d"}]},
            {"subject": "e", "kills": [{"roundTime": 4000, "killer": "e", "victim": "f"}]},
        ]})
    }

    #[test]
    fn opening_kill_is_the_earliest_by_round_time() {
        let round = round_with_kills();
        let opener = opening_kill(&round, |_| true).unwrap();
        assert_eq!(
            opener,
            OpeningKill {
                killer: "b",
                victim: "d"
            }
        );
    }

    #[test]
    fn opening_kill_ignores_untracked_killers() {
        let round = round_with_kills();
        let opener = opening_kill(&round, |puuid| puuid == "a" || puuid == "e").unwrap();
        assert_eq!(opener.killer, "e");
    }

    #[test]
    fn opening_kill_is_none_without_timed_kills() {
        let round = json!({"playerStats": [{"subject": "a", "kills": [{"killer": "a"}]}]});
        assert_eq!(opening_kill(&round, |_| true), None);
    }
}
