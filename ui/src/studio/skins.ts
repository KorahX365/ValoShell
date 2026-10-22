// Menu styles that echo other games' menus. Picking one applies its whole theme bundle
// (colours, display font, corners, grain, sometimes the backdrop) and lays out Home, Play,
// Career and Store with that style's own pieces; TopNav draws the style's own nav and
// theme/skins.css restyles panels and controls. Everything stays editable afterwards, and one
// undo restores the previous setup.
import { DEFAULT_THEME, piece } from "./defaults";
import type { Backdrop, DisplayFont, Piece, Skin, StageId, StudioDoc } from "./types";

export interface SkinBundle {
  id: Skin;
  label: string;
  caption: string;
  accent: string;
  accent2: string;
  ground: string;
  text: string;
  /** PLAY colour, for the picker preview only (the stylesheet sets the real one). */
  play: string;
  displayFont: DisplayFont;
  cut: number;
  isGrainy: boolean;
  vignette: number;
  /** Backdrop fields the style needs to read right (e.g. CS:GO's bright haze). */
  backdrop?: Partial<Backdrop>;
}

export const SKINS: readonly SkinBundle[] = [
  {
    id: "modern",
    label: "Modern",
    caption: "ValoShell",
    accent: "#ff4655",
    accent2: "#ffb4a8",
    ground: "#0a0a0e",
    text: "#ededf0",
    play: "#ff4655",
    displayFont: "saira",
    cut: 18,
    isGrainy: true,
    vignette: 0.7,
  },
  {
    id: "glass",
    label: "VALORANT Glass",
    caption: "Concept · frosted",
    accent: "#ff4655",
    accent2: "#ff9fa8",
    ground: "#08090d",
    text: "#f5f5f7",
    play: "#ff4655",
    displayFont: "inter",
    cut: 0,
    isGrainy: false,
    vignette: 0.45,
    backdrop: { kind: "game", isAscii: false, dim: 0.3, blur: 14, motion: "drift", motionSpeed: 0.5 },
  },
  {
    id: "current",
    label: "VALORANT",
    caption: "Today's client",
    accent: "#ff4655",
    accent2: "#2ee6c5",
    ground: "#0f1923",
    text: "#ece8e1",
    play: "#ff4655",
    displayFont: "tall",
    cut: 0,
    isGrainy: false,
    vignette: 0.25,
    backdrop: { kind: "shader", shader: "diamonds", isAscii: false, dim: 0, blur: 0, motion: "none" },
  },
  {
    id: "beta",
    label: "VALORANT Beta",
    caption: "2020 · Project A",
    accent: "#ecebbd",
    accent2: "#bfe1dd",
    ground: "#0c1315",
    text: "#eef1f0",
    play: "#bfe1dd",
    displayFont: "saira",
    cut: 0,
    isGrainy: true,
    vignette: 0.75,
  },
  {
    id: "valnow",
    label: "VALORANT 2022",
    caption: "Side menu",
    accent: "#ff4655",
    accent2: "#ece8e1",
    ground: "#0f1923",
    text: "#ece8e1",
    play: "#ff4655",
    displayFont: "tall",
    cut: 6,
    isGrainy: false,
    vignette: 0.5,
  },
  {
    id: "csgo",
    label: "CS:GO",
    caption: "Panorama 2018",
    accent: "#de9b35",
    accent2: "#b7c7d6",
    ground: "#1f2a35",
    text: "#e6eaee",
    play: "#5a9a3a",
    displayFont: "saira",
    cut: 0,
    isGrainy: false,
    vignette: 0.35,
    backdrop: { kind: "shader", shader: "haze", isAscii: false, dim: 0, blur: 0 },
  },
  {
    id: "cs2",
    label: "CS2",
    caption: "Counter-Strike 2",
    accent: "#f0f2f5",
    accent2: "#8fb1d4",
    ground: "#0d1015",
    text: "#f0f2f5",
    play: "#4f9d3d",
    displayFont: "saira",
    cut: 0,
    isGrainy: false,
    vignette: 0.5,
  },
  {
    id: "cs16",
    label: "CS 1.6",
    caption: "2003 · VGUI",
    accent: "#c4b550",
    accent2: "#a0aa95",
    ground: "#4c5844",
    text: "#dedfd6",
    play: "#4c5844",
    displayFont: "classic",
    cut: 0,
    isGrainy: false,
    vignette: 0.45,
    // 1.6 sat its menus on a still of a map, washed toward olive.
    backdrop: { kind: "map", mapId: "Duality", isAscii: false, blur: 0, dim: 0.62, motion: "none", hue: 25, saturation: 0.45 },
  },
  {
    id: "terminal",
    label: "Terminal",
    caption: "Phosphor console",
    accent: "#3dff7a",
    accent2: "#1c8f45",
    ground: "#020603",
    text: "#b8ffc9",
    play: "#3dff7a",
    displayFont: "mono",
    cut: 0,
    isGrainy: true,
    vignette: 0.85,
    backdrop: { isAscii: true, asciiSize: 10, dim: 0.55 },
  },
  {
    id: "fortnite",
    label: "Fortnite",
    caption: "Battle Royale lobby",
    accent: "#fbe925",
    accent2: "#4fb3ff",
    ground: "#10237a",
    text: "#ffffff",
    play: "#fbe925",
    displayFont: "tall",
    cut: 0,
    isGrainy: false,
    vignette: 0.45,
    backdrop: { kind: "shader", shader: "burst", isAscii: false, dim: 0, blur: 0 },
  },
  {
    id: "minecraft",
    label: "Minecraft",
    caption: "Java title screen",
    accent: "#7b86bd",
    accent2: "#ffff55",
    ground: "#2b2016",
    text: "#ffffff",
    play: "#6f6f6f",
    displayFont: "pixel",
    cut: 0,
    isGrainy: false,
    vignette: 0.5,
    backdrop: { kind: "map", isAscii: false, blur: 5, dim: 0.15, motion: "pan", motionSpeed: 0.6 },
  },
  {
    id: "halo",
    label: "Halo 3",
    caption: "2007 lobby",
    accent: "#6fd4ff",
    accent2: "#c7ecff",
    ground: "#04101c",
    text: "#d9f2ff",
    play: "#6fd4ff",
    displayFont: "saira",
    cut: 0,
    isGrainy: true,
    vignette: 0.7,
    backdrop: { kind: "shader", shader: "flow", isAscii: false, dim: 0.1, blur: 0 },
  },
  {
    id: "pipboy",
    label: "Pip-Boy",
    caption: "Wrist computer, 2287",
    accent: "#1aff80",
    accent2: "#0b8a45",
    ground: "#031208",
    text: "#1aff80",
    play: "#1aff80",
    displayFont: "robco",
    cut: 0,
    isGrainy: false,
    vignette: 0.8,
    backdrop: { kind: "shader", shader: "crt", isAscii: false, dim: 0, blur: 0, motion: "none" },
  },
  {
    id: "league",
    label: "League client",
    caption: "Windowed · hextech gold",
    accent: "#c8aa6e",
    accent2: "#0ac8b9",
    ground: "#010a13",
    text: "#f0e6d2",
    play: "#0ac8b9",
    displayFont: "beaufort",
    cut: 0,
    isGrainy: false,
    vignette: 0.35,
    backdrop: { kind: "color", isAscii: false, dim: 0, blur: 0, motion: "none" },
  },
];

