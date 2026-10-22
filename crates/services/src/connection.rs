//! Connection supervisor: watch the lockfile, connect when the Riot Client
//! appears, run the websocket, and start over when it is lost. Retrying the same lockfile is
//! throttled to 30 s so a game that is still starting does not stutter the app.

use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use domain::presence::{decode_private, read_own_presence};
use riot::RiotError;
use riot::api::local::events::UriKind;
use riot::lockfile::lockfile_stamp;
use riot::session::RiotSession;
use riot::ws::{RiotEvent, WsEnd, spawn_websocket};
use serde_json::Value;
use tokio_util::sync::CancellationToken;

use crate::ctx::Ctx;
use crate::event::{AppEvent, ConnectionStatus};
use crate::game_state::GameTracker;

const WATCH_INTERVAL: Duration = Duration::from_secs(2);
const RETRY_SAME_LOCKFILE: Duration = Duration::from_secs(30);

/// Run until `stop` fires. Spawn once at startup.
pub async fn supervise(ctx: Ctx, stop: CancellationToken) {
    let mut last_failure: Option<(SystemTime, Instant)> = None;
    let mut status = None;
    while !stop.is_cancelled() {
        let Some(stamp) = lockfile_stamp().await else {
            announce(&ctx, &mut status, ConnectionStatus::WaitingForRiot);
            pause(&stop).await;
            continue;
        };
        let is_throttled = last_failure
            .is_some_and(|(failed, at)| failed == stamp && at.elapsed() < RETRY_SAME_LOCKFILE);
        if is_throttled {
            pause(&stop).await;
            continue;
        }
        announce(&ctx, &mut status, ConnectionStatus::Connecting);
        match RiotSession::connect(ctx.settings.get().region.as_deref()).await {
            Ok(session) => {
                last_failure = None;
                run_session(&ctx, Arc::new(session), &stop).await;
                status = None;
            }
            Err(error) => {
                last_failure = Some((stamp, Instant::now()));
                announce(&ctx, &mut status, failure_status(&error));
            }
        }
    }
}

async fn pause(stop: &CancellationToken) {
    tokio::select! {
        () = stop.cancelled() => {}
        () = tokio::time::sleep(WATCH_INTERVAL) => {}
    }
}

/// Publish only when the status changes.
fn announce(ctx: &Ctx, current: &mut Option<ConnectionStatus>, next: ConnectionStatus) {
    if current.as_ref() != Some(&next) {
        ctx.bus.publish(AppEvent::Connection(next.clone()));
        *current = Some(next);
    }
}

/// The game still starting is the normal case, not an error.
fn failure_status(error: &RiotError) -> ConnectionStatus {
    match error {
        RiotError::Handshake { .. } | RiotError::RegionUnknown => ConnectionStatus::RiotFound,
        other => ConnectionStatus::Failed {
            message: other.to_string(),
        },
    }
}

async fn run_session(ctx: &Ctx, session: Arc<RiotSession>, stop: &CancellationToken) {
    remember_region(ctx, &session);
    ctx.set_session(Some(session.clone()));
    ctx.bus
        .publish(AppEvent::Connection(ConnectionStatus::Connected {
            riot_id: session.riot_id.clone(),
            region: session.http.region().region.clone(),
        }));
    let mut tracker = GameTracker::default();
    publish_game(ctx, tracker.attach());

    let session_stop = stop.child_token();
    let (mut events, task) = spawn_websocket(session.lockfile.clone(), session_stop.clone());
    while let Some(event) = events.recv().await {
        if event.kind == UriKind::Presence {
            apply_presence(ctx, &mut tracker, &session, &event);
        }
        ctx.forward_riot_event(event);
    }
    session_stop.cancel();
    let end = task.await.unwrap_or(WsEnd::Lost);
    tracing::info!(?end, "riot_session_ended");
    ctx.set_session(None);
    publish_game(ctx, tracker.detach());
}

fn remember_region(ctx: &Ctx, session: &RiotSession) {
    let region = session.http.region().region.clone();
    if ctx.settings.get().region.as_deref() == Some(region.as_str()) {
        return;
    }
    if let Err(error) = ctx
        .settings
        .update(|settings| settings.region = Some(region))
    {
        tracing::warn!(%error, "region_not_persisted");
    }
}

/// Only the player's own presence drives the snapshot.
fn apply_presence(ctx: &Ctx, tracker: &mut GameTracker, session: &RiotSession, event: &RiotEvent) {
    let own = event
        .event
        .data
        .get("presences")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|presence| {
            presence.get("puuid").and_then(Value::as_str) == Some(session.puuid.as_str())
        });
    let Some(private) = own
        .and_then(|presence| presence.get("private"))
        .and_then(Value::as_str)
    else {
        return;
    };
    publish_game(
        ctx,
        tracker.apply_presence(&read_own_presence(&decode_private(private))),
    );
}

fn publish_game(ctx: &Ctx, snapshot: Option<domain::phase::GameSnapshot>) {
    if let Some(snapshot) = snapshot {
        ctx.set_game(snapshot.clone());
        ctx.bus.publish(AppEvent::GameState(snapshot));
    }
}
