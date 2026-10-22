//! Agent-select commands: the player's own picks.

use services::pregame::{PregameService, PregameView};
use tauri::State;

type CommandResult<T = ()> = Result<T, String>;

/// Last published agent-select view.
#[tauri::command]
pub fn get_pregame(pregame: State<'_, PregameService>) -> Option<PregameView> {
    pregame.current()
}

#[tauri::command]
pub async fn pregame_select(
    pregame: State<'_, PregameService>,
    agent_uuid: String,
) -> CommandResult {
    pregame
        .select(&agent_uuid)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn pregame_lock(pregame: State<'_, PregameService>, agent_uuid: String) -> CommandResult {
    pregame
        .lock(&agent_uuid)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn pregame_dodge(pregame: State<'_, PregameService>) -> CommandResult {
    pregame.dodge().await.map_err(|error| error.to_string())
}
