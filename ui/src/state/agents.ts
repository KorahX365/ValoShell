// Owned agents and Agent Mastery levels: read on connect, after every match and when a page
// asks for a fresh copy. Until the first answer every agent counts as owned, so nothing is
// greyed out by mistake while offline.
import { createMemo, createRoot, createSignal } from "solid-js";
import type { AgentProgress } from "~/bindings/AgentProgress";
import { call } from "~/ipc/commands";
import { onAppEvent } from "~/ipc/events";

const [progress, setProgress] = createSignal<AgentProgress | null>(null);

/** Re-read ownership and mastery; `isForced` skips the backend's short reuse window. */
export function refreshAgentProgress(isForced = false): void {
  void call("get_agent_progress", { isForced }).then(setProgress, () => undefined);
}

refreshAgentProgress();
onAppEvent("connection", (status) => {
  if (status.state === "connected") refreshAgentProgress();
  else setProgress(null);
});
onAppEvent("post_match", () => refreshAgentProgress(true));

const owned = createRoot(() => createMemo(() => new Set(progress()?.owned ?? [])));
const mastery = createRoot(() =>
  createMemo(() => new Map((progress()?.mastery ?? []).map((row) => [row.agent_uuid, row]))),
);

/** Whether the player can pick this agent; true while ownership is unknown. */
export function isAgentOwned(uuid: string): boolean {
  return progress() === null || owned().has(uuid.toLowerCase());
}

/** Lifetime and act mastery levels; zeros for an agent never played. */
export function masteryOf(uuid: string): { lifetime: number; act: number } {
  const row = mastery().get(uuid.toLowerCase());
  return { lifetime: row?.lifetime_level ?? 0, act: row?.act_level ?? 0 };
}
