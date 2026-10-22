//! The live `GameSnapshot`, rebuilt from the player's own presence.
//! `GameTracker` is a plain reducer so every transition rule is unit-tested.

use domain::phase::{GameSnapshot, Phase, side_from_snapshot};
use domain::presence::{LiveScores, LoopState, OwnPresence};

/// Folds presence updates into a snapshot.
#[derive(Debug, Default)]
pub struct GameTracker {
    snapshot: GameSnapshot,
    /// `Blue`/`Red`, learned at agent select or from old-build presence.
    team_id: String,
}

impl GameTracker {
    /// Current snapshot.
    pub fn snapshot(&self) -> &GameSnapshot {
        &self.snapshot
    }

    /// Connected: the menus are the starting point.
    pub fn attach(&mut self) -> Option<GameSnapshot> {
        let next = GameSnapshot {
            phase: Phase::Menus,
            ..GameSnapshot::default()
        };
        self.publish(next)
    }

    /// Disconnected: forget everything.
    pub fn detach(&mut self) -> Option<GameSnapshot> {
        self.team_id.clear();
        self.publish(GameSnapshot::default())
    }

    /// Team from the pregame roster (`AllyTeam.TeamID`).
    pub fn set_team(&mut self, team_id: &str) -> Option<GameSnapshot> {
        self.team_id = team_id.to_owned();
        let next = self.with_side(self.snapshot.clone());
        self.publish(next)
    }

    /// Apply the player's own presence. Returns the snapshot only when it changed.
    pub fn apply_presence(&mut self, presence: &OwnPresence) -> Option<GameSnapshot> {
        let previous = &self.snapshot;
        // A momentary gap in the loop state must not announce "match ended".
        let phase = presence.loop_state.map_or(
            if previous.phase == Phase::Offline {
                Phase::Menus
            } else {
                previous.phase
            },
            phase_of,
        );
        let is_match_over = previous.phase.is_live() && !phase.is_live();
        if is_match_over {
            self.team_id.clear();
        }
        let mut next = GameSnapshot {
            phase,
            party_size: presence.party_size.unwrap_or(previous.party_size).max(1),
            ..GameSnapshot::default()
        };
        if !is_match_over && phase.is_live() {
            next.map_name.clone_from(&previous.map_name);
            next.queue.clone_from(&previous.queue);
        }
        if let Some(scores) = &presence.live_scores {
            self.merge_scores(&mut next, scores);
        }
        let next = self.with_side(next);
        self.publish(next)
    }

    fn merge_scores(&mut self, next: &mut GameSnapshot, scores: &LiveScores) {
        next.ally_score = Some(scores.ally_score);
        next.enemy_score = Some(scores.enemy_score);
        if matches!(scores.current_team.as_str(), "Blue" | "Red") {
            self.team_id.clone_from(&scores.current_team);
        }
        if scores.map_name != "Unknown" {
            next.map_name.clone_from(&scores.map_name);
        }
        if !scores.queue.is_empty() {
            next.queue.clone_from(&scores.queue);
        }
    }

    fn with_side(&self, mut snapshot: GameSnapshot) -> GameSnapshot {
        snapshot.side = side_from_snapshot(
            &self.team_id,
            snapshot.ally_score,
            snapshot.enemy_score,
            &snapshot.queue,
        );
        snapshot
    }

    fn publish(&mut self, next: GameSnapshot) -> Option<GameSnapshot> {
        if next == self.snapshot {
            return None;
        }
        self.snapshot = next.clone();
        Some(next)
    }
}

fn phase_of(state: LoopState) -> Phase {
    match state {
        LoopState::Menus => Phase::Menus,
        LoopState::Pregame => Phase::Pregame,
        LoopState::Ingame => Phase::Ingame,
    }
}

#[cfg(test)]
mod tests;
