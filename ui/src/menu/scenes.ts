// Scenes of the menu. Tabs appear in the top nav in this order; Q/E cycle
// them. Every scene is a Studio stage, so all of it can be rearranged in edit mode.
import type { StageId } from "~/studio/types";

export type SceneId = StageId;

export interface SceneInfo {
  id: SceneId;
  label: string;
  /** Shown as a top-nav tab. Automatic scenes (agent select) are entered by match flow. */
  isTab: boolean;
}

export const SCENES: readonly SceneInfo[] = [
  { id: "home", label: "Home", isTab: true },
  { id: "play", label: "Play", isTab: true },
  { id: "agents", label: "Agents", isTab: true },
  { id: "collection", label: "Collection", isTab: true },
  { id: "career", label: "Career", isTab: true },
  { id: "store", label: "Store", isTab: true },
  { id: "agent-select", label: "Agent Select", isTab: false },
  { id: "live", label: "Live", isTab: false },
  { id: "post-match", label: "Match summary", isTab: false },
  { id: "ingame", label: "In-game overlay", isTab: false },
];

export const TABS: readonly SceneInfo[] = SCENES.filter((scene) => scene.isTab);

export function getScene(id: string): SceneInfo | undefined {
  return SCENES.find((scene) => scene.id === id);
}

/** The stage shown at a route path (`/play` -> `play`). */
export function sceneForPath(pathname: string): SceneId {
  return getScene(pathname.replace(/^\//, "").split("/")[0] ?? "")?.id ?? "home";
}

/** The tab `step` positions away from `current`, wrapping around (Q = -1, E = +1). */
export function getAdjacentTab(current: string, step: -1 | 1): SceneInfo {
  const index = TABS.findIndex((tab) => tab.id === current);
  const start = index === -1 ? 0 : index;
  const next = (start + step + TABS.length) % TABS.length;
  return TABS[next] ?? TABS[0]!;
}
