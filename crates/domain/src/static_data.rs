//! Fallback agent and map tables, and lookups by name, uuid and internal id.
//!
//! Used when valorant-api.com is unreachable. Key everything by uuid or internal id; display
//! names are for the UI only. Do not edit the uuids: they are Riot's.

use serde::Serialize;
use ts_rs::TS;

use crate::ids::AgentId;
use crate::numeric::title_case;

/// A playable agent: upper-case name and Riot uuid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct AgentEntry {
    pub name: &'static str,
    pub uuid: &'static str,
}

impl AgentEntry {
    /// Title-case name for the UI (`Kay/O`).
    pub fn display_name(&self) -> String {
        title_case(self.name)
    }

    /// The uuid as a typed id.
    pub fn agent_id(&self) -> AgentId {
        AgentId::new(self.uuid)
    }
}

/// A map: upper-case name and the internal id found in a match's `MapID`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct MapEntry {
    pub name: &'static str,
    pub internal_id: &'static str,
}

impl MapEntry {
    /// Title-case name for the UI (`Bind`).
    pub fn display_name(&self) -> String {
        title_case(&self.name.replace('_', " "))
    }

    /// Team Deathmatch maps are listed with the competitive maps.
    pub fn is_team_deathmatch(&self) -> bool {
        self.internal_id.starts_with("HURM_")
    }
}

const fn agent(name: &'static str, uuid: &'static str) -> AgentEntry {
    AgentEntry { name, uuid }
}

const fn map(name: &'static str, internal_id: &'static str) -> MapEntry {
    MapEntry { name, internal_id }
}

/// Playable agents, alphabetical like the live API sort.
pub const FALLBACK_AGENTS: [AgentEntry; 29] = [
    agent("ASTRA", "41fb69c1-4189-7b37-f117-bcaf1e96f1bf"),
    agent("BREACH", "5f8d3a7f-467b-97f3-062c-13acf203c006"),
    agent("BRIMSTONE", "9f0d8ba9-4140-b941-57d3-a7ad57c6b417"),
    agent("CHAMBER", "22697a3d-45bf-8dd7-4fec-84a9e28c69d7"),
    agent("CLOVE", "1dbf2edd-4729-0984-3115-daa5eed44993"),
    agent("CYPHER", "117ed9e3-49f3-6512-3ccf-0cada7e3823b"),
    agent("DEADLOCK", "cc8b64c8-4b25-4ff9-6e7f-37b4da43d235"),
    agent("FADE", "dade69b4-4f5a-8528-247b-219e5a1facd6"),
    agent("GEKKO", "e370fa57-4757-3604-3648-499e1f642d3f"),
    agent("HARBOR", "95b78ed7-4637-86d9-7e41-71ba8c293152"),
    agent("ISO", "0e38b510-41a8-5780-5e8f-568b2a4f2d6c"),
    agent("JETT", "add6443a-41bd-e414-f6ad-e58d267f4e95"),
    agent("KAY/O", "601dbbe7-43ce-be57-2a40-4abd24953621"),
    agent("KILLJOY", "1e58de9c-4950-5125-93e9-a0aee9f98746"),
    agent("MIKS", "7c8a4701-4de6-9355-b254-e09bc2a34b72"),
    agent("NEON", "bb2a4828-46eb-8cd1-e765-15848195d751"),
    agent("OMEN", "8e253930-4c05-31dd-1b6c-968525494517"),
    agent("PHOENIX", "eb93336a-449b-9c1b-0a54-a891f7921d69"),
    agent("RAZE", "f94c3b30-42be-e959-889c-5aa313dba261"),
    agent("REYNA", "a3bfb853-43b2-7238-a4f1-ad90e9e46bcc"),
    agent("SAGE", "569fdd95-4d10-43ab-ca70-79becc718b46"),
    agent("SKYE", "6f2a04ca-43e0-be17-7f36-b3908627744d"),
    agent("SOVA", "320b2a48-4d9b-a075-30f1-1f93a9b638fa"),
    agent("TEJO", "b444168c-4e35-8076-db47-ef9bf368f384"),
    agent("VETO", "92eeef5d-43b5-1d4a-8d03-b3927a09034b"),
    agent("VIPER", "707eab51-4836-f488-046a-cda6bf494859"),
    agent("VYSE", "efba5359-4016-a1e5-7626-b1ae76895940"),
    agent("WAYLAY", "df1cb487-4902-002e-5c17-d28e83e78588"),
    agent("YORU", "7f94d92c-4234-0a36-9646-3a87eb8b5c89"),
];

