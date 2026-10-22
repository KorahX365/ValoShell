//! Wallet balances for the top bar (read-only; nothing is ever bought). Refreshed on connect,
//! when the player returns to the menus, and every few minutes.

use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use domain::phase::Phase;
use riot::CachePolicy;
use riot::api::pd::wallet::{
    CURRENCY_KINGDOM_CREDITS, CURRENCY_RADIANITE_POINTS, CURRENCY_VALORANT_POINTS, Wallet,
    get_wallet,
};
use serde::Serialize;
use tokio_util::sync::CancellationToken;
use ts_rs::TS;

use crate::ctx::Ctx;
use crate::event::AppEvent;

const REFRESH_INTERVAL: Duration = Duration::from_secs(5 * 60);

/// The three balances the menu shows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct WalletView {
    #[ts(type = "number")]
    pub valorant_points: i64,
    #[ts(type = "number")]
    pub radianite_points: i64,
    #[ts(type = "number")]
    pub kingdom_credits: i64,
}

impl From<&Wallet> for WalletView {
    fn from(wallet: &Wallet) -> Self {
        Self {
            valorant_points: wallet.balance(CURRENCY_VALORANT_POINTS),
            radianite_points: wallet.balance(CURRENCY_RADIANITE_POINTS),
            kingdom_credits: wallet.balance(CURRENCY_KINGDOM_CREDITS),
        }
    }
}

/// Cheap to clone.
#[derive(Clone)]
pub struct WalletService {
    ctx: Ctx,
    last: Arc<Mutex<Option<WalletView>>>,
}

impl WalletService {
    pub fn new(ctx: Ctx) -> Self {
        Self {
            ctx,
            last: Arc::new(Mutex::new(None)),
        }
    }

    /// Last known balances.
    pub fn current(&self) -> Option<WalletView> {
        *self.last.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Background task.
    pub async fn run(self, stop: CancellationToken) {
        let mut game = self.ctx.game();
        let mut tick = tokio::time::interval(REFRESH_INTERVAL);
        let mut last_phase = Phase::Offline;
        loop {
            tokio::select! {
                () = stop.cancelled() => return,
                changed = game.changed() => if changed.is_err() { return },
                _ = tick.tick() => {}
            }
            let phase = game.borrow().phase;
            let is_back_in_menus = phase == Phase::Menus && last_phase != Phase::Menus;
            last_phase = phase;
            if is_back_in_menus || phase == Phase::Menus {
                self.refresh().await;
            }
        }
    }

    async fn refresh(&self) {
        let Some(session) = self.ctx.session() else {
            return;
        };
        let fetched = session
            .gate
            .fetch(
                &session.http,
                &get_wallet(&session.puuid),
                CachePolicy::Fresh,
            )
            .await;
        let Ok(wallet) = serde_json::from_value::<Wallet>(fetched.data) else {
            return;
        };
        if !fetched.is_complete {
            return;
        }
        let view = WalletView::from(&wallet);
        let mut last = self.last.lock().unwrap_or_else(PoisonError::into_inner);
        if *last == Some(view) {
            return;
        }
        *last = Some(view);
        drop(last);
        self.ctx.bus.publish(AppEvent::Wallet(view));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balances_are_read_by_currency_id() {
        let body = format!(
            r#"{{"Balances":{{"{CURRENCY_VALORANT_POINTS}":1250,"{CURRENCY_KINGDOM_CREDITS}":7}}}}"#
        );
        let wallet: Wallet = serde_json::from_str(&body).unwrap();

        let view = WalletView::from(&wallet);

        assert_eq!(view.valorant_points, 1250);
        assert_eq!(view.radianite_points, 0);
        assert_eq!(view.kingdom_credits, 7);
    }
}