/** Styles drawn as a desktop client in a window rather than over the whole screen. */
export const WINDOWED_SKINS: readonly Skin[] = ["league"];

/** Tab names a style uses instead of the defaults (CS:GO calls the collection Inventory...). */
const TAB_LABELS: Partial<Record<Skin, Partial<Record<StageId, string>>>> = {
  current: { career: "Progress", play: "Modes" },
  beta: { career: "Progression", agents: "Agents", collection: "Collection" },
  csgo: { collection: "Inventory", career: "Stats" },
  cs2: { collection: "Inventory" },
  cs16: { home: "Main menu", play: "Find match", collection: "Inventory", career: "Stats" },
  fortnite: { home: "Lobby", collection: "Locker", store: "Item Shop" },
  minecraft: { home: "Title", collection: "Inventory", career: "Statistics", store: "Trading" },
  halo: { home: "Lobby", play: "Matchmaking", collection: "Armory", career: "Service Record" },
  pipboy: { home: "Stat", play: "Map", agents: "Special", collection: "Inv", career: "Data", store: "Barter" },
  league: { career: "Profile" },
};

/** Which style a piece kind belongs to (its prefix), or null for the menu's own pieces. */
const FAMILY_PREFIXES: readonly [string, Skin][] = [
  ["cs2-", "cs2"],
  ["csgo-", "csgo"],
  ["cs16-", "cs16"],
  ["beta-", "beta"],
  ["terminal-", "terminal"],
  ["fn-", "fortnite"],
  ["mc-", "minecraft"],
  ["halo-", "halo"],
  ["pip-", "pipboy"],
  ["lol-", "league"],
  ["val-", "current"],
  ["glass-", "glass"],
];

