//! Owned items by item type.

use domain::ids::{ItemId, Puuid};
use serde::Deserialize;

use crate::http::{Host, Request};

/// Agents.
pub const ITEM_TYPE_AGENTS: &str = "01bb38e1-da47-4e6a-9b3d-945fe4655707";
/// Weapon skin levels (level 1 ~ "owns the skin").
pub const ITEM_TYPE_SKIN_LEVELS: &str = "e7c63390-eda7-46e0-bb7a-a6abdacd2433";
/// Weapon skin chromas (variants).
pub const ITEM_TYPE_SKIN_CHROMAS: &str = "3ad1b2b2-acdb-4524-852f-954a76ddae0a";
/// Gun buddies; entries carry an `InstanceID`.
pub const ITEM_TYPE_BUDDIES: &str = "dd3bf334-87f3-40bd-b043-682a57a8dc3a";
/// Sprays.
pub const ITEM_TYPE_SPRAYS: &str = "d5f120f8-ff8c-4aac-92ea-f2b5acbe9475";
/// Player cards.
pub const ITEM_TYPE_PLAYER_CARDS: &str = "3f296c07-64c3-494c-923b-fe692a4fa1bd";
/// Player titles.
pub const ITEM_TYPE_PLAYER_TITLES: &str = "de7caa6b-adf7-4588-bbd1-143831e786c6";
/// Flex (internally "Totems"); the Flex mapping is inferred, not documented.
pub const ITEM_TYPE_FLEX: &str = "03a572de-4234-31ed-d344-ababa488f981";
/// Contract definitions (battle passes, agent contracts).
pub const ITEM_TYPE_CONTRACTS: &str = "f85cb6f7-33e5-4dc8-b609-ec7212301948";

/// `GET /store/v1/entitlements/{puuid}`: every owned type in one call (empty types omitted).
pub fn get_all_entitlements(puuid: &Puuid) -> Request {
    Request::get(Host::Pd, format!("/store/v1/entitlements/{puuid}"))
}

/// `GET /store/v1/entitlements/{puuid}/{item_type_id}`: owned items of one type.
pub fn get_entitlements(puuid: &Puuid, item_type_id: &str) -> Request {
    Request::get(
        Host::Pd,
        format!("/store/v1/entitlements/{puuid}/{item_type_id}"),
    )
}

/// Entitlements answer. The all-types call fills `entitlements_by_types`; the per-type call is
/// tolerated in both layouts (grouped, or flat with `item_type_id` at the top).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Entitlements {
    #[serde(deserialize_with = "super::null_as_default")]
    pub entitlements_by_types: Vec<EntitlementGroup>,
    #[serde(rename = "ItemTypeID")]
    pub item_type_id: Option<String>,
    #[serde(deserialize_with = "super::null_as_default")]
    pub entitlements: Vec<Entitlement>,
}

impl Entitlements {
    /// Owned entries of one item type, whichever layout Riot answered with.
    pub fn of_type(&self, item_type_id: &str) -> Vec<&Entitlement> {
        let grouped = self
            .entitlements_by_types
            .iter()
            .filter(|group| group.item_type_id.eq_ignore_ascii_case(item_type_id))
            .flat_map(|group| group.entitlements.iter());
        let flat_matches = self
            .item_type_id
            .as_deref()
            .is_some_and(|id| id.eq_ignore_ascii_case(item_type_id));
        let flat = self.entitlements.iter().filter(|_| flat_matches);
        grouped.chain(flat).collect()
    }
}

/// Owned items of one type.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct EntitlementGroup {
    #[serde(rename = "ItemTypeID")]
    pub item_type_id: String,
    #[serde(deserialize_with = "super::null_as_default")]
    pub entitlements: Vec<Entitlement>,
}

/// One owned item; buddies have one entry (and `instance_id`) per copy.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Entitlement {
    #[serde(rename = "TypeID")]
    pub type_id: String,
    #[serde(rename = "ItemID")]
    pub item_id: Option<ItemId>,
    #[serde(rename = "InstanceID")]
    pub instance_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_target_all_types_and_one_type() {
        let puuid = Puuid::new("abc");

        assert_eq!(
            get_all_entitlements(&puuid).path,
            "/store/v1/entitlements/abc"
        );
        assert_eq!(
            get_entitlements(&puuid, ITEM_TYPE_SPRAYS).path,
            "/store/v1/entitlements/abc/d5f120f8-ff8c-4aac-92ea-f2b5acbe9475"
        );
    }

    #[test]
    fn item_type_constants_match_the_spec_table() {
        assert_eq!(ITEM_TYPE_AGENTS, "01bb38e1-da47-4e6a-9b3d-945fe4655707");
        assert_eq!(
            ITEM_TYPE_SKIN_LEVELS,
            "e7c63390-eda7-46e0-bb7a-a6abdacd2433"
        );
        assert_eq!(ITEM_TYPE_BUDDIES, "dd3bf334-87f3-40bd-b043-682a57a8dc3a");
        assert_eq!(ITEM_TYPE_FLEX, "03a572de-4234-31ed-d344-ababa488f981");
        assert_eq!(ITEM_TYPE_CONTRACTS, "f85cb6f7-33e5-4dc8-b609-ec7212301948");
    }

    #[test]
    fn grouped_answer_filters_by_type_and_keeps_instance_ids() {
        let json = format!(
            r#"{{"EntitlementsByTypes":[
            {{"ItemTypeID":"{ITEM_TYPE_BUDDIES}","Entitlements":[
              {{"TypeID":"{ITEM_TYPE_BUDDIES}","ItemID":"b-1","InstanceID":"i-1"}},
              {{"TypeID":"{ITEM_TYPE_BUDDIES}","ItemID":"b-1","InstanceID":"i-2"}}]}},
            {{"ItemTypeID":"{ITEM_TYPE_SPRAYS}","Entitlements":[{{"TypeID":"x","ItemID":"s-1"}}]}}]}}"#
        );

        let owned: Entitlements = serde_json::from_str(&json).expect("decodes");

        let buddies = owned.of_type(ITEM_TYPE_BUDDIES);
        assert_eq!(buddies.len(), 2);
        assert_eq!(buddies[1].instance_id.as_deref(), Some("i-2"));
        assert_eq!(
            owned.of_type(ITEM_TYPE_SPRAYS)[0].item_id,
            Some(ItemId::new("s-1"))
        );
        assert!(owned.of_type(ITEM_TYPE_AGENTS).is_empty());
    }

    #[test]
    fn flat_per_type_answer_is_tolerated() {
        let json = format!(
            r#"{{"ItemTypeID":"{ITEM_TYPE_AGENTS}","Entitlements":[{{"TypeID":"t","ItemID":"a-1"}}]}}"#
        );

        let owned: Entitlements = serde_json::from_str(&json).expect("decodes");

        assert_eq!(owned.of_type(ITEM_TYPE_AGENTS).len(), 1);
        assert!(owned.of_type(ITEM_TYPE_SPRAYS).is_empty());
    }
}
