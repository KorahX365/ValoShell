//! When the menu steps aside for the game and when it comes back.
//!
//! Pure: `step(state, input, rules) -> (state, effects)`. The services layer feeds it phase
//! changes and lock confirmations and executes the effects on real windows; tests feed it
//! plain values. Time is an input (seconds), never read from a clock.

use serde::Serialize;
use ts_rs::TS;

use crate::phase::Phase;

/// After the user shows or hides the menu by hand, automation leaves windows alone this long,
/// so it never fights a deliberate choice.
pub const MANUAL_OVERRIDE_S: f64 = 10.0;

/// Where the match flow is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum FlowStage {
    /// Not connected to the game.
    Offline,
    /// Main menu: party, queue, collection...
    Menus,
    /// Agent select, our agent not locked yet.
    AgentSelect,
    /// Our agent is locked; the menu has handed the screen to the game.
    Locked,
    /// The match is being played.
    InMatch,
}

/// Which scene the menu should show.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum FlowScene {
    Home,
    Play,
    AgentSelect,
    Career,
}

/// Something that happened.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlowInput {
    PhaseChanged(Phase),
    /// The pregame roster shows our own agent as locked.
    OwnAgentLocked,
    /// The user showed (`true`) or hid (`false`) the menu by hand, at `at_s`.
    ManualToggle {
        is_visible: bool,
        at_s: f64,
    },
    /// Clock tick, so a pending decision can be re-evaluated after the override expires.
    Tick {
        now_s: f64,
    },
}

/// Something to do about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", content = "scene", rename_all = "snake_case")]
#[ts(export)]
pub enum FlowEffect {
    ShowMenu,
    HideMenu,
    FocusGame,
    MinimizeGame,
    EnterScene(FlowScene),
}

/// The user's match-flow preferences (mirrors `store::settings::MatchFlowSettings`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlowRules {
    pub should_hide_on_lock: bool,
    pub should_hide_on_match_start: bool,
    pub should_return_in_menus: bool,
    pub should_minimize_game_in_menus: bool,
}

/// Machine state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlowState {
    pub stage: FlowStage,
    /// Time of the last manual toggle, if any.
    pub manual_toggle_at_s: Option<f64>,
    /// Latest known time; advanced by every timed input.
    pub now_s: f64,
}

impl Default for FlowState {
    fn default() -> Self {
        Self {
            stage: FlowStage::Offline,
            manual_toggle_at_s: None,
            now_s: 0.0,
        }
    }
}

impl FlowState {
    fn is_overridden(&self) -> bool {
        self.manual_toggle_at_s
            .is_some_and(|at| self.now_s - at < MANUAL_OVERRIDE_S)
    }
}

/// Advance the machine by one input.
pub fn step(state: FlowState, input: FlowInput, rules: &FlowRules) -> (FlowState, Vec<FlowEffect>) {
    match input {
        FlowInput::PhaseChanged(phase) => on_phase(state, phase, rules),
        FlowInput::OwnAgentLocked => on_locked(state, rules),
        FlowInput::ManualToggle {
            is_visible: _,
            at_s,
        } => {
            let next = FlowState {
                manual_toggle_at_s: Some(at_s),
                now_s: at_s,
                ..state
            };
            (next, Vec::new())
        }
        FlowInput::Tick { now_s } => (FlowState { now_s, ..state }, Vec::new()),
    }
}

fn on_phase(state: FlowState, phase: Phase, rules: &FlowRules) -> (FlowState, Vec<FlowEffect>) {
    let stage = match phase {
        Phase::Offline => FlowStage::Offline,
        Phase::Menus => FlowStage::Menus,
        Phase::Pregame if state.stage == FlowStage::Locked => FlowStage::Locked,
        Phase::Pregame => FlowStage::AgentSelect,
        Phase::Ingame => FlowStage::InMatch,
    };
    if stage == state.stage {
        return (state, Vec::new());
    }
    let effects = if state.is_overridden() {
        Vec::new()
    } else {
        transition_effects(state.stage, stage, rules)
    };
    (FlowState { stage, ..state }, effects)
}

fn on_locked(state: FlowState, rules: &FlowRules) -> (FlowState, Vec<FlowEffect>) {
    if state.stage != FlowStage::AgentSelect {
        return (state, Vec::new());
    }
    let next = FlowState {
        stage: FlowStage::Locked,
        ..state
    };
    if state.is_overridden() || !rules.should_hide_on_lock {
        return (next, Vec::new());
    }
    (next, vec![FlowEffect::HideMenu, FlowEffect::FocusGame])
}

fn transition_effects(from: FlowStage, to: FlowStage, rules: &FlowRules) -> Vec<FlowEffect> {
    match (from, to) {
        (_, FlowStage::AgentSelect) => vec![
            FlowEffect::EnterScene(FlowScene::AgentSelect),
            FlowEffect::ShowMenu,
        ],
        // Also from Locked: hide-on-lock may be off, and hiding twice is harmless.
        (FlowStage::AgentSelect | FlowStage::Locked, FlowStage::InMatch)
            if rules.should_hide_on_match_start =>
        {
            vec![FlowEffect::HideMenu, FlowEffect::FocusGame]
        }
        (FlowStage::Locked | FlowStage::InMatch, FlowStage::Menus) => back_to_menus(from, rules),
        // A dodge or remake: back to the play scene without touching the game window.
        (FlowStage::AgentSelect, FlowStage::Menus) => vec![FlowEffect::EnterScene(FlowScene::Play)],
        _ => Vec::new(),
    }
}

fn back_to_menus(from: FlowStage, rules: &FlowRules) -> Vec<FlowEffect> {
    if !rules.should_return_in_menus {
        return Vec::new();
    }
    // After a played match the career scene shows the result; after a dodge, home.
    let scene = if from == FlowStage::InMatch {
        FlowScene::Career
    } else {
        FlowScene::Home
    };
    let mut effects = vec![FlowEffect::EnterScene(scene), FlowEffect::ShowMenu];
    if rules.should_minimize_game_in_menus {
        effects.push(FlowEffect::MinimizeGame);
    }
    effects
}

#[cfg(test)]
mod tests;
