//! Match loadout DTOs shared by pregame and coregame.
//!
//! Pregame and coregame disagree on nesting (rows are `{CharacterID, Loadout: {...}}` or flat),
//! so a row accepts both and `entry()` returns whichever was present.

use std::collections::BTreeMap;

use domain::ids::{AgentId, ItemId, Puuid};
use serde::Deserialize;

use super::common::null_as_default;

/// Response of `.../matches/{id}/loadouts` on pregame and coregame.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct MatchLoadouts {
    #[serde(deserialize_with = "null_as_default")]
    pub loadouts: Vec<LoadoutRow>,
    /// Pregame only: `false` means loadouts are not prepared yet; retry in 1-2 s.
    pub loadouts_valid: bool,
}

/// One player's row, nested or flat.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct LoadoutRow {
    #[serde(rename = "CharacterID")]
    pub character_id: Option<AgentId>,
    pub loadout: Option<LoadoutEntry>,
    #[serde(flatten)]
    flat: LoadoutEntry,
}

impl LoadoutRow {
    /// The loadout, whether the row nested it under `Loadout` or carried it inline.
    pub fn entry(&self) -> &LoadoutEntry {
        self.loadout.as_ref().unwrap_or(&self.flat)
    }
}

/// A player's sprays, expressions and per-weapon items.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct LoadoutEntry {
    pub subject: Option<Puuid>,
    #[serde(deserialize_with = "null_as_default")]
    pub sprays: Sprays,
    #[serde(deserialize_with = "null_as_default")]
    pub expressions: Expressions,
    /// Weapon uuid -> equipped item; skin level, chroma and buddy sit in `sockets`.
    #[serde(deserialize_with = "null_as_default")]
    pub items: BTreeMap<String, LoadoutItem>,
}

/// Spray wheel selections.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Sprays {
    #[serde(deserialize_with = "null_as_default")]
    pub spray_selections: Vec<SpraySelection>,
}

/// One spray slot.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct SpraySelection {
    #[serde(rename = "SocketID")]
    pub socket_id: String,
    #[serde(rename = "SprayID")]
    pub spray_id: Option<ItemId>,
    #[serde(rename = "LevelID")]
    pub level_id: Option<ItemId>,
}

/// Expression (spray/flex) selections.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Expressions {
    #[serde(rename = "AESSelections", deserialize_with = "null_as_default")]
    pub aes_selections: Vec<ExpressionSelection>,
}

/// One expression slot.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct ExpressionSelection {
    #[serde(rename = "SocketID")]
    pub socket_id: String,
    #[serde(rename = "AssetID")]
    pub asset_id: Option<ItemId>,
    #[serde(rename = "TypeID")]
    pub type_id: Option<ItemId>,
}

/// An equipped weapon with its sockets.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct LoadoutItem {
    #[serde(rename = "ID")]
    pub id: Option<ItemId>,
    #[serde(rename = "TypeID")]
    pub type_id: Option<ItemId>,
    /// Socket uuid -> item in that socket.
    #[serde(deserialize_with = "null_as_default")]
    pub sockets: BTreeMap<String, LoadoutSocket>,
}

/// A socket holding one cosmetic.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct LoadoutSocket {
    #[serde(rename = "ID")]
    pub id: Option<ItemId>,
    #[serde(deserialize_with = "null_as_default")]
    pub item: SocketItem,
}

/// The cosmetic inside a socket.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct SocketItem {
    #[serde(rename = "ID")]
    pub id: Option<ItemId>,
    #[serde(rename = "TypeID")]
    pub type_id: Option<ItemId>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const NESTED: &str = include_str!("fixtures/coregame_loadouts.json");

    #[test]
    fn nested_rows_expose_items_and_sockets() {
        let loadouts: MatchLoadouts = serde_json::from_str(NESTED).unwrap();
        let row = &loadouts.loadouts[0];
        assert_eq!(
            row.character_id.as_ref().unwrap().as_str(),
            "add6443a-41bd-e414-f6ad-e58d267f4e95"
        );
        let entry = row.entry();
        assert_eq!(entry.subject.as_ref().unwrap().as_str(), "b47236a3-0000");
        let item = &entry.items["weapon-1"];
        assert_eq!(
            item.sockets["socket-skin"]
                .item
                .id
                .as_ref()
                .unwrap()
                .as_str(),
            "skin-level-1"
        );
        assert_eq!(
            entry.sprays.spray_selections[0]
                .spray_id
                .as_ref()
                .unwrap()
                .as_str(),
            "spray-1"
        );
        assert_eq!(
            entry.expressions.aes_selections[0]
                .asset_id
                .as_ref()
                .unwrap()
                .as_str(),
            "flex-1"
        );
    }

    #[test]
    fn flat_rows_fall_back_to_inline_entry() {
        let body = r#"{"Loadouts":[{"Subject":"p1","Items":{"w":{"ID":"w","Sockets":null}}}],"LoadoutsValid":true}"#;
        let loadouts: MatchLoadouts = serde_json::from_str(body).unwrap();
        let entry = loadouts.loadouts[0].entry();
        assert_eq!(entry.subject.as_ref().unwrap().as_str(), "p1");
        assert!(entry.items["w"].sockets.is_empty());
    }

    #[test]
    fn missing_loadouts_decode_as_not_ready() {
        let loadouts: MatchLoadouts = serde_json::from_str("{}").unwrap();
        assert!(!loadouts.loadouts_valid && loadouts.loadouts.is_empty());
    }
}
