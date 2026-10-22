// Agent select: the live pregame view and the player's own pick actions.
import { createSignal } from "solid-js";
import type { PregameView } from "~/bindings/PregameView";
import { call } from "~/ipc/commands";
import { onAppEvent } from "~/ipc/events";
import { playCue } from "~/menu/sound";
import { attempt } from "./toasts";

const [pregame, setPregame] = createSignal<PregameView | null>(null);

onAppEvent("pregame", setPregame);
void call("get_pregame", {}).then((current) => current && setPregame(current), () => undefined);

export { pregame };

/** Seconds left on the select timer at `now`. */
export function timeLeft(view: PregameView, now: number): number {
  return Math.max(0, view.time_remaining_s - (now - view.observed_at_ms) / 1000);
}

export const pregameActions = {
  select: (agentUuid: string) => {
    playCue("hover");
    return attempt(() => call("pregame_select", { agentUuid }), "Could not select");
  },
  lock: (agentUuid: string) => {
    playCue("confirm");
    return attempt(() => call("pregame_lock", { agentUuid }), "Could not lock");
  },
  dodge: () => {
    playCue("back");
    return attempt(() => call("pregame_dodge", {}), "Could not leave");
  },
};
