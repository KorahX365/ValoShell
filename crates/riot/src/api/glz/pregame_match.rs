//! Pregame match DTO.

use domain::ids::{AgentId, MatchId, Puuid};
use serde::Deserialize;
use serde_json::Value;

use super::common::{PlayerIdentity, SeasonalBadgeInfo, SecretString, null_as_default};

const NANOS_PER_SECOND: f64 = 1_000_000_000.0;

/// `CharacterSelectionState` value once an agent is locked.
const LOCKED_STATE: &str = "locked";

/// One player in agent select.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct PregamePlayer {
    pub subject: Option<Puuid>,
    #[serde(rename = "CharacterID")]
    pub character_id: Option<AgentId>,
    /// `""`, `selected` or `locked`.
    pub character_selection_state: String,
    pub pregame_player_state: String,
    pub competitive_tier: u32,
    #[serde(deserialize_with = "null_as_default")]
    pub player_identity: PlayerIdentity,
    pub seasonal_badge_info: Option<SeasonalBadgeInfo>,
    pub is_captain: bool,
}

impl PregamePlayer {
    /// True once the player locked an agent.
    pub fn is_locked(&self) -> bool {
        self.character_selection_state == LOCKED_STATE
    }
}

/// A team in agent select; `TeamID` is `Blue`/`Red` (or a puuid in deathmatch).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct PregameTeam {
    #[serde(rename = "TeamID")]
    pub team_id: String,
    #[serde(deserialize_with = "null_as_default")]
    pub players: Vec<PregamePlayer>,
}

/// The agent-select match; the official client polls it about once per second.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct PregameMatch {
    #[serde(rename = "ID", alias = "MatchID")]
    pub id: Option<MatchId>,
    pub version: u64,
    #[serde(deserialize_with = "null_as_default")]
    pub teams: Vec<PregameTeam>,
    pub ally_team: Option<PregameTeam>,
    /// Null for almost all of agent select; filled as enemies lock.
    pub enemy_team: Option<PregameTeam>,
    #[serde(deserialize_with = "null_as_default")]
    pub observer_subjects: Vec<Puuid>,
    #[serde(deserialize_with = "null_as_default")]
    pub match_coaches: Vec<Value>,
    pub enemy_team_size: u32,
    pub enemy_team_lock_count: u32,
    /// `character_select_active` or `provisioned`; other values are unverified.
    pub pregame_state: String,
    pub last_updated: String,
    #[serde(rename = "MapID")]
    pub map_id: String,
    #[serde(rename = "ModeID")]
    pub mode_id: String,
    /// Mode asset path, e.g. `/Game/GameModes/Bomb/BombGameMode.BombGameMode_C`.
    pub mode: String,
    #[serde(rename = "QueueID")]
    pub queue_id: String,
    #[serde(rename = "GamePodID")]
    pub game_pod_id: String,
    /// `Matchmaking` or `CustomGame`.
    #[serde(rename = "ProvisioningFlowID")]
    pub provisioning_flow_id: String,
    pub is_ranked: bool,
    #[serde(rename = "VoiceSessionID")]
    pub voice_session_id: String,
    #[serde(rename = "MUCName")]
    pub muc_name: String,
    pub team_match_token: SecretString,
    /// Map-vote queues only; shapes are unverified so they stay raw.
    #[serde(deserialize_with = "null_as_default")]
    pub map_select_pool: Vec<Value>,
    #[serde(rename = "BannedMapIDs", deserialize_with = "null_as_default")]
    pub banned_map_ids: Vec<Value>,
    #[serde(rename = "PhaseTimeRemainingNS")]
    pub phase_time_remaining_ns: u64,
    #[serde(rename = "StepTimeRemainingNS")]
    pub step_time_remaining_ns: u64,
}

impl PregameMatch {
    /// Seconds left in the current phase (Riot reports nanoseconds).
    pub fn phase_time_remaining_seconds(&self) -> f64 {
        self.phase_time_remaining_ns as f64 / NANOS_PER_SECOND
    }

    /// The caller's row in the ally team, used to confirm a lock.
    pub fn ally_player(&self, puuid: &Puuid) -> Option<&PregamePlayer> {
        let team = self.ally_team.as_ref()?;
        team.players
            .iter()
            .find(|player| player.subject.as_ref() == Some(puuid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("fixtures/pregame_match.json");
    const ME: &str = "b47236a3-0000-0000-0000-000000000001";

    fn pregame() -> PregameMatch {
        serde_json::from_str(FIXTURE).unwrap()
    }

    #[test]
    fn fixture_decodes_match_header() {
        let pregame = pregame();
        assert_eq!(
            pregame.id.unwrap().as_str(),
            "6e69661e-a009-4e89-b697-6fd442cd7277"
        );
        assert_eq!(pregame.map_id, "/Game/Maps/Ascent/Ascent");
        assert_eq!(
            pregame.mode,
            "/Game/GameModes/Bomb/BombGameMode.BombGameMode_C"
        );
        assert_eq!(pregame.queue_id, "competitive");
        assert_eq!(pregame.pregame_state, "character_select_active");
        assert_eq!(pregame.provisioning_flow_id, "Matchmaking");
        assert!(pregame.is_ranked && pregame.enemy_team.is_none());
    }

    #[test]
    fn fixture_decodes_ally_players() {
        let pregame = pregame();
        let ally = pregame.ally_team.as_ref().unwrap();
        assert_eq!(ally.team_id, "Blue");
        assert_eq!(ally.players.len(), 2);
        let me = pregame.ally_player(&Puuid::new(ME)).unwrap();
        assert!(me.is_locked() && me.is_captain);
        assert_eq!(
            me.character_id.as_ref().unwrap().as_str(),
            "add6443a-41bd-e414-f6ad-e58d267f4e95"
        );
        assert_eq!(me.player_identity.account_level, 29);
        assert!(!pregame.ally_team.as_ref().unwrap().players[1].is_locked());
    }

    #[test]
    fn phase_timer_converts_nanoseconds() {
        let pregame = pregame();
        assert!((pregame.phase_time_remaining_seconds() - 71.950_865_688).abs() < 1e-9);
    }

    #[test]
    fn unknown_player_is_not_found_and_team_token_is_hidden() {
        let pregame = pregame();
        assert!(pregame.ally_player(&Puuid::new("nobody")).is_none());
        assert!(!format!("{pregame:?}").contains("team.jwt.secret"));
        assert_eq!(pregame.team_match_token.expose(), "team.jwt.secret");
    }

    #[test]
    fn match_id_alias_and_null_teams_decode() {
        let body = r#"{"MatchID":"abc","Teams":null,"AllyTeam":null}"#;
        let pregame: PregameMatch = serde_json::from_str(body).unwrap();
        assert_eq!(pregame.id.as_ref().unwrap().as_str(), "abc");
        assert!(pregame.teams.is_empty());
        assert!(pregame.ally_player(&Puuid::new(ME)).is_none());
    }
}
