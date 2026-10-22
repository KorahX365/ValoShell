use std::collections::{HashMap, HashSet};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};

use super::*;

fn encode(value: &Value) -> String {
    STANDARD.encode(value.to_string())
}

fn nested_ingame_blob() -> Value {
    json!({
        "isValid": true,
        "partySize": 9,
        "matchPresenceData": {
            "sessionLoopState": "INGAME",
            "matchMap": "/Game/Maps/Ascent/Ascent",
            "queueId": " competitive "
        },
        "partyPresenceData": {
            "partyId": "PARTY-1",
            "partySize": 3,
            "partyOwnerMatchMap": "/Game/Maps/Duality/Duality",
            "partyOwnerMatchScoreAllyTeam": 7,
            "partyOwnerMatchScoreEnemyTeam": 5
        }
    })
}

#[test]
fn decode_private_reads_base64_json_objects() {
    let blob = encode(&json!({"a": 1}));
    assert_eq!(decode_private(&blob).get("a"), Some(&json!(1)));
}

#[test]
fn decode_private_tolerates_junk_from_other_games() {
    assert!(decode_private("not base64 at all!").is_empty());
    assert!(decode_private(&STANDARD.encode("[1,2]")).is_empty());
    assert!(decode_private("").is_empty());
}

#[test]
fn nested_blocks_win_over_the_top_level() {
    let private = decode_private(&encode(&nested_ingame_blob()));
    // partySize: the match block has none, the party block has 3, the stale top level has 9.
    assert_eq!(presence_field(&private, "partySize"), Some(&json!(3)));
}

#[test]
fn own_presence_in_match_reports_scores_map_and_queue() {
    let private = decode_private(&encode(&nested_ingame_blob()));

    let presence = read_own_presence(&private);

    assert_eq!(presence.loop_state, Some(LoopState::Ingame));
    assert_eq!(presence.party_size, Some(3));
    assert_eq!(presence.party_id, Some(PartyId::new("party-1")));
    let scores = presence.live_scores.expect("in match");
    assert_eq!((scores.ally_score, scores.enemy_score), (7, 5));
    assert_eq!(scores.map_name, "Bind");
    assert_eq!(scores.queue, "competitive");
    assert_eq!(scores.current_team, "Unknown");
}

#[test]
fn live_scores_are_dropped_in_menus() {
    let mut blob = nested_ingame_blob();
    blob["matchPresenceData"]["sessionLoopState"] = json!("MENUS");
    let presence = read_own_presence(&decode_private(&encode(&blob)));
    assert_eq!(presence.loop_state, Some(LoopState::Menus));
    assert_eq!(presence.live_scores, None);
}

#[test]
fn live_scores_wait_until_presence_names_something() {
    let private = decode_private(&encode(&json!({"sessionLoopState": "PREGAME"})));
    assert_eq!(read_own_presence(&private).live_scores, None);
}

#[test]
fn old_flat_shape_is_still_read() {
    let blob = json!({
        "sessionLoopState": "INGAME",
        "partyOwnerMatchScoreAllyTeam": 2,
        "partyOwnerMatchScoreEnemyTeam": 0,
        "partyOwnerMatchCurrentTeam": "Red",
        "partyOwnerMatchMap": "Triad",
    });
    let scores = read_own_presence(&decode_private(&encode(&blob)))
        .live_scores
        .expect("in match");
    assert_eq!(scores.current_team, "Red");
    assert_eq!(scores.map_name, "Haven");
}

#[test]
fn zero_or_missing_party_size_is_ignored() {
    let private = decode_private(&encode(&json!({"partySize": 0})));
    assert_eq!(read_own_presence(&private).party_size, None);
}

fn presence_entry(puuid: &str, party_id: &str, party_size: i64) -> Value {
    let private = json!({"partyPresenceData": {"partyId": party_id, "partySize": party_size}});
    json!({"puuid": puuid, "private": encode(&private)})
}

#[test]
fn parse_parties_skips_entries_without_a_party_and_respects_the_roster() {
    let presences = vec![
        presence_entry("a", "p1", 2),
        presence_entry("b", "", 1),
        json!({"puuid": "c", "private": "junk"}),
        presence_entry("outsider", "p9", 4),
    ];
    let roster: HashSet<Puuid> = ["a", "b", "c"].into_iter().map(Puuid::new).collect();

    let parties = parse_parties(&presences, Some(&roster));

    assert_eq!(parties.len(), 1);
    assert!(parties[&Puuid::new("a")].is_stack());
}

#[test]
fn stack_sizes_count_only_members_inside_the_match() {
    let parties: HashMap<Puuid, PartyInfo> = [("a", "p1", 5), ("b", "p1", 5), ("c", "p2", 1)]
        .into_iter()
        .map(|(puuid, party, size)| {
            (
                Puuid::new(puuid),
                PartyInfo {
                    party_id: PartyId::new(party),
                    party_size: size,
                },
            )
        })
        .collect();
    let roster: Vec<Puuid> = ["a", "b", "c", "d"].into_iter().map(Puuid::new).collect();

    let stacks = stack_sizes(&parties, &roster);

    assert_eq!(stacks[&Puuid::new("a")], 2);
    assert_eq!(stacks[&Puuid::new("c")], 1);
    assert!(!stacks.contains_key(&Puuid::new("d")));
    assert_eq!(largest_stack(&stacks), 2);
}

#[test]
fn largest_stack_defaults_to_solo() {
    assert_eq!(largest_stack(&HashMap::new()), 1);
}
