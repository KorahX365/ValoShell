//! Agent Mastery ("bloomline", spec 07 K7). Every agent is a "bonsai" whose `Flourish` holds
//! the act level (`ShortLevel`) and the lifetime level (`LongLevel`). Zero-valued fields are
//! omitted by Riot, so every field defaults. Answers an empty list, not an error, when
//! `X-Riot-ClientVersion` is stale.

use domain::ids::Puuid;
use serde::Deserialize;

use crate::http::{Host, Request};

/// `GET /bloomline/v0/players/{puuid}`: mastery of every agent the player has played.
pub fn get_bloomline_player(puuid: &Puuid) -> Request {
    Request::get(Host::Pd, format!("/bloomline/v0/players/{puuid}"))
}

/// The bloomline player answer.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct BloomlinePlayer {
    #[serde(deserialize_with = "super::null_as_default")]
    pub bonsais: Vec<Bonsai>,
}

/// One agent's mastery.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Bonsai {
    /// The agent uuid as 32 upper-case hex digits without dashes.
    #[serde(rename = "BonsaiID")]
    pub bonsai_id: String,
    pub flourish: Flourish,
}

impl Bonsai {
    /// The agent uuid in the usual lower-case dashed form.
    pub fn agent_uuid(&self) -> String {
        dashed_uuid(&self.bonsai_id)
    }
}

/// Mastery levels and points.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Flourish {
    /// Act level; resets every act.
    pub short_level: u32,
    /// Lifetime level; never resets.
    pub long_level: u32,
    pub points: i64,
}

/// `9F0D8BA94140B94157D3A7AD57C6B417` as `9f0d8ba9-4140-b941-57d3-a7ad57c6b417`; ids that are
/// already dashed (or malformed) only get lower-cased.
fn dashed_uuid(id: &str) -> String {
    let lower = id.to_ascii_lowercase();
    if lower.len() != 32 || !lower.chars().all(|char| char.is_ascii_hexdigit()) {
        return lower;
    }
    format!(
        "{}-{}-{}-{}-{}",
        &lower[..8],
        &lower[8..12],
        &lower[12..16],
        &lower[16..20],
        &lower[20..]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_request_targets_the_pd_host() {
        let request = get_bloomline_player(&Puuid::new("abc"));

        assert_eq!(request.host, Host::Pd);
        assert_eq!(request.path, "/bloomline/v0/players/abc");
    }

    #[test]
    fn bonsais_decode_with_omitted_zero_fields() {
        let body = r#"{"Subject":"abc","Bonsais":[
            {"BonsaiID":"9F0D8BA94140B94157D3A7AD57C6B417","Flourish":{"ShortLevel":3,"LongLevel":12}},
            {"BonsaiID":"ADD6443A41BDE414F6ADE58D267F4E95"}]}"#;

        let player: BloomlinePlayer = serde_json::from_str(body).expect("decodes");

        assert_eq!(player.bonsais.len(), 2);
        assert_eq!(player.bonsais[0].flourish.long_level, 12);
        assert_eq!(player.bonsais[1].flourish.short_level, 0);
    }

    #[test]
    fn null_bonsais_decode_as_empty() {
        let player: BloomlinePlayer = serde_json::from_str(r#"{"Bonsais":null}"#).unwrap();
        assert!(player.bonsais.is_empty());
    }

    #[test]
    fn bonsai_ids_become_dashed_agent_uuids() {
        let bonsai = Bonsai {
            bonsai_id: "9F0D8BA94140B94157D3A7AD57C6B417".into(),
            ..Bonsai::default()
        };

        assert_eq!(bonsai.agent_uuid(), "9f0d8ba9-4140-b941-57d3-a7ad57c6b417");
        assert_eq!(dashed_uuid("ADD6443A-41BD"), "add6443a-41bd");
    }
}
