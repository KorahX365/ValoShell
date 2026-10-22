//! Ranker row derivation from MMR and competitive-update JSON.
//!
//! NOTE for the integrator: this belongs with the ranker (`analytics/`), not with match
//! parsing; it lives here only because that is the folder this task owns.

use serde::Serialize;
use serde_json::Value;
use ts_rs::TS;

use super::json::{exact_int, is_truthy, items, lenient_int, number, number_or_zero, whole};
use crate::numeric::round_to_digits;

/// Which queue every per-player figure describes, whatever mode the lobby is in.
pub const STATS_QUEUE: &str = "competitive";

/// Newest competitive matches walked per player for combat stats; each costs a request.
pub const RECENT_MATCHES: usize = 3;

/// Highest competitive tier number (Radiant).
const MAX_TIER: u8 = 27;

/// Rank facts for one player's row in the lobby table.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct RankerRow {
    /// Competitive tier 0-27; 0 is unranked.
    pub current_tier: u8,
    /// RR within the tier.
    pub rank_rating: i32,
    pub games_played: u32,
    pub is_placement: bool,
    /// Ladder position for Immortal and Radiant; 0 when unranked there.
    pub leaderboard_rank: u32,
    /// Ranked wins this act from the live match badge; 0 when absent.
    pub act_wins: u32,
    /// Win rate this act in percent (1 decimal); `None` without ranked games, a real 0% stays 0.0.
    pub win_rate: Option<f64>,
    /// Same figure from the most-played other queue, only when `win_rate` is `None`.
    pub alt_win_rate: Option<f64>,
    pub alt_win_rate_queue: String,
    pub peak_tier: u8,
}

/// Everything the derivation reads.
#[derive(Debug, Clone, Copy)]
pub struct RankerInputs<'a> {
    /// `GET /mmr/v1/players/{puuid}` response.
    pub mmr: &'a Value,
    /// The active ACT id (not the episode).
    pub season_id: &'a str,
    /// `competitiveupdates` response, consulted only when the act block has no tier.
    pub competitive_updates: Option<&'a Value>,
    /// This player's `SeasonalBadgeInfo` from the live match.
    pub live_badge: Option<&'a Value>,
}

/// Derive the rank half of a ranker row.
pub fn derive_ranker_row(inputs: &RankerInputs<'_>) -> RankerRow {
    let info = season_info(inputs.mmr, STATS_QUEUE, inputs.season_id);
    let win_rate = win_rate_from(info);
    let (alt_win_rate, alt_win_rate_queue) = match win_rate {
        Some(_) => (None, None),
        None => alt_win_rate(inputs.mmr, inputs.season_id).unzip(),
    };
    let badge = inputs.live_badge.unwrap_or(&super::json::MISSING);
    RankerRow {
        current_tier: current_tier(inputs),
        rank_rating: to_i32(whole(info.get("RankedRating"))),
        games_played: to_u32(whole(info.get("NumberOfGames"))),
        is_placement: number(info.get("GamesNeededForRating")).is_some_and(|needed| needed > 0.0),
        leaderboard_rank: leaderboard_rank(badge, info),
        act_wins: to_u32(whole(badge.get("NumberOfWins"))),
        win_rate,
        alt_win_rate,
        alt_win_rate_queue: alt_win_rate_queue.unwrap_or_default(),
        peak_tier: peak_tier_from_mmr(inputs.mmr),
    }
}

fn to_i32(value: i64) -> i32 {
    i32::try_from(value).unwrap_or(0)
}

fn to_u32(value: i64) -> u32 {
    u32::try_from(value.max(0)).unwrap_or(0)
}

/// One queue's seasonal block for a season id; `MISSING` (always empty) when absent.
pub fn season_info<'a>(mmr: &'a Value, queue: &str, season_id: &str) -> &'a Value {
    mmr.get("QueueSkills")
        .and_then(|skills| skills.get(queue))
        .and_then(|block| block.get("SeasonalInfoBySeasonID"))
        .and_then(|seasons| seasons.get(season_id))
        .filter(|info| info.is_object())
        .unwrap_or(&super::json::MISSING)
}

fn competitive_seasons(mmr: &Value) -> Option<&serde_json::Map<String, Value>> {
    mmr.get("QueueSkills")?
        .get(STATS_QUEUE)?
        .get("SeasonalInfoBySeasonID")?
        .as_object()
}

/// A tier from a block, valid only when it is a real rank (1..=27).
fn usable_tier(raw: Option<&Value>) -> Option<u8> {
    let tier = number(raw)?;
    (1.0..=f64::from(MAX_TIER))
        .contains(&tier)
        .then_some(tier as u8)
}

/// Current rank, first usable of: this act's tier, the newest ranked match's tier-after, any
/// season's tier. 0 (unranked) when none.
///
/// The third source is the first block in key order; `serde_json` without `preserve_order`
/// sorts keys, so the order Riot sent them in is not kept.
pub fn current_tier(inputs: &RankerInputs<'_>) -> u8 {
    let seasons = competitive_seasons(inputs.mmr);
    let from_act = seasons
        .and_then(|blocks| blocks.get(inputs.season_id))
        .and_then(|block| usable_tier(block.get("CompetitiveTier")));
    let from_updates = || {
        let newest = items(inputs.competitive_updates?, &["Matches"]).first()?;
        usable_tier(newest.get("TierAfterUpdate"))
    };
    let from_any_season = || {
        seasons?
            .values()
            .find_map(|block| usable_tier(block.get("CompetitiveTier")))
    };
    from_act
        .or_else(from_updates)
        .or_else(from_any_season)
        .unwrap_or(0)
}

