//! Storefront v3 and the agent store. View only: nothing here buys.

use domain::ids::{AgentId, ItemId};
use serde::Deserialize;
use serde_json::{Map, Value};

use super::bundle::FeaturedBundle;
use super::offer::Offer;
use domain::ids::Puuid;

use crate::http::{Host, Request};

/// `POST /store/v3/storefront/{puuid}`: daily offers, bundles, accessories, night market.
///
/// The body must be an explicit `{}`: an empty payload reportedly returns nothing.
pub fn post_storefront(puuid: &Puuid) -> Request {
    Request::post(Host::Pd, format!("/store/v3/storefront/{puuid}")).json(Value::Object(Map::new()))
}

/// `GET /store/v1/storefronts/agent`: the agent store (shape from a single source).
pub fn get_agent_storefront() -> Request {
    Request::get(Host::Pd, "/store/v1/storefronts/agent")
}

/// The whole storefront. Cache it until [`Storefront::shortest_remaining_seconds`] elapses.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Storefront {
    pub featured_bundle: FeaturedBundle,
    pub skins_panel_layout: SkinsPanelLayout,
    pub upgrade_currency_store: UpgradeCurrencyStore,
    pub accessory_store: AccessoryStore,
    /// Optional, v3 only, shape unknown.
    #[serde(deserialize_with = "super::null_as_default")]
    pub plugin_stores: Vec<Value>,
    /// Absent unless a night market is live.
    pub bonus_store: Option<BonusStore>,
}

impl Storefront {
    /// `true` while a night market is live.
    pub fn has_night_market(&self) -> bool {
        self.bonus_store.is_some()
    }

    /// Seconds until the first section expires (positive durations only).
    pub fn shortest_remaining_seconds(&self) -> Option<i64> {
        [
            Some(
                self.skins_panel_layout
                    .single_item_offers_remaining_duration_in_seconds,
            ),
            Some(
                self.accessory_store
                    .accessory_store_remaining_duration_in_seconds,
            ),
            Some(self.featured_bundle.bundle_remaining_duration_in_seconds),
            self.bonus_store
                .as_ref()
                .map(|store| store.bonus_store_remaining_duration_in_seconds),
        ]
        .into_iter()
        .flatten()
        .filter(|seconds| *seconds > 0)
        .min()
    }
}

/// The four daily skin offers.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct SkinsPanelLayout {
    /// Skin-level uuids.
    #[serde(deserialize_with = "super::null_as_default")]
    pub single_item_offers: Vec<ItemId>,
    #[serde(deserialize_with = "super::null_as_default")]
    pub single_item_store_offers: Vec<Offer>,
    pub single_item_offers_remaining_duration_in_seconds: i64,
}

/// Radianite offers.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct UpgradeCurrencyStore {
    #[serde(deserialize_with = "super::null_as_default")]
    pub upgrade_currency_offers: Vec<UpgradeCurrencyOffer>,
}

/// One Radianite offer.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct UpgradeCurrencyOffer {
    #[serde(rename = "OfferID")]
    pub offer_id: String,
    #[serde(rename = "StorefrontItemID")]
    pub storefront_item_id: String,
    pub offer: Offer,
    pub discounted_percent: f64,
}

/// Rotating accessories (cards, sprays, buddies, titles).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct AccessoryStore {
    #[serde(deserialize_with = "super::null_as_default")]
    pub accessory_store_offers: Vec<AccessoryOffer>,
    pub accessory_store_remaining_duration_in_seconds: i64,
    #[serde(rename = "StorefrontID")]
    pub storefront_id: String,
}

/// An accessory and the pass or event it belongs to.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct AccessoryOffer {
    pub offer: Offer,
    #[serde(rename = "ContractID")]
    pub contract_id: String,
}

/// The night market.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct BonusStore {
    #[serde(deserialize_with = "super::null_as_default")]
    pub bonus_store_offers: Vec<BonusOffer>,
    pub bonus_store_remaining_duration_in_seconds: i64,
}

/// A discounted night-market skin.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct BonusOffer {
    #[serde(rename = "BonusOfferID")]
    pub bonus_offer_id: String,
    pub offer: Offer,
    /// Integer percent here (30), unlike the fraction used by bundles.
    pub discount_percent: f64,
    #[serde(deserialize_with = "super::null_as_default")]
    pub discount_costs: super::offer::Cost,
    pub is_seen: bool,
}

