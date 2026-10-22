//! Featured store bundles.

use domain::ids::ItemId;
use serde::Deserialize;

use super::offer::{Cost, Offer};

/// Bundles on sale and the time left.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct FeaturedBundle {
    pub bundle: Option<Bundle>,
    #[serde(deserialize_with = "super::null_as_default")]
    pub bundles: Vec<Bundle>,
    pub bundle_remaining_duration_in_seconds: i64,
}

/// One bundle with per-item and total prices.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Bundle {
    #[serde(rename = "ID")]
    pub id: String,
    /// Believed to equal the valorant-api.com bundle uuid (UNVERIFIED).
    #[serde(rename = "DataAssetID")]
    pub data_asset_id: String,
    #[serde(rename = "CurrencyID")]
    pub currency_id: String,
    #[serde(deserialize_with = "super::null_as_default")]
    pub items: Vec<BundleItem>,
    #[serde(deserialize_with = "super::null_as_default")]
    pub item_offers: Vec<BundleItemOffer>,
    #[serde(deserialize_with = "super::null_as_default")]
    pub total_base_cost: Cost,
    #[serde(deserialize_with = "super::null_as_default")]
    pub total_discounted_cost: Cost,
    /// A fraction (0.33) in bundles, unlike the night market's integer percent.
    pub total_discount_percent: f64,
    pub duration_remaining_in_seconds: i64,
    pub wholesale_only: bool,
}

/// An item inside a bundle with its price.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct BundleItem {
    pub item: BundleItemRef,
    pub base_price: i64,
    #[serde(rename = "CurrencyID")]
    pub currency_id: String,
    pub discount_percent: f64,
    pub discounted_price: i64,
    pub is_promo_item: bool,
}

/// The item a bundle line grants.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct BundleItemRef {
    #[serde(rename = "ItemTypeID")]
    pub item_type_id: String,
    #[serde(rename = "ItemID")]
    pub item_id: Option<ItemId>,
    pub amount: u32,
}

/// The offer behind a bundle line.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct BundleItemOffer {
    #[serde(rename = "BundleItemOfferID")]
    pub bundle_item_offer_id: String,
    pub offer: Offer,
    pub discount_percent: f64,
    #[serde(deserialize_with = "super::null_as_default")]
    pub discounted_cost: Cost,
}
