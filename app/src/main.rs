//! ValoShell desktop shell: builds the Tauri app, plugins and windows, starts the services.
//! Feature logic lives in the `services` crate; this crate only wires it to the UI.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bridge;
mod commands;
mod game_window;
mod overlay;
mod protocol;
mod shell;

use commands::documents::DocumentDirs;
use services::career::CareerService;
use services::collection::CollectionService;
use services::history::HistoryService;
use services::lobby::LobbyService;
use services::names::NameBook;
use services::party::PartyService;
use services::post_match::PostMatchService;
use services::pregame::{PregameDeps, PregameService};
use services::social::SocialService;
use services::storefront::StoreService;
use services::wallet::WalletService;
use services::{AssetStore, Ctx};
use store::{DataPaths, SettingsStore};
use tauri::Manager;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::EnvFilter;

pub(crate) const MAIN_WINDOW: &str = "main";
const DEFAULT_LOG_FILTER: &str = "info,app=debug,services=debug,riot=debug,store=debug";

fn main() {
    init_tracing();
    let shortcuts = tauri_plugin_global_shortcut::Builder::new()
        .with_handler(shell::on_hotkey)
        .build();
    let result = protocol::register(tauri::Builder::default())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            focus_main_window(app);
        }))
        .plugin(shortcuts)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(invoke_handler())
        .setup(|app| {
            start_services(app.handle())?;
            shell::register_hotkeys(app.handle());
            if let Err(error) = shell::create_tray(app.handle()) {
                tracing::warn!(%error, "tray_create_failed");
            }
            if let Err(error) = overlay::create(app.handle()) {
                tracing::warn!(%error, "overlay_create_failed");
            }
            Ok(())
        })
        .run(tauri::generate_context!());
    if let Err(error) = result {
        tracing::error!(%error, "app_exit_with_error");
        std::process::exit(1);
    }
}

/// Every command the UI may call. A plain list, so it is exempt from the length rule.
#[allow(clippy::too_many_lines)]
fn invoke_handler() -> impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        commands::core::get_settings,
        commands::core::save_settings,
        commands::core::get_game_snapshot,
        commands::core::get_connection,
        commands::core::get_asset_index_summary,
        commands::core::get_catalog,
        commands::core::get_game_videos,
        commands::core::get_ui_sounds,
        commands::core::open_sounds_folder,
        commands::core::resolve_embed_link,
        commands::core::get_news,
        commands::core::get_wallet,
        commands::core::quit_app,
        commands::career::get_store,
        commands::career::get_career,
        commands::career::get_match_history,
        commands::career::get_lobby,
        commands::career::get_post_match,
        commands::career::dismiss_post_match,
        commands::career::get_friends,
        commands::career::get_collection,
        commands::career::get_skin_options,
        commands::career::equip_skin,
        commands::party::get_party,
        commands::party::party_refresh,
        commands::party::party_change_queue,
        commands::party::party_start_queue,
        commands::party::party_stop_queue,
        commands::party::party_set_ready,
        commands::party::party_set_open,
        commands::party::party_invite,
        commands::party::party_kick,
        commands::party::party_leave,
        commands::party::party_generate_code,
        commands::party::party_join_code,
        commands::party::party_change_team,
        commands::party::party_start_custom,
        commands::pregame::get_pregame,
        commands::pregame::pregame_select,
        commands::pregame::pregame_lock,
        commands::pregame::pregame_dodge,
        commands::documents::list_ui_documents,
        commands::documents::load_ui_document,
        commands::documents::save_ui_document,
        commands::documents::delete_ui_document,
        commands::documents::set_overlay_enabled,
        commands::documents::set_overlay_preview,
        commands::instant_clone::instant_clone_state,
        commands::instant_clone::instant_clone_send,
    ]
}

fn init_tracing() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_FILTER));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

/// Data folders, settings, then the long-running services. Tasks stop when the app exits.
fn start_services(app: &tauri::AppHandle) -> std::io::Result<()> {
    let paths = DataPaths::from_env();
    paths.ensure_dirs()?;
    let (ctx, flow_signals) = Ctx::new(SettingsStore::load(paths.settings_file()));
    let stop = CancellationToken::new();
    let assets =
        AssetStore::new(paths.cache_dir(), ctx.bus.clone()).map_err(std::io::Error::other)?;
    // The bridge subscribes before any service runs so no early event is lost.
    tauri::async_runtime::spawn(bridge::forward_events(app.clone(), ctx.bus.subscribe()));
    tauri::async_runtime::spawn(services::match_flow::run(
        ctx.clone(),
        game_window::AppWindows::new(app.clone()),
        flow_signals,
        stop.clone(),
    ));
    app.manage(stop.clone());
    start_features(app, &ctx, &assets);
    tauri::async_runtime::spawn(assets.clone().run(stop.clone()));
    tauri::async_runtime::spawn(services::connection::supervise(ctx.clone(), stop));
    app.manage(bridge::LastConnection::default());
    app.manage(commands::core::SoundsDir(paths.sounds_dir()));
    app.manage(DocumentDirs::new(&paths));
    app.manage(ctx);
    app.manage(assets);
    Ok(())
}

/// Feature services: background tasks where they keep views live, managed state for commands.
fn start_features(app: &tauri::AppHandle, ctx: &Ctx, assets: &AssetStore) {
    let stop = app.state::<CancellationToken>().inner().clone();
    let names = NameBook::default();
    let party = PartyService::new(ctx.clone(), assets.clone(), names.clone());
    let pregame = PregameService::new(
        ctx.clone(),
        PregameDeps {
            assets: assets.clone(),
            names: names.clone(),
        },
    );
    let wallet = WalletService::new(ctx.clone());
    let lobby = LobbyService::new(ctx.clone(), assets.clone(), names);
    tauri::async_runtime::spawn(party.clone().run(stop.clone()));
    tauri::async_runtime::spawn(lobby.clone().run(stop.clone()));
    let social = SocialService::new(ctx.clone(), assets.clone());
    tauri::async_runtime::spawn(social.clone().run(stop.clone()));
    app.manage(social);
    tauri::async_runtime::spawn(pregame.clone().run(stop.clone()));
    tauri::async_runtime::spawn(wallet.clone().run(stop));
    app.manage(party);
    app.manage(pregame);
    app.manage(wallet);
    start_career(app, ctx, assets, lobby.clone());
    app.manage(lobby);
    app.manage(StoreService::new(ctx.clone(), assets.clone()));
    app.manage(CollectionService::new(ctx.clone(), assets.clone()));
}

/// Career, match history and the post-match summary (both read RR changes through the career).
fn start_career(app: &tauri::AppHandle, ctx: &Ctx, assets: &AssetStore, lobby: LobbyService) {
    let stop = app.state::<CancellationToken>().inner().clone();
    let career = CareerService::new(ctx.clone(), assets.clone());
    let post_match = PostMatchService::new(ctx.clone(), assets.clone(), lobby, career.clone());
    tauri::async_runtime::spawn(post_match.clone().run(stop));
    app.manage(post_match);
    app.manage(HistoryService::new(
        ctx.clone(),
        assets.clone(),
        career.clone(),
    ));
    app.manage(career);
}

/// A second launch (or the tray) brings the running window forward.
pub(crate) fn focus_main_window(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
        return;
    };
    if let Err(error) = window.show().and_then(|()| window.set_focus()) {
        tracing::warn!(%error, "focus_main_window_failed");
    }
}
