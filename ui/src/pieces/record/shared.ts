// What every Service Record layout reads from the career view: the act record, the recent
// ranked games summed up per map and per rank, the streak, and the rank ladder. Plus the
// count-up used for big numbers.
import { createEffect, createSignal, on, onCleanup } from "solid-js";
import type { CareerView } from "~/bindings/CareerView";
import type { RankBadge } from "~/bindings/RankBadge";
import type { RrUpdate } from "~/bindings/RrUpdate";
import { party } from "~/state/party";
import { ownRiotId } from "~/state/session";
import { isMotionReduced } from "~/studio/spring";

export type Outcome = "win" | "loss" | "draw";

export function outcome(update: RrUpdate): Outcome {
  if (update.rr_earned > 0) return "win";
  if (update.rr_earned < 0) return "loss";
  return "draw";
}

export function signed(value: number): string {
  return value > 0 ? `+${value}` : `${value}`;
}

export interface MapLine {
  name: string;
  image: string | null;
  games: number;
  wins: number;
  net: number;
}

export interface RankLine {
  rank: RankBadge;
  wins: number;
  losses: number;
}

export interface RecordStats {
  losses: number;
  winRate: number;
  /** Ranked games in the history (the latest ones, not the whole act). */
  recent: number;
  recentWins: number;
  net: number;
  best: number;
  worst: number;
  /** Current run of wins or losses, newest first. */
  streak: { isWin: boolean; length: number } | null;
  promotions: number;
  maps: MapLine[];
  ranks: RankLine[];
}

export function recordStats(view: CareerView): RecordStats {
  const updates = view.updates;
  const maps = new Map<string, MapLine>();
  const ranks = new Map<number, RankLine>();
  for (const update of updates) {
    const result = outcome(update);
    const map = maps.get(update.map_name) ?? { name: update.map_name, image: update.map_image, games: 0, wins: 0, net: 0 };
    map.games += 1;
    map.wins += result === "win" ? 1 : 0;
    map.net += update.rr_earned;
    maps.set(update.map_name, map);
    const rank = ranks.get(update.rank_after.tier) ?? { rank: update.rank_after, wins: 0, losses: 0 };
    if (result === "win") rank.wins += 1;
    if (result === "loss") rank.losses += 1;
    ranks.set(update.rank_after.tier, rank);
  }
  const first = updates[0];
  let streakLength = 0;
  if (first) {
    const kind = outcome(first);
    while (streakLength < updates.length && outcome(updates[streakLength] as RrUpdate) === kind) streakLength += 1;
  }
  const deltas = updates.map((update) => update.rr_earned);
  return {
    losses: Math.max(0, view.games - view.wins),
    winRate: view.games > 0 ? Math.round((view.wins / view.games) * 100) : 0,
    recent: updates.length,
    recentWins: updates.filter((update) => outcome(update) === "win").length,
    net: deltas.reduce((sum, delta) => sum + delta, 0),
    best: deltas.length > 0 ? Math.max(...deltas) : 0,
    worst: deltas.length > 0 ? Math.min(...deltas) : 0,
    streak: first && outcome(first) !== "draw" ? { isWin: outcome(first) === "win", length: streakLength } : null,
    promotions: updates.filter((update) => update.is_promotion).length,
    maps: [...maps.values()].sort((a, b) => b.games - a.games || b.net - a.net),
    ranks: [...ranks.values()].sort((a, b) => b.rank.tier - a.rank.tier),
  };
}

// ---- the ladder ---------------------------------------------------------------------------

export interface TierGroup {
  name: string;
  color: string;
}

/** The nine rank groups, Iron to Radiant, with a colour close to each emblem. */
export const TIER_GROUPS: readonly TierGroup[] = [
  { name: "Iron", color: "#8d8d8d" },
  { name: "Bronze", color: "#b8875a" },
  { name: "Silver", color: "#d7dcdc" },
  { name: "Gold", color: "#eccf56" },
  { name: "Platinum", color: "#4fbfd4" },
  { name: "Diamond", color: "#c08cf2" },
  { name: "Ascendant", color: "#4fd69a" },
  { name: "Immortal", color: "#e0466f" },
  { name: "Radiant", color: "#fff3a6" },
];

/** Index into TIER_GROUPS for a tier (3 = Iron 1 ... 27 = Radiant), or -1 when unranked. */
export function groupIndex(tier: number): number {
  if (tier < 3) return -1;
  return Math.min(TIER_GROUPS.length - 1, Math.floor((tier - 3) / 3));
}

/** 1-3 within the group; 0 for Radiant and unranked. */
export function division(tier: number): number {
  if (tier < 3 || tier >= 27) return 0;
  return ((tier - 3) % 3) + 1;
}

export function nextRankName(tier: number): string {
  if (tier < 3) return "Iron 1";
  if (tier >= 27) return "Radiant";
  const next = tier + 1;
  const group = TIER_GROUPS[groupIndex(next)]?.name ?? "";
  return division(next) > 0 ? `${group} ${division(next)}` : group;
}

// ---- motion -------------------------------------------------------------------------------

/** A number that counts up to its target (and eases to each new one); instant with reduced motion. */
export function useCountUp(target: () => number, delayMs = 0, durationMs = 1000): () => number {
  const [value, setValue] = createSignal(isMotionReduced() ? target() : 0);
  let frame = 0;
  createEffect(
    on(target, (to) => {
      cancelAnimationFrame(frame);
      const from = value();
      if (isMotionReduced() || from === to) {
        setValue(to);
        return;
      }
      const start = performance.now() + delayMs;
      const tick = (now: number) => {
        const progress = Math.min(1, Math.max(0, (now - start) / durationMs));
        const eased = 1 - (1 - progress) ** 3;
        setValue(Math.round(from + (to - from) * eased));
        if (progress < 1) frame = requestAnimationFrame(tick);
      };
      frame = requestAnimationFrame(tick);
    }),
  );
  onCleanup(() => cancelAnimationFrame(frame));
  return value;
}

/** What each Service Record layout is given. */
export interface RecordProps {
  view: CareerView;
  stats: RecordStats;
  isEditing: boolean;
}

export interface Identity {
  name: string;
  tag: string;
  level: number | null;
  card: string | null;
}

/** The signed-in player as the party shows them, for the record's header. */
export function identity(): Identity {
  const self = party()?.members.find((member) => member.is_self);
  const riotId = ownRiotId();
  return {
    name: riotId?.name || self?.game_name || "Player",
    tag: riotId?.tag || self?.tag_line || "",
    level: self?.account_level ?? null,
    card: self?.card_wide ?? null,
  };
}
