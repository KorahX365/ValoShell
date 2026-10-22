//! MMR: rank per season/queue and competitive updates.

use std::collections::HashMap;

use domain::ids::{MatchId, Puuid};
use serde::Deserialize;

use super::history::IndexWindow;
use crate::http::{Host, Request};

/// `GET /mmr/v1/players/{puuid}`: ranks keyed by queue, then by season (act) id.
pub fn get_player_mmr(puuid: &Puuid) -> Request {
    Request::get(Host::Pd, format!("/mmr/v1/players/{puuid}"))
}

/// `GET /mmr/v1/players/{puuid}/competitiveupdates`: RR changes of recent matches.
///
/// The official client sends `startIndex=0&endIndex=15&queue=competitive`.
pub fn get_competitive_updates(puuid: &Puuid, window: IndexWindow, queue: Option<&str>) -> Request {
    let query = window.query(queue);
    Request::get(
        Host::Pd,
        format!("/mmr/v1/players/{puuid}/competitiveupdates?{query}"),
    )
}

/// The MMR profile of one player.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct MmrPlayer {
    pub version: i64,
    pub subject: Option<Puuid>,
    pub new_player_experience_finished: bool,
    #[serde(deserialize_with = "super::null_as_default")]
    pub queue_skills: HashMap<String, QueueSkill>,
    pub latest_competitive_update: Option<CompetitiveUpdate>,
    pub is_leaderboard_anonymized: bool,
    pub is_act_rank_badge_hidden: bool,
}

impl MmrPlayer {
    /// The season block of one queue (`competitive`) for one act, if the player has it.
    pub fn seasonal_info(&self, queue: &str, season_id: &str) -> Option<&SeasonalInfo> {
        self.queue_skills
            .get(queue)?
            .seasonal_info_by_season_id
            .get(season_id)
    }
}

/// Rating state of one queue.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct QueueSkill {
    pub total_games_needed_for_rating: u32,
    pub total_games_needed_for_leaderboard: u32,
    pub current_season_games_needed_for_rating: u32,
    #[serde(
        rename = "SeasonalInfoBySeasonID",
        deserialize_with = "super::null_as_default"
    )]
    pub seasonal_info_by_season_id: HashMap<String, SeasonalInfo>,
}

/// One queue's results in one season.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct SeasonalInfo {
    #[serde(rename = "SeasonID")]
    pub season_id: String,
    pub number_of_wins: u32,
    pub number_of_wins_with_placements: u32,
    pub number_of_games: u32,
    pub rank: u32,
    pub capstone_wins: u32,
    pub leaderboard_rank: u32,
    pub competitive_tier: u32,
    pub ranked_rating: u32,
    /// Wins per tier number; `null` before placements.
    #[serde(deserialize_with = "super::null_as_default")]
    pub wins_by_tier: HashMap<String, u32>,
    pub games_needed_for_rating: u32,
    pub total_wins_needed_for_rank: u32,
}

/// `competitiveupdates` response.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct CompetitiveUpdates {
    pub version: i64,
    pub subject: Option<Puuid>,
    #[serde(deserialize_with = "super::null_as_default")]
    pub matches: Vec<CompetitiveUpdate>,
}

/// RR change caused by one ranked match.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct CompetitiveUpdate {
    #[serde(rename = "MatchID")]
    pub match_id: Option<MatchId>,
    #[serde(rename = "MapID")]
    pub map_id: String,
    #[serde(rename = "SeasonID")]
    pub season_id: String,
    /// Epoch milliseconds.
    pub match_start_time: i64,
    pub tier_after_update: i32,
    pub tier_before_update: i32,
    pub ranked_rating_after_update: i32,
    pub ranked_rating_before_update: i32,
    pub ranked_rating_earned: i32,
    pub ranked_rating_performance_bonus: i32,
    pub competitive_movement: String,
    #[serde(rename = "AFKPenalty")]
    pub afk_penalty: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    const MMR: &str = include_str!("fixtures/mmr.json");
    const ACT: &str = "d816f426-48ea-f052-117f-9697a155b319";

    #[test]
    fn mmr_request_targets_the_player() {
        let request = get_player_mmr(&Puuid::new("abc"));

        assert_eq!(request.host, Host::Pd);
        assert_eq!(request.path, "/mmr/v1/players/abc");
    }

    #[test]
    fn competitive_updates_request_matches_the_client_call() {
        let request = get_competitive_updates(
            &Puuid::new("abc"),
            IndexWindow::new(0, 15),
            Some("competitive"),
        );

        assert_eq!(
            request.path,
            "/mmr/v1/players/abc/competitiveupdates?startIndex=0&endIndex=15&queue=competitive"
        );
    }

    #[test]
    fn mmr_fixture_exposes_seasonal_rank_fields() {
        let mmr: MmrPlayer = serde_json::from_str(MMR).expect("decodes");

        let info = mmr.seasonal_info("competitive", ACT).expect("act block");

        assert_eq!(info.competitive_tier, 14);
        assert_eq!(info.ranked_rating, 37);
        assert_eq!(info.number_of_games, 52);
        assert_eq!(info.number_of_wins, 28);
        assert_eq!(info.leaderboard_rank, 0);
        assert_eq!(info.wins_by_tier.get("14"), Some(&20));
    }

    #[test]
    fn mmr_fixture_keeps_other_queues_and_null_win_tiers() {
        let mmr: MmrPlayer = serde_json::from_str(MMR).expect("decodes");

        let unrated = mmr.seasonal_info("unrated", ACT).expect("unrated block");

        assert!(unrated.wins_by_tier.is_empty());
        assert!(mmr.seasonal_info("competitive", "missing").is_none());
        assert_eq!(
            mmr.latest_competitive_update
                .expect("latest")
                .ranked_rating_earned,
            -18
        );
    }

    #[test]
    fn competitive_updates_decode_matches() {
        let json = r#"{"Version":1,"Subject":"abc","Matches":[
            {"MatchID":"m-1","MapID":"/Game/Maps/Ascent/Ascent","MatchStartTime":1700000000000,
             "TierAfterUpdate":15,"TierBeforeUpdate":14,"RankedRatingAfterUpdate":4,
             "RankedRatingBeforeUpdate":95,"RankedRatingEarned":9,"AFKPenalty":0}]}"#;

        let updates: CompetitiveUpdates = serde_json::from_str(json).expect("decodes");

        let first = &updates.matches[0];
        assert_eq!(first.match_id, Some(MatchId::new("m-1")));
        assert_eq!(first.tier_after_update, 15);
        assert_eq!(first.ranked_rating_earned, 9);
        assert_eq!(first.match_start_time, 1_700_000_000_000);
    }
}
