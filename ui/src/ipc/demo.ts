// Demo backend for a plain browser: answers every command with believable local data and
// plays a short match loop (queue -> agent select -> match -> menus) so the whole menu,
// including the Studio, can be explored and filmed without VALORANT.
// Loaded only outside Tauri (dynamic import in `call`).
import type { AgentCard } from "~/bindings/AgentCard";
import type { AppEvent } from "~/bindings/AppEvent";
import type { CareerView } from "~/bindings/CareerView";
import type { Catalog } from "~/bindings/Catalog";
import type { CollectionView } from "~/bindings/CollectionView";
import type { ConnectionStatus } from "~/bindings/ConnectionStatus";
import type { FriendView } from "~/bindings/FriendView";
import type { HistoryView } from "~/bindings/HistoryView";
import type { NewsView } from "~/bindings/NewsView";
import type { MatchRow } from "~/bindings/MatchRow";
import type { GameSnapshot } from "~/bindings/GameSnapshot";
import type { LobbyPlayerView } from "~/bindings/LobbyPlayerView";
import type { LobbyView } from "~/bindings/LobbyView";
import type { MapCard } from "~/bindings/MapCard";
import type { MemberView } from "~/bindings/MemberView";
import type { PartyView } from "~/bindings/PartyView";
import type { PostMatchPlayer } from "~/bindings/PostMatchPlayer";
import type { PostMatchView } from "~/bindings/PostMatchView";
import type { PregamePlayerView } from "~/bindings/PregamePlayerView";
import type { PregameView } from "~/bindings/PregameView";
import type { QueueOption } from "~/bindings/QueueOption";
import type { RankBadge } from "~/bindings/RankBadge";
import type { Settings } from "~/bindings/Settings";
import type { SkinOption } from "~/bindings/SkinOption";
import type { StoreItem } from "~/bindings/StoreItem";
import type { StoreView } from "~/bindings/StoreView";
import type { WalletView } from "~/bindings/WalletView";
import type { InstantCloneState } from "~/bindings/InstantCloneState";
import type { ArgsOf, Command, ResultOf } from "./commands";
import { DEMO_WEAPON_ART, type DemoSkin } from "./demoArt";
import { dispatchAppEvent } from "./events";

const MEDIA = "https://media.valorant-api.com";
const STORAGE_PREFIX = "valoshell.demo.";
const TIERS = `${MEDIA}/competitivetiers/03621f52-342b-cf4e-4f86-9350a49c6d04`;

/** A player card's three crops, as the party and friends views carry them. */
function playerCard(uuid: string): Pick<MemberView, "card_tall" | "card_wide" | "card_small"> {
  return {
    card_tall: `${MEDIA}/playercards/${uuid}/largeart.png`,
    card_wide: `${MEDIA}/playercards/${uuid}/wideart.png`,
    card_small: `${MEDIA}/playercards/${uuid}/smallart.png`,
  };
}

const CARDS = {
  champions: "7ae0ca6c-44fc-9fe2-9361-9abaecee53bb",
  reyna: "e9dcc215-4b83-90e4-ba3b-4f8d32568eb6",
  oni: "aeca98db-4737-5e0f-3476-7eb8a16eb696",
  clove: "faa3c3b5-4b0b-1f20-b383-01b7b83126ff",
  neon: "cec2424b-483e-da2e-4369-f0a8822f96b5",
  glitchpop: "df3adf8c-4fa3-5f57-544d-eabf8b68713d",
  spectrum: "006e03e5-4fdb-c400-4c5b-5f815980f403",
  jett: "ab5a453a-f726-4e04-b277-f30d56a8b152",
  prime: "d93ad22d-4db7-b6bc-5e9c-e5959bb9dd76",
  rgx: "cada2e3f-4b1d-8279-3e1a-49984a71d4d3",
  hush: "d1794848-4323-429a-269a-83b81a87a4c6",
};

/** Every demo skin by name, with the weapon it belongs to. */
const SKINS_BY_NAME = new Map<string, DemoSkin>(Object.values(DEMO_WEAPON_ART).flatMap(([, skins]) => skins.map((skin) => [skin[0], skin] as const)));

function skinArt(name: string): string | null {
  const skin = SKINS_BY_NAME.get(name);
  return skin ? `${MEDIA}/${skin[1]}` : null;
}

const AGENTS: ReadonlyArray<[string, string, string]> = [
  ["ASTRA", "41fb69c1-4189-7b37-f117-bcaf1e96f1bf", "Controller"],
  ["BREACH", "5f8d3a7f-467b-97f3-062c-13acf203c006", "Initiator"],
  ["BRIMSTONE", "9f0d8ba9-4140-b941-57d3-a7ad57c6b417", "Controller"],
  ["CHAMBER", "22697a3d-45bf-8dd7-4fec-84a9e28c69d7", "Sentinel"],
  ["CLOVE", "1dbf2edd-4729-0984-3115-daa5eed44993", "Controller"],
  ["CYPHER", "117ed9e3-49f3-6512-3ccf-0cada7e3823b", "Sentinel"],
  ["DEADLOCK", "cc8b64c8-4b25-4ff9-6e7f-37b4da43d235", "Sentinel"],
  ["FADE", "dade69b4-4f5a-8528-247b-219e5a1facd6", "Initiator"],
  ["GEKKO", "e370fa57-4757-3604-3648-499e1f642d3f", "Initiator"],
  ["HARBOR", "95b78ed7-4637-86d9-7e41-71ba8c293152", "Controller"],
  ["ISO", "0e38b510-41a8-5780-5e8f-568b2a4f2d6c", "Duelist"],
  ["JETT", "add6443a-41bd-e414-f6ad-e58d267f4e95", "Duelist"],
  ["KAY/O", "601dbbe7-43ce-be57-2a40-4abd24953621", "Initiator"],
  ["KILLJOY", "1e58de9c-4950-5125-93e9-a0aee9f98746", "Sentinel"],
  ["NEON", "bb2a4828-46eb-8cd1-e765-15848195d751", "Duelist"],
  ["OMEN", "8e253930-4c05-31dd-1b6c-968525494517", "Controller"],
  ["PHOENIX", "eb93336a-449b-9c1b-0a54-a891f7921d69", "Duelist"],
  ["RAZE", "f94c3b30-42be-e959-889c-5aa313dba261", "Duelist"],
  ["REYNA", "a3bfb853-43b2-7238-a4f1-ad90e9e46bcc", "Duelist"],
  ["SAGE", "569fdd95-4d10-43ab-ca70-79becc718b46", "Sentinel"],
  ["SKYE", "6f2a04ca-43e0-be17-7f36-b3908627744d", "Initiator"],
  ["SOVA", "320b2a48-4d9b-a075-30f1-1f93a9b638fa", "Initiator"],
  ["TEJO", "b444168c-4e35-8076-db47-ef9bf368f384", "Initiator"],
  ["VIPER", "707eab51-4836-f488-046a-cda6bf494859", "Controller"],
  ["VYSE", "efba5359-4016-a1e5-7626-b1ae76895940", "Sentinel"],
  ["WAYLAY", "df1cb487-4902-002e-5c17-d28e83e78588", "Duelist"],
  ["YORU", "7f94d92c-4234-0a36-9646-3a87eb8b5c89", "Duelist"],
];

