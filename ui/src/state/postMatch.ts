// The summary of the match just played: arrives a few seconds to a minute after the match
// (when Riot publishes it), again when the RR change is known, and once the player closed it.
import { createSignal } from "solid-js";
import type { PostMatchView } from "~/bindings/PostMatchView";
import { call } from "~/ipc/commands";
import { onAppEvent } from "~/ipc/events";

const [postMatch, setPostMatch] = createSignal<PostMatchView | null>(null);

onAppEvent("post_match", setPostMatch);
void call("get_post_match", {}).then((current) => current && setPostMatch(current), () => undefined);

export { postMatch };

/** The player closed the screen; the summary stays readable as the last match. */
export function dismissPostMatch(): void {
  const current = postMatch();
  if (current && !current.is_seen) setPostMatch({ ...current, is_seen: true });
  void call("dismiss_post_match", {}).catch(() => undefined);
}
