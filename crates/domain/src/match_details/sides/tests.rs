use serde_json::json;

use super::*;

/// Rounds 0..count where Red wins the even rounds.
fn rounds(count: i64) -> Vec<Value> {
    (0..count)
        .map(|number| {
            json!({"roundNum": number,
                             "winningTeam": if number % 2 == 0 { "Red" } else { "Blue" }})
        })
        .collect()
}

fn detail_with(rounds: Vec<Value>) -> Value {
    json!({"players": [{"subject": "red-1", "teamId": "Red"},
                       {"subject": "blue-1", "teamId": "Blue"}],
           "roundResults": rounds})
}

#[test]
fn competitive_swaps_after_round_twelve() {
    let split = derive_sides(&detail_with(rounds(24)), "Red", "competitive");
    assert_eq!(split.started_side, Some(StartingSide::Attack));
    assert_eq!((split.attack_played, split.attack_won), (12, 6));
    assert_eq!((split.defence_played, split.defence_won), (12, 6));
}

#[test]
fn swiftplay_swaps_after_round_four() {
    let split = derive_sides(&detail_with(rounds(9)), "Red", "swiftplay");
    assert_eq!((split.attack_played, split.attack_won), (4, 2));
    assert_eq!((split.defence_played, split.defence_won), (5, 3));
}

#[test]
fn overtime_rounds_past_twenty_three_are_excluded() {
    let split = derive_sides(&detail_with(rounds(30)), "Red", "competitive");
    assert_eq!(split.attack_played + split.defence_played, 24);
}

#[test]
fn blue_starts_on_defence() {
    let split = derive_sides(&detail_with(rounds(24)), "Blue", "competitive");
    assert_eq!(split.started_side, Some(StartingSide::Defence));
    assert_eq!((split.defence_played, split.defence_won), (12, 6));
}

#[test]
fn retakes_and_replication_have_no_sides() {
    for queue in ["fortcollins", "onefa", "deathmatch", "hurm"] {
        assert_eq!(
            derive_sides(&detail_with(rounds(8)), "Red", queue),
            SideSplit::default()
        );
    }
}

#[test]
fn no_rounds_means_no_sides() {
    assert_eq!(
        derive_sides(&detail_with(vec![]), "Red", "competitive"),
        SideSplit::default()
    );
}

#[test]
fn an_unknown_queue_uses_competitive_rules() {
    let split = derive_sides(&detail_with(rounds(24)), "Red", "");
    assert_eq!(split.attack_played, 12);
}

#[test]
fn a_first_half_plant_by_blue_flips_the_convention() {
    let mut all = rounds(24);
    all[1]["bombPlanter"] = json!("blue-1");
    let detail = detail_with(all);
    assert_eq!(
        first_half_attacker_from_plants(&detail, 12).as_deref(),
        Some("Blue")
    );
    assert_eq!(
        derive_sides(&detail, "Red", "competitive").started_side,
        Some(StartingSide::Defence)
    );
}

#[test]
fn a_second_half_plant_is_normalised_to_the_first_half() {
    let mut all = rounds(24);
    all[13]["bombPlanter"] = json!("blue-1");
    assert_eq!(
        first_half_attacker_from_plants(&detail_with(all), 12).as_deref(),
        Some("Red")
    );
}

#[test]
fn overtime_plants_are_not_evidence() {
    let mut all = rounds(26);
    all[25]["bombPlanter"] = json!("blue-1");
    assert_eq!(first_half_attacker_from_plants(&detail_with(all), 12), None);
}
