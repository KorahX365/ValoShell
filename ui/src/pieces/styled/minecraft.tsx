// Minecraft-style pieces, after the Java edition: the title screen (blocky logo, the yellow
// splash text, stone buttons in a column with half-width pairs, version line), the party
// as inventory slots (account level as the item count, a purple-edged tooltip on hover) and
// the store as a villager trading screen (VP in, skin out).
import { For, Index, Show, createSignal, onMount, type JSX } from "solid-js";
import { useNavigate } from "@solidjs/router";
import type { MatchRow } from "~/bindings/MatchRow";
import type { PieceProps } from "~/studio/registry";
import { call } from "~/ipc/commands";
import { openEscMenu } from "~/menu/EscMenu";
import { playCue } from "~/menu/sound";
import { formatDuration } from "~/pieces/store";
import { friendActivity, friends, setDrawerOpen } from "~/state/friends";
import { party, partyActions } from "~/state/party";
import { settings, updateSettings } from "~/state/settings";
import { careerView, historyView, storeView } from "~/state/views";
import { Art } from "~/ui/Art";
import { usePlay } from "./usePlay";
import styles from "./minecraft.module.css";

const SPLASHES = [
  "Also try CS2!",
  "Now with stream delay!",
  "Plant the spike!",
  "Radiant or bust!",
  "Check the night market!",
  "Eco round!",
  "Open source!",
  "One tap!",
  "Ult is ready!",
  "Rotate B!",
];

function McButton(props: { children: JSX.Element; onClick: () => void; isDisabled?: boolean; isPlay?: boolean }) {
  return (
    <button
      class={styles.button}
      data-part={props.isPlay ? "play" : undefined}
      disabled={props.isDisabled ?? false}
      onMouseEnter={() => playCue("hover")}
      onClick={() => {
        playCue("confirm");
        props.onClick();
      }}
    >
      <span>{props.children}</span>
    </button>
  );
}

/** Steve's face in 8 x 8 pixels: the stand-in for a player without card art. */
const STEVE = ["HHHHHHHH", "HHHHHHHH", "HSSSSSSH", "SSSSSSSS", "SWBSSBWS", "SSSNNSSS", "SSMSSMSS", "SSMMMMSS"];
const STEVE_COLORS: Record<string, string> = { H: "#3b2810", S: "#c69680", W: "#ffffff", B: "#4a3a8c", N: "#94604a", M: "#5e3a26" };

export function SteveHead(props: { class?: string | undefined }) {
  return (
    <svg class={props.class} viewBox="0 0 8 8" shape-rendering="crispEdges" aria-hidden="true">
      <Index each={STEVE}>
        {(row, y) => <Index each={[...row()]}>{(cell, x) => <rect x={x} y={y} width="1" height="1" fill={STEVE_COLORS[cell()] ?? "#000"} />}</Index>}
      </Index>
    </svg>
  );
}

// ---- title screen -----------------------------------------------------------------------

