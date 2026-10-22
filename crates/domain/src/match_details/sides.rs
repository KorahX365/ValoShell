//! Attack/defence split of a finished match.
//!
//! Riot publishes no side field. Red attacks the first half and Blue defends it, confirmed
//! against the spike plants; sides swap one round before the race target (12 of 13, 4 of 5).

use serde::Serialize;
use serde_json::Value;
use ts_rs::TS;

use super::json::{exact_int, optional_str, roster, subject, team_id};
use super::rounds::round_results;
use crate::game_modes::{ModeRules, is_round_based, rules_for};

/// Team that attacks the first half.
pub const ATTACKING_TEAM_FIRST_HALF: &str = "Red";

/// Retakes documents no half-time rule and Replication's race length is unverified, so a half
/// derived for them would be a guess on a guess.
const NO_CONFIRMED_SWAP_QUEUES: [&str; 2] = ["fortcollins", "onefa"];

/// Competitive half-time: sides swap after this round.
const COMPETITIVE_HALF_LENGTH: u32 = 12;

/// Overtime side rotation is undocumented, so rounds past this are never tallied.
const LAST_REGULATION_ROUND: i64 = 23;

/// Side a team starts the match on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum StartingSide {
    Attack,
    Defence,
}

impl StartingSide {
    /// The side as text (`attack` or `defence`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Attack => "attack",
            Self::Defence => "defence",
        }
    }
}

/// Which side a team started on and how its rounds split by side.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct SideSplit {
    /// `None` when the mode has no sides.
    pub started_side: Option<StartingSide>,
    pub attack_won: u32,
    pub attack_played: u32,
    pub defence_won: u32,
    pub defence_played: u32,
}

fn other_team(team: &str) -> &'static str {
    if team == "Blue" { "Red" } else { "Blue" }
}

/// Rounds per half: one less than the race target. Unknown queues use competitive rules.
fn half_length(rules: &ModeRules) -> i64 {
    rules
        .rounds_to_win
        .map_or(i64::from(COMPETITIVE_HALF_LENGTH), |wins| {
            i64::from(wins) - 1
        })
}

/// The team that attacked the first half, judged by the first regulation spike plant. Callers
/// can compare it with `ATTACKING_TEAM_FIRST_HALF` to notice if Riot ever changes the rule.
pub fn first_half_attacker_from_plants(detail: &Value, half: i64) -> Option<String> {
    let players = roster(detail);
    for round in round_results(detail) {
        let number = exact_int(round.get("roundNum"));
        let planter = optional_str(round, &["bombPlanter"]);
        let (Some(number), Some(planter)) = (number, planter) else {
            continue;
        };
        if number > LAST_REGULATION_ROUND {
            continue;
        }
        let team = players
            .iter()
            .find(|entry| subject(entry) == planter)
            .map(|e| team_id(e));
        let Some(team) = team.filter(|team| !team.is_empty()) else {
            continue;
        };
        // Sides swap after `half` rounds, so normalise a second-half plant back to the first.
        return Some(if number < half {
            team
        } else {
            other_team(&team).to_owned()
        });
    }
    None
}

/// Which side `own_team` started on and how its rounds split by side. Overtime is excluded,
/// and modes without rounds or without a confirmed swap have no sides at all.
pub fn derive_sides(detail: &Value, own_team: &str, queue: &str) -> SideSplit {
    let has_no_sides = !queue.is_empty()
        && (!is_round_based(queue, "") || NO_CONFIRMED_SWAP_QUEUES.contains(&queue));
    let rounds: Vec<&Value> = round_results(detail).collect();
    if has_no_sides || rounds.is_empty() {
        return SideSplit::default();
    }
    let half = half_length(&rules_for(queue, ""));
    let first_half_attacker = first_half_attacker_from_plants(detail, half)
        .unwrap_or_else(|| ATTACKING_TEAM_FIRST_HALF.to_owned());
    let started_attacking = !own_team.is_empty() && first_half_attacker == own_team;
    let mut split = SideSplit {
        started_side: Some(if started_attacking {
            StartingSide::Attack
        } else {
            StartingSide::Defence
        }),
        ..SideSplit::default()
    };
    for round in rounds {
        let Some(number) = exact_int(round.get("roundNum")).filter(|n| *n <= LAST_REGULATION_ROUND)
        else {
            continue;
        };
        let on_attack = if number < half {
            started_attacking
        } else {
            !started_attacking
        };
        let did_win = optional_str(round, &["winningTeam"]) == Some(own_team);
        tally_round(&mut split, on_attack, did_win);
    }
    split
}

fn tally_round(split: &mut SideSplit, on_attack: bool, did_win: bool) {
    let (played, won) = if on_attack {
        (&mut split.attack_played, &mut split.attack_won)
    } else {
        (&mut split.defence_played, &mut split.defence_won)
    };
    *played += 1;
    *won += u32::from(did_win);
}

#[cfg(test)]
mod tests;
