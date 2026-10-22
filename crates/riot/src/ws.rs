//! Local Riot Client websocket: subscribe to `OnJsonApiEvent`, classify each event
//! by URI and forward it. Reconnects after 2/4/6/8 s and gives up after 5 consecutive failures,
//! or at once when the lockfile no longer matches (client closed or restarted).
//! Debouncing is the consumer's job: every event is forwarded so none is silently lost.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{Connector, connect_async_tls_with_config};
use tokio_util::sync::CancellationToken;

use crate::api::local::events::{
    JsonApiEvent, UriKind, classify_uri, parse_event_frame, subscribe_frame,
};
use crate::lockfile::{Lockfile, read_lockfile};
use crate::tls;

const MAX_RECONNECT_ATTEMPTS: u32 = 5;
const RECONNECT_DELAY_STEP: Duration = Duration::from_secs(2);
/// Receive timeout used as a liveness tick, so cancellation is noticed promptly.
const RECEIVE_TICK: Duration = Duration::from_secs(30);
const EVENT_BUFFER: usize = 256;

/// One classified event from the Riot Client.
#[derive(Debug, Clone)]
pub struct RiotEvent {
    pub kind: UriKind,
    pub event: JsonApiEvent,
}

/// Why the websocket task ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WsEnd {
    /// `CancellationToken` fired (app shutdown or session replaced).
    Stopped,
    /// The Riot Client is gone, or reconnecting failed 5 times in a row.
    Lost,
}

/// Start the websocket task. Events arrive on the returned receiver; the task's result says
/// how it ended. Cancel `stop` to end it deliberately.
pub fn spawn_websocket(
    lockfile: Lockfile,
    stop: CancellationToken,
) -> (mpsc::Receiver<RiotEvent>, tokio::task::JoinHandle<WsEnd>) {
    let (sender, receiver) = mpsc::channel(EVENT_BUFFER);
    let task = tokio::spawn(run_with_reconnect(lockfile, sender, stop));
    (receiver, task)
}

async fn run_with_reconnect(
    lockfile: Lockfile,
    sender: mpsc::Sender<RiotEvent>,
    stop: CancellationToken,
) -> WsEnd {
    let mut failures = 0;
    loop {
        let outcome = tokio::select! {
            () = stop.cancelled() => return WsEnd::Stopped,
            outcome = connect_and_listen(&lockfile, &sender, &mut failures) => outcome,
        };
        if sender.is_closed() {
            return WsEnd::Stopped;
        }
        failures += 1;
        if failures >= MAX_RECONNECT_ATTEMPTS || !is_same_client(&lockfile).await {
            tracing::warn!(failures, reason = %outcome.reason, "riot_ws_lost");
            return WsEnd::Lost;
        }
        let delay = RECONNECT_DELAY_STEP * failures;
        tracing::warn!(failures, delay_s = delay.as_secs(), reason = %outcome.reason, "riot_ws_reconnecting");
        tokio::select! {
            () = stop.cancelled() => return WsEnd::Stopped,
            () = tokio::time::sleep(delay) => {}
        }
    }
}

struct Disconnect {
    reason: String,
}

impl Disconnect {
    fn new(reason: impl std::fmt::Display) -> Self {
        Self {
            reason: reason.to_string(),
        }
    }
}

/// The client that wrote `lockfile` still runs: a restart writes a new port and password, so
/// reconnecting with the old ones can only fail.
async fn is_same_client(lockfile: &Lockfile) -> bool {
    read_lockfile()
        .await
        .is_ok_and(|current| current == *lockfile)
}

/// One connection's lifetime. Resets `failures` once connected (the budget is consecutive).
async fn connect_and_listen(
    lockfile: &Lockfile,
    sender: &mpsc::Sender<RiotEvent>,
    failures: &mut u32,
) -> Disconnect {
    let mut socket = match open_socket(lockfile).await {
        Ok(socket) => socket,
        Err(disconnect) => return disconnect,
    };
    *failures = 0;
    tracing::info!("riot_ws_connected");
    if let Err(error) = socket.send(Message::text(subscribe_frame())).await {
        return Disconnect::new(error);
    }
    loop {
        let message = match tokio::time::timeout(RECEIVE_TICK, socket.next()).await {
            Err(_elapsed) => continue,
            Ok(None) => return Disconnect::new("closed by the Riot Client"),
            Ok(Some(Err(error))) => return Disconnect::new(error),
            Ok(Some(Ok(message))) => message,
        };
        if let Some(event) = classify(&message)
            && sender.send(event).await.is_err()
        {
            return Disconnect::new("consumer dropped");
        }
    }
}

type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn open_socket(lockfile: &Lockfile) -> Result<Socket, Disconnect> {
    let config = tls::local_client_config().map_err(Disconnect::new)?;
    let mut request = format!("wss://127.0.0.1:{}", lockfile.port)
        .into_client_request()
        .map_err(Disconnect::new)?;
    let authorization = crate::http::basic_authorization(&lockfile.password)
        .parse()
        .map_err(Disconnect::new)?;
    request.headers_mut().insert("Authorization", authorization);
    let connector = Some(Connector::Rustls(config));
    match connect_async_tls_with_config(request, None, false, connector).await {
        Ok((socket, _response)) => Ok(socket),
        Err(error) => Err(Disconnect::new(error)),
    }
}

/// Text frames that are JSON API events with a URI we care about.
fn classify(message: &Message) -> Option<RiotEvent> {
    let Message::Text(text) = message else {
        return None;
    };
    let event = parse_event_frame(text.as_str())?;
    let kind = classify_uri(&event.uri)?;
    Some(RiotEvent { kind, event })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pregame_event_frames_are_classified() {
        let frame = r#"[8,"OnJsonApiEvent",{"uri":"/riot-messaging-service/v1/message/ares-pregame/pregame/v1/matches/abc","eventType":"Update","data":{}}]"#;
        let event = classify(&Message::text(frame)).expect("classified");
        assert_eq!(event.kind, UriKind::Pregame);
    }

    #[test]
    fn binary_and_unknown_frames_are_ignored() {
        assert!(classify(&Message::binary(vec![1, 2, 3])).is_none());
        assert!(
            classify(&Message::text(
                "[8,\"OnJsonApiEvent\",{\"uri\":\"/nothing\"}]"
            ))
            .is_none()
        );
    }
}
