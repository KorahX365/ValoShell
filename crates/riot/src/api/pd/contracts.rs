//! Contracts, battle pass progress and processed matches.
//! Unlock/upgrade/item-upgrade endpoints spend currency and are not implemented.

use std::collections::HashMap;

use domain::ids::{MatchId, Puuid};
use serde::Deserialize;
use serde_json::Value;

use crate::http::{Host, Request};

/// `GET /contracts/v1/contracts/{puuid}`: battle pass, agent contracts, missions, recent matches.
pub fn get_contracts(puuid: &Puuid) -> Request {
    Request::get(Host::Pd, format!("/contracts/v1/contracts/{puuid}"))
}

/// `POST /contracts/v1/contracts/{puuid}/special/{contract_id}` (no body): make this contract
/// the XP target. Free and reversible; returns the contracts.
///
/// UNVERIFIED on 13.08: behaviour after the 13.06 Agent Gear revamp is unknown.
pub fn activate_special_contract(puuid: &Puuid, contract_id: &str) -> Request {
    Request::post(
        Host::Pd,
        format!("/contracts/v1/contracts/{puuid}/special/{contract_id}"),
    )
}

/// `GET /contracts/v1/players/{puuid}/matches/{match_id}`: what one match earned.
///
/// The shape is assumed to equal a [`ProcessedMatch`] (single source); all fields tolerate absence.
pub fn get_contract_match(puuid: &Puuid, match_id: &MatchId) -> Request {
    Request::get(
        Host::Pd,
        format!("/contracts/v1/players/{puuid}/matches/{match_id}"),
    )
}

/// `GET /contracts/v1/agents/activeRecruitmentEvent`: 204 when no event runs; the body of an
/// active event is UNVERIFIED, so decode it as raw JSON.
pub fn get_active_recruitment_event() -> Request {
    Request::get(Host::Pd, "/contracts/v1/agents/activeRecruitmentEvent")
}

/// The contracts answer.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Contracts {
    pub version: i64,
    pub subject: Option<Puuid>,
    #[serde(deserialize_with = "super::null_as_default")]
    pub contracts: Vec<ContractProgress>,
    #[serde(deserialize_with = "super::null_as_default")]
    pub processed_matches: Vec<ProcessedMatch>,
    pub active_special_contract: String,
    #[serde(deserialize_with = "super::null_as_default")]
    pub missions: Vec<Mission>,
    pub mission_metadata: MissionMetadata,
}

impl Contracts {
    /// Progress of one contract definition (uuid from valorant-api.com `/v1/contracts`).
    pub fn progress_of(&self, definition_id: &str) -> Option<&ContractProgress> {
        self.contracts.iter().find(|contract| {
            contract
                .contract_definition_id
                .eq_ignore_ascii_case(definition_id)
        })
    }
}

/// Progress in one contract (battle pass, agent gear, event pass).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct ContractProgress {
    #[serde(rename = "ContractDefinitionID")]
    pub contract_definition_id: String,
    pub contract_progression: ContractProgression,
    /// Whether this is 1- or 0-based relative to the UI tier is UNVERIFIED.
    pub progression_level_reached: u32,
    pub progression_towards_next_level: i64,
}

/// Total XP and rewarded levels of a contract.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct ContractProgression {
    pub total_progression_earned: i64,
    pub total_progression_earned_version: i64,
    /// Keyed by chapter or schedule id.
    #[serde(deserialize_with = "super::null_as_default")]
    pub highest_rewarded_level: HashMap<String, RewardedLevel>,
}

/// Highest rewarded level in one chapter.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct RewardedLevel {
    pub amount: u32,
    pub version: i64,
}

/// A match Riot already counted towards contracts.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct ProcessedMatch {
    #[serde(rename = "ID")]
    pub id: Option<MatchId>,
    /// Epoch milliseconds.
    pub start_time: i64,
    #[serde(rename = "XPGrants")]
    pub xp_grants: XpGrants,
    pub reward_grants: Value,
    pub mission_deltas: Value,
    /// Keyed by contract id; `null` when no contract moved.
    #[serde(deserialize_with = "super::null_as_default")]
    pub contract_deltas: HashMap<String, ContractDelta>,
    pub could_progress_missions: bool,
}

