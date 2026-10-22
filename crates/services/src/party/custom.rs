//! Custom-game lobby view: map, mode, server and who sits on which team, shaped for the
//! client's three columns. Team names were checked on a live 13.08 lobby: the player alone in
//! `teamOne` shows under Defenders, so `teamTwo` are the attackers.

use riot::api::glz::custom_game::CustomTeam;
use riot::api::glz::party_dto::CustomGameData;
use serde::Serialize;
use serde_json::Value;
use ts_rs::TS;

use crate::assets::{AssetIndex, image_url};

/// The custom lobby as the play screen shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct CustomLobbyView {
    /// English asset name ("Ascent"); empty when the map is unknown.
    pub map_name: String,
    pub map_splash: Option<String>,
    /// English asset name ("Standard"); the UI translates it like queue labels.
    pub mode_name: String,
    /// The game server's city ("Madrid"), read from the pod id.
    pub server_name: String,
    /// Puuids per column, in Riot's order.
    pub attackers: Vec<String>,
    pub defenders: Vec<String>,
    pub spectators: Vec<String>,
    pub is_autobalance_on: bool,
    pub autobalance_min_players: u32,
}

/// A column of the lobby, as the UI names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LobbyTeam {
    Attackers,
    Defenders,
    Spectators,
}

impl LobbyTeam {
    /// `attackers`, `defenders` or `spectators`; anything else is not a column.
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "attackers" => Some(Self::Attackers),
            "defenders" => Some(Self::Defenders),
            "spectators" => Some(Self::Spectators),
            _ => None,
        }
    }

    /// The slot Riot calls this column.
    pub fn riot_team(self) -> CustomTeam {
        match self {
            Self::Attackers => CustomTeam::TeamTwo,
            Self::Defenders => CustomTeam::TeamOne,
            Self::Spectators => CustomTeam::TeamSpectate,
        }
    }
}

/// The lobby view of a party in custom setup.
pub fn custom_lobby_view(data: &CustomGameData, index: &AssetIndex) -> CustomLobbyView {
    let settings = &data.settings;
    let map = index.map_by_url(&settings.map);
    CustomLobbyView {
        map_name: map
            .as_ref()
            .map(|map| map.display_name.clone())
            .unwrap_or_default(),
        map_splash: map
            .as_ref()
            .and_then(|map| map.splash.as_deref().map(image_url)),
        mode_name: index
            .game_mode_by_path(&settings.mode)
            .map(|mode| mode.display_name.clone())
            .unwrap_or_default(),
        server_name: server_name(&settings.game_pod),
        attackers: team_subjects(&data.membership, "teamTwo"),
        defenders: team_subjects(&data.membership, "teamOne"),
        spectators: team_subjects(&data.membership, "teamSpectate"),
        is_autobalance_on: data.autobalance_enabled,
        autobalance_min_players: data.autobalance_min_players,
    }
}

/// `{"teamOne": [{"Subject": "..."}], ...}` -> the puuids of one team.
fn team_subjects(membership: &Value, team: &str) -> Vec<String> {
    membership
        .get(team)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|slot| slot.get("Subject")?.as_str().map(str::to_owned))
        .collect()
}

/// `aresriot.aws-eus2-prod.eu-gp-madrid-1` -> `Madrid`: the last dotted part holds region,
/// `gp`, the city words and a pod number.
fn server_name(game_pod: &str) -> String {
    let last = game_pod.rsplit('.').next().unwrap_or_default();
    let words: Vec<&str> = last
        .split('-')
        .skip(2)
        .filter(|word| !word.chars().all(|c| c.is_ascii_digit()))
        .collect();
    words
        .iter()
        .map(|word| capitalized(word))
        .collect::<Vec<_>>()
        .join(" ")
}

fn capitalized(word: &str) -> String {
    let mut chars = word.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::{AssetSource, fixtures};
    use serde_json::json;

    fn lobby_data() -> CustomGameData {
        serde_json::from_value(json!({
            "Settings": {
                "Map": "/Game/Maps/Duality/Duality",
                "Mode": "/Game/GameModes/Bomb/BombGameMode.BombGameMode_C",
                "GamePod": "aresriot.aws-eus2-prod.eu-gp-madrid-1",
                "GameRules": null
            },
            "Membership": {
                "teamOne": [{ "Subject": "me" }],
                "teamTwo": [{ "Subject": "a" }, { "Subject": "b" }],
                "teamSpectate": []
            },
            "AutobalanceEnabled": true,
            "AutobalanceMinPlayers": 6
        }))
        .unwrap()
    }

    #[test]
    fn a_live_lobby_reads_as_map_mode_server_and_columns() {
        let index = AssetIndex::from_documents(&fixtures::documents(), AssetSource::Cache);

        let view = custom_lobby_view(&lobby_data(), &index);

        assert_eq!(view.map_name, "Bind");
        assert_eq!(view.mode_name, "Standard");
        assert_eq!(view.server_name, "Madrid");
        assert_eq!(view.defenders, ["me"]);
        assert_eq!(view.attackers, ["a", "b"]);
        assert!(view.spectators.is_empty());
        assert_eq!(
            (view.is_autobalance_on, view.autobalance_min_players),
            (true, 6)
        );
    }

    #[test]
    fn unknown_map_and_mode_read_as_empty_names() {
        let mut data = lobby_data();
        data.settings.map = "/Game/Maps/Nowhere/Nowhere".into();
        data.settings.mode = "garbage".into();

        let view = custom_lobby_view(&data, &AssetIndex::bundled());

        assert_eq!((view.map_name.as_str(), view.mode_name.as_str()), ("", ""));
        assert_eq!(view.map_splash, None);
    }

    #[test]
    fn pod_ids_name_their_city() {
        assert_eq!(
            server_name("aresriot.aws-eus2-prod.eu-gp-madrid-1"),
            "Madrid"
        );
        assert_eq!(
            server_name("aresriot.aws-usw2-prod.na-gp-north-california-1"),
            "North California"
        );
        assert_eq!(server_name(""), "");
    }

    #[test]
    fn only_the_three_columns_parse() {
        assert_eq!(
            LobbyTeam::parse("defenders").map(LobbyTeam::riot_team),
            Some(CustomTeam::TeamOne)
        );
        assert_eq!(
            LobbyTeam::parse("attackers").map(LobbyTeam::riot_team),
            Some(CustomTeam::TeamTwo)
        );
        assert_eq!(LobbyTeam::parse("coaches"), None);
    }
}
