//! Small valorant-api.com bodies for the tests (shapes from; nulls kept on purpose).

use super::Endpoint;
use super::index::Documents;

pub const VERSION: &str = r#"{"status":200,"data":{"manifestId":"MANIFEST-A","version":"13.08.00.5698904","riotClientVersion":"release-13.08-shipping-19-5698904","buildDate":"2026-10-14T00:00:00Z"}}"#;

pub const AGENTS: &str = r#"{"status":200,"data":[
 {"uuid":"ADD6443A-41BD-E414-F6AD-E58D267F4E95","displayName":"Jett","description":"Fast.",
  "displayIcon":"https://media.valorant-api.com/agents/add6443a/displayicon.png",
  "killfeedPortrait":"https://media.valorant-api.com/agents/add6443a/killfeedportrait.png",
  "fullPortrait":null,"isPlayableCharacter":true,"isBaseContent":true,
  "role":{"uuid":"role-1","displayName":"Duelist","displayIcon":"https://media.valorant-api.com/role.png"},
  "abilities":[{"slot":"Ability1","displayName":"Cloudburst","description":"Smoke","displayIcon":null}]},
 {"uuid":"601dbbe7-43ce-be57-2a40-4abd24953621","displayName":"KAY/O","abilities":null,"isPlayableCharacter":true}
]}"#;

pub const MAPS: &str = r#"{"status":200,"data":[
 {"uuid":"map-bind","displayName":"Bind","mapUrl":"/Game/Maps/Duality/Duality",
  "splash":"https://media.valorant-api.com/maps/bind/splash.png","listViewIcon":null,"displayIcon":null},
 {"uuid":"map-nourl","displayName":"Mystery","mapUrl":null}
]}"#;

pub const WEAPONS: &str = r#"{"status":200,"data":[
 {"uuid":"weapon-vandal","displayName":"Vandal","category":"EEquippableCategory::Rifle",
  "skins":[{"uuid":"skin-reaver"},{"uuid":"skin-standard"}]}
]}"#;

pub const SKINS: &str = r#"{"status":200,"data":[
 {"uuid":"skin-reaver","displayName":"Reaver Vandal","themeUuid":"theme-1","contentTierUuid":"tier-1",
  "chromas":[{"uuid":"CHROMA-1","displayName":"Reaver Vandal","swatch":null,
              "streamedVideo":"https://media.valorant-api.com/video/chroma1.mp4"}],
  "levels":[{"uuid":"LEVEL-1","displayName":"Reaver Vandal","displayIcon":"https://media.valorant-api.com/skinlevels/l1.png","streamedVideo":null},
            {"uuid":"level-2","displayName":"Reaver Vandal Level 2","streamedVideo":"https://media.valorant-api.com/video/l2.mp4"}]}
]}"#;

pub const BUDDIES: &str = r#"{"status":200,"data":[
 {"uuid":"buddy-1","displayName":"Ace Buddy","displayIcon":"https://media.valorant-api.com/buddies/b1.png",
  "levels":[{"uuid":"buddy-level-1","charmLevel":1,"displayName":"Ace Buddy","displayIcon":"https://media.valorant-api.com/buddies/l1.png"}]}
]}"#;

pub const PLAYER_TITLES: &str = r#"{"status":200,"data":[
 {"uuid":"title-1","displayName":"Radiant Title","titleText":"Radiant"},
 {"uuid":"title-2","displayName":"Hidden Title","titleText":null}
]}"#;

pub const LEVEL_BORDERS: &str = r#"{"status":200,"data":[
 {"uuid":"border-1","startingLevel":1},{"uuid":"border-20","startingLevel":20},{"uuid":"border-50","startingLevel":50}
]}"#;

pub const TIERS: &str = r#"{"status":200,"data":[
 {"uuid":"old","assetObjectName":"Episode1_CompetitiveTierDataTable","tiers":[{"tier":0,"tierName":"UNRANKED"}]},
 {"uuid":"new","assetObjectName":"Episode5_CompetitiveTierDataTable","tiers":[
   {"tier":0,"tierName":"UNRANKED","division":"ECompetitiveDivision::UNRANKED","divisionName":"UNRANKED","color":"ffffff00","smallIcon":null},
   {"tier":24,"tierName":"IMMORTAL 1","division":"ECompetitiveDivision::IMMORTAL","divisionName":"IMMORTAL",
    "largeIcon":"https://media.valorant-api.com/competitivetiers/new/24/largeicon.png"}]}
]}"#;

pub const CURRENCIES: &str = r#"{"status":200,"data":[
 {"uuid":"85ad13f7-3d1b-5128-9eb2-7cd8ee0b5741","displayName":"VALORANT Points","displayIcon":"https://media.valorant-api.com/currencies/vp.png"}
]}"#;

pub const QUEUES: &str = r#"{"status":200,"data":[
 {"uuid":"q1","queueId":"competitive","displayName":"Competitive",
  "displayIcon":"https://media.valorant-api.com/gamemodequeues/q1/displayicon.png",
  "listViewIconTall":"https://media.valorant-api.com/gamemodequeues/q1/listviewicontall.png"},
 {"uuid":"q2","queueId":"swiftplay","displayName":"Swiftplay"}
]}"#;

pub const GAME_MODES: &str = r#"{"status":200,"data":[
 {"uuid":"mode-bomb","displayName":"Standard",
  "assetPath":"ShooterGame/Content/GameModes/Bomb/BombGameMode_PrimaryAsset"},
 {"uuid":"mode-quickbomb","displayName":"Spike Rush",
  "assetPath":"ShooterGame/Content/GameModes/QuickBomb/QuickBombGameMode_PrimaryAsset"}
]}"#;

/// Everything above as cached documents.
pub fn documents() -> Documents {
    [
        (Endpoint::Version, VERSION),
        (Endpoint::Agents, AGENTS),
        (Endpoint::Maps, MAPS),
        (Endpoint::Weapons, WEAPONS),
        (Endpoint::Skins, SKINS),
        (Endpoint::Buddies, BUDDIES),
        (Endpoint::PlayerTitles, PLAYER_TITLES),
        (Endpoint::LevelBorders, LEVEL_BORDERS),
        (Endpoint::CompetitiveTiers, TIERS),
        (Endpoint::Currencies, CURRENCIES),
        (Endpoint::Queues, QUEUES),
        (Endpoint::GameModes, GAME_MODES),
    ]
    .into_iter()
    .map(|(endpoint, body)| (endpoint, body.to_owned()))
    .collect()
}
