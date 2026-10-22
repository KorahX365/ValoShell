//! Game phase, snapshot and attack/defence side derivation.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::game_modes::rules_for;
use crate::numeric::title_case;

/// Team id Riot reports for the team that attacks the first half.
const ATTACKING_TEAM_FIRST_HALF: &str = "Red";
/// Team id Riot reports for the team that defends the first half.
const DEFENDING_TEAM_FIRST_HALF: &str = "Blue";

/// Where the player is in the game loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Phase {
    #[default]
    Offline,
    Menus,
    Pregame,
    Ingame,
}

impl Phase {
    /// True while a match exists (agent select or in the match itself).
    pub fn is_live(self) -> bool {
        matches!(self, Self::Pregame | Self::Ingame)
    }
}

/// Which half-role a team currently plays. Riot publishes no such field; it is derived.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Side {
    Attack,
    Defence,
}

impl Side {
    /// The other side (halves swap once).
    pub fn flipped(self) -> Self {
        match self {
            Self::Attack => Self::Defence,
            Self::Defence => Self::Attack,
        }
    }

    /// Short badge text: `ATK` or `DEF`.
    pub fn label(self) -> &'static str {
        match self {
            Self::Attack => "ATK",
            Self::Defence => "DEF",
        }
    }
}

/// Immutable view of the game the UI renders; compared by equality so unchanged states are not republished.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GameSnapshot {
    pub phase: Phase,
    pub map_name: String,
    pub ally_score: Option<u32>,
    pub enemy_score: Option<u32>,
    pub party_size: u32,
    pub queue: String,
    pub side: Option<Side>,
}

impl Default for GameSnapshot {
    fn default() -> Self {
        Self {
            phase: Phase::Offline,
            map_name: String::new(),
            ally_score: None,
            enemy_score: None,
            party_size: 1,
            queue: String::new(),
            side: None,
        }
    }
}

impl GameSnapshot {
    /// True while a match exists (agent select or in the match itself).
    pub fn is_live(&self) -> bool {
        self.phase.is_live()
    }

    /// True once both teams have a score (never true at agent select).
    pub fn has_score(&self) -> bool {
        self.ally_score.is_some() && self.enemy_score.is_some()
    }

    /// `ATK`, `DEF`, or an empty string when the side is unknown.
    pub fn side_label(&self) -> &'static str {
        self.side.map_or("", Side::label)
    }

    /// Short caption such as `Competitive · party of 3`; empty when nothing is worth saying.
    pub fn describe(&self) -> String {
        let mut parts = Vec::new();
        if !self.queue.is_empty() {
            parts.push(title_case(&self.queue));
        }
        if self.party_size > 1 {
            parts.push(format!("party of {}", self.party_size));
        }
        parts.join(" · ")
    }
}

/// The side a team starts on: Red attacks first, Blue defends first, anything else is unknown.
pub fn starting_side(team_id: &str) -> Option<Side> {
    match team_id {
        ATTACKING_TEAM_FIRST_HALF => Some(Side::Attack),
        DEFENDING_TEAM_FIRST_HALF => Some(Side::Defence),
        _ => None,
    }
}

/// The side after `rounds_played` rounds. Overtime is deliberately unknown: Riot's
/// overtime side rotation is undocumented and a wrong answer is worse than none.
pub fn current_side(team_id: &str, rounds_played: u32, rounds_to_win: u32) -> Option<Side> {
    let start = starting_side(team_id)?;
    let half = rounds_to_win.saturating_sub(1).max(1);
    if rounds_played >= 2 * half {
        None
    } else if rounds_played < half {
        Some(start)
    } else {
        Some(start.flipped())
    }
}

/// Side for a live snapshot. Kill races have no sides; with no score yet (agent select)
/// the team's starting side applies.
///
/// Retakes (`fortcollins`) answers like a 5-round-half mode, even
/// though its match log treats it as sideless.
pub fn side_from_snapshot(
    team_id: &str,
    ally_score: Option<u32>,
    enemy_score: Option<u32>,
    queue: &str,
) -> Option<Side> {
    if team_id.is_empty() {
        return None;
    }
    let rules = rules_for(queue, "");
    if !rules.is_race_to_rounds() {
        return None;
    }
    let (Some(ally), Some(enemy)) = (ally_score, enemy_score) else {
        return starting_side(team_id);
    };
    current_side(team_id, ally + enemy, rules.rounds_to_win?)
}

#[cfg(test)]
mod tests;
