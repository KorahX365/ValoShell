//! Wallet balances and currency ids.

use std::collections::HashMap;

use domain::ids::Puuid;
use serde::Deserialize;

use crate::http::{Host, Request};

/// VALORANT Points.
pub const CURRENCY_VALORANT_POINTS: &str = "85ad13f7-3d1b-5128-9eb2-7cd8ee0b5741";
/// Radianite Points.
pub const CURRENCY_RADIANITE_POINTS: &str = "e59aa87c-4cbf-517a-5983-6e81511be9b7";
/// Kingdom Credits (called "Dough" internally).
pub const CURRENCY_KINGDOM_CREDITS: &str = "85ca954a-41f2-ce94-9b45-8ca3dd39a00d";
/// Agent tokens (legacy "free agents").
pub const CURRENCY_AGENT_TOKENS: &str = "f08d4ae3-939c-4576-ab26-09ce1f23bb37";

/// `GET /store/v1/wallet/{puuid}`: balances keyed by currency id.
pub fn get_wallet(puuid: &Puuid) -> Request {
    Request::get(Host::Pd, format!("/store/v1/wallet/{puuid}"))
}

/// The wallet answer.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Wallet {
    #[serde(deserialize_with = "super::null_as_default")]
    pub balances: HashMap<String, i64>,
}

impl Wallet {
    /// Balance of one currency; 0 when the wallet has no entry for it.
    pub fn balance(&self, currency_id: &str) -> i64 {
        self.balances.get(currency_id).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wallet_request_targets_the_player() {
        let request = get_wallet(&Puuid::new("abc"));

        assert_eq!(request.host, Host::Pd);
        assert_eq!(request.path, "/store/v1/wallet/abc");
    }

    #[test]
    fn balances_decode_and_missing_currency_is_zero() {
        let json = format!(r#"{{"Balances":{{"{CURRENCY_VALORANT_POINTS}":1450}}}}"#);

        let wallet: Wallet = serde_json::from_str(&json).expect("decodes");

        assert_eq!(wallet.balance(CURRENCY_VALORANT_POINTS), 1450);
        assert_eq!(wallet.balance(CURRENCY_KINGDOM_CREDITS), 0);
    }

    #[test]
    fn currency_constants_match_the_spec_table() {
        assert_eq!(
            CURRENCY_RADIANITE_POINTS,
            "e59aa87c-4cbf-517a-5983-6e81511be9b7"
        );
        assert_eq!(
            CURRENCY_AGENT_TOKENS,
            "f08d4ae3-939c-4576-ab26-09ce1f23bb37"
        );
    }
}
