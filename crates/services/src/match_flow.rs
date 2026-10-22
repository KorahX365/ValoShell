//! Executes `domain::match_flow`: feeds it phase changes, lock confirmations and manual toggles,
//! and applies its effects through a `WindowPort` (real windows in the app, a fake in tests).

use std::time::Instant;

use domain::match_flow::{FlowEffect, FlowInput, FlowRules, FlowState, step};
use store::settings::MatchFlowSettings;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::ctx::Ctx;
use crate::event::AppEvent;

/// What the flow can do to windows. Implemented by the app shell.
pub trait WindowPort: Send + 'static {
    fn show_menu(&self);
    fn hide_menu(&self);
    fn focus_game(&self);
    fn minimize_game(&self);
}

/// Signals other services send to the flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowSignal {
    OwnAgentLocked,
    ManualToggle { is_visible: bool },
}

fn rules_from(settings: &MatchFlowSettings) -> FlowRules {
    FlowRules {
        should_hide_on_lock: settings.should_hide_on_lock,
        should_hide_on_match_start: settings.should_hide_on_match_start,
        should_return_in_menus: settings.should_return_in_menus,
        should_minimize_game_in_menus: settings.should_minimize_game_in_menus,
    }
}

/// Run until `stop` fires. `signals` comes from `Ctx::flow_signals`.
pub async fn run(
    ctx: Ctx,
    windows: impl WindowPort,
    mut signals: mpsc::Receiver<FlowSignal>,
    stop: CancellationToken,
) {
    let started = Instant::now();
    let mut game = ctx.game();
    let mut state = FlowState::default();
    loop {
        let input = tokio::select! {
            () = stop.cancelled() => return,
            changed = game.changed() => match changed {
                Ok(()) => FlowInput::PhaseChanged(game.borrow_and_update().phase),
                Err(_) => return,
            },
            signal = signals.recv() => match signal {
                Some(signal) => to_input(signal, started.elapsed().as_secs_f64()),
                None => return,
            },
        };
        let now_s = started.elapsed().as_secs_f64();
        let rules = rules_from(&ctx.settings.get().match_flow);
        (state, _) = step(state, FlowInput::Tick { now_s }, &rules);
        let effects;
        (state, effects) = step(state, input, &rules);
        for effect in effects {
            apply(&windows, effect);
            ctx.bus.publish(AppEvent::MatchFlow(effect));
        }
    }
}

fn to_input(signal: FlowSignal, now_s: f64) -> FlowInput {
    match signal {
        FlowSignal::OwnAgentLocked => FlowInput::OwnAgentLocked,
        FlowSignal::ManualToggle { is_visible } => FlowInput::ManualToggle {
            is_visible,
            at_s: now_s,
        },
    }
}

/// Scene changes are UI-only; the bus event carries them.
fn apply(windows: &impl WindowPort, effect: FlowEffect) {
    match effect {
        FlowEffect::ShowMenu => windows.show_menu(),
        FlowEffect::HideMenu => windows.hide_menu(),
        FlowEffect::FocusGame => windows.focus_game(),
        FlowEffect::MinimizeGame => windows.minimize_game(),
        FlowEffect::EnterScene(_) => {}
    }
}
