// CS2-style pieces: the play panel (mode tabs, map pool grid, green GO), a rating badge in
// the banded parallelogram look, and the friends rail of square tiles down the right edge.
import { For, Index, Show, createMemo, createSignal, onCleanup, onMount, type JSX } from "solid-js";
import { Portal } from "solid-js/web";
import { Check, ChevronDown, Plus, Search, Users, X } from "lucide-solid";
import type { FriendView } from "~/bindings/FriendView";
import type { PieceProps } from "~/studio/registry";
import { playCue } from "~/menu/sound";
import { catalog } from "~/state/catalog";
import { friendActivity, friends, isElsewhere, onlineCount } from "~/state/friends";
import { party, partyActions } from "~/state/party";
import { isConnected } from "~/state/session";
import { careerView } from "~/state/views";
import { Art } from "~/ui/Art";
import { Silhouette } from "./avatars";
import { ladderPosition, usePlay } from "./usePlay";
import styles from "./cs2.module.css";

const MAX_MAPS = 10;

/** One line on each mode, like CS2's mode cards. */
const MODE_NOTES: Record<string, string> = {
  competitive: "5v5 · first to 13 · ranked",
  unrated: "5v5 · first to 13 · unranked",
  swiftplay: "5v5 · first to 5 · quick games",
  spikerush: "5v5 · first to 4 · random guns",
  deathmatch: "Free for all · first to 40 kills",
  hurm: "5v5 · respawns · first to 100 kills",
  ggteam: "5v5 · weapon ladder · first to the top",
  premier: "Your team · weekly matches",
};

// ---- play panel -------------------------------------------------------------------------

export function Cs2Play(props: PieceProps) {
  const play = usePlay();
  const showMaps = () => props.piece.props["showMaps"] !== false;
  const maps = createMemo(() =>
    catalog()
      .maps.filter((map) => !map.is_team_deathmatch && map.upper_name !== null)
      .slice(0, MAX_MAPS),
  );
  const buttonLabel = () => {
    if (play.searchClock() !== null) return "Cancel";
    if (play.penaltyClock()) return play.penaltyClock();
    if (!play.isOwner()) return play.isReady() ? "Ready" : "Ready up";
    return "Go";
  };
  return (
    <section class={styles.play} data-searching={play.searchClock() !== null ? "" : undefined}>
      <header
        class={styles.modes}
        role="tablist"
        onWheel={(event) => {
          if (Math.abs(event.deltaX) > Math.abs(event.deltaY)) return;
          event.preventDefault();
          event.currentTarget.scrollLeft += event.deltaY;
        }}
      >
        <For each={play.queues()}>
          {(queue) => (
            <button
              role="tab"
              aria-selected={play.current()?.id === queue.id}
              class={styles.mode}
              data-active={play.current()?.id === queue.id ? "" : undefined}
              disabled={!play.isOwner()}
              onMouseEnter={() => playCue("hover")}
              onClick={(event) => {
                play.pickQueue(queue);
                event.currentTarget.scrollIntoView({ inline: "nearest", block: "nearest", behavior: "smooth" });
              }}
            >
              {queue.label}
            </button>
          )}
        </For>
      </header>
      <Show when={!showMaps() && play.current()}>
        {(queue) => (
          <div class={styles.modeInfo}>
            <b>{queue().label}</b>
            <span>{MODE_NOTES[queue().id] ?? "Matchmaking"}</span>
            <span class={styles.modeMaps}>
              <For each={maps().slice(0, 7)}>
                {(map) => <Art src={map.list_icon ?? map.splash} class={styles.modeMap} fallback={<i class={styles.modeMap} />} alt={map.display_name} />}
              </For>
            </span>
          </div>
        )}
      </Show>
      <Show when={showMaps()}>
        <div class={styles.maps}>
          <span class={styles.mapsTitle}>Map pool</span>
          <div class={styles.mapGrid}>
            <For each={maps()}>
              {(map) => (
                <figure class={styles.map}>
                  <Art src={map.list_icon ?? map.splash} class={styles.mapImage} fallback={<span class={styles.mapImage} />} />
                  <figcaption class={styles.mapName}>{map.display_name}</figcaption>
                </figure>
              )}
            </For>
          </div>
        </div>
      </Show>
      <footer class={styles.footer}>
        <div class={styles.party}>
          <span class={styles.slots}>
            <Index each={Array.from({ length: play.maxSize() })}>
              {(_, index) => <i data-on={index < play.partySize() ? "" : undefined} />}
            </Index>
          </span>
          <span class={styles.partyText}>
            <Show when={play.searchClock()} fallback={play.current()?.label ?? "Not connected"}>
              {(clock) => <span class="num">Searching {clock()}</span>}
            </Show>
          </span>
        </div>
        <button
          class={styles.go}
          data-part="play"
          disabled={play.searchClock() === null && !play.canStart()}
          onMouseEnter={() => playCue("hover")}
          onClick={() => play.press()}
        >
          {buttonLabel()}
        </button>
      </footer>
    </section>
  );
}

// ---- rating badge -----------------------------------------------------------------------

