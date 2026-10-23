//! Store, career, collection, agents, friends, live-lobby and post-match commands. Nothing here
//! can buy.

use services::agents::{AgentProgress, AgentsService};
use services::career::{CareerService, CareerView};
use services::collection::{CollectionService, CollectionView, SkinOption};
use services::history::{HistoryService, HistoryView};
use services::lobby::{LobbyService, LobbyView};
use services::post_match::{PostMatchService, PostMatchView};
use services::social::{FriendView, SocialService};
use services::storefront::{StoreService, StoreView};
use tauri::State;

/// Today's store; `is_forced` skips the short reuse window.
#[tauri::command]
pub async fn get_store(
    store: State<'_, StoreService>,
    is_forced: bool,
) -> Result<StoreView, String> {
    store
        .view(is_forced)
        .await
        .map_err(|error| error.to_string())
}

/// Rank, RR and recent ranked matches; `is_forced` skips the short reuse window.
#[tauri::command]
pub async fn get_career(
    career: State<'_, CareerService>,
    is_forced: bool,
) -> Result<CareerView, String> {
    career
        .view(is_forced)
        .await
        .map_err(|error| error.to_string())
}

/// The last matches in every mode; `is_forced` skips the short reuse window.
#[tauri::command]
pub async fn get_match_history(
    history: State<'_, HistoryService>,
    is_forced: bool,
) -> Result<HistoryView, String> {
    history
        .view(is_forced)
        .await
        .map_err(|error| error.to_string())
}

/// The live lobby (or the last match's), if any.
#[tauri::command]
pub fn get_lobby(lobby: State<'_, LobbyService>) -> Option<LobbyView> {
    lobby.current()
}

/// The last match's summary, if one was collected since the app started.
#[tauri::command]
pub fn get_post_match(post_match: State<'_, PostMatchService>) -> Option<PostMatchView> {
    post_match.current()
}

/// The player closed the post-match screen.
#[tauri::command]
pub fn dismiss_post_match(post_match: State<'_, PostMatchService>) {
    post_match.dismiss();
}

/// Friends with live status.
#[tauri::command]
pub fn get_friends(social: State<'_, SocialService>) -> Vec<FriendView> {
    social.current()
}

/// Every weapon with its equipped skin.
#[tauri::command]
pub async fn get_collection(
    collection: State<'_, CollectionService>,
) -> Result<CollectionView, String> {
    collection.view().await.map_err(|error| error.to_string())
}

/// Owned skins for one weapon.
#[tauri::command]
pub async fn get_skin_options(
    collection: State<'_, CollectionService>,
    weapon_id: String,
) -> Result<Vec<SkinOption>, String> {
    collection
        .options(&weapon_id)
        .await
        .map_err(|error| error.to_string())
}

/// Owned agents and Agent Mastery levels; `is_forced` skips the short reuse window.
#[tauri::command]
pub async fn get_agent_progress(
    agents: State<'_, AgentsService>,
    is_forced: bool,
) -> Result<AgentProgress, String> {
    agents
        .progress(is_forced)
        .await
        .map_err(|error| error.to_string())
}

/// Equip an owned skin (and chroma) on a weapon; returns the updated collection.
#[tauri::command]
pub async fn equip_skin(
    collection: State<'_, CollectionService>,
    weapon_id: String,
    skin_id: String,
    chroma_id: String,
) -> Result<CollectionView, String> {
    collection
        .equip(&weapon_id, &skin_id, &chroma_id)
        .await
        .map_err(|error| error.to_string())
}
