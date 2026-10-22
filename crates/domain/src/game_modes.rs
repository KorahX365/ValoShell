//! Queue rules: how many rounds win a match and whether the mode is a round race.

use serde::Serialize;
use ts_rs::TS;

use crate::numeric::title_case;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Tiebreak {
    WinByTwo,
    SuddenDeath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct ModeRules {
    pub label: &'static str,
    pub rounds_to_win: Option<u32>,
    pub is_round_based: bool,
    pub tiebreak: Tiebreak,
}

impl ModeRules {
    const fn rounds(label: &'static str, rounds_to_win: u32, tiebreak: Tiebreak) -> Self {
        Self {
            label,
            rounds_to_win: Some(rounds_to_win),
            is_round_based: true,
            tiebreak,
        }
    }

    const fn not_rounds(label: &'static str) -> Self {
        Self {
            label,
            rounds_to_win: None,
            is_round_based: false,
            tiebreak: Tiebreak::WinByTwo,
        }
    }

    /// A race to N rounds (sides, halves and a score make sense).
    pub fn is_race_to_rounds(&self) -> bool {
        self.is_round_based && self.rounds_to_win.is_some_and(|rounds| rounds > 0)
    }
}

/// Default for unrecognised queues: competitive rules are the safe guess.
pub const STANDARD: ModeRules = ModeRules::rounds("Competitive", 13, Tiebreak::WinByTwo);

/// Keys are lowercased Riot queue ids. `onefa` = 9 is unverified upstream (TODO 2026-09-28).
const QUEUE_RULES: [(&str, ModeRules); 12] = [
    ("competitive", STANDARD),
    (
        "unrated",
        ModeRules::rounds("Unrated", 13, Tiebreak::SuddenDeath),
    ),
    (
        "premier",
        ModeRules::rounds("Premier", 13, Tiebreak::WinByTwo),
    ),
    (
        "newmap",
        ModeRules::rounds("New Map", 13, Tiebreak::WinByTwo),
    ),
    (
        "swiftplay",
        ModeRules::rounds("Swiftplay", 5, Tiebreak::WinByTwo),
    ),
    (
        "spikerush",
        ModeRules::rounds("Spike Rush", 4, Tiebreak::WinByTwo),
    ),
    (
        "onefa",
        ModeRules::rounds("Replication", 9, Tiebreak::WinByTwo),
    ),
    (
        "fortcollins",
        ModeRules::rounds("Retakes", 5, Tiebreak::WinByTwo),
    ),
    ("hurm", ModeRules::not_rounds("Team Deathmatch")),
    ("deathmatch", ModeRules::not_rounds("Deathmatch")),
    ("ggteam", ModeRules::not_rounds("Escalation")),
    ("snowball", ModeRules::not_rounds("Snowball Fight")),
];

/// Ordered: first substring hit in the lowercased mode path wins. `bomb` must stay last
/// because `QuickBomb` contains it.
const MODE_PATH_HINTS: [(&str, &str); 12] = [
    ("quickbomb", "spikerush"),
    ("spikerush", "spikerush"),
    ("hurm", "hurm"),
    ("deathmatch", "deathmatch"),
    ("swiftplay", "swiftplay"),
    ("gungame", "ggteam"),
    ("escalation", "ggteam"),
    ("fortcollins", "fortcollins"),
    ("oneforall", "onefa"),
    ("onefa", "onefa"),
    ("snowball", "snowball"),
    ("bomb", "unrated"),
];

fn find_rules(key: &str) -> Option<ModeRules> {
    QUEUE_RULES
        .iter()
        .find(|(queue, _)| *queue == key)
        .map(|(_, rules)| *rules)
}

/// Canonical queue key from a queue id, falling back to the mode asset path. `""` if unknown.
pub fn resolve_queue(queue_id: &str, mode_path: &str) -> &'static str {
    let key = queue_id.trim().to_lowercase();
    if let Some((queue, _)) = QUEUE_RULES.iter().find(|(queue, _)| *queue == key) {
        return queue;
    }
    let path = mode_path.to_lowercase();
    MODE_PATH_HINTS
        .iter()
        .find(|(hint, _)| path.contains(hint))
        .map_or("", |(_, queue)| queue)
}

/// Rules for a queue; unrecognised queues get `STANDARD`.
pub fn rules_for(queue_id: &str, mode_path: &str) -> ModeRules {
    find_rules(resolve_queue(queue_id, mode_path)).unwrap_or(STANDARD)
}

/// Whether the queue is known at all.
pub fn is_recognized(queue_id: &str, mode_path: &str) -> bool {
    !resolve_queue(queue_id, mode_path).is_empty()
}

pub fn is_round_based(queue_id: &str, mode_path: &str) -> bool {
    rules_for(queue_id, mode_path).is_round_based
}

/// Display label. Never claims "Competitive" for a queue it does not know.
pub fn label_for(queue_id: &str) -> String {
    let key = queue_id.trim().to_lowercase();
    match find_rules(&key) {
        Some(rules) => rules.label.to_owned(),
        None if key.is_empty() => "Unknown".to_owned(),
        None => title_case(&key.replace('_', " ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_queue_resolves_directly() {
        assert_eq!(resolve_queue(" Competitive ", ""), "competitive");
        assert_eq!(rules_for("swiftplay", "").rounds_to_win, Some(5));
    }

    #[test]
    fn quickbomb_path_is_spike_rush_not_unrated() {
        assert_eq!(
            resolve_queue("", "/Game/GameModes/QuickBomb/QuickBombGameMode"),
            "spikerush"
        );
        assert_eq!(
            resolve_queue("", "/Game/GameModes/Bomb/BombGameMode"),
            "unrated"
        );
    }

    #[test]
    fn unknown_queue_uses_standard_rules_but_is_not_recognized() {
        assert_eq!(rules_for("mystery", ""), STANDARD);
        assert!(!is_recognized("mystery", ""));
    }

    #[test]
    fn deathmatch_is_not_a_race_to_rounds() {
        assert!(!rules_for("deathmatch", "").is_race_to_rounds());
        assert!(rules_for("unrated", "").is_race_to_rounds());
    }

    #[test]
    fn label_for_unknown_queue_is_titled_key() {
        assert_eq!(label_for("team_rush"), "Team Rush");
        assert_eq!(label_for(""), "Unknown");
        assert_eq!(label_for("onefa"), "Replication");
    }
}
