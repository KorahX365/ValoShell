//! Shared context every service holds. The Riot session is swapped (never mutated) on each
//! Riot Client launch, so services read `session()` per operation and never cache it.

use std::sync::Arc;

use arc_swap::ArcSwapOption;
use domain::phase::GameSnapshot;
use riot::session::RiotSession;
use riot::ws::RiotEvent;
use store::SettingsStore;
use tokio::sync::{broadcast, mpsc, watch};

use crate::event::Bus;
use crate::match_flow::FlowSignal;

const RIOT_EVENT_CAPACITY: usize = 256;
const FLOW_SIGNAL_CAPACITY: usize = 16;

#[derive(Clone)]
pub struct Ctx {
    session: Arc<ArcSwapOption<RiotSession>>,
    pub bus: Bus,
    pub settings: SettingsStore,
    riot_events: broadcast::Sender<RiotEvent>,
    game: Arc<watch::Sender<GameSnapshot>>,
    flow: mpsc::Sender<FlowSignal>,
}

impl Ctx {
    /// Fresh context (disconnected, offline snapshot) plus the receiver the match-flow task
    /// consumes.
    pub fn new(settings: SettingsStore) -> (Self, mpsc::Receiver<FlowSignal>) {
        let (flow, flow_signals) = mpsc::channel(FLOW_SIGNAL_CAPACITY);
        let ctx = Self {
            session: Arc::new(ArcSwapOption::empty()),
            bus: Bus::default(),
            settings,
            riot_events: broadcast::channel(RIOT_EVENT_CAPACITY).0,
            game: Arc::new(watch::channel(GameSnapshot::default()).0),
            flow,
        };
        (ctx, flow_signals)
    }

    /// Tell the match flow something happened (own lock, manual show/hide).
    pub fn signal_flow(&self, signal: FlowSignal) {
        if self.flow.try_send(signal).is_err() {
            tracing::warn!(?signal, "match_flow_signal_dropped");
        }
    }

    /// The current Riot session, `None` while disconnected.
    pub fn session(&self) -> Option<Arc<RiotSession>> {
        self.session.load_full()
    }

    pub(crate) fn set_session(&self, session: Option<Arc<RiotSession>>) {
        self.session.store(session);
    }

    /// Every classified Riot Client event (pregame, party, presence, chat...).
    pub fn riot_events(&self) -> broadcast::Receiver<RiotEvent> {
        self.riot_events.subscribe()
    }

    pub(crate) fn forward_riot_event(&self, event: RiotEvent) {
        let _ = self.riot_events.send(event);
    }

    /// The live game snapshot; `changed()` wakes on every new snapshot.
    pub fn game(&self) -> watch::Receiver<GameSnapshot> {
        self.game.subscribe()
    }

    pub(crate) fn set_game(&self, snapshot: GameSnapshot) {
        self.game.send_replace(snapshot);
    }
}
