// What the store pieces know about an item beyond its price: its edition (read from the
// price, as VALORANT tints its cards), its weapon and finish ("Reaver Vandal" -> Vandal,
// Reaver), and a silhouette to draw when its picture can't load, so a store without images
// still reads as rifles, knives and cards rather than initials.
import { For, createUniqueId } from "solid-js";
import type { StoreItem } from "~/bindings/StoreItem";
import type { BundleView } from "~/bindings/BundleView";
import { Art } from "~/ui/Art";
import styles from "./storeArt.module.css";

export type EditionTone = "select" | "deluxe" | "premium" | "exclusive" | "ultra" | "accessory";

export interface Edition {
  name: string;
  tone: EditionTone;
}

/** Melee prices, which run at about twice the gun price of the same edition. */
const MELEE_PRICES = new Set([1750, 2550, 3550, 4350, 5350]);
const MELEE_FLOOR = 3000;

/** VALORANT edition from a skin's price (guns 875 Select ... 2475+ Ultra; melees double). */
export function editionOf(price: number): Edition {
  const gunPrice = MELEE_PRICES.has(price) || price >= MELEE_FLOOR ? price / 2 : price;
  if (gunPrice >= 2475) return { name: "Ultra", tone: "ultra" };
  if (gunPrice >= 2175) return { name: "Exclusive", tone: "exclusive" };
  if (gunPrice >= 1775) return { name: "Premium", tone: "premium" };
  if (gunPrice >= 1275) return { name: "Deluxe", tone: "deluxe" };
  return { name: "Select", tone: "select" };
}

/** Accessories (Kingdom Credits) have no edition; they get their own quiet tone. */
export function gradeOf(item: StoreItem): Edition {
  return item.currency === "kc" ? { name: "Accessory", tone: "accessory" } : editionOf(item.price);
}

/** The game's own edition colours. */
export const EDITION_TINT: Record<EditionTone, string> = {
  select: "#5a9fe2",
  deluxe: "#009587",
  premium: "#d1548d",
  exclusive: "#f5955b",
  ultra: "#fad663",
  accessory: "#9aa7b4",
};

export type ItemKind = "rifle" | "sniper" | "smg" | "pistol" | "shotgun" | "heavy" | "melee" | "card" | "spray" | "buddy" | "title";

const WEAPON_KINDS: Record<string, ItemKind> = {
  Classic: "pistol",
  Shorty: "pistol",
  Frenzy: "pistol",
  Ghost: "pistol",
  Sheriff: "pistol",
  Stinger: "smg",
  Spectre: "smg",
  Bucky: "shotgun",
  Judge: "shotgun",
  Bulldog: "rifle",
  Guardian: "rifle",
  Phantom: "rifle",
  Vandal: "rifle",
  Marshal: "sniper",
  Outlaw: "sniper",
  Operator: "sniper",
  Ares: "heavy",
  Odin: "heavy",
};

const ACCESSORY_KINDS: [string, ItemKind][] = [
  ["Player Card", "card"],
  ["Card", "card"],
  ["Spray", "spray"],
  ["Buddy", "buddy"],
  ["Title", "title"],
];

export interface ItemParts {
  kind: ItemKind;
  /** The weapon or accessory type: "Vandal", "Axe", "Player Card". */
  base: string;
  /** The skin line: "Reaver", "Neo Frontier"; empty when the name is only the base. */
  finish: string;
}

/** Split a store name into weapon and finish, and guess the silhouette. */
export function partsOf(item: StoreItem): ItemParts {
  const words = item.name.trim().split(/\s+/);
  const weapon = words.find((word) => word in WEAPON_KINDS);
  if (weapon) {
    return { kind: WEAPON_KINDS[weapon] ?? "rifle", base: weapon, finish: words.filter((word) => word !== weapon).join(" ") };
  }
  for (const [suffix, kind] of ACCESSORY_KINDS) {
    if (item.name.endsWith(suffix)) return { kind, base: suffix, finish: item.name.slice(0, -suffix.length).trim() };
  }
  // A VP item that isn't a gun is a melee ("Neo Frontier Axe", "Reaver Karambit").
  const base = words.length > 1 ? (words.at(-1) ?? item.name) : item.name;
  return { kind: item.currency === "kc" ? "card" : "melee", base, finish: words.slice(0, -1).join(" ") };
}

