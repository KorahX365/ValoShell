//! Agent and map catalog for the UI: names, lookup keys and image urls in one call, so scenes
//! never join asset tables themselves. Images go through the `asset-cache` protocol.

use domain::static_data::{agent_by_uuid, map_by_internal_id};
use serde::Serialize;
use ts_rs::TS;

use crate::assets::{Agent, AssetIndex, GameMap, image_url};

/// One playable agent as the roster grid shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct AgentCard {
    pub uuid: String,
    pub display_name: String,
    /// Upper-case name (`KAY/O`), the stable key the UI looks agents up by.
    pub upper_name: String,
    pub role: String,
    pub icon: Option<String>,
    pub portrait: Option<String>,
    pub background: Option<String>,
    /// The agent's bio, as their page shows it.
    pub description: String,
    /// The page's tint as `#rrggbb` stops, darkest first as the API lists them.
    pub colors: Vec<String>,
    pub role_icon: Option<String>,
    /// Abilities in slot order: (name, icon).
    pub abilities: Vec<AbilityCard>,
}

/// One ability of an agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct AbilityCard {
    pub slot: String,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
}

/// One map; `upper_name` is set only for maps in the known map list (`BIND`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct MapCard {
    pub internal_id: String,
    pub display_name: String,
    pub upper_name: Option<String>,
    pub splash: Option<String>,
    pub list_icon: Option<String>,
    pub is_team_deathmatch: bool,
}

/// Everything the roster and map pickers need.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct Catalog {
    pub agents: Vec<AgentCard>,
    pub maps: Vec<MapCard>,
}

fn image(url: Option<&String>) -> Option<String> {
    url.filter(|url| !url.is_empty()).map(|url| image_url(url))
}

/// `ff4655ff` (the API's RRGGBBAA) as `#ff4655`; anything else is dropped.
fn css_color(rgba: &str) -> Option<String> {
    let rgb = rgba.get(..6)?;
    rgb.chars()
        .all(|char| char.is_ascii_hexdigit())
        .then(|| format!("#{}", rgb.to_ascii_lowercase()))
}

/// Agent row for the UI.
pub fn agent_card(agent: &Agent) -> AgentCard {
    let upper_name = agent_by_uuid(&agent.uuid).map_or_else(
        || agent.display_name.to_uppercase(),
        |entry| entry.name.to_owned(),
    );
    let role = agent.role.as_ref();
    AgentCard {
        uuid: agent.uuid.to_lowercase(),
        display_name: agent.display_name.clone(),
        upper_name,
        role: role
            .map(|role| role.display_name.clone())
            .unwrap_or_default(),
        icon: image(agent.display_icon.as_ref()),
        portrait: image(agent.full_portrait.as_ref()),
        background: image(agent.background.as_ref()),
        description: agent.description.clone().unwrap_or_default(),
        colors: agent
            .background_gradient_colors
            .iter()
            .filter_map(|color| css_color(color))
            .collect(),
        role_icon: image(role.and_then(|role| role.display_icon.as_ref())),
        abilities: agent
            .abilities
            .iter()
            .filter(|ability| ability.slot != "Passive")
            .map(|ability| AbilityCard {
                slot: ability.slot.clone(),
                name: ability.display_name.clone(),
                description: ability.description.clone().unwrap_or_default(),
                icon: image(ability.display_icon.as_ref()),
            })
            .collect(),
    }
}

/// Map row for the UI.
pub fn map_card(map: &GameMap) -> MapCard {
    let entry = map_by_internal_id(&map.internal_id);
    MapCard {
        internal_id: map.internal_id.clone(),
        display_name: map.display_name.clone(),
        upper_name: entry.map(|entry| entry.name.to_owned()),
        splash: image(map.splash.as_ref()),
        list_icon: image(map.list_view_icon.as_ref()),
        is_team_deathmatch: entry.is_some_and(|entry| entry.is_team_deathmatch()),
    }
}

/// The catalog from the current asset index: playable agents and every map with an id.
pub fn build_catalog(index: &AssetIndex) -> Catalog {
    Catalog {
        agents: index
            .agents()
            .iter()
            .filter(|agent| agent.is_playable_character)
            .map(|agent| agent_card(agent))
            .collect(),
        maps: index
            .maps()
            .iter()
            .filter(|map| !map.internal_id.is_empty())
            .map(|map| map_card(map))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_catalog_lists_every_fallback_agent_with_upper_names() {
        let catalog = build_catalog(&AssetIndex::bundled());

        assert_eq!(
            catalog.agents.len(),
            domain::static_data::FALLBACK_AGENTS.len()
        );
        let kayo = catalog
            .agents
            .iter()
            .find(|agent| agent.upper_name == "KAY/O")
            .expect("kay/o");
        assert!(kayo.icon.is_none());
    }

    #[test]
    fn bundled_maps_carry_upper_names() {
        let catalog = build_catalog(&AssetIndex::bundled());

        let bind = catalog
            .maps
            .iter()
            .find(|map| map.internal_id == "Duality")
            .expect("bind");
        assert_eq!(bind.upper_name.as_deref(), Some("BIND"));
        assert!(!bind.is_team_deathmatch);
    }

    #[test]
    fn images_are_routed_through_the_asset_protocol() {
        let agent = Agent {
            uuid: "ADD6443A-41BD-E414-F6AD-E58D267F4E95".into(),
            display_name: "Jett".into(),
            display_icon: Some("https://media.valorant-api.com/agents/x/displayicon.png".into()),
            ..Agent::default()
        };

        let card = agent_card(&agent);

        assert_eq!(card.uuid, "add6443a-41bd-e414-f6ad-e58d267f4e95");
        assert_eq!(card.upper_name, "JETT");
        assert!(card.icon.expect("icon").contains("asset-cache"));
    }
}