const MAPS: ReadonlyArray<[string, string, string]> = [
  ["ABYSS", "Infinity", "224b0a95-48b9-f703-1bd8-67aca101a61f"],
  ["ASCENT", "Ascent", "7eaecc1b-4337-bbf6-6ab9-04b8f06b3319"],
  ["BIND", "Duality", "2c9d57ec-4431-9c5e-2939-8f9ef6dd5cba"],
  ["BREEZE", "Foxtrot", "2fb9a4fd-47b8-4e7d-a969-74b4046ebd53"],
  ["CORRODE", "Rook", "1c18ab1f-420d-0d8b-71d0-77ad3c439115"],
  ["FRACTURE", "Canyon", "b529448b-4d60-346e-e89e-00a4c527a405"],
  ["HAVEN", "Triad", "2bee0dc9-4ffe-519b-1cbd-7fbe763a6047"],
  ["ICEBOX", "Port", "e2ad5c54-4114-a870-9641-8ea21279579a"],
  ["LOTUS", "Jam", "2fe4ed3a-450a-948b-6d6b-e89a78e680a9"],
  ["PEARL", "Pitt", "fd267378-4d1d-484f-ff52-77821ed10dc2"],
  ["SPLIT", "Bonsai", "d960549e-485c-e861-8d71-aa9d1aed12a2"],
  ["SUNSET", "Juliett", "92584fbe-486a-b1b2-9faa-39b0f486b498"],
];

/** A demo queue with the asset tables' card art and glyph (valorant-api queue uuids). */
function demoQueue(id: string, label: string, uuid: string, hasIcon: boolean): QueueOption {
  const base = `${MEDIA}/gamemodequeues/${uuid}`;
  return { id, label, art: `${base}/listviewicontall.png`, icon: hasIcon ? `${base}/displayicon.png` : null };
}

const QUEUES: QueueOption[] = [
  demoQueue("unrated", "Unrated", "494b69f1-4e3a-1b03-c2cd-a4875d6e9cb6", false),
  demoQueue("competitive", "Competitive", "d2faff85-4964-f52e-b6b5-73a5d66ccad6", true),
  demoQueue("swiftplay", "Swiftplay", "2d257217-464c-7c4b-efc6-51a55365d44a", true),
  demoQueue("spikerush", "Spike Rush", "3938d5da-43bf-1e8c-f09f-858caf145975", true),
  demoQueue("deathmatch", "Deathmatch", "f3126c5e-4a6c-1f02-b216-cb9bf58df856", false),
  demoQueue("hurm", "Team Deathmatch", "1400f385-4905-96ff-8e1e-f8a4acb8c64d", true),
  demoQueue("custom", "Custom", "63d60a3e-4838-695d-9077-e9af5ed523ca", false),
];

/** The custom lobby the demo opens: you defending on Ascent, Nova attacking, Kite watching. */
function demoCustomLobby(): NonNullable<PartyView["custom"]> {
  return {
    map_name: "Ascent",
    map_splash: mapArt("Ascent", "splash"),
    mode_name: "Standard",
    server_name: "Madrid",
    attackers: ["demo-nova"],
    defenders: ["demo-self"],
    spectators: ["demo-kite"],
    is_autobalance_on: true,
    autobalance_min_players: 6,
  };
}

function title(name: string): string {
  return name.charAt(0) + name.slice(1).toLowerCase().replace(/\/(.)/, (_, letter: string) => `/${letter.toUpperCase()}`);
}

/** Page tints by role for the demo; the real ones come per agent from the asset tables. */
const DEMO_TINTS: Record<string, string[]> = {
  Duelist: ["#5e2a3c", "#3a1a2a", "#8a3a52", "#21101a"],
  Controller: ["#4b2a7a", "#2c1a52", "#6a3fa8", "#170f2e"],
  Initiator: ["#2a5a4a", "#183a30", "#3f7a62", "#0c1f1a"],
  Sentinel: ["#2a4a6a", "#1a2e44", "#3f6a90", "#0f1a28"],
};

function agentCard([name, uuid, role]: readonly [string, string, string]): AgentCard {
  return {
    uuid,
    display_name: title(name),
    upper_name: name,
    role,
    icon: `${MEDIA}/agents/${uuid}/displayicon.png`,
    portrait: `${MEDIA}/agents/${uuid}/fullportrait.png`,
    background: `${MEDIA}/agents/${uuid}/background.png`,
    description: `${title(name)} brings their own toolkit to every round. The full bio shows when ValoShell has the game's asset tables.`,
    colors: DEMO_TINTS[role] ?? ["#2a3c52", "#1b2a3a", "#3d5570", "#0f1923"],
    role_icon: null,
    abilities: [],
  };
}

function mapCard([name, internalId, uuid]: readonly [string, string, string]): MapCard {
  return {
    internal_id: internalId,
    display_name: title(name),
    upper_name: name,
    splash: `${MEDIA}/maps/${uuid}/splash.png`,
    list_icon: `${MEDIA}/maps/${uuid}/listviewicon.png`,
    is_team_deathmatch: false,
  };
}

let catalog: Catalog = {
  agents: AGENTS.map(agentCard),
  maps: MAPS.map(mapCard),
};

interface ApiAgent {
  uuid: string;
  description?: string;
  backgroundGradientColors?: string[];
  role?: { displayIcon?: string } | null;
  abilities?: { slot: string; displayName: string; description?: string; displayIcon?: string | null }[];
}

/** The real bios, colours and abilities from valorant-api.com, the same source the app's asset
 *  cache reads; offline, the demo keeps its placeholders. */
