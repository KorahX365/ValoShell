//! Match history for the career page: the last few matches in every mode, one row each (agent,
//! mode, map, K/D/A, combat score, result, MVP, placement and the RR change when ranked).
//! Rows are built from Riot's match details with the post-match view's rules and kept for the
//! session, since a finished match never changes.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use tokio::task::JoinSet;

use domain::ids::{MatchId, Puuid};
use riot::CachePolicy;
use riot::api::pd::history::{IndexWindow, MatchHistory, get_match_details, get_match_history};
use riot::session::RiotSession;
use serde::Serialize;
use ts_rs::TS;

use crate::action::ActionError;
use crate::assets::AssetStore;
use crate::career::CareerService;
use crate::ctx::Ctx;
use crate::post_match::{MatchResult, PostMatchInputs, PostMatchView, post_match_view};

const VIEW_TTL: Duration = Duration::from_secs(60);
const MATCHES_SHOWN: u32 = 10;
const DEATHMATCH: &str = "deathmatch";

/// Best player of the match, or of the player's own team.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Mvp {
    Match,
    Team,
}

/// One match as the career's history lists it.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct MatchRow {
    pub match_id: String,
    pub queue_id: String,
    pub queue_label: String,
    pub map_name: String,
    pub map_splash: Option<String>,
    pub agent_name: Option<String>,
    pub agent_icon: Option<String>,
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    /// Average combat score, as the client's "score" line shows it.
    pub score: u32,
    pub result: MatchResult,
    pub rounds_won: u32,
    pub rounds_lost: u32,
    /// Finishing place in free-for-all modes.
    pub placement: Option<u32>,
    pub mvp: Option<Mvp>,
    /// RR won or lost in ranked; `None` elsewhere.
    pub rr_change: Option<i32>,
    /// Epoch ms.
    #[ts(type = "number")]
    pub started_at_ms: i64,
}

/// The career page's history.
#[derive(Debug, Clone, Default, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct HistoryView {
    pub rows: Vec<MatchRow>,
}

/// The row for one match, from its post-match view.
pub fn match_row(view: &PostMatchView) -> MatchRow {
    // Players come highest combat score first, which is how the client picks its MVPs.
    let own_rank = view.players.iter().position(|player| player.is_self);
    let team_best = view.players.iter().find(|player| player.is_ally);
    let mvp = match (own_rank, team_best) {
        (Some(0), _) => Some(Mvp::Match),
        (_, Some(best)) if best.is_self => Some(Mvp::Team),
        _ => None,
    };
    let placement = (view.queue_id == DEATHMATCH)
        .then(|| deathmatch_place(view))
        .flatten();
    MatchRow {
        match_id: view.match_id.clone(),
        queue_id: view.queue_id.clone(),
        queue_label: view.queue_label.clone(),
        map_name: view.map_name.clone(),
        map_splash: view.map_splash.clone(),
        agent_name: view.agent_name.clone(),
        agent_icon: view.agent_icon.clone(),
        kills: view.stats.kills,
        deaths: view.stats.deaths,
        assists: view.stats.assists,
        score: score_of(view.stats.acs),
        result: view.result,
        rounds_won: view.rounds_won,
        rounds_lost: view.rounds_lost,
        placement,
        mvp: if placement.is_some() { None } else { mvp },
        rr_change: view.rr_change,
        started_at_ms: view.started_at_ms,
    }
}

/// Free-for-all places go by kills: one plus everyone who got more.
fn deathmatch_place(view: &PostMatchView) -> Option<u32> {
    let own = view.players.iter().find(|player| player.is_self)?;
    let ahead = view
        .players
        .iter()
        .filter(|player| player.kills > own.kills)
        .count();
    u32::try_from(ahead + 1).ok()
}

fn score_of(acs: f64) -> u32 {
    if acs.is_finite() && acs > 0.0 {
        // Combat score per round is a few hundred at most, so the rounded value fits.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let score = acs.round() as u32;
        score
    } else {
        0
    }
}

struct Inner {
    ctx: Ctx,
    assets: AssetStore,
    career: CareerService,
    /// Built rows by account and match: a finished match never changes.
    rows: Mutex<HashMap<(Puuid, MatchId), MatchRow>>,
    last: Mutex<Option<(Instant, Puuid, HistoryView)>>,
}

/// Cheap to clone.
#[derive(Clone)]
pub struct HistoryService {
    inner: Arc<Inner>,
}

impl HistoryService {
    pub fn new(ctx: Ctx, assets: AssetStore, career: CareerService) -> Self {
        Self {
            inner: Arc::new(Inner {
                ctx,
                assets,
                career,
                rows: Mutex::new(HashMap::new()),
                last: Mutex::new(None),
            }),
        }
    }

    /// The recent matches, reusing a recent answer unless `is_forced`.
    pub async fn view(&self, is_forced: bool) -> Result<HistoryView, ActionError> {
        let session = self.inner.ctx.session().ok_or(ActionError::NotConnected)?;
        if !is_forced && let Some(view) = self.fresh(&session.puuid) {
            return Ok(view);
        }
        let ids = recent_ids(&session).await?;
        let rr = self.rr_changes().await;
        let rows = self
            .rows_for(&session, ids)
            .await
            .into_iter()
            .map(|(id, row)| {
                let rr_change = rr.get(&id).copied().or(row.rr_change);
                MatchRow { rr_change, ..row }
            })
            .collect();
        let view = HistoryView { rows };
        *self
            .inner
            .last
            .lock()
            .unwrap_or_else(PoisonError::into_inner) =
            Some((Instant::now(), session.puuid.clone(), view.clone()));
        Ok(view)
    }

