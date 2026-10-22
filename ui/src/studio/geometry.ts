// Pixel geometry for the editor: converting stage fractions to px, resizing from an edge,
// clamping into the stage, and snapping to alignment guides.
import type { Rect } from "./types";

export interface PxRect {
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface StageSize {
  w: number;
  h: number;
}

export type Edge = "n" | "ne" | "e" | "se" | "s" | "sw" | "w" | "nw";
export const EDGES: readonly Edge[] = ["n", "ne", "e", "se", "s", "sw", "w", "nw"];

export function toPx(rect: Rect, stage: StageSize): PxRect {
  return { x: rect.x * stage.w, y: rect.y * stage.h, w: rect.w * stage.w, h: rect.h * stage.h };
}

export function toFraction(rect: PxRect, stage: StageSize): Rect {
  const round = (value: number) => Math.round(value * 10000) / 10000;
  return {
    x: round(rect.x / Math.max(stage.w, 1)),
    y: round(rect.y / Math.max(stage.h, 1)),
    w: round(rect.w / Math.max(stage.w, 1)),
    h: round(rect.h / Math.max(stage.h, 1)),
  };
}

/** Resize `start` by dragging `edge` by (dx, dy); optionally keep the aspect ratio. */
export function resizeFromEdge(start: PxRect, edge: Edge, dx: number, dy: number, isProportional: boolean): PxRect {
  let { x, y, w, h } = start;
  if (edge.includes("e")) w = start.w + dx;
  if (edge.includes("w")) w = start.w - dx;
  if (edge.includes("s")) h = start.h + dy;
  if (edge.includes("n")) h = start.h - dy;
  const isCorner = edge.length === 2;
  if (isProportional && isCorner && start.h > 0) {
    const ratio = start.w / start.h;
    if (Math.abs(w / ratio - start.h) > Math.abs(h * ratio - start.w)) h = w / ratio;
    else w = h * ratio;
  }
  if (edge.includes("w")) x = start.x + start.w - w;
  if (edge.includes("n")) y = start.y + start.h - h;
  return { x, y, w, h };
}

/** Hard clamp: inside the stage, at least the minimum size. */
export function clampRect(rect: PxRect, stage: StageSize, min: { w: number; h: number }): PxRect {
  const w = Math.min(Math.max(rect.w, Math.min(min.w, stage.w)), stage.w);
  const h = Math.min(Math.max(rect.h, Math.min(min.h, stage.h)), stage.h);
  return {
    x: Math.min(Math.max(rect.x, 0), stage.w - w),
    y: Math.min(Math.max(rect.y, 0), stage.h - h),
    w,
    h,
  };
}

export interface Guides {
  xs: number[];
  ys: number[];
}

/** Lines worth aligning to: stage edges and centre, and every other piece's edges/centre. */
export function collectGuides(stage: StageSize, others: readonly PxRect[]): Guides {
  const xs = [0, stage.w / 2, stage.w];
  const ys = [0, stage.h / 2, stage.h];
  for (const rect of others) {
    xs.push(rect.x, rect.x + rect.w / 2, rect.x + rect.w);
    ys.push(rect.y, rect.y + rect.h / 2, rect.y + rect.h);
  }
  return { xs, ys };
}

/** The smallest offset that puts one of `points` on one of `lines`, if within `threshold`. */
function nearestOffset(points: readonly number[], lines: readonly number[], threshold: number): number | null {
  let best: number | null = null;
  for (const point of points) {
    for (const line of lines) {
      const offset = line - point;
      if (Math.abs(offset) <= threshold && (best === null || Math.abs(offset) < Math.abs(best))) best = offset;
    }
  }
  return best;
}

/** Snap a moved rect so an edge or its centre lands on a guide. */
export function snapMove(rect: PxRect, guides: Guides, threshold: number): PxRect {
  const dx = nearestOffset([rect.x, rect.x + rect.w / 2, rect.x + rect.w], guides.xs, threshold);
  const dy = nearestOffset([rect.y, rect.y + rect.h / 2, rect.y + rect.h], guides.ys, threshold);
  return { ...rect, x: rect.x + (dx ?? 0), y: rect.y + (dy ?? 0) };
}

/** Snap only the edges being dragged during a resize. */
export function snapResize(rect: PxRect, edge: Edge, guides: Guides, threshold: number): PxRect {
  const next = { ...rect };
  if (edge.includes("e")) next.w += nearestOffset([rect.x + rect.w], guides.xs, threshold) ?? 0;
  if (edge.includes("s")) next.h += nearestOffset([rect.y + rect.h], guides.ys, threshold) ?? 0;
  if (edge.includes("w")) {
    const offset = nearestOffset([rect.x], guides.xs, threshold) ?? 0;
    next.x += offset;
    next.w -= offset;
  }
  if (edge.includes("n")) {
    const offset = nearestOffset([rect.y], guides.ys, threshold) ?? 0;
    next.y += offset;
    next.h -= offset;
  }
  return next;
}

/** Guide lines the rect currently touches (for drawing). */
export function touchingGuides(rect: PxRect, guides: Guides, tolerance: number): Guides {
  const xPoints = [rect.x, rect.x + rect.w / 2, rect.x + rect.w];
  const yPoints = [rect.y, rect.y + rect.h / 2, rect.y + rect.h];
  return {
    xs: [...new Set(guides.xs.filter((line) => xPoints.some((point) => Math.abs(point - line) <= tolerance)))],
    ys: [...new Set(guides.ys.filter((line) => yPoints.some((point) => Math.abs(point - line) <= tolerance)))],
  };
}