const fullCatalog: Promise<Catalog> = fetch("https://valorant-api.com/v1/agents?isPlayableCharacter=true")
  .then((response) => (response.ok ? (response.json() as Promise<{ data: ApiAgent[] }>) : null))
  .then((json) => {
    const byUuid = new Map((json?.data ?? []).map((agent) => [agent.uuid, agent]));
    catalog = {
      ...catalog,
      agents: catalog.agents.map((agent) => {
        const api = byUuid.get(agent.uuid);
        if (!api) return agent;
        return {
          ...agent,
          description: api.description ?? agent.description,
          colors: (api.backgroundGradientColors ?? []).map((color) => `#${color.slice(0, 6).toLowerCase()}`),
          role_icon: api.role?.displayIcon ?? null,
          abilities: (api.abilities ?? [])
            .filter((ability) => ability.slot !== "Passive")
            .map((ability) => ({ slot: ability.slot, name: ability.displayName, description: ability.description ?? "", icon: ability.displayIcon ?? null })),
        };
      }),
    };
    return catalog;
  })
  .catch(() => catalog);

function rank(tier: number, name: string, color: string): RankBadge {
  return { tier, name, icon: `${TIERS}/${tier}/largeicon.png`, color };
}

function mapArt(name: string, crop: "splash" | "listviewicon"): string | null {
  const map = MAPS.find(([upperName]) => upperName === name.toUpperCase());
  return map ? `${MEDIA}/maps/${map[2]}/${crop}.png` : null;
}

function member(name: string, tag: string, extra: Partial<MemberView>): MemberView {
  return {
    puuid: `demo-${name.toLowerCase()}`,
    game_name: name,
    tag_line: tag,
    is_self: false,
    is_owner: false,
    is_ready: true,
    account_level: 120,
    card_tall: null,
    card_wide: null,
    card_small: null,
    title: null,
    title_es: null,
    level_border: `${MEDIA}/levelborders/7e7feff1-44c2-301e-767d-d9b2b1cd9051/smallplayercardappearance.png`,
    rank: rank(14, "Gold 2", "e5c46bff"),
    ping_ms: 24,
    ...extra,
  };
}

function defaultSettings(): Settings {
  return {
    region: "eu",
    match_flow: {
      should_hide_on_lock: true,
      should_hide_on_match_start: true,
      should_return_in_menus: true,
      should_minimize_game_in_menus: false,
    },
    lobby: { should_rank_enemies_in_agent_select: false },
    interface: { theme: "valorant-dark", ui_volume: 0.6, motion: "system", backdrop: "default" },
    streamer: { is_enabled: false, should_hide_own_name: true },
  };
}

function load<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(STORAGE_PREFIX + key);
    return raw ? (JSON.parse(raw) as T) : fallback;
  } catch {
    return fallback;
  }
}

function persist(key: string, value: unknown): void {
  try {
    localStorage.setItem(STORAGE_PREFIX + key, JSON.stringify(value));
  } catch {
    // Storage can be unavailable (private mode); the demo just forgets.
  }
}

let settings: Settings = load("settings", defaultSettings());
const connection: ConnectionStatus = { state: "connected", riot_id: "s1moscs#EUW", region: "eu" };
let game: GameSnapshot = {
  phase: "menus",
  map_name: "",
  ally_score: null,
  enemy_score: null,
  party_size: 3,
  queue: "",
  side: null,
};
const wallet: WalletView = { valorant_points: 1844, radianite_points: 770, kingdom_credits: 6179 };
// InstantClone streaming to two destinations, no delay yet: the stream delay widget's preview.
let demoDelay: InstantCloneState = {
  phase: "idle",
  armed_delay_ms: 0,
  current_delay_ms: 0,
  buffer_fill_ms: 0,
  ingest_alive: true,
  destinations_alive: 2,
  destinations_total: 2,
};
let party: PartyView = {
  party_id: "demo-party",
  is_owner: true,
  is_open: false,
  is_queueing: false,
  is_custom_game: false,
  queue: QUEUES[1] ?? { id: "competitive", label: "Competitive", art: null, icon: null },
  queue_entry_time: null,
  eligible_queues: QUEUES,
  invite_code: null,
  members: [
    member("s1moscs", "EUW", { puuid: "demo-self", is_self: true, is_owner: true, rank: rank(18, "Diamond 1", "b87de8ff"), title: "Say It With Your Chest", ...playerCard(CARDS.champions) }),
    member("Nova", "4242", { rank: rank(16, "Platinum 2", "4fbfd4ff"), ping_ms: 31, title: "Braba", ...playerCard(CARDS.reyna) }),
    member("Kite", "777", { rank: rank(19, "Diamond 2", "b87de8ff"), is_ready: false, ping_ms: 18, title: "Cooked", ...playerCard(CARDS.oni) }),
  ],
  max_size: 5,
  restricted_seconds: 0,
  custom: null,
};
let pregame: PregameView | null = null;
const timers: number[] = [];

function emit(event: AppEvent): void {
  dispatchAppEvent(event);
}

function later(ms: number, run: () => void): void {
  timers.push(window.setTimeout(run, ms));
}

function setParty(next: Partial<PartyView>): void {
  party = { ...party, ...next };
  emit({ type: "party", data: party });
}

function setGame(next: Partial<GameSnapshot>): void {
  game = { ...game, ...next };
  emit({ type: "game_state", data: game });
}

// ---- store and career -------------------------------------------------------------------

function storeItem(name: string, price: number, discounted: number | null = null, currency = "vp", edition: string | null = null, image: string | null = skinArt(name)): StoreItem {
  return {
    id: `demo-${name.toLowerCase().replace(/\W+/g, "-")}`,
    name,
    image,
    price,
    discounted_price: discounted,
    discount_percent: discounted === null ? null : Math.round((1 - discounted / price) * 100),
    currency,
    edition,
  };
}