export function pieceFamily(kind: string): Skin | null {
  return FAMILY_PREFIXES.find(([prefix]) => kind.startsWith(prefix))?.[1] ?? null;
}

export function skinLabel(id: Skin): string {
  return SKINS.find((skin) => skin.id === id)?.label ?? id;
}

export function tabLabel(skin: Skin, stage: StageId, fallback: string): string {
  return TAB_LABELS[skin]?.[stage] ?? fallback;
}

/** Stages a style may lay out; any it doesn't keep whatever the player made. */
export const STYLED_STAGES: readonly StageId[] = ["home", "play", "agents", "collection", "career", "store"];

/** The current client's scene is the screen minus its 60 px social strip (--nav-right). */
const VAL_W = 1860 / 1920;

/** A rect measured on the whole 1920-wide client screen, as a fraction of that scene. */
function onClient(x: number, y: number, w: number, h: number) {
  return { x: x / VAL_W, y, w: w / VAL_W, h };
}

type Layout = Partial<Record<StageId, () => Piece[]>>;

/** Where a style's store front sits: nearly the whole screen; the piece picks the look. */
const FRONT_RECT = { x: 0.03, y: 0.04, w: 0.94, h: 0.92 };

const LAYOUTS: Record<Skin, Layout> = {
  // Modern uses the default stages.
  modern: {},
  glass: {
    home: () => [
      piece("glass-hello", { x: 0.03, y: 0.03, w: 0.5, h: 0.12 }, 1),
      piece("glass-rank", { x: 0.535, y: 0.04, w: 0.215, h: 0.1 }, 6),
      piece("glass-party", { x: 0.03, y: 0.18, w: 0.72, h: 0.58 }, 2),
      piece("glass-play", { x: 0.17, y: 0.79, w: 0.44, h: 0.18 }, 3),
      piece("glass-friends", { x: 0.775, y: 0.03, w: 0.205, h: 0.94 }, 4),
    ],
    play: () => [
      piece("queue-picker", { x: 0.03, y: 0.03, w: 0.72, h: 0.72 }, 1),
      piece("glass-play", { x: 0.17, y: 0.79, w: 0.44, h: 0.18 }, 2),
      piece("glass-friends", { x: 0.775, y: 0.03, w: 0.205, h: 0.94 }, 3),
    ],
    career: () => [
      piece("glass-hello", { x: 0.03, y: 0.03, w: 0.6, h: 0.12 }, 1),
      piece("service-record", { x: 0.03, y: 0.18, w: 0.94, h: 0.78 }, 2),
    ],
    store: () => [piece("store-front", FRONT_RECT, 1, { layout: "auto" })],
  },
  // Measured off the 1920x1080 client: 48 px top bar, 60 px social strip on the right.
  current: {
    home: () => [
      piece("val-banner", onClient(0, 0, 0.2405, 0.4118), 1),
      piece("val-corner", onClient(0.78125, 0.6783, 0.1875, 0.3217), 1),
      piece("val-dailies", onClient(0.4, 0.0445, 0.17, 0.06), 2),
      piece("val-mode", onClient(0.0208, 0.2907, 0.125, 0.42), 3),
      piece("val-lineup", onClient(0.223, 0.2043, 0.522, 0.581), 4),
      // Only drawn in a custom lobby, where the lineup steps aside: 353, 307, 1150 x 496 px on
      // screen, so y counts from under the 48 px bar (1032 px tall).
      piece("val-custom", onClient(0.18385, 0.25097, 0.59896, 0.48062), 4),
      piece("val-find", onClient(0.427, 0.8858, 0.114, 0.0484), 5),
      piece("val-news", onClient(0.823, 0.1307, 0.124, 0.688), 6),
      piece("val-chat", onClient(0, 0.961, 0.232, 0.039), 7),
    ],
    play: () => [
      piece("val-modes", onClient(0.06, 0.06, 0.86, 0.72), 1),
      piece("val-find", onClient(0.427, 0.8858, 0.114, 0.0484), 2),
      piece("val-chat", onClient(0, 0.961, 0.232, 0.039), 3),
    ],
    agents: () => [piece("val-agents", { x: 0, y: 0, w: 1, h: 1 }, 1)],
    collection: () => [
      piece("val-collection", { x: 0, y: 0, w: 1, h: 1 }, 1),
      piece("val-chat", onClient(0, 0.965, 0.232, 0.035), 2),
    ],
    career: () => [
      piece("val-career", { x: 0, y: 0, w: 1, h: 1 }, 1),
      piece("val-chat", onClient(0, 0.961, 0.232, 0.039), 2),
    ],
    store: () => [
      piece("val-store", { x: 0, y: 0, w: 1, h: 1 }, 1),
      piece("val-chat", onClient(0, 0.961, 0.232, 0.039), 2),
    ],
  },
  beta: {
    home: () => [
      piece("beta-pills", { x: 0.36, y: 0.12, w: 0.24, h: 0.05 }, 1),
      piece("beta-lineup", { x: 0.11, y: 0.22, w: 0.74, h: 0.64 }, 2),
      piece("beta-social", { x: 0.885, y: 0, w: 0.115, h: 1 }, 3),
    ],
    play: () => [
      piece("queue-picker", { x: 0.04, y: 0.2, w: 0.5, h: 0.74 }, 1),
      piece("party-tools", { x: 0.57, y: 0.2, w: 0.28, h: 0.74 }, 2),
      piece("beta-social", { x: 0.885, y: 0, w: 0.115, h: 1 }, 3),
    ],
    career: () => [
      piece("service-record", { x: 0.04, y: 0.15, w: 0.81, h: 0.8 }, 1),
      piece("beta-social", { x: 0.885, y: 0, w: 0.115, h: 1 }, 3),
    ],
    store: () => [piece("store-front", FRONT_RECT, 1, { layout: "auto" })],
  },
  valnow: {
    home: () => [
      piece("party-lineup", { x: 0.04, y: 0.04, w: 0.92, h: 0.76 }, 1),
    ],
    play: () => [
      piece("queue-picker", { x: 0.03, y: 0.04, w: 0.62, h: 0.9 }, 1),
      piece("party-tools", { x: 0.68, y: 0.04, w: 0.29, h: 0.9 }, 2),
    ],
    career: () => [
      piece("service-record", { x: 0.03, y: 0.04, w: 0.94, h: 0.9 }, 1),
    ],
    store: () => [piece("store-front", FRONT_RECT, 1, { layout: "auto" })],
  },
  csgo: {
    home: () => [
      piece("csgo-profile", { x: 0.146, y: 0.1, w: 0.203, h: 0.17 }, 1),
      piece("csgo-store", { x: 0.364, y: 0.1, w: 0.487, h: 0.17 }, 2),
      piece("csgo-social", { x: 0.146, y: 0.3, w: 0.203, h: 0.67 }, 3),
      piece("csgo-news", { x: 0.364, y: 0.3, w: 0.487, h: 0.67 }, 4),
    ],
    play: () => [
      piece("csgo-play", { x: 0.146, y: 0.1, w: 0.42, h: 0.82 }, 1, { showModes: true }),
      piece("party-tools", { x: 0.58, y: 0.1, w: 0.27, h: 0.56 }, 2),
      piece("csgo-skill", { x: 0.58, y: 0.69, w: 0.27, h: 0.23 }, 3),
    ],
    career: () => [
      piece("csgo-profile", { x: 0.146, y: 0.1, w: 0.3, h: 0.2 }, 1),
      piece("csgo-skill", { x: 0.146, y: 0.33, w: 0.3, h: 0.24 }, 2),
      piece("rr-history", { x: 0.47, y: 0.1, w: 0.38, h: 0.82 }, 3),
    ],
    store: () => [piece("store-front", FRONT_RECT, 1, { layout: "auto" })],
  },
  cs2: {
    home: () => [
      piece("cs2-rating", { x: 0.015, y: 0.03, w: 0.22, h: 0.24 }, 1),
      piece("store-bundle", { x: 0.015, y: 0.3, w: 0.22, h: 0.42 }, 2),
      piece("party-lineup", { x: 0.26, y: 0.04, w: 0.66, h: 0.68 }, 3),
      piece("cs2-friends", { x: 0.945, y: 0.03, w: 0.045, h: 0.7 }, 4),
      piece("cs2-play", { x: 0.48, y: 0.8, w: 0.5, h: 0.15 }, 5, { showMaps: false }),
    ],
    play: () => [
      piece("cs2-play", { x: 0.03, y: 0.04, w: 0.64, h: 0.92 }, 1, { showMaps: true }),
      piece("party-tools", { x: 0.7, y: 0.04, w: 0.22, h: 0.62 }, 2),
      piece("cs2-rating", { x: 0.7, y: 0.7, w: 0.22, h: 0.26 }, 3),
      piece("cs2-friends", { x: 0.945, y: 0.04, w: 0.045, h: 0.7 }, 4),
    ],
    career: () => [
      piece("cs2-rating", { x: 0.04, y: 0.04, w: 0.36, h: 0.3 }, 1),
      piece("rank-card", { x: 0.04, y: 0.38, w: 0.36, h: 0.34 }, 2),
      piece("rr-history", { x: 0.44, y: 0.04, w: 0.52, h: 0.9 }, 3),
    ],
    store: () => [piece("store-front", FRONT_RECT, 1, { layout: "auto" })],
  },
  cs16: {
    home: () => [
      piece("cs16-browser", { x: 0.42, y: 0.06, w: 0.52, h: 0.42 }, 1),
      piece("cs16-play", { x: 0.6, y: 0.52, w: 0.34, h: 0.42 }, 2),
    ],
    play: () => [
      piece("cs16-play", { x: 0.06, y: 0.12, w: 0.36, h: 0.6 }, 1),
      piece("cs16-browser", { x: 0.46, y: 0.12, w: 0.48, h: 0.6 }, 2),
    ],
    career: () => [
      piece("service-record", { x: 0.08, y: 0.08, w: 0.82, h: 0.82 }, 1),
    ],
    store: () => [piece("store-front", FRONT_RECT, 1, { layout: "auto" })],
  },
  terminal: {
    home: () => [
      piece("terminal-console", { x: 0.02, y: 0.03, w: 0.4, h: 0.93 }, 1),
      piece("party-lineup", { x: 0.44, y: 0.03, w: 0.54, h: 0.64 }, 2),
    ],
    play: () => [
      piece("terminal-console", { x: 0.02, y: 0.03, w: 0.5, h: 0.93 }, 1),
      piece("queue-picker", { x: 0.54, y: 0.03, w: 0.44, h: 0.93 }, 2),
    ],
    career: () => [
      piece("terminal-console", { x: 0.02, y: 0.03, w: 0.34, h: 0.93 }, 1),
      piece("service-record", { x: 0.38, y: 0.03, w: 0.6, h: 0.93 }, 2),
    ],
    store: () => [piece("store-front", FRONT_RECT, 1, { layout: "auto" })],
  },
  fortnite: {
    home: () => [
      piece("fn-party", { x: 0.12, y: 0.05, w: 0.76, h: 0.62 }, 1),
      piece("fn-play", { x: 0.66, y: 0.7, w: 0.32, h: 0.27 }, 2),
    ],
    play: () => [
      piece("queue-picker", { x: 0.03, y: 0.04, w: 0.6, h: 0.9 }, 1),
      piece("party-tools", { x: 0.66, y: 0.04, w: 0.32, h: 0.58 }, 2),
      piece("fn-play", { x: 0.66, y: 0.67, w: 0.32, h: 0.29 }, 3),
    ],
    career: () => [
      piece("service-record", { x: 0.03, y: 0.04, w: 0.94, h: 0.92 }, 1),
    ],
    store: () => [piece("fn-shop", { x: 0.03, y: 0.04, w: 0.94, h: 0.92 }, 1, { showBundle: true })],
  },
  minecraft: {
    home: () => [
      piece("mc-menu", { x: 0.3, y: 0.02, w: 0.4, h: 0.94 }, 1),
      piece("mc-party", { x: 0.015, y: 0.64, w: 0.27, h: 0.3 }, 2),
    ],
    play: () => [piece("mc-worlds", { x: 0.22, y: 0.03, w: 0.56, h: 0.93 }, 1)],
    career: () => [piece("mc-stats", { x: 0.22, y: 0.03, w: 0.56, h: 0.93 }, 1)],
    store: () => [piece("mc-trades", { x: 0.2, y: 0.08, w: 0.6, h: 0.8 }, 1)],
  },
  halo: {
    home: () => [
      piece("halo-lobby", { x: 0.03, y: 0.06, w: 0.36, h: 0.56 }, 1),
      piece("halo-roster", { x: 0.6, y: 0.06, w: 0.37, h: 0.6 }, 2),
      piece("halo-preview", { x: 0.03, y: 0.66, w: 0.36, h: 0.29 }, 4),
    ],
    play: () => [
      piece("halo-lobby", { x: 0.03, y: 0.06, w: 0.36, h: 0.56 }, 1),
      piece("queue-picker", { x: 0.42, y: 0.04, w: 0.55, h: 0.9 }, 2),
    ],
    career: () => [
      piece("service-record", { x: 0.03, y: 0.06, w: 0.94, h: 0.88 }, 1),
    ],
    store: () => [piece("store-front", FRONT_RECT, 1, { layout: "auto" })],
  },
  // The STAT screen: figure and limb bars in the middle, actions over the status bar.
  pipboy: {
    home: () => [
      piece("pip-stat", { x: 0.2, y: 0.02, w: 0.6, h: 0.76 }, 1),
      piece("pip-hints", { x: 0.03, y: 0.8, w: 0.94, h: 0.07 }, 2),
      piece("pip-bar", { x: 0.03, y: 0.895, w: 0.94, h: 0.075 }, 3),
    ],
    play: () => [
      piece("queue-picker", { x: 0.03, y: 0.03, w: 0.94, h: 0.74 }, 1),
      piece("pip-hints", { x: 0.03, y: 0.8, w: 0.94, h: 0.07 }, 2),
      piece("pip-bar", { x: 0.03, y: 0.895, w: 0.94, h: 0.075 }, 3),
    ],
    career: () => [
      piece("service-record", { x: 0.03, y: 0.03, w: 0.94, h: 0.84 }, 1),
      piece("pip-bar", { x: 0.03, y: 0.895, w: 0.94, h: 0.075 }, 2),
    ],
    store: () => [piece("store-front", { x: 0.03, y: 0.03, w: 0.94, h: 0.84 }, 1, { layout: "auto" }), piece("pip-bar", { x: 0.03, y: 0.895, w: 0.94, h: 0.075 }, 2)],
  },
  // The client's home hub and lobby fill the space between the bar and the social column.
  league: {
    home: () => [
      piece("lol-home", { x: 0.02, y: 0.03, w: 0.96, h: 0.94 }, 1),
    ],
    play: () => [piece("lol-lobby", { x: 0, y: 0, w: 1, h: 1 }, 1)],
    career: () => [piece("service-record", { x: 0.02, y: 0.03, w: 0.96, h: 0.94 }, 1)],
    store: () => [piece("store-front", { x: 0.02, y: 0.03, w: 0.96, h: 0.94 }, 1, { layout: "auto" })],
  },
};

