//! Post-match screen: when the player comes back from a match (in game -> menus), fetch that
//! match's details, build the summary and keep it as the last match. Riot publishes details
//! 5-60 s after the end (404 until then), so the read is retried for a few minutes. The RR
//! change of a ranked match arrives later still and is added when Riot reports it.

mod view;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use domain::ids::{MatchId, Puuid};
use domain::phase::Phase;
use riot::CachePolicy;
use riot::api::pd::history::{IndexWindow, MatchHistory, get_match_details, get_match_history};
use riot::session::RiotSession;
use serde_json::Value;
use tokio_util::sync::CancellationToken;

pub use view::{
    KnownPlayer, MatchResult, OwnStats, PostMatchInputs, PostMatchPlayer, PostMatchView,
    post_match_view, with_rr,
};

use crate::assets::AssetStore;
use crate::career::CareerService;
use crate::ctx::Ctx;
use crate::event::AppEvent;
use crate::lobby::LobbyService;
use crate::names::{RiotName, split_riot_id};

/// Wait between reads while Riot has not published the match yet.
const DETAILS_RETRY: Duration = Duration::from_secs(8);
/// About three minutes of reads before giving up on a match.
const DETAILS_ATTEMPTS: u32 = 23;
/// About half a minute of history reads while it still lists the previous match.
const HISTORY_ATTEMPTS: u32 = 4;
/// Wait between career reads while the RR change is not reported yet.
const RR_RETRY: Duration = Duration::from_secs(15);
const RR_ATTEMPTS: u32 = 4;

#[derive(Default)]
struct PostMatchState {
    view: Option<PostMatchView>,
}

struct Inner {
    ctx: Ctx,
    assets: AssetStore,
    lobby: LobbyService,
    career: CareerService,
    state: Mutex<PostMatchState>,
}

/// Cheap to clone.
#[derive(Clone)]
pub struct PostMatchService {
    inner: Arc<Inner>,
}

impl PostMatchService {
    pub fn new(ctx: Ctx, assets: AssetStore, lobby: LobbyService, career: CareerService) -> Self {
        Self {
            inner: Arc::new(Inner {
                ctx,
                assets,
                lobby,
                career,
                state: Mutex::new(PostMatchState::default()),
            }),
        }
    }

    fn state(&self) -> std::sync::MutexGuard<'_, PostMatchState> {
        self.inner
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// The last match's summary, seen or not.
    pub fn current(&self) -> Option<PostMatchView> {
        self.state().view.clone()
    }

    /// The player closed the screen: keep the summary, marked as seen.
    pub fn dismiss(&self) {
        let mut state = self.state();
        let Some(view) = state.view.as_mut().filter(|view| !view.is_seen) else {
            return;
        };
        view.is_seen = true;
        let view = view.clone();
        drop(state);
        self.inner.ctx.bus.publish(AppEvent::PostMatch(Some(view)));
    }

    /// Background task: one collection per return from a match; a newer one replaces it.
    pub async fn run(self, stop: CancellationToken) {
        let mut game = self.inner.ctx.game();
        let mut previous = game.borrow().phase;
        let mut collecting: Option<CancellationToken> = None;
        loop {
            tokio::select! {
                () = stop.cancelled() => return,
                changed = game.changed() => if changed.is_err() { return },
            }
            let phase = game.borrow_and_update().phase;
            if previous == Phase::Ingame && phase == Phase::Menus {
                if let Some(running) = collecting.take() {
                    running.cancel();
                }
                let token = stop.child_token();
                tokio::spawn(self.clone().collect(token.clone()));
                collecting = Some(token);
            }
            previous = phase;
        }
    }

    async fn collect(self, stop: CancellationToken) {
        let Some(session) = self.inner.ctx.session() else {
            return;
        };
        let Some(match_id) = self.ended_match(&session, &stop).await else {
            tracing::debug!("post_match_no_match_id");
            return;
        };
        let Some(detail) = read_details(&session, &match_id, &stop).await else {
            tracing::info!(%match_id, "post_match_details_unavailable");
            return;
        };
        let Some(view) = self.build(&session, &match_id, &detail) else {
            return;
        };
        let is_ranked = view.is_ranked;
        self.publish(view);
        if is_ranked {
            self.add_rr(&match_id, &stop).await;
        }
    }

    fn is_known(&self, match_id: &MatchId) -> bool {
        self.state()
            .view
            .as_ref()
            .is_some_and(|view| MatchId::new(&view.match_id) == *match_id)
    }

