//! Chat presence decoding.
//!
//! `GET /chat/v4/presences` is the FRIENDS feed, not the lobby: it holds you, your party and
//! your friends, never strangers. Absence of a player therefore means "not visible", never
//! "queued alone".

use std::collections::{HashMap, HashSet};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Map, Value};

use crate::ids::{PartyId, Puuid};
use crate::static_data::map_display_name;

/// Nested blocks of the current presence shape, searched in this order before the top level.
const PRESENCE_BLOCKS: [&str; 3] = [
    "matchPresenceData",
    "partyPresenceData",
    "playerPresenceData",
];
const UNKNOWN_MAP: &str = "Unknown";

/// Decode a presence `private` blob (base64 JSON object). Anything else decodes to empty:
/// other games' friends carry blobs that are not VALORANT JSON.
pub fn decode_private(blob: &str) -> Map<String, Value> {
    let Ok(bytes) = STANDARD.decode(blob.trim()) else {
        return Map::new();
    };
    match serde_json::from_slice::<Value>(&bytes) {
        Ok(Value::Object(object)) => object,
        _ => Map::new(),
    }
}

/// A presence field from the nested blocks (current builds) or the top level (old builds).
pub fn presence_field<'a>(private: &'a Map<String, Value>, key: &str) -> Option<&'a Value> {
    PRESENCE_BLOCKS
        .iter()
        .filter_map(|block| private.get(*block).and_then(Value::as_object))
        .find_map(|block| block.get(key))
        .or_else(|| private.get(key))
}

fn field_str<'a>(private: &'a Map<String, Value>, key: &str) -> &'a str {
    presence_field(private, key)
        .and_then(Value::as_str)
        .unwrap_or("")
}

fn field_u32(private: &Map<String, Value>, key: &str) -> Option<u32> {
    presence_field(private, key)
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
}

/// The game-loop state the client reports for itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopState {
    Menus,
    Pregame,
    Ingame,
}

impl LoopState {
    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "MENUS" => Some(Self::Menus),
            "PREGAME" => Some(Self::Pregame),
            "INGAME" => Some(Self::Ingame),
            _ => None,
        }
    }
}

/// Live match facts from the player's own presence; only meaningful during a match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveScores {
    pub ally_score: u32,
    pub enemy_score: u32,
    /// `Blue`/`Red` on old builds; `Unknown` on current builds (team comes from agent select).
    pub current_team: String,
    pub map_name: String,
    pub queue: String,
}

/// What the player's own presence says right now.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OwnPresence {
    pub loop_state: Option<LoopState>,
    /// Only set when the blob holds a positive integer.
    pub party_size: Option<u32>,
    pub party_id: Option<PartyId>,
    pub live_scores: Option<LiveScores>,
}

/// Read the own-presence blob.
///
/// Live scores exist only while the loop state is PREGAME or INGAME and presence has named
/// something (a score or a map); otherwise the previous match's numbers would leak.
pub fn read_own_presence(private: &Map<String, Value>) -> OwnPresence {
    let loop_state = LoopState::parse(field_str(private, "sessionLoopState"));
    let is_in_match = matches!(loop_state, Some(LoopState::Pregame | LoopState::Ingame));
    let party_id = Some(field_str(private, "partyId"))
        .filter(|id| !id.is_empty())
        .map(PartyId::new);
    OwnPresence {
        loop_state,
        party_size: field_u32(private, "partySize").filter(|size| *size > 0),
        party_id,
        live_scores: if is_in_match {
            live_scores(private)
        } else {
            None
        },
    }
}

fn live_scores(private: &Map<String, Value>) -> Option<LiveScores> {
    let ally_score = field_u32(private, "partyOwnerMatchScoreAllyTeam").unwrap_or(0);
    let enemy_score = field_u32(private, "partyOwnerMatchScoreEnemyTeam").unwrap_or(0);
    let map_name = presence_map_name(private);
    if ally_score == 0 && enemy_score == 0 && map_name == UNKNOWN_MAP {
        return None;
    }
    let current_team = Some(field_str(private, "partyOwnerMatchCurrentTeam"))
        .filter(|team| !team.is_empty())
        .unwrap_or(UNKNOWN_MAP);
    Some(LiveScores {
        ally_score,
        enemy_score,
        current_team: current_team.to_owned(),
        map_name,
        queue: field_str(private, "queueId").trim().to_owned(),
    })
}

/// Map display name from `partyOwnerMatchMap`, falling back to `matchMap`, then `Unknown`.
fn presence_map_name(private: &Map<String, Value>) -> String {
    let raw = [
        field_str(private, "partyOwnerMatchMap"),
        field_str(private, "matchMap"),
    ]
    .into_iter()
    .find(|value| !value.is_empty())
    .unwrap_or(UNKNOWN_MAP);
    let internal_id = raw.rsplit('/').next().unwrap_or(raw);
    map_display_name(internal_id)
}

/// Party membership of one visible player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartyInfo {
    pub party_id: PartyId,
    /// Members of the whole party, including people outside this match.
    pub party_size: u32,
}

impl PartyInfo {
    /// More than one person queued together (possibly not all in this match).
    pub fn is_stack(&self) -> bool {
        self.party_size > 1
    }
}

/// Party per visible player from a `presences` array, optionally restricted to a roster.
pub fn parse_parties(
    presences: &[Value],
    roster: Option<&HashSet<Puuid>>,
) -> HashMap<Puuid, PartyInfo> {
    presences
        .iter()
        .filter_map(|entry| {
            let puuid = Puuid::new(entry.get("puuid")?.as_str()?);
            let is_wanted = roster.is_none_or(|members| members.contains(&puuid));
            let blob = entry.get("private")?.as_str()?;
            is_wanted
                .then(|| party_from_blob(blob))
                .flatten()
                .map(|party| (puuid, party))
        })
        .collect()
}

fn party_from_blob(blob: &str) -> Option<PartyInfo> {
    let decoded = decode_private(blob);
    let source = decoded
        .get("partyPresenceData")
        .and_then(Value::as_object)
        .unwrap_or(&decoded);
    let party_id = source
        .get("partyId")?
        .as_str()
        .filter(|id| !id.is_empty())?;
    let party_size = source.get("partySize")?.as_i64()?;
    Some(PartyInfo {
        party_id: PartyId::new(party_id),
        party_size: u32::try_from(party_size.max(1)).unwrap_or(1),
    })
}

/// Size of each roster member's stack WITHIN this match: members sharing a party id.
/// `partySize` alone would count party members who are not in the match.
pub fn stack_sizes(parties: &HashMap<Puuid, PartyInfo>, roster: &[Puuid]) -> HashMap<Puuid, u32> {
    let mut members_per_party: HashMap<&PartyId, u32> = HashMap::new();
    for puuid in roster {
        if let Some(party) = parties.get(puuid) {
            *members_per_party.entry(&party.party_id).or_default() += 1;
        }
    }
    roster
        .iter()
        .filter_map(|puuid| {
            let party = parties.get(puuid)?;
            Some((
                puuid.clone(),
                members_per_party.get(&party.party_id).copied().unwrap_or(1),
            ))
        })
        .collect()
}

/// Largest stack in the roster; 1 when nobody's party is visible.
pub fn largest_stack(stacks: &HashMap<Puuid, u32>) -> u32 {
    stacks.values().copied().max().unwrap_or(1)
}

#[cfg(test)]
mod tests;
