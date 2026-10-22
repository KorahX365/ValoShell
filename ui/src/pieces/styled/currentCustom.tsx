// The current client's custom-game lobby: Attackers, Defenders and Spectators columns with a
// row per player, the swap arrows that move you to a column, invite and autobalance under it,
// and the mode card's MODE / MAP / OPTIONS / SERVER rows. Measured on a 1920 x 1080 client.
import { For, Index, Show, type Component } from "solid-js";
import { ChevronDown, Plus } from "lucide-solid";
import type { MemberView } from "~/bindings/MemberView";
import type { PieceProps } from "~/studio/registry";
import { playCue } from "~/menu/sound";
import { setDrawerOpen } from "~/state/friends";
import { party, partyActions } from "~/state/party";
import { pushToast } from "~/state/toasts";
import { Art } from "~/ui/Art";
import { modeLabel, vt } from "./valText";
import styles from "./currentCustom.module.css";
import shared from "./current.module.css";

type Column = "attackers" | "defenders" | "spectators";

/** Rows per column: five a side, two spectator seats shown. */
const COLUMNS: readonly { id: Column; rows: number }[] = [
  { id: "attackers", rows: 5 },
  { id: "defenders", rows: 5 },
  { id: "spectators", rows: 2 },
];

/** True while the party sits in a custom lobby; the home pieces hand the screen over. */
export const isCustomLobby = () => party()?.custom != null;

/** A home piece the client takes off the screen in a custom lobby (news, dailies, lineup). */
export function hiddenInCustomLobby(Piece: Component<PieceProps>): Component<PieceProps> {
  return (props) => (
    <Show when={!isCustomLobby()}>
      <Piece {...props} />
    </Show>
  );
}

// ---- mode card rows ------------------------------------------------------------------------

function settingsNote() {
  playCue("confirm");
  pushToast("info", vt("options"), vt("settingsInClient"));
}

/** MODE, MAP, OPTIONS and SERVER under the custom card, as the client lists them. */
export function CustomModeRows() {
  const lobby = () => party()?.custom;
  const rows = () => [
    { label: `${vt("modeRow")}:`, value: modeLabel(lobby()?.mode_name || "—") },
    { label: `${vt("mapRow")}:`, value: lobby()?.map_name || "—" },
    { label: vt("options"), value: "" },
  ];
  return (
    <>
      <For each={rows()}>
        {(row) => (
          <button class={shared.modeButton} data-size="short" onMouseEnter={() => playCue("hover")} onClick={settingsNote}>
            <span>
              {row.label} <b class={styles.rowValue}>{row.value}</b>
            </span>
          </button>
        )}
      </For>
      <button class={shared.modeButton} data-size="short" onMouseEnter={() => playCue("hover")} onClick={settingsNote}>
        <span>
          {vt("server")}: <b class={styles.rowValue}>{lobby()?.server_name || "—"}</b>
        </span>
        <ChevronDown size={16} strokeWidth={1.8} />
      </button>
    </>
  );
}

// ---- team columns --------------------------------------------------------------------------

function SwapGlyph() {
  return (
    <svg width="30" height="22" viewBox="0 0 30 22" fill="none" stroke="currentColor" stroke-width="2.2" aria-hidden="true">
      <path d="M28 6 H3 M9 1 L3 6 L9 11" />
      <path d="M2 16 H27 M21 11 L27 16 L21 21" />
    </svg>
  );
}

function MemberRow(props: { member: MemberView }) {
  return (
    <div class={styles.member} data-self={props.member.is_self ? "" : undefined}>
      <Art src={props.member.card_small} class={styles.avatar} fallback={<span class={styles.avatar} />} />
      <span class={styles.who}>
        <b class={styles.name} data-sensitive>
          {props.member.game_name || "Player"}
        </b>
        <span class={styles.state} data-ready={props.member.is_ready ? "" : undefined}>
          {props.member.is_ready ? vt("ready") : vt("notReady")}
        </span>
      </span>
      <Show when={props.member.ping_ms !== null}>
        <span class={styles.ping}>{props.member.ping_ms} ms</span>
      </Show>
      <span class={styles.mute} aria-hidden="true">
        <svg width="22" height="18" viewBox="0 0 22 18" fill="currentColor">
          <path d="M1 6 H5 L10.5 1.5 V16.5 L5 12 H1 Z" />
          <path d="M14 6.5 L19 11.5 M19 6.5 L14 11.5" stroke="currentColor" stroke-width="1.6" fill="none" />
        </svg>
      </span>
    </div>
  );
}

/** An empty row: the first one offers a move here (or an invite, in your own column). */
function EmptyRow(props: { column: Column; isFirst: boolean; isOwnColumn: boolean }) {
  const self = () => party()?.members.find((member) => member.is_self);
  return (
    <Show when={props.isFirst} fallback={<div class={styles.row} />}>
      <Show
        when={!props.isOwnColumn}
        fallback={
          <button
            class={styles.row}
            title={vt("invite")}
            onMouseEnter={() => playCue("hover")}
            onClick={() => {
              playCue("confirm");
              setDrawerOpen(true);
            }}
          >
            <Plus size={22} strokeWidth={2.6} />
          </button>
        }
      >
        <button
          class={styles.row}
          title={vt("joinTeam")}
          disabled={!self()}
          onMouseEnter={() => playCue("hover")}
          onClick={() => {
            const own = self();
            if (!own) return;
            playCue("tab");
            void partyActions.changeTeam(props.column, own.puuid);
          }}
        >
          <SwapGlyph />
        </button>
      </Show>
    </Show>
  );
}

function TeamColumn(props: { id: Column; rows: number }) {
  const lobby = () => party()?.custom;
  const members = () => {
    const ids = lobby()?.[props.id] ?? [];
    const everyone = party()?.members ?? [];
    return ids.map((puuid) => everyone.find((member) => member.puuid === puuid)).filter((member): member is MemberView => !!member);
  };
  const isOwnColumn = () => members().some((member) => member.is_self);
  const slots = () => Array.from({ length: props.rows }, (_, index) => members()[index]);
  return (
    <section class={styles.column} data-team={props.id}>
      <header class={styles.header}>{vt(props.id)}</header>
      <Index each={slots()}>
        {(member, index) => (
          <Show when={member()} fallback={<EmptyRow column={props.id} isFirst={index === members().length} isOwnColumn={isOwnColumn()} />}>
            {(current) => <MemberRow member={current()} />}
          </Show>
        )}
      </Index>
    </section>
  );
}

export function ValCustomLobby(_props: PieceProps) {
  const lobby = () => party()?.custom;
  return (
    <Show when={lobby()}>
      {(current) => (
        <div class={styles.lobby}>
          <div class={styles.columns}>
            <For each={COLUMNS}>{(column) => <TeamColumn id={column.id} rows={column.rows} />}</For>
          </div>
          <div class={styles.actions}>
            <button
              class={styles.action}
              onMouseEnter={() => playCue("hover")}
              onClick={() => {
                playCue("confirm");
                setDrawerOpen(true);
              }}
            >
              {vt("invite")} <Plus size={17} strokeWidth={2} />
            </button>
            <button class={styles.action} disabled title={current().is_autobalance_on ? `${current().autobalance_min_players}+` : undefined}>
              {vt("autobalance")} <BalanceGlyph />
            </button>
          </div>
        </div>
      )}
    </Show>
  );
}

function BalanceGlyph() {
  return (
    <svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.6" aria-hidden="true">
      <path d="M10 2 V18 M5 18 H15 M3 6 H17 M3 6 L1 12 H5 Z M17 6 L15 12 H19 Z" />
    </svg>
  );
}