const RATING_FLOOR = 1000;
const RATING_SPAN = 32000;
/** Colour bands by rating, lowest first. */
const BANDS: readonly [number, string][] = [
  [0, "#b0c3d9"],
  [5000, "#8cc6ff"],
  [10000, "#6a7dff"],
  [15000, "#c166ff"],
  [20000, "#f03cff"],
  [25000, "#eb4b4b"],
  [30000, "#fed700"],
];
const RECENT_GAMES = 10;

function bandColor(rating: number): string {
  let color = BANDS[0]?.[1] ?? "#b0c3d9";
  for (const [from, band] of BANDS) if (rating >= from) color = band;
  return color;
}

export function Cs2Rating(_props: PieceProps) {
  onMount(() => {
    if (!careerView.value()) void careerView.load();
  });
  const view = () => careerView.value();
  const isRated = () => {
    const current = view();
    return current !== null && current.rank.tier >= 3 && current.placements_left === 0;
  };
  /** The rank and RR as one number on a CS2-like scale. */
  const rating = () => {
    const current = view();
    return current ? Math.round(RATING_FLOOR + ladderPosition(current.rank.tier, current.rr) * RATING_SPAN) : 0;
  };
  const recent = () => view()?.updates.slice(0, RECENT_GAMES) ?? [];
  const wins = () => recent().filter((update) => update.rr_earned > 0).length;
  const losses = () => recent().filter((update) => update.rr_earned < 0).length;
  const draws = () => recent().length - wins() - losses();
  return (
    <section class={styles.rating}>
      <span class={styles.ratingTitle}>Rating</span>
      <div class={styles.badge} style={{ "--band": isRated() ? bandColor(rating()) : "#5b6370" }}>
        <span class={styles.badgeStripes} />
        <span class={`${styles.badgeValue} num`}>
          <Show when={isRated()} fallback={<b>---</b>}>
            <b>{Math.floor(rating() / 1000)}</b>
            <small>,{(rating() % 1000).toString().padStart(3, "0")}</small>
          </Show>
        </span>
      </div>
      <span class={styles.ratingRank}>
        <Show when={view()} fallback={careerView.isLoading() ? "Loading…" : "Connect to see your rating"}>
          {(current) => (
            <Show when={isRated()} fallback={`${current().placements_left} placement games left`}>
              {current().rank.name} · {current().rr} RR
            </Show>
          )}
        </Show>
      </span>
      <Show when={recent().length > 0}>
        <div class={styles.recent}>
          <For each={recent()}>
            {(update) => (
              <i
                data-result={update.rr_earned > 0 ? "win" : update.rr_earned < 0 ? "loss" : "draw"}
                title={`${update.map_name} ${update.rr_earned > 0 ? "+" : ""}${update.rr_earned} RR`}
              />
            )}
          </For>
        </div>
        <span class={styles.recentText}>
          {wins()} W · {losses()} L{draws() > 0 ? ` · ${draws()} D` : ""} last {recent().length}
        </span>
      </Show>
    </section>
  );
}

// ---- friends rail and panel -------------------------------------------------------------

const RAIL_SIZE = 12;

type Cs2Group = "ingame" | "online" | "other" | "offline";

/** Steam's buckets: in a match (green), online (blue), another game (green), offline (grey). */
function cs2Group(friend: FriendView): Cs2Group {
  if (friend.status === "in_match" || friend.status === "agent_select") return "ingame";
  if (isElsewhere(friend)) return "other";
  if (friend.status === "offline") return "offline";
  return "online";
}

const GROUP_LABELS: readonly [Cs2Group, string][] = [
  ["ingame", "In game"],
  ["online", "Online"],
  ["other", "Other games"],
  ["offline", "Offline"],
];

const [isPanelOpen, setPanelOpen] = createSignal(false);

export function Cs2Friends(props: PieceProps) {
  let rail: HTMLElement | undefined;
  const [anchor, setAnchor] = createSignal({ top: 0, right: 0, height: 0 });
  const shown = () => friends().filter((friend) => friend.status !== "offline").slice(0, RAIL_SIZE);
  const isPinned = () => props.piece.props["openOnStart"] === true;
  onMount(() => {
    if (isPinned() && !props.isEditing) toggle(true);
  });
  const toggle = (next = !isPanelOpen()) => {
    if (rail) {
      const box = rail.getBoundingClientRect();
      setAnchor({ top: box.top, right: window.innerWidth - box.left + 6, height: box.height });
    }
    playCue(next ? "confirm" : "back");
    setPanelOpen(next);
  };
  return (
    <aside ref={rail} class={styles.rail} data-open={isPanelOpen() ? "" : undefined}>
      <button class={styles.railHead} title="Friends" onClick={() => toggle()}>
        <Users size={16} />
        <span class="num">{onlineCount()}</span>
      </button>
      <For each={shown()}>
        {(friend) => (
          <button
            class={styles.tile}
            data-status={friend.status}
            title={`${friend.game_name}#${friend.tag_line} · ${friendActivity(friend)}`}
            onMouseEnter={() => playCue("hover")}
            onClick={() => toggle(true)}
          >
            <Art src={friend.card_small} class={styles.tileImage} fallback={<Silhouette />} />
          </button>
        )}
      </For>
      <Show when={isPanelOpen()}>
        <Portal>
          <Cs2FriendPanel
            style={{ top: `${anchor().top}px`, right: `${anchor().right}px`, height: `${Math.max(anchor().height, 520)}px` }}
            onClose={() => toggle(false)}
          />
        </Portal>
      </Show>
    </aside>
  );
}

