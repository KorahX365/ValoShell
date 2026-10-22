use super::*;

#[test]
fn starting_side_maps_red_and_blue_only() {
    assert_eq!(starting_side("Red"), Some(Side::Attack));
    assert_eq!(starting_side("Blue"), Some(Side::Defence));
    assert_eq!(starting_side("red"), None);
    assert_eq!(starting_side(""), None);
}

#[test]
fn current_side_swaps_after_twelve_rounds_in_a_13_round_mode() {
    assert_eq!(current_side("Red", 11, 13), Some(Side::Attack));
    assert_eq!(current_side("Red", 12, 13), Some(Side::Defence));
    assert_eq!(current_side("Blue", 12, 13), Some(Side::Attack));
}

#[test]
fn current_side_is_unknown_in_overtime() {
    assert_eq!(current_side("Red", 24, 13), None);
    assert_eq!(current_side("Red", 30, 13), None);
}

#[test]
fn current_side_swiftplay_swaps_after_four_rounds() {
    assert_eq!(current_side("Red", 3, 5), Some(Side::Attack));
    assert_eq!(current_side("Red", 4, 5), Some(Side::Defence));
    assert_eq!(current_side("Red", 8, 5), None);
}

#[test]
fn current_side_clamps_half_to_one_round() {
    assert_eq!(current_side("Red", 0, 1), Some(Side::Attack));
    assert_eq!(current_side("Red", 1, 1), Some(Side::Defence));
    assert_eq!(current_side("Red", 2, 0), None);
}

#[test]
fn current_side_is_unknown_for_unknown_team() {
    assert_eq!(current_side("Green", 0, 13), None);
}

#[test]
fn side_from_snapshot_without_scores_is_starting_side() {
    assert_eq!(
        side_from_snapshot("Red", None, None, "competitive"),
        Some(Side::Attack)
    );
    assert_eq!(
        side_from_snapshot("Blue", Some(1), None, ""),
        Some(Side::Defence)
    );
}

#[test]
fn side_from_snapshot_uses_total_rounds_played() {
    assert_eq!(
        side_from_snapshot("Red", Some(7), Some(5), "competitive"),
        Some(Side::Defence)
    );
    assert_eq!(
        side_from_snapshot("Red", Some(6), Some(5), "competitive"),
        Some(Side::Attack)
    );
}

#[test]
fn side_from_snapshot_is_unknown_for_kill_races_and_missing_team() {
    assert_eq!(side_from_snapshot("Red", None, None, "deathmatch"), None);
    assert_eq!(side_from_snapshot("Red", None, None, "hurm"), None);
    assert_eq!(side_from_snapshot("", None, None, "competitive"), None);
}

#[test]
fn side_from_snapshot_unknown_queue_uses_standard_rules() {
    assert_eq!(
        side_from_snapshot("Red", Some(12), Some(0), "mystery"),
        Some(Side::Defence)
    );
}

#[test]
fn side_from_snapshot_retakes_answers_with_five_round_halves() {
    assert_eq!(
        side_from_snapshot("Red", Some(2), Some(2), "fortcollins"),
        Some(Side::Defence)
    );
}

#[test]
fn snapshot_default_is_offline_party_of_one() {
    let snapshot = GameSnapshot::default();
    assert_eq!(snapshot.phase, Phase::Offline);
    assert_eq!(snapshot.party_size, 1);
    assert!(!snapshot.is_live());
    assert!(!snapshot.has_score());
    assert_eq!(snapshot.side_label(), "");
}

#[test]
fn snapshot_is_live_only_in_pregame_and_ingame() {
    let live = |phase| {
        GameSnapshot {
            phase,
            ..GameSnapshot::default()
        }
        .is_live()
    };
    assert!(live(Phase::Pregame));
    assert!(live(Phase::Ingame));
    assert!(!live(Phase::Menus));
    assert!(!live(Phase::Offline));
}

#[test]
fn snapshot_has_score_needs_both_scores() {
    let mut snapshot = GameSnapshot {
        ally_score: Some(0),
        ..GameSnapshot::default()
    };
    assert!(!snapshot.has_score());
    snapshot.enemy_score = Some(0);
    assert!(snapshot.has_score());
}

#[test]
fn snapshot_side_label_is_atk_or_def() {
    let with_side = |side| {
        GameSnapshot {
            side,
            ..GameSnapshot::default()
        }
        .side_label()
    };
    assert_eq!(with_side(Some(Side::Attack)), "ATK");
    assert_eq!(with_side(Some(Side::Defence)), "DEF");
    assert_eq!(with_side(None), "");
}

#[test]
fn snapshot_describe_joins_queue_and_party() {
    let snapshot = GameSnapshot {
        queue: "competitive".to_owned(),
        party_size: 3,
        ..GameSnapshot::default()
    };
    assert_eq!(snapshot.describe(), "Competitive · party of 3");
}

#[test]
fn snapshot_describe_omits_empty_parts() {
    let queue_only = GameSnapshot {
        queue: "spikerush".to_owned(),
        ..GameSnapshot::default()
    };
    let party_only = GameSnapshot {
        party_size: 2,
        ..GameSnapshot::default()
    };
    assert_eq!(queue_only.describe(), "Spikerush");
    assert_eq!(party_only.describe(), "party of 2");
    assert_eq!(GameSnapshot::default().describe(), "");
}