function demoStore(): StoreView {
  return {
    daily: [
      storeItem("Reaver Vandal", 1775, null, "vp", "Premium"),
      storeItem("Prime Spectre", 1775, null, "vp", "Premium"),
      storeItem("Oni Phantom", 1775, null, "vp", "Premium"),
      storeItem("Glitchpop Odin", 2175, null, "vp", "Exclusive"),
    ],
    daily_ends_in_s: 9 * 3600 + 1234,
    bundles: [
      {
        name: "Neo Frontier",
        image: `${MEDIA}/bundles/4d368017-4f98-1e89-dbec-31abd2533eb9/displayicon.png`,
        price: 8700,
        discounted_price: 5829,
        ends_in_s: 5 * 86400 + 3600,
        items: [storeItem("Neo Frontier Sheriff", 2175, 1458), storeItem("Neo Frontier Phantom", 2175, 1458), storeItem("Neo Frontier Axe", 4350, 2915, "vp", null, `${MEDIA}/weaponskins/cdcfab50-425d-6410-7a54-6aa913b7ce48/displayicon.png`)],
      },
      {
        name: "VCT x LOUD",
        image: `${MEDIA}/bundles/8feb9ed1-441a-73b6-55ed-ff836011701e/displayicon.png`,
        price: 2175,
        discounted_price: null,
        ends_in_s: 9 * 86400,
        items: [storeItem("VCT x LOUD Classic", 2175, null, "vp", null, `${MEDIA}/weaponskins/0aa66a21-41d9-8c50-f2ef-3988b57bfd52/displayicon.png`)],
      },
    ],
    night_market: [storeItem("Elderflame Operator", 2475, 1237), storeItem("Ion Phantom", 1775, 1065), storeItem("Sakura Classic", 875, 525)],
    night_market_ends_in_s: 6 * 86400,
    accessories: [
      storeItem("Hush Card", 4000, null, "kc", null, `${MEDIA}/playercards/${CARDS.hush}/largeart.png`),
      storeItem("Clutch Spray", 2000, null, "kc", null, `${MEDIA}/sprays/890c4f6d-4794-3d88-617b-1b906c7a8ea6/fulltransparenticon.png`),
      storeItem("The Spike Buddy", 3000, null, "kc", null, `${MEDIA}/buddies/0ec28d81-498c-af58-722c-baa60a84151c/displayicon.png`),
    ],
    accessories_end_in_s: 3 * 86400,
    observed_at_ms: Date.now(),
  };
}

function demoCareer(): CareerView {
  const diamond = rank(18, "Diamond 1", "b87de8ff");
  const plat = rank(17, "Platinum 3", "4fbfd4ff");
  const maps = ["Ascent", "Bind", "Lotus", "Haven", "Split", "Sunset", "Icebox", "Pearl"];
  const earned = [21, -14, 18, 23, -9, 17, -19, 24, 15, -12];
  let rr = 41;
  const updates = earned.map((delta, index) => {
    const after = rr;
    rr -= delta;
    return {
      match_id: `demo-match-${index}`,
      map_name: maps[index % maps.length] ?? "Ascent",
      map_image: mapArt(maps[index % maps.length] ?? "Ascent", "listviewicon"),
      started_at_ms: Date.now() - index * 20 * 3600 * 1000,
      rr_earned: delta,
      rr_after: after < 0 ? after + 100 : after,
      rank_after: after < 0 ? plat : diamond,
      is_promotion: index === 3,
      is_demotion: false,
      afk_penalty: 0,
    };
  });
  return { rank: diamond, rr: 41, leaderboard_rank: 0, wins: 34, games: 61, peak: rank(19, "Diamond 2", "b87de8ff"), placements_left: 0, updates };
}

/** playvalorant.com's news as the app reads it (October 2026, Spanish). */
function demoNews(): NewsView {
  const image = (id: string) => `https://cmsassets.rgpub.io/sanity/images/dsfx7636/news_live/${id}.jpg?accountingTag=VAL&w=640`;
  const item = (title: string, category: string, id: string, url: string, published_at: string, summary: string) => ({ title, category, summary, image: image(id), url, published_at });
  return {
    items: [
      item("VOCÊ É BRABA // Game Changers Championship", "esports", "549e358e98eb8249c230b805a493bb11ec65b765-3840x2160", "https://www.youtube.com/watch?v=69sKYwsxUM8", "2026-10-20T15:00:00.000Z", "VCT Game Changers Championship 2026: del 22 de octubre al 1 de noviembre en São Paulo, Brasil."),
      item("¡ÁLZATE Y CAE! // Tráiler de Hush - VALORANT", "game_updates", "511ed3c6f31074a8225f6b9597d686f17eeef8d3-1920x1080", "https://www.youtube.com/watch?v=pyir73S9kFU", "2026-10-18T06:00:00.000Z", "Nueva agente duelista disponible en la Temporada 26 // ACTO VI"),
      item("Clasificatorias del Kickoff de VCT EMEA 2027", "esports", "a12af144cabbe2b642e799a596ed2d69c2712a25-3840x2160", "https://valorantesports.com/es-es/news/2027-vct-emea-kickoff-qualifiers", "2026-10-15T16:00:00.000Z", "Dad el primer paso hacia el VCT EMEA 2027. Inscribíos ya en las clasificatorias abiertas de EMEA"),
      item("LA ÚLTIMA PALABRA // Avance de agente - VALORANT", "game_updates", "561c2fe972b26b563bf2a0c58c329de7d234082a-1920x1080", "https://www.youtube.com/watch?v=kiaLEAFx_FA", "2026-10-15T12:00:00.000Z", "Pronto descubriréis quién tiene la última palabra."),
      item("VCT 2027: la gira mundial", "esports", "3ac79a3f7c8f8e4c452febf366629442e0ba2a87-1920x1080", "https://valorantesports.com/es-es/news/vct-2027-the-global-tour/", "2026-10-15T13:00:00.000Z", "El VALORANT Champions Tour se va de gira en 2027 con más torneos, más formas de competir y más VCT cerca de vosotros."),
    ],
    patch_version: "13.08",
    patch_url: "https://playvalorant.com/es-es/news/game-updates/valorant-patch-notes-13-08",
  };
}

const HISTORY_MAPS = ["Abyss", "Corrode", "Lotus", "Haven", "Bind", "Sunset", "Ascent", "Pearl", "Split"];