/// Agent store answer.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct AgentStorefront {
    pub agent_store: AgentStore,
}

/// Agents currently on sale.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct AgentStore {
    #[serde(deserialize_with = "super::null_as_default")]
    pub agent_store_offers: Vec<AgentStoreOffer>,
    /// Shapes not verified, kept raw.
    pub current_featured_agent: Value,
    pub next_featured_agent: Value,
}

/// Offers for one agent.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct AgentStoreOffer {
    #[serde(rename = "AgentID")]
    pub agent_id: Option<AgentId>,
    #[serde(deserialize_with = "super::null_as_default")]
    pub store_offers: Vec<Offer>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("fixtures/storefront.json");

    fn storefront() -> Storefront {
        serde_json::from_str(FIXTURE).expect("fixture decodes")
    }

    #[test]
    fn storefront_request_posts_an_explicit_empty_object() {
        let request = post_storefront(&Puuid::new("abc"));

        assert_eq!(request.method, reqwest::Method::POST);
        assert_eq!(request.host, Host::Pd);
        assert_eq!(request.path, "/store/v3/storefront/abc");
        assert_eq!(request.body, Some(serde_json::json!({})));
    }

    #[test]
    fn agent_storefront_request_is_a_get() {
        let request = get_agent_storefront();

        assert_eq!(request.method, reqwest::Method::GET);
        assert_eq!(request.path, "/store/v1/storefronts/agent");
    }

    #[test]
    fn daily_offers_decode_with_prices() {
        let store = storefront();

        let layout = &store.skins_panel_layout;
        assert_eq!(layout.single_item_offers.len(), 2);
        let offer = &layout.single_item_store_offers[0];
        assert_eq!(offer.cost["85ad13f7-3d1b-5128-9eb2-7cd8ee0b5741"], 1775);
        assert_eq!(
            offer.first_reward().expect("reward").item_type_id,
            "e7c63390-eda7-46e0-bb7a-a6abdacd2433"
        );
    }

    #[test]
    fn bundle_decodes_items_and_totals() {
        let store = storefront();

        let bundle = store.featured_bundle.bundle.as_ref().expect("bundle");
        assert_eq!(bundle.items[0].discounted_price, 1793);
        assert!((bundle.total_discount_percent - 0.33).abs() < f64::EPSILON);
        assert_eq!(bundle.item_offers[0].discounted_cost.len(), 1);
        assert_eq!(store.featured_bundle.bundles.len(), 1);
    }

    #[test]
    fn night_market_is_optional_and_exposes_discounts() {
        let with_market = storefront();
        let without_market: Storefront = serde_json::from_str("{}").expect("decodes");

        assert!(with_market.has_night_market());
        let offer = &with_market.bonus_store.expect("bonus").bonus_store_offers[0];
        assert_eq!(offer.discount_percent, 30.0);
        assert!(!offer.is_seen);
        assert!(!without_market.has_night_market());
    }

    #[test]
    fn shortest_remaining_picks_the_earliest_expiry() {
        assert_eq!(storefront().shortest_remaining_seconds(), Some(40_000));
    }

    #[test]
    fn accessory_store_decodes_contract_ids() {
        let store = storefront();

        let accessory = &store.accessory_store.accessory_store_offers[0];
        assert_eq!(accessory.contract_id, "contract-1");
        assert_eq!(
            store
                .accessory_store
                .accessory_store_remaining_duration_in_seconds,
            500_000
        );
    }

    #[test]
    fn agent_storefront_decodes_offers() {
        let json = r#"{"AgentStore":{"AgentStoreOffers":[{"AgentID":"A-1",
            "StoreOffers":[{"OfferID":"o","Cost":{"c":8000},"Rewards":[]}]}],
            "CurrentFeaturedAgent":"a","NextFeaturedAgent":null}}"#;

        let store: AgentStorefront = serde_json::from_str(json).expect("decodes");

        let offer = &store.agent_store.agent_store_offers[0];
        assert_eq!(offer.store_offers[0].cost["c"], 8000);
        assert!(offer.agent_id.is_some());
    }
}