export function McMenu(_props: PieceProps) {
  const play = usePlay();
  const splash = SPLASHES[Math.floor(Math.random() * SPLASHES.length)] ?? SPLASHES[0];
  const isOpen = () => party()?.is_open ?? false;
  const isStreamerMode = () => settings()?.streamer.is_enabled ?? false;
  const cycleQueue = () => {
    const queues = play.queues();
    const index = queues.findIndex((queue) => queue.id === play.current()?.id);
    const next = queues[(index + 1) % Math.max(1, queues.length)];
    if (next) play.pickQueue(next);
  };
  const playLabel = () => {
    const clock = play.searchClock();
    if (clock !== null) return `Cancel search (${clock})`;
    if (play.penaltyClock()) return `Locked for ${play.penaltyClock()}`;
    if (!play.isOwner()) return play.isReady() ? "Not ready" : "Ready";
    return "Play";
  };
  return (
    <section class={styles.title}>
      <div class={styles.logo}>
        <span class={styles.logoText}>ValoShell</span>
        <span class={styles.splash}>{splash}</span>
      </div>
      <div class={styles.buttons}>
        <McButton isPlay isDisabled={play.searchClock() === null && !play.canStart()} onClick={() => play.press()}>
          {playLabel()}
        </McButton>
        <McButton isDisabled={!play.isOwner()} onClick={cycleQueue}>
          Mode: {play.current()?.label ?? "None"}
        </McButton>
        {/* An options toggle, worded like the game's own: "Streamer mode: ON". */}
        <McButton
          onClick={() =>
            updateSettings((draft) => {
              draft.streamer.is_enabled = !draft.streamer.is_enabled;
            })
          }
        >
          Streamer mode: {isStreamerMode() ? "ON" : "OFF"}
        </McButton>
        <div class={styles.pair}>
          <McButton onClick={() => setDrawerOpen(true)}>Friends...</McButton>
          <McButton isDisabled={!play.isOwner()} onClick={() => void partyActions.setOpen(!isOpen())}>
            Party: {isOpen() ? "Open" : "Closed"}
          </McButton>
        </div>
        <div class={styles.pair}>
          <McButton onClick={openEscMenu}>Options...</McButton>
          <McButton onClick={() => void call("quit_app", {})}>Quit Game</McButton>
        </div>
      </div>
      <footer class={styles.version}>
        <span>
          ValoShell {__APP_VERSION__} ({play.searchClock() ? `searching ${play.searchClock()}` : `${play.partySize()}/${play.maxSize()} in party`})
        </span>
        <span>Not affiliated with Riot Games.</span>
      </footer>
    </section>
  );
}

// ---- party inventory --------------------------------------------------------------------

const FRIEND_SLOTS = 9;

export function McParty(props: PieceProps) {
  const title = () => (typeof props.piece.props["title"] === "string" && props.piece.props["title"]) || "Party";
  const slots = () => {
    const members = party()?.members ?? [];
    return Array.from({ length: party()?.max_size ?? 5 }, (_, index) => members[index] ?? null);
  };
  const online = () => friends().filter((friend) => friend.status !== "offline");
  const friendSlots = () => Array.from({ length: FRIEND_SLOTS }, (_, index) => online()[index] ?? null);
  const invite = () => {
    playCue("confirm");
    setDrawerOpen(true);
  };
  return (
    <section class={styles.panel}>
      <span class={styles.panelTitle}>{title()}</span>
      <div class={styles.slots}>
        {/* By position, so a party event updates a slot in place and keeps its tooltip open. */}
        <Index each={slots()}>
          {(slot) => (
            <Show
              when={slot()}
              fallback={
                <button class={styles.slot} title="Invite" onClick={invite}>
                  <span class={styles.slotPlus}>+</span>
                </button>
              }
            >
              {(member) => (
                <div class={styles.slot} tabIndex={0} data-self={member().is_self ? "" : undefined}>
                  <Art
                    src={member().card_small ?? member().card_tall}
                    class={styles.slotItem}
                    fallback={<SteveHead class={styles.slotItem} />}
                  />
                  <span class={styles.count}>{member().account_level ?? ""}</span>
                  <span class={styles.tooltip}>
                    <b>{member().game_name}</b>
                    <small>{member().rank.tier >= 3 ? member().rank.name : "Unranked"}</small>
                    <small data-ready={member().is_ready ? "" : undefined}>{member().is_ready ? "Ready" : "Not ready"}</small>
                  </span>
                </div>
              )}
            </Show>
          )}
        </Index>
      </div>
      <span class={styles.panelTitle}>Friends online</span>
      <div class={styles.slots} data-grid="">
        <For each={friendSlots()}>
          {(friend) =>
            friend ? (
              <button class={styles.slot} data-status={friend.status} onClick={invite}>
                <Art src={friend.card_small} class={styles.slotItem} fallback={<SteveHead class={styles.slotItem} />} />
                <span class={styles.tooltip}>
                  <b>{friend.game_name}</b>
                  <small>{friendActivity(friend)}</small>
                </span>
              </button>
            ) : (
              <span class={styles.slot} />
            )
          }
        </For>
      </div>
    </section>
  );
}

