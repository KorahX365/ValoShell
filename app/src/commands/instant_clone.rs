//! InstantClone stream delay widget: state reads and delay commands on this machine.

use services::instant_clone::{self, InstantCloneAction, InstantCloneState, InstantCloneTarget};

/// InstantClone's state; `None` when it is not running or refuses the token.
#[tauri::command]
pub async fn instant_clone_state(target: InstantCloneTarget) -> Option<InstantCloneState> {
    instant_clone::read_state(&target).await
}

/// Arm a delay, cut back to live, schedule a cut or drop a filling delay.
#[tauri::command]
pub async fn instant_clone_send(
    target: InstantCloneTarget,
    action: InstantCloneAction,
    delay_ms: u32,
) -> Result<(), String> {
    instant_clone::send(&target, action, delay_ms)
        .await
        .map_err(|error| error.to_string())
}
