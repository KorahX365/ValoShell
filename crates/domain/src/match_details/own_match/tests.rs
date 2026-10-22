use serde_json::json;

use super::*;

fn player(subject: &str, team: &str, party: &str) -> Value {
    json!({"subject": subject, "teamId": team, "partyId": party, "characterId": "agent-1",
           "stats": {"kills": 20, "deaths": 10, "assists": 5, "score": 5400, "roundsPlayed": 24},
           "behaviorFactors": {"afkRounds": 2.0}})
}

fn finished_match() -> Value {
    json!({
        "matchInfo": {"matchId": "m1", "mapId": "/Game/Maps/Ascent/Ascent",
                      "gameStartMillis": 5_000_000, "queueID": "competitive"},
        "players": [player("me", "Red", "p"), player("duo", "Red", "p"),
                    player("solo", "Red", ""), player("foe", "Blue", "q")],
        "teams": [{"teamId": "Red", "won": true, "roundsWon": 13},
                  {"teamId": "Blue", "won": false, "roundsWon": 11}],
        "roundResults": [
            {"roundNum": 0, "winningTeam": "Red", "playerStats": [
                {"subject": "me", "damage": [{"headshots": 2, "bodyshots": 3, "legshots": 1, "damage": 400}],
                 "kills": [{"roundTime": 8000, "killer": "me", "victim": "foe"}]},
                {"subject": "foe", "kills": [{"roundTime": 12000, "killer": "foe", "victim": "duo"}]}]},
            {"roundNum": 1, "winningTeam": "Blue", "playerStats": [
                {"subject": "foe", "kills": [{"roundTime": 3000, "killer": "foe", "victim": "me"}]},
                {"subject": "me", "damage": [{"headshots": 1, "bodyshots": 0, "legshots": 0, "damage": 150}]}]},
        ],
    })
}

#[test]
fn own_row_maps_score_stats_and_attribution() {
    let row = extract_own_match(&finished_match(), "me", None).unwrap();
    assert_eq!(
        (row.match_id.as_str(), row.queue.as_str()),
        ("m1", "competitive")
    );
    assert_eq!((row.won, row.rounds_won, row.rounds_lost), (true, 13, 11));
    assert_eq!((row.kills, row.deaths, row.assists), (20, 10, 5));
    assert_eq!(
        (row.combat_score, row.rounds_played, row.afk_rounds),
        (5400, 24, 2)
    );
    assert_eq!(row.account_puuid.as_deref(), Some("me"));
    assert_eq!(row.agent_id, "agent-1");
    assert_eq!(row.started_at, 5000.0);
}

#[test]
fn own_row_sums_shots_and_damage_for_me_only() {
    let row = extract_own_match(&finished_match(), "me", None).unwrap();
    assert_eq!(
        (row.headshots, row.bodyshots, row.legshots, row.damage),
        (3, 3, 1, 550)
    );
}

#[test]
fn opening_duels_use_the_earliest_kill_of_every_round() {
    let row = extract_own_match(&finished_match(), "me", None).unwrap();
    assert_eq!((row.first_bloods, row.first_deaths), (1, 1));
}

#[test]
fn party_size_counts_players_sharing_my_party() {
    assert_eq!(
        extract_own_match(&finished_match(), "me", None)
            .unwrap()
            .party_size,
        2
    );
    assert_eq!(
        extract_own_match(&finished_match(), "solo", None)
            .unwrap()
            .party_size,
        1
    );
    assert_eq!(
        extract_own_match(&finished_match(), "me", Some(5))
            .unwrap()
            .party_size,
        5
    );
    assert_eq!(
        extract_own_match(&finished_match(), "me", Some(0))
            .unwrap()
            .party_size,
        2
    );
}

#[test]
fn starting_side_comes_from_the_team_convention() {
    let row = extract_own_match(&finished_match(), "me", None).unwrap();
    assert_eq!(row.started_side, Some(StartingSide::Attack));
    assert_eq!((row.attack_played, row.attack_won), (2, 1));
}

#[test]
fn unknown_queue_is_the_unknown_placeholder() {
    let mut detail = finished_match();
    detail["matchInfo"]["queueID"] = json!("");
    assert_eq!(
        extract_own_match(&detail, "me", None).unwrap().queue,
        "unknown"
    );
}

#[test]
fn a_match_without_me_or_an_empty_puuid_yields_nothing() {
    assert_eq!(extract_own_match(&finished_match(), "ghost", None), None);
    assert_eq!(extract_own_match(&finished_match(), "", None), None);
    assert_eq!(extract_own_match(&json!([]), "me", None), None);
}

#[test]
fn a_draw_is_neither_won_nor_lost_by_flag() {
    let mut detail = finished_match();
    detail["teams"] = json!([{"teamId": "Red", "won": false, "roundsWon": 12},
                             {"teamId": "Blue", "won": false, "roundsWon": 12}]);
    let row = extract_own_match(&detail, "me", None).unwrap();
    assert_eq!((row.won, row.rounds_won, row.rounds_lost), (false, 12, 12));
}