function demoHistory(): HistoryView {
  const brimstone = "9f0d8ba9-4140-b941-57d3-a7ad57c6b417";
  const clove = "1dbf2edd-4729-0984-3115-daa5eed44993";
  const lines: [string, string, number, number, number, number, number, number, MatchRow["mvp"], number | null, number | null][] = [
    [brimstone, "swiftplay", 3, 5, 0, 231, 0, 5, null, null, null],
    [brimstone, "swiftplay", 3, 5, 2, 243, 1, 5, null, null, null],
    [brimstone, "swiftplay", 11, 6, 5, 436, 4, 5, "match", null, null],
    [brimstone, "swiftplay", 6, 4, 5, 328, 5, 4, null, null, null],
    [brimstone, "swiftplay", 10, 2, 4, 494, 5, 1, "match", null, null],
    [brimstone, "swiftplay", 10, 6, 9, 424, 4, 5, "match", null, null],
    [brimstone, "competitive", 15, 9, 11, 358, 13, 6, null, 19, null],
    [clove, "deathmatch", 5, 9, 3, 0, 0, 0, null, null, 7],
    [brimstone, "competitive", 8, 10, 13, 253, 13, 6, null, 16, null],
  ];
  const labels: Record<string, string> = { swiftplay: "Swiftplay", competitive: "Competitive", deathmatch: "Deathmatch" };
  return {
    rows: lines.map(([agent, queue, kills, deaths, assists, score, won, lost, mvp, rr, place], index) => ({
      match_id: `demo-history-${index}`,
      queue_id: queue,
      queue_label: labels[queue] ?? queue,
      map_name: HISTORY_MAPS[index % HISTORY_MAPS.length] ?? "Abyss",
      map_splash: mapArt(HISTORY_MAPS[index % HISTORY_MAPS.length] ?? "Abyss", "splash"),
      agent_name: agent === clove ? "Clove" : "Brimstone",
      agent_icon: `${MEDIA}/agents/${agent}/displayicon.png`,
      kills,
      deaths,
      assists,
      score,
      result: queue === "deathmatch" ? "defeat" : won > lost ? "victory" : won < lost ? "defeat" : "draw",
      rounds_won: won,
      rounds_lost: lost,
      placement: place,
      mvp: mvp ?? null,
      rr_change: rr,
      started_at_ms: Date.now() - index * 3 * 3600 * 1000,
    })),
  };
}

const WEAPONS: ReadonlyArray<[string, string]> = [
  ["Classic", "Sidearm"], ["Shorty", "Sidearm"], ["Frenzy", "Sidearm"], ["Ghost", "Sidearm"], ["Sheriff", "Sidearm"],
  ["Stinger", "SMG"], ["Spectre", "SMG"], ["Bucky", "Shotgun"], ["Judge", "Shotgun"],
  ["Bulldog", "Rifle"], ["Guardian", "Rifle"], ["Phantom", "Rifle"], ["Vandal", "Rifle"],
  ["Marshal", "Sniper"], ["Outlaw", "Sniper"], ["Operator", "Sniper"], ["Ares", "Heavy"], ["Odin", "Heavy"], ["Melee", "Melee"],
];
/** Which of a weapon's demo skins it wears (-1: the standard skin), so the loadout looks lived in. */
const EQUIPPED: Record<string, number> = { Classic: 0, Shorty: -1, Frenzy: 2, Ghost: 1, Sheriff: 0, Stinger: -1, Spectre: 1, Bucky: -1, Judge: 1, Bulldog: 2, Guardian: 0, Phantom: 0, Vandal: 0, Marshal: 1, Outlaw: -1, Operator: 3, Ares: -1, Odin: 1, Melee: 0 };

function chromaArt(uuid: string, crop: "swatch" | "fullrender"): string {
  return `${MEDIA}/weaponskinchromas/${uuid}/${crop}.png`;
}

function weaponSkins(name: string): [string, DemoSkin[]] {
  return DEMO_WEAPON_ART[name] ?? ["", []];
}

let collection: CollectionView = {
  weapons: WEAPONS.map(([name, category]) => {
    const [weaponUuid, skins] = weaponSkins(name);
    const skin = skins[EQUIPPED[name] ?? -1];
    if (!skin) {
      const image = weaponUuid ? `${MEDIA}/weapons/${weaponUuid}/displayicon.png` : null;
      return { weapon_id: `w-${name.toLowerCase()}`, weapon_name: name, category, skin_id: `s-${name}`, skin_name: name, chroma_id: `c-${name}-0`, image, edition: null };
    }
    const chroma = skin[4][0]?.[1] ?? "";
    return { weapon_id: `w-${name.toLowerCase()}`, weapon_name: name, category, skin_id: `s-${skin[0]}`, skin_name: skin[0], chroma_id: chroma, image: `${MEDIA}/${skin[1]}`, edition: skin[2] };
  }),
};

function demoSkinOptions(weaponId: string): SkinOption[] {
  const name = WEAPONS.find(([weapon]) => `w-${weapon.toLowerCase()}` === weaponId)?.[0] ?? "Weapon";
  const [weaponUuid, skins] = weaponSkins(name);
  const standard: SkinOption = {
    skin_id: `s-${name}`,
    name,
    image: weaponUuid ? `${MEDIA}/weapons/${weaponUuid}/displayicon.png` : null,
    chromas: [],
    owned_levels: 1,
    total_levels: 1,
    levels: [{ level_id: `l-${name}-0`, name, effect: null, video: null, is_owned: true }],
    edition: null,
  };
  return [
    standard,
    ...skins.map(([skin, icon, edition, levelCount, chromas], index): SkinOption => {
      const owned = Math.min(levelCount, 2 + (index % 3));
      return {
        skin_id: `s-${skin}`,
        name: skin,
        image: `${MEDIA}/${icon}`,
        chromas: chromas.map(([chromaName, uuid, hasSwatch], chroma) => ({
          chroma_id: uuid,
          name: chromaName,
          swatch: hasSwatch ? chromaArt(uuid, "swatch") : null,
          image: chromaArt(uuid, "fullrender"),
          video: null,
          is_owned: chroma < 2,
        })),
        owned_levels: owned,
        total_levels: levelCount,
        levels: Array.from({ length: levelCount }, (_, level) => ({
          level_id: `l-${skin}-${level}`,
          name: level === 0 ? skin : `${skin} Level ${level + 1}`,
          effect: level === 0 ? null : (["VFX", "Animation", "Finisher", "Kill Banner"][level - 1] ?? null),
          video: null,
          is_owned: level < owned,
        })),
        edition,
      };
    }),
  ];
}

// ---- match loop -------------------------------------------------------------------------

function demoAgent(name: string): AgentCard | undefined {
  return catalog.agents.find((agent) => agent.upper_name === name);
}

function pregamePlayer(memberView: MemberView): PregamePlayerView {
  return {
    puuid: memberView.puuid,
    is_self: memberView.is_self,
    game_name: memberView.game_name,
    tag_line: memberView.tag_line,
    agent_uuid: null,
    agent_name: null,
    agent_icon: null,
    selection: "",
    account_level: memberView.account_level,
    rank: memberView.rank,
  };
}

function setPregame(next: PregameView | null): void {
  pregame = next;
  emit({ type: "pregame", data: next });
}

