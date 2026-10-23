//! Collection: what each weapon wears, the skins you own for it, and
//! equipping one. Equipping is a read-modify-write of the whole loadout (fresh GET, change
//! one gun, PUT) so nothing else in the loadout can be lost or reverted.

mod view;

use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use riot::CachePolicy;
use riot::api::pd::entitlements::{
    Entitlements, ITEM_TYPE_SKIN_CHROMAS, ITEM_TYPE_SKIN_LEVELS, get_entitlements,
};
use riot::api::pd::loadout::{PlayerLoadout, get_player_loadout, put_player_loadout};
use riot::session::RiotSession;

pub(crate) use view::edition_of;
pub use view::{ChromaOption, CollectionView, Owned, SkinOption, WeaponSlot};
use view::{collection_view, equipped_gun, skin_options};

use crate::action::{ActionError, send_ok};
use crate::assets::AssetStore;
use crate::ctx::Ctx;

/// Owned items change rarely (a purchase); re-read them this often at most.
const OWNED_TTL: Duration = Duration::from_secs(5 * 60);

/// Cheap to clone.
#[derive(Clone)]
pub struct CollectionService {
    ctx: Ctx,
    assets: AssetStore,
    owned: Arc<Mutex<Option<(Instant, Owned)>>>,
}

impl CollectionService {
    pub fn new(ctx: Ctx, assets: AssetStore) -> Self {
        Self {
            ctx,
            assets,
            owned: Arc::new(Mutex::new(None)),
        }
    }

    fn session(&self) -> Result<Arc<RiotSession>, ActionError> {
        self.ctx.session().ok_or(ActionError::NotConnected)
    }

    /// Every weapon with its equipped skin.
    pub async fn view(&self) -> Result<CollectionView, ActionError> {
        let session = self.session()?;
        let loadout = read_loadout(&session).await?;
        Ok(collection_view(&self.assets.index(), &loadout))
    }

    /// Owned skins for one weapon.
    pub async fn options(&self, weapon_id: &str) -> Result<Vec<SkinOption>, ActionError> {
        let session = self.session()?;
        let owned = self.owned(&session).await?;
        Ok(skin_options(&self.assets.index(), weapon_id, &owned))
    }

    /// Put `skin_id` (in `chroma_id` when owned) on `weapon_id`.
    pub async fn equip(
        &self,
        weapon_id: &str,
        skin_id: &str,
        chroma_id: &str,
    ) -> Result<CollectionView, ActionError> {
        let session = self.session()?;
        let index = self.assets.index();
        let weapon = index
            .weapon_of_skin(skin_id)
            .filter(|weapon| weapon.uuid.eq_ignore_ascii_case(weapon_id))
            .ok_or_else(|| {
                ActionError::could_not("equip that skin", "it does not belong to this weapon")
            })?;
        let owned = self.owned(&session).await?;
        let is_owned = skin_options(&index, &weapon.uuid, &owned)
            .iter()
            .any(|option| option.skin_id.eq_ignore_ascii_case(skin_id));
        let skin = index
            .skin(skin_id)
            .filter(|_| is_owned)
            .ok_or_else(|| ActionError::could_not("equip that skin", "you do not own it"))?;
        let (skin_id, level_id, chroma_id) = equipped_gun(&skin, chroma_id, &owned)
            .ok_or_else(|| ActionError::could_not("equip that skin", "its data is incomplete"))?;
        let mut loadout = read_loadout(&session).await?;
        let gun = loadout.gun_mut(&weapon.uuid).ok_or_else(|| {
            ActionError::could_not("equip that skin", "the weapon is missing from your loadout")
        })?;
        gun.skin_id = skin_id;
        gun.skin_level_id = level_id;
        gun.chroma_id = chroma_id;
        let request = put_player_loadout(&session.puuid, &loadout)
            .map_err(|error| ActionError::could_not("equip that skin", error))?;
        let reply = send_ok(&session.http, &request, "equip that skin").await?;
        let saved: PlayerLoadout = serde_json::from_str(&reply.body).unwrap_or(loadout);
        Ok(collection_view(&index, &saved))
    }

    async fn owned(&self, session: &RiotSession) -> Result<Owned, ActionError> {
        if let Some((at, owned)) = self
            .owned
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_ref()
            && at.elapsed() < OWNED_TTL
        {
            return Ok(owned.clone());
        }
        let owned = Owned {
            skin_levels: read_owned(session, ITEM_TYPE_SKIN_LEVELS).await?,
            chromas: read_owned(session, ITEM_TYPE_SKIN_CHROMAS).await?,
        };
        *self.owned.lock().unwrap_or_else(PoisonError::into_inner) =
            Some((Instant::now(), owned.clone()));
        Ok(owned)
    }
}

/// The loadout, always fresh (the game may have changed it since).
async fn read_loadout(session: &RiotSession) -> Result<PlayerLoadout, ActionError> {
    session
        .http
        .send_json(&get_player_loadout(&session.puuid))
        .await
        .map_err(|error| ActionError::could_not("read your loadout", error))
}

/// Lower-case ids of every owned item of one entitlement type.
pub(crate) async fn read_owned(
    session: &RiotSession,
    item_type: &str,
) -> Result<std::collections::HashSet<String>, ActionError> {
    let fetched = session
        .gate
        .fetch(
            &session.http,
            &get_entitlements(&session.puuid, item_type),
            CachePolicy::Fresh,
        )
        .await;
    if !fetched.is_complete {
        return Err(ActionError::could_not(
            "read your collection",
            format!("Riot answered HTTP {}", fetched.status),
        ));
    }
    let entitlements: Entitlements = serde_json::from_value(fetched.data)
        .map_err(|error| ActionError::could_not("read your collection", error))?;
    Ok(entitlements
        .of_type(item_type)
        .into_iter()
        .filter_map(|entry| {
            entry
                .item_id
                .as_ref()
                .map(|id| id.to_string().to_lowercase())
        })
        .collect())
}
