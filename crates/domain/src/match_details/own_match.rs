//! The signed-in user's own line of a finished match: result, score and personal stats.

use serde::Serialize;
use serde_json::Value;

use super::context::{played_at, queue_of, won_flag};
use super::json::{
    first_present, is_truthy, items, lenient_int, match_id, match_info, object, roster, subject,
    team_id, text,
};
use super::rounds::{opening_kill, player_stats, round_results};
use super::sides::{StartingSide, derive_sides};

/// The player's own line of a match, every value known.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExtractedOwnMatch {
    pub match_id: String,
    pub started_at: f64,
    /// Resolved queue, or the placeholder `"unknown"` that a later pass may correct.
    pub queue: String,
    /// The `mapId` path; the display name is derived at read time.
    pub map_id: String,
    /// Agent uuid; the display name is derived at read time.
    pub agent_id: String,
    pub won: bool,
    pub rounds_won: i64,
    pub rounds_lost: i64,
    pub kills: i64,
    pub deaths: i64,
    pub assists: i64,
    pub combat_score: i64,
    pub rounds_played: i64,
    pub headshots: i64,
    pub bodyshots: i64,
    pub legshots: i64,
    pub damage: i64,
    pub party_size: i64,
    pub first_bloods: i64,
    pub first_deaths: i64,
    pub afk_rounds: i64,
    pub started_side: Option<StartingSide>,
    pub attack_won: i64,
    pub attack_played: i64,
    pub defence_won: i64,
    pub defence_played: i64,
    /// The account that played it; always the extracting puuid.
    pub account_puuid: Option<String>,
}

/// How many players shared this player's party (at least 1; 1 when their party id is missing).
pub fn party_size_from_details(detail: &Value, puuid: &str) -> i64 {
    let players = roster(detail);
    let own_party = players
        .iter()
        .find(|entry| subject(entry) == puuid)
        .map(|entry| text(entry, &["partyId", "PartyID"]))
        .filter(|party| !party.is_empty());
    let Some(own_party) = own_party else { return 1 };
    let shared = players
        .iter()
        .filter(|e| text(e, &["partyId", "PartyID"]) == own_party)
        .count();
    i64::try_from(shared).unwrap_or(i64::MAX).max(1)
}

/// Build the row for `puuid`, or `None` when the response is not a match containing them.
/// `party_size` of `None` or `Some(0)` is derived from the response.
pub fn extract_own_match(
    detail: &Value,
    puuid: &str,
    party_size: Option<u32>,
) -> Option<ExtractedOwnMatch> {
    if !detail.is_object() || puuid.is_empty() {
        return None;
    }
    let players = roster(detail);
    let me = players.iter().find(|entry| subject(entry) == puuid)?;
    let party = party_size
        .filter(|size| *size > 0)
        .map_or_else(|| party_size_from_details(detail, puuid), i64::from);
    Some(build_row(detail, me, puuid, party))
}

fn build_row(detail: &Value, me: &Value, puuid: &str, party: i64) -> ExtractedOwnMatch {
    let own_team = team_id(me);
    let stats = object(me, &["stats", "Stats"]);
    let count = |key: &str| lenient_int(stats.get(key)).unwrap_or(0);
    let queue = queue_of(detail);
    let (won, rounds_won, rounds_lost) = own_score(detail, &own_team);
    let (shots, duels) = (shot_breakdown(detail, puuid), opening_duels(detail, puuid));
    let sides = derive_sides(detail, &own_team, queue);
    ExtractedOwnMatch {
        match_id: match_id(detail),
        started_at: played_at(detail),
        queue: if queue.is_empty() { "unknown" } else { queue }.to_owned(),
        map_id: text(match_info(detail), &["mapId", "MapID"]),
        agent_id: text(me, &["characterId", "CharacterID"]),
        won,
        rounds_won,
        rounds_lost,
        kills: count("kills"),
        deaths: count("deaths"),
        assists: count("assists"),
        combat_score: count("score"),
        rounds_played: count("roundsPlayed"),
        headshots: shots.headshots,
        bodyshots: shots.bodyshots,
        legshots: shots.legshots,
        damage: shots.damage,
        party_size: party.max(1),
        first_bloods: duels.0,
        first_deaths: duels.1,
        afk_rounds: lenient_int(object(me, &["behaviorFactors"]).get("afkRounds")).unwrap_or(0),
        started_side: sides.started_side,
        attack_won: i64::from(sides.attack_won),
        attack_played: i64::from(sides.attack_played),
        defence_won: i64::from(sides.defence_won),
        defence_played: i64::from(sides.defence_played),
        account_puuid: Some(puuid.to_owned()),
    }
}

/// `(won, rounds_won, rounds_lost)`: own team's score, and the best score among the others.
fn own_score(detail: &Value, own_team: &str) -> (bool, i64, i64) {
    let (mut won, mut rounds_won, mut rounds_lost) = (false, 0, 0);
    for team in items(detail, &["teams", "Teams"])
        .iter()
        .filter(|team| team.is_object())
    {
        let rounds = first_present(team, &["roundsWon", "RoundsWon"])
            .filter(|value| is_truthy(value))
            .and_then(|value| lenient_int(Some(value)))
            .unwrap_or(0);
        if team_id(team) == own_team {
            won = won_flag(team);
            rounds_won = rounds;
        } else {
            rounds_lost = rounds_lost.max(rounds);
        }
    }
    (won, rounds_won, rounds_lost)
}

#[derive(Debug, Default, Clone, Copy)]
struct Shots {
    headshots: i64,
    bodyshots: i64,
    legshots: i64,
    damage: i64,
}

/// Every shot and point of damage this player dealt across the match.
fn shot_breakdown(detail: &Value, puuid: &str) -> Shots {
    let mut totals = Shots::default();
    for round in round_results(detail) {
        for stat in player_stats(round).filter(|stat| subject(stat) == puuid) {
            for entry in items(stat, &["damage", "Damage"])
                .iter()
                .filter(|e| e.is_object())
            {
                let amount = |key: &str| lenient_int(entry.get(key)).unwrap_or(0);
                totals.headshots += amount("headshots");
                totals.bodyshots += amount("bodyshots");
                totals.legshots += amount("legshots");
                totals.damage += amount("damage");
            }
        }
    }
    totals
}

/// `(first_bloods, first_deaths)`: rounds where this player took, or lost, the first fight.
fn opening_duels(detail: &Value, puuid: &str) -> (i64, i64) {
    let (mut first_bloods, mut first_deaths) = (0, 0);
    for round in round_results(detail) {
        let Some(opener) = opening_kill(round, |_| true) else {
            continue;
        };
        if opener.killer == puuid {
            first_bloods += 1;
        } else if opener.victim == puuid {
            first_deaths += 1;
        }
    }
    (first_bloods, first_deaths)
}

#[cfg(test)]
mod tests;