/// Maps in menu order. Internal ids are case-sensitive (`Duality` is Bind).
pub const FALLBACK_MAPS: [MapEntry; 18] = [
    map("ABYSS", "Infinity"),
    map("ASCENT", "Ascent"),
    map("BIND", "Duality"),
    map("BREEZE", "Foxtrot"),
    map("CORRODE", "Rook"),
    map("DISTRICT", "HURM_Alley"),
    map("DRIFT", "HURM_Helix"),
    map("FRACTURE", "Canyon"),
    map("GLITCH", "HURM_HighTide"),
    map("HAVEN", "Triad"),
    map("ICEBOX", "Port"),
    map("KASBAH", "HURM_Bowl"),
    map("LOTUS", "Jam"),
    map("PEARL", "Pitt"),
    map("PIAZZA", "HURM_Yard"),
    map("SPLIT", "Bonsai"),
    map("SUMMIT", "Plummet"),
    map("SUNSET", "Juliett"),
];

/// Lookup key: upper case, `/` and spaces folded to `_` so
/// `KAY/O`, `kay/o` and `Kay O` all meet.
fn canonical_key(name: &str) -> String {
    name.trim().to_uppercase().replace(['/', ' '], "_")
}

/// Agent by name; case-insensitive (`jett`, `KAY/O`, `kay/o`).
pub fn agent_by_name(name: &str) -> Option<&'static AgentEntry> {
    let key = canonical_key(name);
    FALLBACK_AGENTS
        .iter()
        .find(|entry| canonical_key(entry.name) == key)
}

/// Agent by uuid; case-insensitive.
pub fn agent_by_uuid(uuid: &str) -> Option<&'static AgentEntry> {
    let key = uuid.trim().to_lowercase();
    FALLBACK_AGENTS.iter().find(|entry| entry.uuid == key)
}

/// Map by upper-case or display name; case-insensitive (`ascent`, `Ascent`, `ASCENT`).
pub fn map_by_name(name: &str) -> Option<&'static MapEntry> {
    let key = canonical_key(name);
    FALLBACK_MAPS.iter().find(|entry| entry.name == key)
}

/// Map by internal id. Exact and case-sensitive on purpose: an unknown map must fail
/// closed, because guessing once locked the Abyss agent on every map.
pub fn map_by_internal_id(internal_id: &str) -> Option<&'static MapEntry> {
    FALLBACK_MAPS
        .iter()
        .find(|entry| entry.internal_id == internal_id)
}

/// Map from a match's `MapID` path such as `/Game/Maps/Duality/Duality`: the internal id is
/// the second to last segment.
pub fn map_by_map_id_path(map_id_path: &str) -> Option<&'static MapEntry> {
    map_by_internal_id(map_id_path.rsplit('/').nth(1)?)
}

/// Display name for an internal id. Tolerant of case and underscores; unknown ids are
/// titled as given. Empty and `Unknown` pass through unchanged.
pub fn map_display_name(internal_id: &str) -> String {
    if internal_id.is_empty() || internal_id == "Unknown" {
        return internal_id.to_owned();
    }
    let squashed = |text: &str| text.trim().to_lowercase().replace(['_', ' '], "");
    let wanted = squashed(internal_id);
    match FALLBACK_MAPS
        .iter()
        .find(|entry| squashed(entry.internal_id) == wanted)
    {
        Some(entry) => entry.display_name(),
        None => title_case(&internal_id.replace('_', " ")),
    }
}

#[cfg(test)]
mod tests;
