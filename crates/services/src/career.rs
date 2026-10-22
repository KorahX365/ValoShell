//! Career: current rank and RR, the act's record, peak rank this act,
//! and the RR history of recent ranked matches.

use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use domain::static_data::map_by_map_id_path;
use riot::api::pd::history::IndexWindow;
use riot::api::pd::mmr::{
    CompetitiveUpdate, CompetitiveUpdates, MmrPlayer, get_competitive_updates, get_player_mmr,
};
use riot::{CachePolicy, Request};
use serde::Serialize;
use serde::de::DeserializeOwned;
use ts_rs::TS;

use crate::action::ActionError;
use crate::assets::{AssetIndex, AssetStore, image_url};
use crate::ctx::Ctx;
use crate::party::{RankBadge, rank_badge};

const VIEW_TTL: Duration = Duration::from_secs(90);
const COMPETITIVE: &str = "competitive";
const UPDATES_SHOWN: u32 = 20;

/// RR change of one ranked match.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct RrUpdate {
    pub match_id: String,
    pub map_name: String,
    pub map_image: Option<String>,
    /// Epoch ms.
    #[ts(type = "number")]
    pub started_at_ms: i64,
    pub rr_earned: i32,
    pub rr_after: i32,
    pub rank_after: RankBadge,
    pub is_promotion: bool,
    pub is_demotion: bool,
    pub afk_penalty: i32,
}

/// The career scene.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct CareerView {
    pub rank: RankBadge,
    pub rr: u32,
    /// 0 when not on the leaderboard.
    pub leaderboard_rank: u32,
    pub wins: u32,
    pub games: u32,
    /// Highest tier with a win this act.
    pub peak: Option<RankBadge>,
    /// Ranked games still needed before a rank is shown.
    pub placements_left: u32,
    pub updates: Vec<RrUpdate>,
}

fn rr_update(index: &AssetIndex, update: &CompetitiveUpdate) -> RrUpdate {
    let internal_id = update.map_id.rsplit('/').nth(1).unwrap_or_default();
    let map = index.map_by_internal_id(internal_id);
    let tier_after = u32::try_from(update.tier_after_update).unwrap_or(0);
    RrUpdate {
        match_id: update
            .match_id
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default(),
        map_name: map
            .as_ref()
            .map(|map| map.display_name.clone())
            .or_else(|| map_by_map_id_path(&update.map_id).map(|entry| entry.display_name()))
            .unwrap_or_else(|| "Unknown map".to_owned()),
        map_image: map.and_then(|map| map.list_view_icon.as_deref().map(image_url)),
        started_at_ms: update.match_start_time,
        rr_earned: update.ranked_rating_earned,
        rr_after: update.ranked_rating_after_update,
        rank_after: rank_badge(index, tier_after),
        is_promotion: update.tier_after_update > update.tier_before_update,
        is_demotion: update.tier_after_update < update.tier_before_update,
        afk_penalty: update.afk_penalty,
    }
}

/// Build the view. The act is the one of the latest ranked match (the act the player is in).
pub fn career_view(
    index: &AssetIndex,
    mmr: &MmrPlayer,
    updates: &CompetitiveUpdates,
) -> CareerView {
    let season_id = mmr
        .latest_competitive_update
        .as_ref()
        .map(|update| update.season_id.clone())
        .or_else(|| {
            updates
                .matches
                .first()
                .map(|update| update.season_id.clone())
        })
        .unwrap_or_default();
    let season = mmr.seasonal_info(COMPETITIVE, &season_id);
    let tier = season.map_or(0, |info| info.competitive_tier);
    let peak = season.and_then(|info| {
        info.wins_by_tier
            .keys()
            .filter_map(|tier| tier.parse::<u32>().ok())
            .max()
            .map(|tier| rank_badge(index, tier))
    });
    CareerView {
        rank: rank_badge(index, tier),
        rr: season.map_or(0, |info| info.ranked_rating),
        leaderboard_rank: season.map_or(0, |info| info.leaderboard_rank),
        wins: season.map_or(0, |info| {
            info.number_of_wins_with_placements.max(info.number_of_wins)
        }),
        games: season.map_or(0, |info| info.number_of_games),
        peak,
        placements_left: mmr
            .queue_skills
            .get(COMPETITIVE)
            .map_or(0, |skill| skill.current_season_games_needed_for_rating),
        updates: updates
            .matches
            .iter()
            .map(|update| rr_update(index, update))
            .collect(),
    }
}

