use super::*;

const RULES: FlowRules = FlowRules {
    should_hide_on_lock: true,
    should_hide_on_match_start: true,
    should_return_in_menus: true,
    should_minimize_game_in_menus: false,
};

fn at(stage: FlowStage) -> FlowState {
    FlowState {
        stage,
        ..FlowState::default()
    }
}

fn run(state: FlowState, input: FlowInput) -> (FlowState, Vec<FlowEffect>) {
    step(state, input, &RULES)
}

#[test]
fn match_found_shows_the_agent_select_scene() {
    let (state, effects) = run(
        at(FlowStage::Menus),
        FlowInput::PhaseChanged(Phase::Pregame),
    );
    assert_eq!(state.stage, FlowStage::AgentSelect);
    assert_eq!(
        effects,
        vec![
            FlowEffect::EnterScene(FlowScene::AgentSelect),
            FlowEffect::ShowMenu
        ]
    );
}

#[test]
fn locking_hands_the_screen_to_the_game() {
    let (state, effects) = run(at(FlowStage::AgentSelect), FlowInput::OwnAgentLocked);
    assert_eq!(state.stage, FlowStage::Locked);
    assert_eq!(effects, vec![FlowEffect::HideMenu, FlowEffect::FocusGame]);
}

#[test]
fn repeated_pregame_events_after_locking_change_nothing() {
    let (state, effects) = run(
        at(FlowStage::Locked),
        FlowInput::PhaseChanged(Phase::Pregame),
    );
    assert_eq!(state.stage, FlowStage::Locked);
    assert!(effects.is_empty());
}

#[test]
fn match_start_hides_even_when_hide_on_lock_is_off() {
    let rules = FlowRules {
        should_hide_on_lock: false,
        ..RULES
    };
    let (locked, lock_effects) = step(
        at(FlowStage::AgentSelect),
        FlowInput::OwnAgentLocked,
        &rules,
    );
    let (_, start_effects) = step(locked, FlowInput::PhaseChanged(Phase::Ingame), &rules);
    assert!(lock_effects.is_empty());
    assert_eq!(
        start_effects,
        vec![FlowEffect::HideMenu, FlowEffect::FocusGame]
    );
}

#[test]
fn match_end_returns_to_career() {
    let (state, effects) = run(
        at(FlowStage::InMatch),
        FlowInput::PhaseChanged(Phase::Menus),
    );
    assert_eq!(state.stage, FlowStage::Menus);
    assert_eq!(
        effects,
        vec![
            FlowEffect::EnterScene(FlowScene::Career),
            FlowEffect::ShowMenu
        ]
    );
}

#[test]
fn dodge_before_locking_goes_back_to_play_without_touching_the_game() {
    let (_, effects) = run(
        at(FlowStage::AgentSelect),
        FlowInput::PhaseChanged(Phase::Menus),
    );
    assert_eq!(effects, vec![FlowEffect::EnterScene(FlowScene::Play)]);
}

#[test]
fn dodge_after_locking_brings_the_menu_back_home() {
    let (_, effects) = run(at(FlowStage::Locked), FlowInput::PhaseChanged(Phase::Menus));
    assert_eq!(
        effects,
        vec![
            FlowEffect::EnterScene(FlowScene::Home),
            FlowEffect::ShowMenu
        ]
    );
}

#[test]
fn minimise_game_in_menus_is_opt_in() {
    let rules = FlowRules {
        should_minimize_game_in_menus: true,
        ..RULES
    };
    let (_, effects) = step(
        at(FlowStage::InMatch),
        FlowInput::PhaseChanged(Phase::Menus),
        &rules,
    );
    assert_eq!(effects.last(), Some(&FlowEffect::MinimizeGame));
}

#[test]
fn manual_toggle_suppresses_automation_for_ten_seconds() {
    let toggled = run(
        at(FlowStage::AgentSelect),
        FlowInput::ManualToggle {
            is_visible: true,
            at_s: 100.0,
        },
    )
    .0;

    let (_, during) = run(toggled, FlowInput::OwnAgentLocked);
    let later = run(
        toggled,
        FlowInput::Tick {
            now_s: 100.0 + MANUAL_OVERRIDE_S,
        },
    )
    .0;
    let (_, after) = run(later, FlowInput::OwnAgentLocked);

    assert!(during.is_empty());
    assert_eq!(after, vec![FlowEffect::HideMenu, FlowEffect::FocusGame]);
}

#[test]
fn lock_outside_agent_select_is_ignored() {
    let (state, effects) = run(at(FlowStage::Menus), FlowInput::OwnAgentLocked);
    assert_eq!(state.stage, FlowStage::Menus);
    assert!(effects.is_empty());
}

#[test]
fn disconnect_goes_offline_quietly() {
    let (state, effects) = run(
        at(FlowStage::Menus),
        FlowInput::PhaseChanged(Phase::Offline),
    );
    assert_eq!(state.stage, FlowStage::Offline);
    assert!(effects.is_empty());
}
