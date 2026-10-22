// Connection and live game state, kept current by backend events.
import { createSignal } from "solid-js";
import type { ConnectionStatus } from "~/bindings/ConnectionStatus";
import type { GameSnapshot } from "~/bindings/GameSnapshot";
import { call } from "~/ipc/commands";
import { onAppEvent } from "~/ipc/events";

const [connection, setConnection] = createSignal<ConnectionStatus>({ state: "waiting_for_riot" });
const [game, setGame] = createSignal<GameSnapshot | undefined>();

onAppEvent("connection", setConnection);
onAppEvent("game_state", setGame);
void call("get_connection", {}).then(setConnection, () => undefined);
void call("get_game_snapshot", {}).then(setGame, () => undefined);

export { connection, game };

export const CONNECTION_COPY: Record<ConnectionStatus["state"], string> = {
  waiting_for_riot: "Waiting for the Riot Client",
  riot_found: "Riot Client found, waiting for VALORANT",
  connecting: "Connecting",
  connected: "Connected",
  failed: "Connection failed",
};

/** One line describing the connection, for status text. */
export function connectionLine(status: ConnectionStatus): string {
  return status.state === "failed" ? status.message : CONNECTION_COPY[status.state];
}

export const isConnected = () => connection().state === "connected";

/** `Name` and `Tag` of the signed-in player, when known. */
export function ownRiotId(): { name: string; tag: string } | null {
  const status = connection();
  if (status.state !== "connected" || !status.riot_id) return null;
  const [name = "", tag = ""] = status.riot_id.split("#");
  return { name, tag };
}
