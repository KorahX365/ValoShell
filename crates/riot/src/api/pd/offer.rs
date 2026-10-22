//! The store OFFER shape shared by storefront, bundles and the agent store.

use std::collections::HashMap;

use domain::ids::ItemId;
use serde::Deserialize;

/// Prices keyed by currency id (see `wallet` for the id constants).
pub type Cost = HashMap<String, i64>;

/// A purchasable offer. Viewing only: this crate has no purchase endpoints.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Offer {
    #[serde(rename = "OfferID")]
    pub offer_id: String,
    pub is_direct_purchase: bool,
    pub start_date: String,
    #[serde(deserialize_with = "super::null_as_default")]
    pub cost: Cost,
    #[serde(deserialize_with = "super::null_as_default")]
    pub rewards: Vec<OfferReward>,
}

/// What an offer grants; skin offers use the skin-level item type.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct OfferReward {
    #[serde(rename = "ItemTypeID")]
    pub item_type_id: String,
    #[serde(rename = "ItemID")]
    pub item_id: Option<ItemId>,
    pub quantity: u32,
}

impl Offer {
    /// The first reward, which identifies single-item offers.
    pub fn first_reward(&self) -> Option<&OfferReward> {
        self.rewards.first()
    }
}