    fn fresh(&self, puuid: &Puuid) -> Option<HistoryView> {
        let last = self
            .inner
            .last
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        last.as_ref()
            .filter(|(at, owner, _)| at.elapsed() < VIEW_TTL && owner == puuid)
            .map(|(_, _, view)| view.clone())
    }

    /// Every match's row, read in parallel and returned in history order.
    async fn rows_for(
        &self,
        session: &Arc<RiotSession>,
        ids: Vec<MatchId>,
    ) -> Vec<(MatchId, MatchRow)> {
        let mut tasks = JoinSet::new();
        for (order, id) in ids.into_iter().enumerate() {
            let service = self.clone();
            let session = Arc::clone(session);
            tasks.spawn(async move {
                let row = service.row(&session, &id).await;
                (order, id, row)
            });
        }
        let mut found = Vec::new();
        while let Some(joined) = tasks.join_next().await {
            if let Ok((order, id, Some(row))) = joined {
                found.push((order, id, row));
            }
        }
        found.sort_by_key(|(order, _, _)| *order);
        found.into_iter().map(|(_, id, row)| (id, row)).collect()
    }

    /// RR changes of recent ranked matches, from the career (best effort).
    async fn rr_changes(&self) -> HashMap<MatchId, i32> {
        let Ok(career) = self.inner.career.view(false).await else {
            return HashMap::new();
        };
        career
            .updates
            .iter()
            .map(|update| (MatchId::new(&update.match_id), update.rr_earned))
            .collect()
    }

    /// One match's row: from the session's cache, else from its details.
    async fn row(&self, session: &RiotSession, id: &MatchId) -> Option<MatchRow> {
        let key = (session.puuid.clone(), id.clone());
        if let Some(row) = self
            .inner
            .rows
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&key)
        {
            return Some(row.clone());
        }
        let fetched = session
            .gate
            .fetch(&session.http, &get_match_details(id), CachePolicy::Cached)
            .await;
        if fetched.status != 200 {
            return None;
        }
        let index = self.inner.assets.index();
        let known = HashMap::new();
        let inputs = PostMatchInputs {
            index: &index,
            own: &session.puuid,
            known: &known,
        };
        let row = match_row(&post_match_view(&fetched.data, &inputs)?);
        self.inner
            .rows
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(key, row.clone());
        Some(row)
    }
}

/// Ids of the newest matches in every mode, newest first.
async fn recent_ids(session: &RiotSession) -> Result<Vec<MatchId>, ActionError> {
    let request = get_match_history(&session.puuid, IndexWindow::new(0, MATCHES_SHOWN), None);
    let fetched = session
        .gate
        .fetch(&session.http, &request, CachePolicy::Fresh)
        .await;
    if !fetched.is_complete {
        return Err(ActionError::could_not(
            "load your match history",
            "Riot did not answer",
        ));
    }
    let history: MatchHistory = serde_json::from_value(fetched.data)
        .map_err(|error| ActionError::could_not("read your match history", error))?;
    Ok(history
        .history
        .into_iter()
        .filter_map(|entry| entry.match_id)
        .collect())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_json::Value;

    use super::*;
    use crate::assets::AssetIndex;

    const DETAILS: &str = include_str!("post_match/fixtures/match_details.json");

    fn view() -> PostMatchView {
        let detail: Value = serde_json::from_str(DETAILS).unwrap();
        let index = AssetIndex::bundled();
        let own = Puuid::new("me");
        let known = HashMap::new();
        let inputs = PostMatchInputs {
            index: &index,
            own: &own,
            known: &known,
        };
        post_match_view(&detail, &inputs).unwrap()
    }

    #[test]
    fn a_row_keeps_the_result_line_and_own_numbers() {
        let view = view();
        let row = match_row(&view);

        assert_eq!(row.result, MatchResult::Victory);
        assert_eq!((row.rounds_won, row.rounds_lost), (13, 9));
        assert_eq!(row.map_name, "Ascent");
        assert_eq!(
            (row.kills, row.deaths, row.assists),
            (view.stats.kills, view.stats.deaths, view.stats.assists)
        );
        assert_eq!(row.placement, None);
    }

    #[test]
    fn mvp_follows_the_combat_score_order() {
        let mut view = view();
        let own = view
            .players
            .iter()
            .position(|player| player.is_self)
            .unwrap();
        let mine = view.players.remove(own);
        view.players.insert(0, mine);
        assert_eq!(match_row(&view).mvp, Some(Mvp::Match));

        let first = view.players.remove(0);
        let enemy = view
            .players
            .iter()
            .position(|player| !player.is_ally)
            .unwrap();
        view.players.insert(enemy + 1, first);
        let best_ally = view.players.iter().find(|player| player.is_ally).unwrap();
        let expected = best_ally.is_self.then_some(Mvp::Team);
        assert_eq!(match_row(&view).mvp, expected);
    }

    #[test]
    fn deathmatch_reports_a_place_instead_of_mvp() {
        let mut view = view();
        view.queue_id = DEATHMATCH.to_owned();
        let row = match_row(&view);
        let own = view.players.iter().find(|player| player.is_self).unwrap();
        let place = view
            .players
            .iter()
            .filter(|player| player.kills > own.kills)
            .count()
            + 1;

        assert_eq!(row.placement, u32::try_from(place).ok());
        assert_eq!(row.mvp, None);
    }
}
