//! Identifier newtypes so a match id can never be passed where a puuid is expected.
//! Riot ids are lowercase hyphenated UUIDs; constructors normalise case.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

macro_rules! id_type {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
        #[serde(transparent)]
        #[ts(export, type = "string")]
        pub struct $name(String);

        impl $name {
            pub fn new(raw: impl AsRef<str>) -> Self {
                Self(raw.as_ref().trim().to_lowercase())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

id_type!(
    /// Player universally unique id.
    Puuid
);
id_type!(
    /// Match id (pregame, core-game and match-details share it).
    MatchId
);
id_type!(
    /// Party id.
    PartyId
);
id_type!(
    /// Agent (character) uuid.
    AgentId
);
id_type!(
    /// Cosmetic or content item uuid (skins, buddies, cards, sprays, titles...).
    ItemId
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_normalised_to_lowercase() {
        assert_eq!(Puuid::new(" ABC-Def ").as_str(), "abc-def");
    }
}