/// Cheap to clone.
#[derive(Clone)]
pub struct CareerService {
    ctx: Ctx,
    assets: AssetStore,
    last: Arc<Mutex<Option<(Instant, CareerView)>>>,
}

impl CareerService {
    pub fn new(ctx: Ctx, assets: AssetStore) -> Self {
        Self {
            ctx,
            assets,
            last: Arc::new(Mutex::new(None)),
        }
    }

    /// The career, reusing a recent answer unless `is_forced`.
    pub async fn view(&self, is_forced: bool) -> Result<CareerView, ActionError> {
        if !is_forced
            && let Some((at, view)) = self
                .last
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .as_ref()
            && at.elapsed() < VIEW_TTL
        {
            return Ok(view.clone());
        }
        let session = self.ctx.session().ok_or(ActionError::NotConnected)?;
        let mmr: MmrPlayer = fetch(&session, &get_player_mmr(&session.puuid)).await?;
        let window = IndexWindow::new(0, UPDATES_SHOWN);
        let updates: CompetitiveUpdates = fetch(
            &session,
            &get_competitive_updates(&session.puuid, window, Some(COMPETITIVE)),
        )
        .await?;
        let view = career_view(&self.assets.index(), &mmr, &updates);
        *self.last.lock().unwrap_or_else(PoisonError::into_inner) =
            Some((Instant::now(), view.clone()));
        Ok(view)
    }
}

async fn fetch<T: DeserializeOwned>(
    session: &riot::session::RiotSession,
    request: &Request,
) -> Result<T, ActionError> {
    let fetched = session
        .gate
        .fetch(&session.http, request, CachePolicy::Fresh)
        .await;
    if !fetched.is_complete {
        let reason = match fetched.status {
            0 => "Riot did not answer in time".to_owned(),
            status => format!("Riot answered HTTP {status}"),
        };
        return Err(ActionError::could_not("load your career", reason));
    }
    serde_json::from_value(fetched.data)
        .map_err(|error| ActionError::could_not("read your career", error))
}

#[cfg(test)]
mod tests {
    use super::*;

    const MMR: &str = include_str!("../../riot/src/api/pd/fixtures/mmr.json");

    fn view() -> CareerView {
        let mmr: MmrPlayer = serde_json::from_str(MMR).unwrap();
        let updates = CompetitiveUpdates {
            matches: mmr.latest_competitive_update.clone().into_iter().collect(),
            ..CompetitiveUpdates::default()
        };
        career_view(&AssetIndex::bundled(), &mmr, &updates)
    }

    #[test]
    fn current_act_rank_and_record_are_read() {
        let view = view();

        assert_eq!(view.rank.tier, 14);
        assert_eq!(view.rr, 37);
        assert_eq!(view.wins, 28);
        assert_eq!(view.games, 52);
        assert_eq!(view.placements_left, 0);
    }

    #[test]
    fn peak_is_the_highest_tier_with_a_win() {
        assert_eq!(view().peak.map(|peak| peak.tier), Some(14));
    }

    #[test]
    fn rr_updates_name_the_map_and_the_change() {
        let view = view();

        let update = &view.updates[0];
        assert_eq!(update.map_name, "Ascent");
        assert_eq!(update.rr_earned, -18);
        assert!(!update.is_promotion && !update.is_demotion);
    }
}