function updateAlly(puuid: string, agentName: string, selection: "selected" | "locked"): void {
  if (!pregame) return;
  const agent = demoAgent(agentName);
  const allies = pregame.allies.map((ally) =>
    ally.puuid === puuid && agent
      ? { ...ally, agent_uuid: agent.uuid, agent_name: agent.display_name, agent_icon: agent.icon, selection }
      : ally,
  );
  const own = allies.find((ally) => ally.is_self);
  setPregame({
    ...pregame,
    allies,
    own_agent_uuid: own?.agent_uuid ?? null,
    is_own_locked: own?.selection === "locked",
    taken_agents: allies.filter((ally) => !ally.is_self && ally.selection === "locked").flatMap((ally) => (ally.agent_uuid ? [ally.agent_uuid] : [])),
  });
}

function startAgentSelect(): void {
  setParty({ is_queueing: false, queue_entry_time: null });
  setGame({ phase: "pregame", map_name: "Ascent", queue: party.queue.id });
  emit({ type: "match_flow", data: { kind: "enter_scene", scene: "agent-select" } });
  setPregame({
    match_id: `demo-${Date.now()}`,
    map_name: "Ascent",
    map_splash: mapArt("Ascent", "splash"),
    queue_label: party.queue.label,
    is_ranked: party.queue.id === "competitive",
    team_id: "Blue",
    time_remaining_s: 85,
    observed_at_ms: Date.now(),
    allies: [
      ...party.members.map(pregamePlayer),
      pregamePlayer(member("Echo", "11", { puuid: "demo-echo", rank: rank(17, "Platinum 3", "4fbfd4ff") })),
      pregamePlayer(member("Byte", "22", { puuid: "demo-byte", rank: rank(15, "Gold 3", "e5c46bff") })),
    ],
    enemy_lock_count: 0,
    enemy_team_size: 5,
    own_agent_uuid: null,
    is_own_locked: false,
    taken_agents: [],
  });
  later(2500, () => updateAlly("demo-nova", "OMEN", "selected"));
  later(4200, () => updateAlly("demo-kite", "SOVA", "locked"));
  later(5200, () => updateAlly("demo-nova", "OMEN", "locked"));
  later(7800, () => updateAlly("demo-echo", "KILLJOY", "locked"));
  later(9600, () => updateAlly("demo-byte", "BREACH", "locked"));
  later(1200, () => setLobby(demoLobby(true)));
}

function friend(name: string, tag: string, status: FriendView["status"], card: string | null, extra: Partial<FriendView> = {}): FriendView {
  const isValorant = status !== "offline" && status !== "online";
  return {
    puuid: `f-${name.toLowerCase()}`,
    game_name: name,
    tag_line: tag,
    status,
    detail: "",
    party_size: 1,
    note: "",
    last_online_ms: null,
    card_small: card ? `${MEDIA}/playercards/${card}/smallart.png` : null,
    account_level: isValorant ? 40 + ((name.length * 53) % 300) : null,
    queue: isValorant ? "Competitive" : "",
    product: status === "offline" ? "" : status === "online" ? "league_of_legends" : "valorant",
    ...extra,
  };
}

const FRIENDS: FriendView[] = [
  friend("Luna", "EUW", "menus", CARDS.clove),
  friend("Ghost", "0001", "menus", CARDS.neon, { detail: "Party of 2", party_size: 2, queue: "Unrated" }),
  friend("BoomBot", "777", "queueing", CARDS.glitchpop, { queue: "Swiftplay" }),
  friend("Skyline", "333", "in_match", CARDS.spectrum, { detail: "Lotus 9-6", party_size: 3 }),
  friend("Mia", "FR", "agent_select", CARDS.jett),
  friend("Rook", "LOL", "online", null, { detail: "league_of_legends" }),
  friend("Tom", "1312", "away", CARDS.prime),
  friend("Ivy", "IVY", "offline", CARDS.rgx, { last_online_ms: Date.now() - 5 * 3600 * 1000 }),
  friend("Max", "99", "offline", CARDS.hush, { last_online_ms: Date.now() - 3 * 86400 * 1000 }),
];

let lobby: LobbyView | null = null;

function lobbyPlayer(name: string, tag: string, isAlly: boolean, agent: string, tier: number, rankName: string, color: string, extra: Partial<LobbyPlayerView> = {}): LobbyPlayerView {
  const card = demoAgent(agent);
  return {
    puuid: `demo-${name.toLowerCase()}`,
    game_name: name,
    tag_line: tag,
    is_self: false,
    is_ally: isAlly,
    is_incognito: false,
    agent_name: card?.display_name ?? null,
    agent_icon: card?.icon ?? null,
    account_level: 40 + ((name.length * 37) % 300),
    is_loaded: true,
    is_rank_hidden: false,
    rank: rank(tier, rankName, color),
    rr: (name.length * 13) % 100,
    peak: rank(Math.min(27, tier + 1), rankName.replace(/\d$/, (digit) => String(Math.min(3, Number(digit) + 1))), color),
    win_rate: 42 + ((name.length * 7) % 20),
    games: 20 + ((name.length * 11) % 60),
    is_placement: false,
    leaderboard_rank: 0,
    stack_size: 1,
    stack_group: null,
    ...extra,
  };
}

function demoLobby(isPregame: boolean): LobbyView {
  const stack = { stack_size: 3, stack_group: 0 };
  const allies = [
    lobbyPlayer("s1moscs", "EUW", true, "JETT", 18, "Diamond 1", "b87de8ff", { is_self: true, ...stack }),
    lobbyPlayer("Kite", "777", true, "SOVA", 19, "Diamond 2", "b87de8ff", stack),
    lobbyPlayer("Nova", "4242", true, "OMEN", 16, "Platinum 2", "4fbfd4ff", stack),
    lobbyPlayer("Echo", "11", true, "KILLJOY", 17, "Platinum 3", "4fbfd4ff"),
    lobbyPlayer("Byte", "22", true, "BREACH", 15, "Gold 3", "e5c46bff"),
  ];
  const duo = { stack_size: 2, stack_group: 1 };
  const enemies = isPregame
    ? []
    : [
        lobbyPlayer("Vortex", "EU1", false, "RAZE", 19, "Diamond 2", "b87de8ff", duo),
        lobbyPlayer("Mirage", "EU1", false, "VIPER", 18, "Diamond 1", "b87de8ff", duo),
        lobbyPlayer("", "", false, "CYPHER", 17, "Platinum 3", "4fbfd4ff", { is_incognito: true }),
        lobbyPlayer("Hex", "404", false, "SKYE", 21, "Ascendant 1", "2fc46bff"),
        lobbyPlayer("Pulse", "99", false, "NEON", 16, "Platinum 2", "4fbfd4ff"),
      ];
  return {
    match_id: pregame?.match_id ?? "demo-match",
    is_pregame: isPregame,
    is_live: true,
    map_name: "Ascent",
    queue_label: party.queue.label,
    allies,
    enemies,
    ally_average: rank(17, "Platinum 3", "4fbfd4ff"),
    enemy_average: isPregame ? null : rank(18, "Diamond 1", "b87de8ff"),
    is_complete: true,
  };
}