function Cs2FriendPanel(props: { style: JSX.CSSProperties; onClose: () => void }) {
  const [query, setQuery] = createSignal("");
  const [isOfflineOpen, setOfflineOpen] = createSignal(false);
  const matches = createMemo(() => {
    const text = query().trim().toLowerCase();
    return friends().filter((friend) => !text || `${friend.game_name}#${friend.tag_line}`.toLowerCase().includes(text));
  });
  const members = () => party()?.members ?? [];
  const onKey = (event: KeyboardEvent) => {
    if (event.key !== "Escape") return;
    event.stopPropagation();
    props.onClose();
  };
  onMount(() => window.addEventListener("keydown", onKey, true));
  onCleanup(() => window.removeEventListener("keydown", onKey, true));
  return (
    <section class={styles.panel} style={props.style} role="dialog" aria-label="Friends">
      <header class={styles.panelHead}>
        <span class={styles.panelTitle}>Friends</span>
        <span class={`${styles.panelCount} num`}>{onlineCount()} online</span>
        <button class={styles.panelClose} title="Close" onClick={() => props.onClose()}>
          <X size={16} />
        </button>
      </header>
      <div class={styles.panelParty}>
        <span class={styles.panelLabel}>Your party</span>
        <div class={styles.partyFaces}>
          <Index each={Array.from({ length: party()?.max_size ?? 5 })}>
            {(_, index) => (
              <Show
                when={members()[index]}
                fallback={
                  <button class={styles.partyEmpty} title="Invite a friend below" onClick={() => document.querySelector<HTMLInputElement>(`.${styles.panelSearch} input`)?.focus()}>
                    <Plus size={14} />
                  </button>
                }
              >
                {(member) => (
                  <span class={styles.partyFace} data-ready={member().is_ready ? "" : undefined} title={member().game_name}>
                    <Art src={member().card_small} class={styles.tileImage} fallback={<Silhouette />} />
                  </span>
                )}
              </Show>
            )}
          </Index>
        </div>
      </div>
      <label class={styles.panelSearch}>
        <Search size={14} />
        <input placeholder="Search friends" value={query()} spellcheck={false} onInput={(event) => setQuery(event.currentTarget.value)} />
      </label>
      <div class={styles.panelList}>
        <For each={GROUP_LABELS}>
          {([group, label]) => {
            const list = () => matches().filter((friend) => cs2Group(friend) === group);
            const isFolded = () => group === "offline" && !isOfflineOpen() && !query();
            return (
              <Show when={list().length > 0}>
                <button
                  class={styles.groupHead}
                  data-group={group}
                  onClick={() => group === "offline" && setOfflineOpen(!isOfflineOpen())}
                >
                  <span>{label}</span>
                  <span class="num">({list().length})</span>
                  <Show when={group === "offline"}>
                    <ChevronDown size={14} class={styles.groupChevron ?? ""} data-open={isFolded() ? undefined : ""} />
                  </Show>
                </button>
                <Show when={!isFolded()}>
                  <For each={list()}>{(friend) => <Cs2FriendRow friend={friend} group={group} />}</For>
                </Show>
              </Show>
            );
          }}
        </For>
      </div>
    </section>
  );
}

function Cs2FriendRow(props: { friend: FriendView; group: Cs2Group }) {
  const [isSent, setSent] = createSignal(false);
  const canInvite = () =>
    isConnected() && (props.friend.status === "menus" || props.friend.status === "queueing") && !(party()?.members.some((member) => member.puuid === props.friend.puuid) ?? false);
  return (
    <div class={styles.friendRow} data-group={props.group} onMouseEnter={() => playCue("hover")}>
      <span class={styles.friendFace}>
        <Art src={props.friend.card_small} class={styles.tileImage} fallback={<Silhouette />} />
      </span>
      <span class={styles.friendText}>
        <b data-sensitive>{props.friend.game_name}</b>
        <small>{props.group === "ingame" ? `VALORANT · ${friendActivity(props.friend)}` : friendActivity(props.friend)}</small>
      </span>
      <Show when={canInvite()}>
        <button
          class={styles.friendInvite}
          data-sent={isSent() ? "" : undefined}
          disabled={isSent()}
          title="Invite to party"
          onClick={async () => {
            playCue("confirm");
            if (await partyActions.invite(`${props.friend.game_name}#${props.friend.tag_line}`)) setSent(true);
          }}
        >
          <Show when={isSent()} fallback={<Plus size={14} />}>
            <Check size={14} />
          </Show>
        </button>
      </Show>
    </div>
  );
}
