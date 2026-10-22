// Every backend command with its argument and result types. `call` is the only way the UI
// talks to Rust; in a plain browser (dev preview, filming without the game) the demo backend
// answers instead, so every screen stays usable.
import { invoke } from "@tauri-apps/api/core";
import type { CareerView } from "~/bindings/CareerView";
import type { Catalog } from "~/bindings/Catalog";
import type { CollectionView } from "~/bindings/CollectionView";
import type { ConnectionStatus } from "~/bindings/ConnectionStatus";
import type { FriendView } from "~/bindings/FriendView";
import type { GameSnapshot } from "~/bindings/GameSnapshot";
import type { GameVideo } from "~/bindings/GameVideo";
import type { UiSound } from "~/bindings/UiSound";
import type { LobbyView } from "~/bindings/LobbyView";
import type { PartyView } from "~/bindings/PartyView";
import type { PostMatchView } from "~/bindings/PostMatchView";
import type { PregameView } from "~/bindings/PregameView";
import type { Settings } from "~/bindings/Settings";
import type { SkinOption } from "~/bindings/SkinOption";
import type { StoreView } from "~/bindings/StoreView";
import type { HistoryView } from "~/bindings/HistoryView";
import type { InstantCloneAction } from "~/bindings/InstantCloneAction";
import type { InstantCloneState } from "~/bindings/InstantCloneState";
import type { InstantCloneTarget } from "~/bindings/InstantCloneTarget";
import type { NewsView } from "~/bindings/NewsView";
import type { WalletView } from "~/bindings/WalletView";
import { isTauri } from "./runtime";

type NoArgs = Record<string, never>;

export interface CommandMap {
  get_settings: [NoArgs, Settings];
  save_settings: [{ settings: Settings }, Settings];
  get_game_snapshot: [NoArgs, GameSnapshot];
  get_connection: [NoArgs, ConnectionStatus];
  get_catalog: [NoArgs, Catalog];
  get_game_videos: [NoArgs, GameVideo[]];
  get_ui_sounds: [NoArgs, UiSound[]];
  open_sounds_folder: [NoArgs, null];
  resolve_embed_link: [{ url: string }, string | null];
  get_wallet: [NoArgs, WalletView | null];
  quit_app: [NoArgs, null];
  get_store: [{ isForced: boolean }, StoreView];
  get_career: [{ isForced: boolean }, CareerView];
  get_match_history: [{ isForced: boolean }, HistoryView];
  get_news: [{ locale: string }, NewsView | null];
  get_lobby: [NoArgs, LobbyView | null];
  get_post_match: [NoArgs, PostMatchView | null];
  dismiss_post_match: [NoArgs, null];
  get_friends: [NoArgs, FriendView[]];
  get_collection: [NoArgs, CollectionView];
  get_skin_options: [{ weaponId: string }, SkinOption[]];
  equip_skin: [{ weaponId: string; skinId: string; chromaId: string }, CollectionView];
  get_party: [NoArgs, PartyView | null];
  party_refresh: [NoArgs, PartyView | null];
  party_change_queue: [{ queueId: string }, null];
  party_start_queue: [NoArgs, null];
  party_stop_queue: [NoArgs, null];
  party_set_ready: [{ isReady: boolean }, null];
  party_set_open: [{ isOpen: boolean }, null];
  party_invite: [{ riotId: string }, null];
  party_kick: [{ puuid: string }, null];
  party_leave: [NoArgs, null];
  party_generate_code: [NoArgs, null];
  party_join_code: [{ code: string }, null];
  party_change_team: [{ team: "attackers" | "defenders" | "spectators"; puuid: string }, null];
  party_start_custom: [NoArgs, null];
  get_pregame: [NoArgs, PregameView | null];
  pregame_select: [{ agentUuid: string }, null];
  pregame_lock: [{ agentUuid: string }, null];
  pregame_dodge: [NoArgs, null];
  list_ui_documents: [{ kind: DocumentKind }, string[]];
  load_ui_document: [{ kind: DocumentKind; name: string }, unknown];
  save_ui_document: [{ kind: DocumentKind; name: string; document: unknown }, null];
  delete_ui_document: [{ kind: DocumentKind; name: string }, null];
  set_overlay_enabled: [{ isEnabled: boolean }, null];
  set_overlay_preview: [{ isVisible: boolean }, null];
  instant_clone_state: [{ target: InstantCloneTarget }, InstantCloneState | null];
  instant_clone_send: [{ target: InstantCloneTarget; action: InstantCloneAction; delayMs: number }, null];
}

export type DocumentKind = "layouts" | "themes";
export type Command = keyof CommandMap;
export type ArgsOf<C extends Command> = CommandMap[C][0];
export type ResultOf<C extends Command> = CommandMap[C][1];

/** Run a backend command. Rejects with the backend's plain-language message. */
export async function call<C extends Command>(command: C, args: ArgsOf<C>): Promise<ResultOf<C>> {
  if (isTauri) {
    return invoke<ResultOf<C>>(command, args);
  }
  const demo = await import("./demo");
  return demo.handle(command, args);
}

/** The message of a rejected `call`, whatever shape it came in. */
export function errorText(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return "Something went wrong.";
}