// ---- villager trading -------------------------------------------------------------------

export function McTrades(_props: PieceProps) {
  const navigate = useNavigate();
  const [picked, setPicked] = createSignal(0);
  onMount(() => {
    if (!storeView.value()) void storeView.load();
  });
  const offers = () => storeView.value()?.daily ?? [];
  const current = () => offers()[picked()] ?? offers()[0];
  return (
    <section class={styles.panel}>
      <span class={styles.panelTitle}>
        Store <small>· refreshes in {formatDuration(storeView.value()?.daily_ends_in_s ?? 0)}</small>
      </span>
      <div class={styles.trade}>
        <ul class={styles.tradeList}>
          <For each={offers()} fallback={<li class={styles.tradeEmpty}>{storeView.isLoading() ? "Loading trades..." : "No trades yet."}</li>}>
            {(item, index) => (
              <li>
                <button
                  class={styles.tradeRow}
                  data-active={index() === picked() ? "" : undefined}
                  onMouseEnter={() => playCue("hover")}
                  onClick={() => setPicked(index())}
                >
                  <span class={styles.vp}>VP</span>
                  <span class={`${styles.tradeCost} num`}>{item.price}</span>
                  <span class={styles.arrow} aria-hidden="true" />
                  <Art src={item.image} class={styles.tradeIcon} fallback={<span class={styles.tradeIcon} />} />
                </button>
              </li>
            )}
          </For>
        </ul>
        <Show when={current()}>
          {(item) => (
            <div class={styles.tradeDetail}>
              <span class={styles.tradeName}>{item().name}</span>
              <div class={styles.tradeSlots}>
                <span class={styles.slot}>
                  <span class={styles.vp}>VP</span>
                  <span class={styles.count}>{item().price}</span>
                </span>
                <span class={styles.bigArrow} aria-hidden="true" />
                <span class={styles.slot} data-big="">
                  <Art src={item().image} class={styles.slotItem} fallback={<span class={styles.slotLetter}>?</span>} />
                </span>
              </div>
              <button
                class={styles.button}
                onClick={() => {
                  playCue("confirm");
                  navigate("/store");
                }}
              >
                <span>Open store</span>
              </button>
            </div>
          )}
        </Show>
      </div>
    </section>
  );
}

// ---- select world (modes) -----------------------------------------------------------------

/** One line on each mode, as a world's subtitle. */
const MODE_LINES: Record<string, [string, string]> = {
  unrated: ["Survival Mode", "13 rounds to win, no rank on the line"],
  competitive: ["Hardcore Mode!", "Ranked. Every round counts"],
  swiftplay: ["Survival Mode", "Short 5-round matches"],
  spikerush: ["Adventure Mode", "Random weapons, orbs, fast rounds"],
  deathmatch: ["Creative Mode", "Free-for-all warmup"],
  hurm: ["Survival Mode", "Teams, respawns and loadouts"],
};