function setLobby(next: LobbyView | null): void {
  lobby = next;
  emit({ type: "lobby", data: next });
}

let matchStarted = false;

function afterOwnLock(): void {
  if (matchStarted) return;
  matchStarted = true;
  later(4000, () => {
    setPregame(null);
    setGame({ phase: "ingame", ally_score: 0, enemy_score: 0, side: "attack" });
    setLobby(demoLobby(false));
    emit({ type: "match_flow", data: { kind: "hide_menu" } });
    playRounds(0);
  });
}

let postMatch: PostMatchView | null = null;

/** One scoreboard line, with numbers loosely tied to the seat for believability. */
function postMatchLine(player: LobbyPlayerView, rounds: number, seed: number): PostMatchPlayer {
  const kills = Math.max(2, Math.round(rounds * (0.55 + ((seed * 37) % 50) / 100)));
  const deaths = Math.max(3, Math.round(rounds * (0.5 + ((seed * 53) % 40) / 100)));
  const score = Math.round(kills * 190 + rounds * 55 + ((seed * 71) % 600));
  return {
    puuid: player.puuid,
    game_name: player.is_incognito ? "" : player.game_name,
    tag_line: player.is_incognito ? "" : player.tag_line,
    is_self: player.is_self,
    is_ally: player.is_ally,
    is_incognito: player.is_incognito,
    team_id: player.is_ally ? "Blue" : "Red",
    agent_name: player.agent_name,
    agent_icon: player.agent_icon,
    kills,
    deaths,
    assists: (seed * 13) % 9,
    score,
    acs: Math.round((score / rounds) * 10) / 10,
  };
}

function resultOf(won: number, lost: number): PostMatchView["result"] {
  if (won === lost) return "draw";
  return won > lost ? "victory" : "defeat";
}

function demoPostMatch(won: number, lost: number): PostMatchView {
  const rounds = Math.max(1, won + lost);
  const roster = lobby ?? demoLobby(false);
  const players = [...roster.allies, ...roster.enemies]
    .map((player, index) => postMatchLine(player, rounds, index + 3))
    .sort((a, b) => b.score - a.score);
  const me = players.find((player) => player.is_self);
  const jett = demoAgent("JETT");
  return {
    match_id: `demo-post-${Date.now()}`,
    result: resultOf(won, lost),
    rounds_won: won,
    rounds_lost: lost,
    map_name: "Ascent",
    map_splash: `${MEDIA}/maps/7eaecc1b-4337-bbf6-6ab9-04b8f06b3319/splash.png`,
    queue_id: party.queue.id,
    queue_label: party.queue.label,
    is_ranked: party.queue.id === "competitive",
    started_at_ms: Date.now() - rounds * 100_000,
    agent_name: jett?.display_name ?? "Jett",
    agent_icon: jett?.icon ?? null,
    agent_portrait: jett?.portrait ?? null,
    stats: {
      kills: me?.kills ?? 0,
      deaths: me?.deaths ?? 0,
      assists: me?.assists ?? 0,
      combat_score: me?.score ?? 0,
      rounds_played: rounds,
      acs: me?.acs ?? 0,
      adr: 142.6,
      headshot_percent: 27.4,
      first_bloods: 3,
      first_deaths: 2,
    },
    rr_change: null,
    rr_after: null,
    rank_after: null,
    players,
    is_seen: false,
  };
}

/** Like the backend: the summary a moment after the menus return, the RR change later. */
function announcePostMatch(won: number, lost: number): void {
  later(1800, () => {
    postMatch = demoPostMatch(won, lost);
    emit({ type: "post_match", data: postMatch });
    if (!postMatch.is_ranked) return;
    later(2600, () => {
      if (!postMatch) return;
      const delta = { victory: 21, defeat: -16, draw: 2 }[postMatch.result];
      postMatch = { ...postMatch, rr_change: delta, rr_after: 41 + delta, rank_after: rank(18, "Diamond 1", "b87de8ff") };
      emit({ type: "post_match", data: postMatch });
    });
  });
}

function playRounds(round: number): void {
  if (round >= 9) {
    later(2500, () => {
      matchStarted = false;
      const won = game.ally_score ?? 0;
      const lost = game.enemy_score ?? 0;
      setGame({ phase: "menus", map_name: "", ally_score: null, enemy_score: null, side: null, queue: "" });
      if (lobby) setLobby({ ...lobby, is_live: false });
      emit({ type: "match_flow", data: { kind: "show_menu" } });
      emit({ type: "match_flow", data: { kind: "enter_scene", scene: "home" } });
      announcePostMatch(won, lost);
    });
    return;
  }
  later(2200, () => {
    const isWin = Math.random() < 0.6;
    setGame({
      ally_score: (game.ally_score ?? 0) + (isWin ? 1 : 0),
      enemy_score: (game.enemy_score ?? 0) + (isWin ? 0 : 1),
    });
    playRounds(round + 1);
  });
}

// ---- commands ---------------------------------------------------------------------------

type Handler<C extends Command> = (args: ArgsOf<C>) => ResultOf<C> | Promise<ResultOf<C>>;
type HandlerTable = { [C in Command]: Handler<C> };

function needPregame(): PregameView {
  if (!pregame) throw new Error("Could not pick: you are not in agent select.");
  return pregame;
}

