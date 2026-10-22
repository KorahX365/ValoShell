//! Forwards every backend `AppEvent` to the webviews as the `app-event` Tauri event, and
//! remembers the last connection status for windows that open later.

use std::sync::{Mutex, PoisonError};

use services::{AppEvent, ConnectionStatus};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::error::RecvError;

/// Name of the Tauri event the UI listens to.
pub const APP_EVENT: &str = "app-event";

/// Last connection status published (managed state).
pub struct LastConnection(Mutex<ConnectionStatus>);

impl Default for LastConnection {
    fn default() -> Self {
        Self(Mutex::new(ConnectionStatus::WaitingForRiot))
    }
}

impl LastConnection {
    pub fn get(&self) -> ConnectionStatus {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn set(&self, status: ConnectionStatus) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = status;
    }
}

/// Run until the bus closes. Lagging (the UI was busy) skips old events instead of failing.
pub async fn forward_events(app: AppHandle, mut events: Receiver<AppEvent>) {
    loop {
        match events.recv().await {
            Ok(event) => {
                if let AppEvent::Connection(status) = &event
                    && let Some(last) = app.try_state::<LastConnection>()
                {
                    last.set(status.clone());
                }
                if let Err(error) = app.emit(APP_EVENT, &event) {
                    tracing::warn!(%error, "app_event_emit_failed");
                }
            }
            Err(RecvError::Lagged(skipped)) => tracing::debug!(skipped, "app_event_lagged"),
            Err(RecvError::Closed) => return,
        }
    }
}
