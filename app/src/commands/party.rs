//! Party lobby commands: thin wrappers over `PartyService`.

use services::party::{PartyService, PartyView};
use tauri::State;

type CommandResult<T = ()> = Result<T, String>;

/// Last published party (the UI's initial read).
#[tauri::command]
pub fn get_party(party: State<'_, PartyService>) -> Option<PartyView> {
    party.current()
}

/// Fetch the party now.
#[tauri::command]
pub async fn party_refresh(party: State<'_, PartyService>) -> CommandResult<Option<PartyView>> {
    party.refresh().await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn party_change_queue(party: State<'_, PartyService>, queue_id: String) -> CommandResult {
    party
        .change_queue(&queue_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn party_start_queue(party: State<'_, PartyService>) -> CommandResult {
    party.start_queue().await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn party_stop_queue(party: State<'_, PartyService>) -> CommandResult {
    party.stop_queue().await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn party_set_ready(party: State<'_, PartyService>, is_ready: bool) -> CommandResult {
    party
        .set_ready(is_ready)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn party_set_open(party: State<'_, PartyService>, is_open: bool) -> CommandResult {
    party
        .set_open(is_open)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn party_invite(party: State<'_, PartyService>, riot_id: String) -> CommandResult {
    party
        .invite(&riot_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn party_kick(party: State<'_, PartyService>, puuid: String) -> CommandResult {
    party.kick(&puuid).await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn party_leave(party: State<'_, PartyService>) -> CommandResult {
    party.leave().await.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn party_generate_code(party: State<'_, PartyService>) -> CommandResult {
    party
        .generate_code()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn party_join_code(party: State<'_, PartyService>, code: String) -> CommandResult {
    party
        .join_by_code(&code)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn party_change_team(
    party: State<'_, PartyService>,
    team: String,
    puuid: String,
) -> CommandResult {
    party
        .change_team(&team, &puuid)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn party_start_custom(party: State<'_, PartyService>) -> CommandResult {
    party
        .start_custom_game()
        .await
        .map_err(|error| error.to_string())
}
