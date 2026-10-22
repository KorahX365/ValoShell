//! Region and shard resolution and the three remote base URLs.

use serde::Serialize;
use ts_rs::TS;

/// Regions the Riot services accept.
pub const REGIONS: [&str; 7] = ["na", "eu", "latam", "br", "ap", "kr", "pbe"];

/// Where the player's remote services live: `region` picks the GLZ cluster, `shard` the rest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct Region {
    pub region: String,
    pub shard: String,
}

impl Region {
    /// Resolve a region id; `None` for anything outside `REGIONS` (never a crash).
    ///
    /// latam and br live on the na shard; pbe is its own shard served by the na GLZ cluster.
    pub fn resolve(raw: &str) -> Option<Self> {
        let input = raw.trim().to_lowercase();
        if !REGIONS.contains(&input.as_str()) {
            return None;
        }
        let (region, shard) = match input.as_str() {
            "latam" | "br" => (input.clone(), "na".to_owned()),
            "pbe" => ("na".to_owned(), "pbe".to_owned()),
            _ => (input.clone(), input),
        };
        Some(Self { region, shard })
    }

    /// Player data host: MMR, history, loadout, store, contracts, names.
    pub fn pd_url(&self) -> String {
        format!("https://pd.{}.a.pvp.net", self.shard)
    }

    /// Game-loop host: parties, pregame, core-game.
    pub fn glz_url(&self) -> String {
        format!("https://glz-{}-1.{}.a.pvp.net", self.region, self.shard)
    }

    /// Shared host: content service and config.
    pub fn shared_url(&self) -> String {
        format!("https://shared.{}.a.pvp.net", self.shard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn urls(raw: &str) -> (String, String, String) {
        let region = Region::resolve(raw).expect("known region");
        (region.pd_url(), region.glz_url(), region.shared_url())
    }

    #[test]
    fn eu_uses_its_own_shard() {
        assert_eq!(
            urls("eu"),
            (
                "https://pd.eu.a.pvp.net".to_owned(),
                "https://glz-eu-1.eu.a.pvp.net".to_owned(),
                "https://shared.eu.a.pvp.net".to_owned()
            )
        );
    }

    #[test]
    fn latam_and_br_live_on_the_na_shard() {
        assert_eq!(urls("latam").1, "https://glz-latam-1.na.a.pvp.net");
        assert_eq!(urls("br").0, "https://pd.na.a.pvp.net");
    }

    #[test]
    fn pbe_is_its_own_shard_on_the_na_cluster() {
        assert_eq!(urls("pbe").1, "https://glz-na-1.pbe.a.pvp.net");
    }

    #[test]
    fn unknown_region_is_none_not_a_crash() {
        assert_eq!(Region::resolve("mars"), None);
        assert_eq!(Region::resolve(""), None);
    }

    #[test]
    fn input_is_trimmed_and_case_insensitive() {
        assert_eq!(
            Region::resolve(" EU ").map(|region| region.shard),
            Some("eu".to_owned())
        );
    }
}