/** Bumped when a style's built-in layouts change, so saved documents pick the new ones up. */
const LAYOUT_REVISIONS: Partial<Record<Skin, number>> = { glass: 1, current: 4, csgo: 1, cs16: 2, minecraft: 2, halo: 3 };

function applyLayouts(doc: StudioDoc, skin: Skin, previousLayout: Layout): void {
  const layout = LAYOUTS[skin];
  for (const stage of STYLED_STAGES) {
    const build = layout[stage];
    if (build) doc.stages[stage] = build();
    // Only clear a screen the previous style laid out; one the player built stays.
    else if (previousLayout[stage]) delete doc.stages[stage];
  }
  const revision = LAYOUT_REVISIONS[skin];
  if (revision !== undefined) doc.layouts = { ...doc.layouts, [skin]: revision };
}

/** Re-lay a loaded document's screens when its style's layouts were reworked since they were
    applied. Returns whether anything changed (the caller saves). */
export function refreshLayouts(doc: StudioDoc): boolean {
  const skin = doc.theme.skin;
  const revision = LAYOUT_REVISIONS[skin];
  if (revision === undefined || (doc.layouts?.[skin] ?? 0) >= revision) return false;
  applyLayouts(doc, skin, LAYOUTS[skin] ?? {});
  return true;
}

/** Apply a style's whole bundle and its layouts to a document draft. */
export function applySkin(doc: StudioDoc, skin: SkinBundle): void {
  const draft = doc.theme;
  // Undo the backdrop the previous style forced (CS:GO's haze, the terminal's ASCII).
  const previous = SKINS.find((option) => option.id === draft.skin)?.backdrop;
  const previousLayout: Layout = LAYOUTS[draft.skin] ?? {};
  if (previous) {
    for (const key of Object.keys(previous) as (keyof Backdrop)[]) {
      Object.assign(draft.backdrop, { [key]: DEFAULT_THEME.backdrop[key] });
    }
  }
  draft.skin = skin.id;
  draft.preset = skin.id === "modern" ? "valorant" : "custom";
  draft.accent = skin.accent;
  draft.accent2 = skin.accent2;
  draft.ground = skin.ground;
  draft.text = skin.text;
  draft.displayFont = skin.displayFont;
  draft.cut = skin.cut;
  draft.isGrainy = skin.isGrainy;
  draft.vignette = skin.vignette;
  if (skin.backdrop) Object.assign(draft.backdrop, skin.backdrop);
  applyLayouts(doc, skin.id, previousLayout);
}