    /// The match that just ended: the live lobby's, else the newest one in the history. An id
    /// already summarised is stale (the lobby never read this match, or the history lags), so
    /// it is skipped and the history read again until the new match shows up.
    async fn ended_match(
        &self,
        session: &RiotSession,
        stop: &CancellationToken,
    ) -> Option<MatchId> {
        let lobby_id = self
            .inner
            .lobby
            .current()
            .map(|lobby| MatchId::new(&lobby.match_id))
            .filter(|id| !id.as_str().is_empty() && !self.is_known(id));
        if lobby_id.is_some() {
            return lobby_id;
        }
        for attempt in 0..HISTORY_ATTEMPTS {
            if attempt > 0 && !wait(stop, DETAILS_RETRY).await {
                return None;
            }
            let newest = newest_match(session).await.filter(|id| !self.is_known(id));
            if newest.is_some() {
                return newest;
            }
        }
        None
    }

    fn build(
        &self,
        session: &RiotSession,
        match_id: &MatchId,
        detail: &Value,
    ) -> Option<PostMatchView> {
        let index = self.inner.assets.index();
        let known = self.known_players(session, match_id);
        let inputs = PostMatchInputs {
            index: &index,
            own: &session.puuid,
            known: &known,
        };
        post_match_view(detail, &inputs)
    }

    /// Names (and who hid theirs) as the live lobby showed them; nothing is looked up here.
    fn known_players(
        &self,
        session: &RiotSession,
        match_id: &MatchId,
    ) -> HashMap<Puuid, KnownPlayer> {
        let mut known: HashMap<Puuid, KnownPlayer> = HashMap::new();
        let lobby = self
            .inner
            .lobby
            .current()
            .filter(|lobby| MatchId::new(&lobby.match_id) == *match_id);
        for player in lobby
            .iter()
            .flat_map(|lobby| lobby.allies.iter().chain(&lobby.enemies))
        {
            let name = RiotName {
                game_name: player.game_name.clone(),
                tag_line: player.tag_line.clone(),
            };
            let entry = KnownPlayer {
                name,
                is_incognito: player.is_incognito,
            };
            known.insert(Puuid::new(&player.puuid), entry);
        }
        if let Some(riot_id) = &session.riot_id {
            let own = KnownPlayer {
                name: split_riot_id(riot_id),
                is_incognito: false,
            };
            known.insert(session.puuid.clone(), own);
        }
        known
    }

    fn publish(&self, view: PostMatchView) {
        self.state().view = Some(view.clone());
        self.inner.ctx.bus.publish(AppEvent::PostMatch(Some(view)));
    }

    /// Ranked only: add the RR change once the career lists this match.
    async fn add_rr(&self, match_id: &MatchId, stop: &CancellationToken) {
        for _ in 0..RR_ATTEMPTS {
            if !wait(stop, RR_RETRY).await {
                return;
            }
            let Ok(career) = self.inner.career.view(true).await else {
                continue;
            };
            let update = career
                .updates
                .iter()
                .find(|update| MatchId::new(&update.match_id) == *match_id);
            let Some(update) = update else {
                continue;
            };
            let current = self
                .current()
                .filter(|view| MatchId::new(&view.match_id) == *match_id);
            if let Some(view) = current {
                self.publish(with_rr(view, update));
            }
            return;
        }
    }
}

/// The newest match in the player's history, read fresh.
async fn newest_match(session: &RiotSession) -> Option<MatchId> {
    let request = get_match_history(&session.puuid, IndexWindow::new(0, 1), None);
    let fetched = session
        .gate
        .fetch(&session.http, &request, CachePolicy::Fresh)
        .await;
    let history: MatchHistory = serde_json::from_value(fetched.data).ok()?;
    history.history.into_iter().next()?.match_id
}

/// Sleep unless stopped first; `false` when stopped.
async fn wait(stop: &CancellationToken, pause: Duration) -> bool {
    tokio::select! {
        () = stop.cancelled() => false,
        () = tokio::time::sleep(pause) => true,
    }
}

/// The match details, read until Riot publishes them (404 until then) or the attempts run out.
async fn read_details(
    session: &RiotSession,
    match_id: &MatchId,
    stop: &CancellationToken,
) -> Option<Value> {
    let request = get_match_details(match_id);
    for attempt in 0..DETAILS_ATTEMPTS {
        if attempt > 0 && !wait(stop, DETAILS_RETRY).await {
            return None;
        }
        let fetched = session
            .gate
            .fetch(&session.http, &request, CachePolicy::Cached)
            .await;
        if fetched.status == 200 && fetched.data.is_object() {
            return Some(fetched.data);
        }
        tracing::debug!(%match_id, status = fetched.status, attempt, "post_match_details_pending");
    }
    None
}
