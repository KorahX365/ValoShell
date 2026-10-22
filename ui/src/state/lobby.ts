// The live lobby (everyone's rank and stacks), filled progressively by the backend.
import { createSignal } from "solid-js";
import type { LobbyView } from "~/bindings/LobbyView";
import { call } from "~/ipc/commands";
import { onAppEvent } from "~/ipc/events";

const [lobby, setLobby] = createSignal<LobbyView | null>(null);

onAppEvent("lobby", setLobby);
void call("get_lobby", {}).then((current) => current && setLobby(current), () => undefined);

export { lobby };