/** Silhouettes, muzzle to the right. Guns share a 200x64 box so they line up across tiles. */
const GLYPHS: Record<ItemKind, { box: string; d: string }> = {
  rifle: {
    box: "0 0 200 64",
    d: "M4 25 L44 20 L62 18 L66 13 L92 13 L94 18 L148 18 L152 21 L196 21 L196 26 L152 26 L148 29 L116 29 L110 50 L97 48 L101 29 L86 29 L84 45 L73 45 L71 30 L50 30 L44 33 L10 43 L4 41 Z",
  },
  sniper: {
    box: "0 0 200 64",
    d: "M2 28 L30 24 L46 23 L50 20 L120 20 L122 23 L198 23 L198 27 L124 28 L118 31 L96 31 L94 44 L84 44 L82 31 L56 31 L48 35 L8 44 L2 42 Z M58 8 L104 8 L108 11 L108 15 L104 18 L58 18 L54 15 L54 11 Z M70 17 L75 17 L75 21 L70 21 Z M88 17 L93 17 L93 21 L88 21 Z",
  },
  smg: {
    box: "0 0 200 64",
    d: "M20 24 L52 22 L60 18 L134 18 L136 22 L172 22 L172 28 L136 28 L130 30 L104 30 L100 52 L90 52 L92 30 L80 30 L78 44 L68 44 L66 30 L50 30 L24 36 L20 34 Z",
  },
  pistol: {
    box: "0 0 200 64",
    d: "M58 16 L146 16 L148 20 L146 27 L104 27 L100 33 L90 33 L88 30 L84 30 L80 52 L60 52 L66 30 L58 27 Z",
  },
  shotgun: {
    box: "0 0 200 64",
    d: "M4 28 L40 22 L60 20 L180 20 L184 22 L196 22 L196 27 L184 27 L180 30 L150 30 L150 35 L110 35 L110 30 L80 30 L78 44 L68 44 L66 31 L50 31 L10 42 L4 40 Z",
  },
  heavy: {
    box: "0 0 200 64",
    d: "M6 26 L40 20 L56 20 L58 14 L130 14 L132 18 L150 18 L150 16 L182 16 L182 20 L196 20 L196 28 L182 28 L182 32 L150 32 L150 30 L130 30 L126 34 L108 34 L108 52 L78 52 L78 34 L70 34 L68 46 L58 46 L56 32 L44 32 L12 42 L6 40 Z",
  },
  melee: {
    box: "0 0 200 64",
    d: "M30 30 L80 28 L84 22 L90 22 L92 28 L100 27 L170 21 L190 27 L170 35 L100 37 L92 36 L90 42 L84 42 L82 36 L30 36 L26 33 Z",
  },
  card: {
    box: "0 0 64 64",
    d: "M18 4 H46 V60 H18 Z M25 20 a7 7 0 1 0 14 0 a7 7 0 1 0 -14 0 Z M21 52 C23 38 41 38 43 52 Z",
  },
  spray: {
    box: "0 0 64 64",
    d: "M32 6 L37 20 L52 12 L44 27 L60 32 L44 37 L52 52 L37 44 L32 58 L27 44 L12 52 L20 37 L4 32 L20 27 L12 12 L27 20 Z",
  },
  buddy: {
    box: "0 0 64 64",
    d: "M28 6 a4 4 0 1 0 8 0 a4 4 0 1 0 -8 0 Z M30 9 H34 V18 H30 Z M32 18 C43 18 48 27 48 36 C48 48 40 60 32 60 C24 60 16 48 16 36 C16 27 21 18 32 18 Z",
  },
  title: {
    box: "0 0 64 64",
    d: "M8 24 H56 A4 4 0 0 1 60 28 V36 A4 4 0 0 1 56 40 H8 A4 4 0 0 1 4 36 V28 A4 4 0 0 1 8 24 Z",
  },
};

/** The silhouette alone; layouts colour it through --glyph-a / --glyph-b (and may stroke it). */
export function ItemGlyph(props: { item: StoreItem; class?: string | undefined }) {
  const id = createUniqueId();
  const glyph = () => GLYPHS[partsOf(props.item).kind];
  return (
    <svg class={`${styles.glyph} ${props.class ?? ""}`} viewBox={glyph().box} aria-hidden="true" data-part="item-glyph">
      <defs>
        <linearGradient id={`g${id}`} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" style={{ "stop-color": "var(--glyph-a, currentColor)" }} />
          <stop offset="1" style={{ "stop-color": "var(--glyph-b, currentColor)" }} />
        </linearGradient>
      </defs>
      <path d={glyph().d} fill={`url(#g${id})`} fill-rule="evenodd" />
    </svg>
  );
}

/** The item's picture, or its silhouette while that loads or when it can't. */
export function ItemArt(props: { item: StoreItem; class?: string | undefined }) {
  return <Art src={props.item.image} class={props.class} fallback={<ItemGlyph item={props.item} class={props.class} />} />;
}

/** Long guns read best up front; pistols and melees go behind. */
const HERO_RANK: Record<ItemKind, number> = { sniper: 0, rifle: 1, heavy: 2, shotgun: 3, smg: 4, melee: 5, pistol: 6, card: 7, spray: 8, buddy: 9, title: 10 };

function heroOrder(items: StoreItem[]): StoreItem[] {
  return [...items].sort((a, b) => HERO_RANK[partsOf(a).kind] - HERO_RANK[partsOf(b).kind]);
}

/** A bundle's key art, or its first items fanned out when there's none. */
export function BundleArt(props: { bundle: BundleView; class?: string | undefined }) {
  return (
    <Art
      src={props.bundle.image}
      class={props.class}
      fallback={
        <span class={styles.fan} data-part="bundle-fan">
          <For each={heroOrder(props.bundle.items).slice(0, 3)}>
            {(item) => <ItemGlyph item={item} class={styles.fanItem} />}
          </For>
        </span>
      }
    />
  );
}