/// Contract XP granted by one match.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct XpGrants {
    pub game_played: i64,
    pub game_won: i64,
    pub round_played: i64,
    pub round_won: i64,
    pub missions: Value,
    pub modifier: XpModifier,
    #[serde(rename = "NumAFKRounds")]
    pub num_afk_rounds: u32,
}

/// XP multiplier with its named parts (`SOCIAL_XP`, `PREMIUM_CONTRACT_XP`, ...).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct XpModifier {
    pub value: f64,
    pub base_multiplier_value: f64,
    #[serde(deserialize_with = "super::null_as_default")]
    pub modifiers: Vec<XpModifierPart>,
}

/// One multiplier part.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct XpModifierPart {
    pub value: f64,
    pub name: String,
    pub base_only: bool,
}

/// XP of a contract before and after a match.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct ContractDelta {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "TotalXPBefore")]
    pub total_xp_before: i64,
    #[serde(rename = "TotalXPAfter")]
    pub total_xp_after: i64,
}

/// An active daily/weekly mission.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Mission {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(deserialize_with = "super::null_as_default")]
    pub objectives: HashMap<String, i64>,
    pub complete: bool,
    pub expiration_time: String,
}

/// Mission bookkeeping timestamps.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct MissionMetadata {
    #[serde(rename = "NPECompleted")]
    pub npe_completed: bool,
    pub weekly_checkpoint: String,
    pub weekly_refill_time: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("fixtures/contracts.json");

    fn contracts() -> Contracts {
        serde_json::from_str(FIXTURE).expect("fixture decodes")
    }

    #[test]
    fn requests_use_the_contract_paths() {
        let puuid = Puuid::new("abc");

        assert_eq!(get_contracts(&puuid).path, "/contracts/v1/contracts/abc");
        assert_eq!(
            get_contract_match(&puuid, &MatchId::new("M-1")).path,
            "/contracts/v1/players/abc/matches/m-1"
        );
        assert_eq!(
            get_active_recruitment_event().path,
            "/contracts/v1/agents/activeRecruitmentEvent"
        );
    }

    #[test]
    fn activate_special_is_a_bodyless_post() {
        let request = activate_special_contract(&Puuid::new("abc"), "def");

        assert_eq!(request.method, reqwest::Method::POST);
        assert_eq!(request.path, "/contracts/v1/contracts/abc/special/def");
        assert!(request.body.is_none());
    }

    #[test]
    fn progress_decodes_levels_and_xp() {
        let all = contracts();

        let pass = all
            .progress_of("3F04583C-4C7A-6BDF-65CE-D4B6FF53C5E9")
            .expect("pass");

        assert_eq!(pass.progression_level_reached, 14);
        assert_eq!(pass.progression_towards_next_level, 3200);
        assert_eq!(pass.contract_progression.total_progression_earned, 123_456);
        assert_eq!(
            pass.contract_progression.highest_rewarded_level["chapter-1"].amount,
            12
        );
        assert_eq!(all.active_special_contract, "agent-contract");
    }

    #[test]
    fn processed_match_decodes_xp_and_deltas() {
        let all = contracts();

        let processed = &all.processed_matches[0];

        assert_eq!(processed.id, Some(MatchId::new("m-1")));
        assert_eq!(processed.xp_grants.game_won, 1000);
        assert_eq!(processed.xp_grants.modifier.modifiers[0].name, "SOCIAL_XP");
        assert_eq!(processed.contract_deltas["c-1"].total_xp_after, 5000);
        assert!(processed.reward_grants.is_null());
        assert!(all.processed_matches[1].contract_deltas.is_empty());
    }

    #[test]
    fn missions_decode_objectives_and_metadata() {
        let all = contracts();

        assert_eq!(all.missions[0].objectives["obj-1"], 3);
        assert!(!all.missions[0].complete);
        assert!(all.mission_metadata.npe_completed);
        assert_eq!(
            all.mission_metadata.weekly_refill_time,
            "2026-10-25T00:00:00Z"
        );
    }
}