/// Highest tier a player has won a game at, plus the act-end badge, across every season.
/// 0 when the response says nothing or the result is not a valid tier.
pub fn peak_tier_from_mmr(mmr: &Value) -> u8 {
    let Some(seasons) = competitive_seasons(mmr) else {
        return 0;
    };
    let mut peak = 0_i64;
    for block in seasons.values().filter(|block| block.is_object()) {
        let wins_by_tier = block.get("WinsByTier").and_then(Value::as_object);
        for (tier, wins) in wins_by_tier.into_iter().flatten() {
            let won = lenient_int(Some(wins)).is_some_and(|wins| wins > 0);
            if let (true, Ok(tier)) = (won, tier.trim().parse::<i64>()) {
                peak = peak.max(tier);
            }
        }
        // Applied unconditionally, like the original, despite its "fall back" comment.
        if let Some(badge) = exact_int(block.get("Rank")) {
            peak = peak.max(badge);
        }
    }
    u8::try_from(peak)
        .ok()
        .filter(|tier| *tier <= MAX_TIER)
        .unwrap_or(0)
}

/// Win rate of a seasonal block in percent, 1 decimal; `None` when it has no games.
pub fn win_rate_from(info: &Value) -> Option<f64> {
    let games = number(info.get("NumberOfGames")).filter(|games| *games > 0.0)?;
    let wins = number_or_zero(info.get("NumberOfWins"));
    Some(round_to_digits(wins / games * 100.0, 1))
}

/// Win rate from the queue they play most, for players with no ranked record.
/// Strictly more games wins, so the first queue in key order keeps a tie.
pub fn alt_win_rate(mmr: &Value, season_id: &str) -> Option<(f64, String)> {
    let queues = mmr.get("QueueSkills")?.as_object()?;
    let mut best: Option<(f64, String)> = None;
    let mut best_games = 0.0;
    for queue in queues
        .iter()
        .filter(|(queue, block)| *queue != STATS_QUEUE && block.is_object())
        .map(|(queue, _)| queue)
    {
        let info = season_info(mmr, queue, season_id);
        let games = number_or_zero(info.get("NumberOfGames"));
        if let Some(rate) = win_rate_from(info).filter(|_| games > best_games) {
            best_games = games;
            best = Some((rate, queue.clone()));
        }
    }
    best
}

/// Ladder position: the live match badge first, else the MMR block. 0 when neither has one.
fn leaderboard_rank(badge: &Value, info: &Value) -> u32 {
    let live = badge.get("LeaderboardRank").filter(|rank| is_truthy(rank));
    to_u32(whole(live.or_else(|| info.get("LeaderboardRank"))))
}

/// One match's contribution to a player's combat stats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchCombat {
    pub kills: i64,
    pub deaths: i64,
    pub headshots: i64,
    pub shots: i64,
}

/// K/D, kills per match and headshot percentage over a player's newest matches.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct CombatStats {
    /// Rounded to 1 decimal after `deaths` is floored at 1.
    pub kills_per_death: f64,
    pub kills_per_match: f64,
    pub head_shot: f64,
}

/// A match's combat line for `puuid`, or `None` when the response lacks their `stats`.
pub fn match_combat_from_detail(detail: &Value, puuid: &str) -> Option<MatchCombat> {
    let entry = items(detail, &["players", "Players"])
        .iter()
        .find(|entry| super::json::subject(entry) == puuid)?;
    let stats = entry
        .get("stats")
        .or_else(|| entry.get("Stats"))
        .filter(|s| s.is_object())?;
    let mut combat = MatchCombat {
        kills: whole(stats.get("kills")),
        deaths: whole(stats.get("deaths")),
        headshots: 0,
        shots: 0,
    };
    for round in super::rounds::round_results(detail) {
        let own = super::rounds::player_stats(round).find(|s| super::json::subject(s) == puuid);
        for damage in own.map_or(&[][..], |s| items(s, &["damage", "Damage"])) {
            let (head, body, leg) = (
                whole(damage.get("headshots")),
                whole(damage.get("bodyshots")),
                whole(damage.get("legshots")),
            );
            combat.headshots += head;
            combat.shots += head + body + leg;
        }
    }
    Some(combat)
}

/// Combine matches into the ranker's K/D, K/M and HS%. `None` when there are no matches.
pub fn summarise_combat(matches: &[MatchCombat]) -> Option<CombatStats> {
    if matches.is_empty() {
        return None;
    }
    let sum = |pick: fn(&MatchCombat) -> i64| matches.iter().map(pick).sum::<i64>() as f64;
    let kills = sum(|m| m.kills);
    // Floored at 1 like the original: a deathless player gets `kills / 1`.
    let deaths = sum(|m| m.deaths).max(1.0);
    let shots = sum(|m| m.shots).max(1.0);
    Some(CombatStats {
        kills_per_death: round_to_digits(kills / deaths, 1),
        kills_per_match: round_to_digits(kills / matches.len() as f64, 1),
        head_shot: round_to_digits(sum(|m| m.headshots) / shots * 100.0, 1),
    })
}

#[cfg(test)]
mod tests;