const HANDLERS: HandlerTable = {
  get_settings: () => settings,
  save_settings: ({ settings: next }) => {
    settings = next;
    persist("settings", settings);
    return settings;
  },
  get_game_snapshot: () => game,
  get_connection: () => connection,
  get_catalog: () => fullCatalog,
  get_game_videos: () => [],
  get_ui_sounds: () => [],
  open_sounds_folder: () => null,
  resolve_embed_link: () => null,
  get_wallet: () => wallet,
  quit_app: () => null,
  get_store: () => demoStore(),
  get_career: () => demoCareer(),
  get_match_history: () => demoHistory(),
  get_news: () => demoNews(),
  get_lobby: () => lobby,
  // A finished match is already on record, so the screen can be designed before playing one.
  get_post_match: () =>
    (postMatch ??= { ...demoPostMatch(13, 9), is_seen: true, rr_change: 21, rr_after: 62, rank_after: rank(18, "Diamond 1", "b87de8ff") }),
  dismiss_post_match: () => {
    if (postMatch && !postMatch.is_seen) {
      postMatch = { ...postMatch, is_seen: true };
      emit({ type: "post_match", data: postMatch });
    }
    return null;
  },
  get_friends: () => FRIENDS,
  get_collection: () => collection,
  get_skin_options: ({ weaponId }) => demoSkinOptions(weaponId),
  equip_skin: ({ weaponId, skinId, chromaId }) => {
    const option = demoSkinOptions(weaponId).find((entry) => entry.skin_id === skinId);
    if (!option) throw new Error("Could not equip that skin: you do not own it.");
    collection = {
      weapons: collection.weapons.map((weapon) =>
        weapon.weapon_id === weaponId
          ? { ...weapon, skin_id: skinId, skin_name: option.name, chroma_id: chromaId, edition: option.edition, image: option.chromas.find((chroma) => chroma.chroma_id === chromaId && chroma.image)?.image ?? option.image }
          : weapon,
      ),
    };
    return collection;
  },
  get_party: () => party,
  party_refresh: () => party,
  party_change_queue: ({ queueId }) => {
    const queue = QUEUES.find((option) => option.id === queueId);
    if (!queue) return null;
    const isCustom = queue.id === "custom";
    setParty({ queue, is_custom_game: isCustom, max_size: isCustom ? 12 : 5, custom: isCustom ? demoCustomLobby() : null });
    return null;
  },
  party_change_team: ({ team, puuid }) => {
    const lobby = party.custom;
    if (!lobby) throw new Error("Could not change team: the party is not in a custom game.");
    const without = (ids: string[]) => ids.filter((id) => id !== puuid);
    const next = { ...lobby, attackers: without(lobby.attackers), defenders: without(lobby.defenders), spectators: without(lobby.spectators) };
    next[team] = [...next[team], puuid];
    setParty({ custom: next });
    return null;
  },
  party_start_custom: () => {
    if (!party.custom) throw new Error("Could not start the custom game: the party is not in one.");
    later(1500, startAgentSelect);
    return null;
  },
  party_start_queue: () => {
    setParty({ is_queueing: true, queue_entry_time: new Date().toISOString() });
    later(7000, () => {
      if (party.is_queueing) startAgentSelect();
    });
    return null;
  },
  party_stop_queue: () => {
    setParty({ is_queueing: false, queue_entry_time: null });
    return null;
  },
  party_set_ready: ({ isReady }) => {
    setParty({ members: party.members.map((m) => (m.is_self ? { ...m, is_ready: isReady } : m)) });
    return null;
  },
  party_set_open: ({ isOpen }) => {
    setParty({ is_open: isOpen });
    return null;
  },
  party_invite: ({ riotId }) => {
    const [name, tag] = riotId.split("#");
    if (!name || !tag) throw new Error("Could not invite: type the Riot ID as Name#Tag.");
    if (party.members.length >= party.max_size) throw new Error("Could not send the invite: the party is full.");
    later(1600, () => setParty({ members: [...party.members, member(name, tag, { puuid: `demo-${name}`, is_ready: false })] }));
    return null;
  },
  party_kick: ({ puuid }) => {
    setParty({ members: party.members.filter((m) => m.puuid !== puuid) });
    return null;
  },
  party_leave: () => {
    setParty({ members: party.members.filter((m) => m.is_self) });
    return null;
  },
  party_generate_code: () => {
    setParty({ invite_code: "VL7Q2X" });
    return null;
  },
  party_join_code: ({ code }) => {
    if (!code.trim()) throw new Error("Could not join: type a party code.");
    return null;
  },
  get_pregame: () => pregame,
  pregame_select: ({ agentUuid }) => {
    const agent = catalog.agents.find((entry) => entry.uuid === agentUuid);
    needPregame();
    if (agent) updateAlly("demo-self", agent.upper_name, "selected");
    return null;
  },
  pregame_lock: ({ agentUuid }) => {
    const agent = catalog.agents.find((entry) => entry.uuid === agentUuid);
    needPregame();
    if (agent) updateAlly("demo-self", agent.upper_name, "locked");
    afterOwnLock();
    return null;
  },
  pregame_dodge: () => {
    needPregame();
    setPregame(null);
    setGame({ phase: "menus", map_name: "" });
    emit({ type: "match_flow", data: { kind: "enter_scene", scene: "play" } });
    return null;
  },
  list_ui_documents: ({ kind }) => load<string[]>(`docs.${kind}`, []),
  load_ui_document: ({ kind, name }) => load<unknown>(`doc.${kind}.${name}`, null),
  save_ui_document: ({ kind, name, document }) => {
    persist(`doc.${kind}.${name}`, document);
    const names = load<string[]>(`docs.${kind}`, []);
    if (!names.includes(name)) persist(`docs.${kind}`, [...names, name]);
    return null;
  },
  delete_ui_document: ({ kind, name }) => {
    try {
      localStorage.removeItem(`${STORAGE_PREFIX}doc.${kind}.${name}`);
    } catch {
      // Nothing to delete.
    }
    persist(`docs.${kind}`, load<string[]>(`docs.${kind}`, []).filter((entry) => entry !== name));
    return null;
  },
  set_overlay_enabled: () => null,
  set_overlay_preview: () => null,
  instant_clone_state: () => demoDelay,
  instant_clone_send: ({ action, delayMs }) => {
    const delay = action === "delay" ? delayMs : 0;
    demoDelay = { ...demoDelay, phase: delay > 0 ? "active" : "idle", armed_delay_ms: delay, current_delay_ms: delay, buffer_fill_ms: delay };
    return null;
  },
};

/** Answer one command like the Rust backend would (errors are thrown as messages). */
export async function handle<C extends Command>(command: C, args: ArgsOf<C>): Promise<ResultOf<C>> {
  const handler = HANDLERS[command] as Handler<C>;
  await new Promise((resolve) => setTimeout(resolve, 40));
  try {
    return await handler(args);
  } catch (error) {
    throw error instanceof Error ? error.message : String(error);
  }
}

// The demo announces itself like a real connection a moment after the UI starts.
setTimeout(() => {
  emit({ type: "connection", data: connection });
  emit({ type: "game_state", data: game });
  emit({ type: "party", data: party });
  emit({ type: "wallet", data: wallet });
  emit({ type: "friends", data: FRIENDS });
}, 300);
