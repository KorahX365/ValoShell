use serde_json::json;

use super::*;

const ACT: &str = "act-now";

fn mmr() -> Value {
    json!({"QueueSkills": {
        "competitive": {"SeasonalInfoBySeasonID": {
            "act-now": {"CompetitiveTier": 14, "RankedRating": 45, "NumberOfGames": 20,
                        "NumberOfWins": 11, "GamesNeededForRating": 0, "LeaderboardRank": 0,
                        "Rank": 14, "WinsByTier": {"13": 5, "14": 6}},
            "act-old": {"CompetitiveTier": 21, "Rank": 20, "WinsByTier": {"21": 2, "22": 0}},
        }},
        "unrated": {"SeasonalInfoBySeasonID": {"act-now": {"NumberOfGames": 40, "NumberOfWins": 10}}},
        "swiftplay": {"SeasonalInfoBySeasonID": {"act-now": {"NumberOfGames": 12, "NumberOfWins": 6}}},
    }})
}

fn inputs<'a>(mmr: &'a Value, updates: Option<&'a Value>) -> RankerInputs<'a> {
    RankerInputs {
        mmr,
        season_id: ACT,
        competitive_updates: updates,
        live_badge: None,
    }
}

#[test]
fn peak_is_highest_tier_with_wins_or_badge_across_seasons() {
    assert_eq!(peak_tier_from_mmr(&mmr()), 21);
}

#[test]
fn peak_ignores_zero_win_tiers_and_applies_rank_unconditionally() {
    let only_badge = json!({"QueueSkills": {"competitive": {"SeasonalInfoBySeasonID": {
        "s": {"WinsByTier": {"24": 0}, "Rank": 17}}}}});
    assert_eq!(peak_tier_from_mmr(&only_badge), 17);
}

#[test]
fn peak_is_zero_without_data_or_above_radiant() {
    assert_eq!(peak_tier_from_mmr(&json!({})), 0);
    assert_eq!(peak_tier_from_mmr(&json!(null)), 0);
    let absurd = json!({"QueueSkills": {"competitive": {"SeasonalInfoBySeasonID": {
        "s": {"WinsByTier": {"99": 3}}}}}});
    assert_eq!(peak_tier_from_mmr(&absurd), 0);
}

#[test]
fn peak_tolerates_junk_tiers_and_wins() {
    let junk = json!({"QueueSkills": {"competitive": {"SeasonalInfoBySeasonID": {
        "s": {"WinsByTier": {"x": 3, "12": "2", "13": null, "14": "many"}}, "t": 5}}}});
    assert_eq!(peak_tier_from_mmr(&junk), 12);
}

#[test]
fn row_reads_the_act_block() {
    let data = mmr();
    let row = derive_ranker_row(&inputs(&data, None));
    assert_eq!(
        (row.current_tier, row.rank_rating, row.games_played),
        (14, 45, 20)
    );
    assert_eq!(row.win_rate, Some(55.0));
    assert_eq!(
        (row.alt_win_rate, row.alt_win_rate_queue.as_str()),
        (None, "")
    );
    assert!(!row.is_placement);
    assert_eq!(row.peak_tier, 21);
}

#[test]
fn current_tier_falls_back_to_the_newest_update_then_any_season() {
    let no_act = json!({"QueueSkills": {"competitive": {"SeasonalInfoBySeasonID": {
        "act-now": {"CompetitiveTier": 0}, "act-old": {"CompetitiveTier": 9}}}}});
    let updates = json!({"Matches": [{"TierAfterUpdate": 16}, {"TierAfterUpdate": 3}]});
    assert_eq!(current_tier(&inputs(&no_act, Some(&updates))), 16);
    assert_eq!(current_tier(&inputs(&no_act, None)), 9);
    assert_eq!(current_tier(&inputs(&json!({}), None)), 0);
}

