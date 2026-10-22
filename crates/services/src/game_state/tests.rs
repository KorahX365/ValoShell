use domain::phase::Side;

use super::*;

fn presence(state: LoopState, scores: Option<(u32, u32)>) -> OwnPresence {
    OwnPresence {
        loop_state: Some(state),
        party_size: Some(2),
        party_id: None,
        live_scores: scores.map(|(ally_score, enemy_score)| LiveScores {
            ally_score,
            enemy_score,
            current_team: "Unknown".to_owned(),
            map_name: "Bind".to_owned(),
            queue: "competitive".to_owned(),
        }),
    }
}

#[test]
fn attach_starts_in_menus() {
    let mut tracker = GameTracker::default();
    assert_eq!(
        tracker.attach().map(|snapshot| snapshot.phase),
        Some(Phase::Menus)
    );
}

#[test]
fn ingame_presence_reports_score_map_queue_and_side() {
    let mut tracker = GameTracker::default();
    tracker.attach();
    tracker.set_team("Red");

    let snapshot = tracker
        .apply_presence(&presence(LoopState::Ingame, Some((3, 1))))
        .expect("changed");

    assert_eq!(
        (snapshot.ally_score, snapshot.enemy_score),
        (Some(3), Some(1))
    );
    assert_eq!(snapshot.map_name, "Bind");
    assert_eq!(snapshot.side, Some(Side::Attack));
    assert_eq!(snapshot.party_size, 2);
}

#[test]
fn unchanged_presence_is_not_republished() {
    let mut tracker = GameTracker::default();
    tracker.apply_presence(&presence(LoopState::Ingame, Some((3, 1))));
    assert!(
        tracker
            .apply_presence(&presence(LoopState::Ingame, Some((3, 1))))
            .is_none()
    );
}

#[test]
fn returning_to_menus_clears_match_facts_and_the_queue() {
    let mut tracker = GameTracker::default();
    tracker.set_team("Blue");
    tracker.apply_presence(&presence(LoopState::Ingame, Some((13, 9))));

    let snapshot = tracker
        .apply_presence(&presence(LoopState::Menus, None))
        .expect("changed");

    assert_eq!(snapshot.phase, Phase::Menus);
    assert_eq!(snapshot.map_name, "");
    assert_eq!(snapshot.queue, "");
    assert_eq!(snapshot.side, None);
    assert_eq!(snapshot.ally_score, None);
}

#[test]
fn a_gap_in_the_loop_state_keeps_the_phase() {
    let mut tracker = GameTracker::default();
    tracker.apply_presence(&presence(LoopState::Pregame, None));
    let gap = OwnPresence {
        loop_state: None,
        ..presence(LoopState::Pregame, None)
    };
    tracker.apply_presence(&gap);
    assert_eq!(tracker.snapshot().phase, Phase::Pregame);
}

#[test]
fn detach_goes_offline() {
    let mut tracker = GameTracker::default();
    tracker.attach();
    assert_eq!(
        tracker.detach().map(|snapshot| snapshot.phase),
        Some(Phase::Offline)
    );
}