export function McWorlds(_props: PieceProps) {
  const play = usePlay();
  const [picked, setPicked] = createSignal<string | null>(null);
  const selected = () => picked() ?? play.current()?.id ?? null;
  const confirm = () => {
    const queue = play.queues().find((entry) => entry.id === selected());
    if (queue && queue.id !== play.current()?.id) play.pickQueue(queue);
    else play.press();
  };
  return (
    <section class={styles.worlds}>
      <h2 class={styles.worldsTitle}>Select World</h2>
      <div class={styles.worldList}>
        <For each={play.queues()}>
          {(queue) => {
            const line = () => MODE_LINES[queue.id] ?? ["Survival Mode", "Matchmaking"];
            return (
              <button
                class={styles.world}
                data-selected={selected() === queue.id ? "" : undefined}
                onMouseEnter={() => playCue("hover")}
                onClick={() => {
                  playCue("tab");
                  setPicked(queue.id);
                }}
                onDblClick={() => {
                  setPicked(queue.id);
                  confirm();
                }}
              >
                <span class={styles.worldIcon} data-mode={queue.id} />
                <span class={styles.worldText}>
                  <b>{queue.label}</b>
                  <small>{queue.id === play.current()?.id ? "Current mode" : `${queue.id}.vlw`}</small>
                  <small data-mode={queue.id}>
                    {line()[0]}, {line()[1]}
                  </small>
                </span>
              </button>
            );
          }}
        </For>
      </div>
      <div class={styles.worldButtons}>
        <McButton isPlay isDisabled={!play.isOwner() && selected() !== play.current()?.id} onClick={confirm}>
          {selected() === play.current()?.id ? (play.searchClock() ? `Cancel (${play.searchClock()})` : "Play Selected World") : "Select This Mode"}
        </McButton>
        <McButton isDisabled={picked() === null} onClick={() => setPicked(null)}>
          Cancel
        </McButton>
      </div>
    </section>
  );
}

// ---- statistics (career) ------------------------------------------------------------------

type StatsTab = "general" | "matches" | "agents";

export function McStats(_props: PieceProps) {
  const [tab, setTab] = createSignal<StatsTab>("general");
  onMount(() => {
    if (!careerView.value() && !careerView.isLoading()) void careerView.load();
    if (!historyView.value() && !historyView.isLoading()) void historyView.load();
  });
  const rows = () => historyView.value()?.rows ?? [];
  const sum = (pick: (row: MatchRow) => number) => rows().reduce((total, row) => total + pick(row), 0);
  const general = (): [string, string][] => {
    const view = careerView.value();
    const deaths = sum((row) => row.deaths);
    return [
      ["Rank", view && view.rank.tier >= 3 ? `${view.rank.name} (${view.rr} RR)` : "Unranked"],
      ["Peak Rank This Act", view?.peak?.name ?? "-"],
      ["Games Played This Act", String(view?.games ?? 0)],
      ["Games Won This Act", String(view?.wins ?? 0)],
      ["Mobs Killed (recent)", String(sum((row) => row.kills))],
      ["Deaths (recent)", String(deaths)],
      ["Assists (recent)", String(sum((row) => row.assists))],
      ["K/D Ratio", deaths ? (sum((row) => row.kills) / deaths).toFixed(2) : "-"],
      ["Times Named MVP", String(rows().filter((row) => row.mvp !== null).length)],
    ];
  };
  const agents = (): [string, string][] => {
    const counts = new Map<string, number>();
    for (const row of rows()) if (row.agent_name) counts.set(row.agent_name, (counts.get(row.agent_name) ?? 0) + 1);
    return [...counts.entries()].sort((a, b) => b[1] - a[1]).map(([name, games]) => [name, `${games} played`]);
  };
  const matches = (): [string, string][] =>
    rows().map((row) => [`${row.queue_label} on ${row.map_name}`, `${row.result === "victory" ? "Won" : row.result === "draw" ? "Drew" : "Lost"} ${row.kills}/${row.deaths}/${row.assists}`]);
  const shown = () => (tab() === "general" ? general() : tab() === "matches" ? matches() : agents());
  return (
    <section class={styles.stats}>
      <h2 class={styles.worldsTitle}>Statistics</h2>
      <div class={styles.statTabs}>
        <For each={[["general", "General"], ["matches", "Matches"], ["agents", "Agents"]] as const}>
          {([id, label]) => (
            <McButton isDisabled={tab() === id} onClick={() => setTab(id)}>
              {label}
            </McButton>
          )}
        </For>
      </div>
      <div class={styles.statList}>
        <For each={shown()} fallback={<p class={styles.statEmpty}>{historyView.isLoading() ? "Loading terrain..." : "Nothing here yet."}</p>}>
          {([label, value]) => (
            <div class={styles.statRow}>
              <span>{label}</span>
              <span>{value}</span>
            </div>
          )}
        </For>
      </div>
    </section>
  );
}