#[test]
fn win_rate_is_none_without_games_and_zero_is_real() {
    assert_eq!(
        win_rate_from(&json!({"NumberOfGames": 0, "NumberOfWins": 0})),
        None
    );
    assert_eq!(win_rate_from(&json!({})), None);
    assert_eq!(
        win_rate_from(&json!({"NumberOfGames": 7, "NumberOfWins": 0})),
        Some(0.0)
    );
    assert_eq!(
        win_rate_from(&json!({"NumberOfGames": 3, "NumberOfWins": 1})),
        Some(33.3)
    );
}

#[test]
fn alt_win_rate_uses_the_most_played_queue_when_unranked() {
    let data = json!({"QueueSkills": {
        "competitive": {"SeasonalInfoBySeasonID": {}},
        "unrated": {"SeasonalInfoBySeasonID": {"act-now": {"NumberOfGames": 40, "NumberOfWins": 10}}},
        "swiftplay": {"SeasonalInfoBySeasonID": {"act-now": {"NumberOfGames": 12, "NumberOfWins": 6}}}}});
    let row = derive_ranker_row(&inputs(&data, None));
    assert_eq!(row.win_rate, None);
    assert_eq!(
        (row.alt_win_rate, row.alt_win_rate_queue.as_str()),
        (Some(25.0), "unrated")
    );
}

#[test]
fn placement_and_leaderboard_come_from_the_right_sources() {
    let data = json!({"QueueSkills": {"competitive": {"SeasonalInfoBySeasonID": {
        "act-now": {"GamesNeededForRating": 3, "LeaderboardRank": 120}}}}});
    let badge = json!({"LeaderboardRank": 42, "NumberOfWins": 9});
    let with_badge = RankerInputs {
        live_badge: Some(&badge),
        ..inputs(&data, None)
    };
    let row = derive_ranker_row(&with_badge);
    assert!(row.is_placement);
    assert_eq!((row.leaderboard_rank, row.act_wins), (42, 9));
    assert_eq!(
        derive_ranker_row(&inputs(&data, None)).leaderboard_rank,
        120
    );
}

#[test]
fn missing_everything_is_an_unranked_empty_row() {
    let row = derive_ranker_row(&inputs(&json!(null), None));
    assert_eq!(
        (row.current_tier, row.peak_tier, row.rank_rating),
        (0, 0, 0)
    );
    assert_eq!(row.win_rate, None);
}

#[test]
fn combat_summary_rounds_to_one_decimal_and_floors_deaths() {
    let matches = [
        MatchCombat {
            kills: 20,
            deaths: 10,
            headshots: 10,
            shots: 60,
        },
        MatchCombat {
            kills: 13,
            deaths: 14,
            headshots: 5,
            shots: 40,
        },
    ];
    let stats = summarise_combat(&matches).unwrap();
    assert_eq!(stats.kills_per_death, 1.4);
    assert_eq!(stats.kills_per_match, 16.5);
    assert_eq!(stats.head_shot, 15.0);
    let deathless = [MatchCombat {
        kills: 7,
        deaths: 0,
        headshots: 0,
        shots: 0,
    }];
    assert_eq!(summarise_combat(&deathless).unwrap().kills_per_death, 7.0);
    assert_eq!(summarise_combat(&[]), None);
}

#[test]
fn combat_from_detail_sums_headshots_from_round_damage() {
    let detail = json!({"players": [{"subject": "p", "stats": {"kills": 12, "deaths": 6}}],
        "roundResults": [{"playerStats": [{"subject": "p", "damage": [
            {"headshots": 1, "bodyshots": 2, "legshots": 1}]}]},
            {"playerStats": [{"subject": "p", "damage": [{"headshots": 0, "bodyshots": 4, "legshots": 0}]}]}]});
    let combat = match_combat_from_detail(&detail, "p").unwrap();
    assert_eq!(
        combat,
        MatchCombat {
            kills: 12,
            deaths: 6,
            headshots: 1,
            shots: 8
        }
    );
    assert_eq!(match_combat_from_detail(&detail, "other"), None);
    assert_eq!(
        match_combat_from_detail(&json!({"players": [{"subject": "p"}]}), "p"),
        None
    );
}
