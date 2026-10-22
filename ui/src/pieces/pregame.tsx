// Agent-select pieces: header (map, mode, timer), ally roster, and the agent picker with the
// LOCK IN button.
import { For, Index, Show, createMemo, createSignal, onCleanup } from "solid-js";
import { DoorOpen } from "lucide-solid";
import type { PieceProps } from "~/studio/registry";
import { playCue } from "~/menu/sound";
import { ROLE_ORDER, agentByUuid, catalog } from "~/state/catalog";
import { pregame, pregameActions, timeLeft } from "~/state/pregame";
import { Empty, controls } from "~/ui/controls";
import { rankColor } from "./home";
import styles from "./pregame.module.css";
import { Art } from "~/ui/Art";

function useTick(intervalMs: number): () => number {
  const [now, setNow] = createSignal(Date.now());
  const timer = window.setInterval(() => setNow(Date.now()), intervalMs);
  onCleanup(() => window.clearInterval(timer));
  return now;
}

export function PregameHeader(_props: PieceProps) {
  const now = useTick(200);
  return (
    <Show when={pregame()} fallback={<Empty title="Agent select">Appears here when a match is found.</Empty>}>
      {(view) => {
        const seconds = () => timeLeft(view(), now());
        return (
          <div class={styles.header} data-part="pregame-header">
            <span class={styles.headerMode}>
              {view().queue_label}
              <Show when={view().team_id}>
                <span class={styles.side}>{view().team_id === "Blue" ? "Attack first" : "Defend first"}</span>
              </Show>
            </span>
            <span class={styles.headerMap}>{view().map_name}</span>
            <span class={`${styles.timer} num`} data-low={seconds() < 10 ? "" : undefined}>
              {Math.ceil(seconds())}
            </span>
            <span class={styles.enemy}>
              Enemy locked <b class="num">{view().enemy_lock_count}</b>/<span class="num">{view().enemy_team_size || 5}</span>
            </span>
          </div>
        );
      }}
    </Show>
  );
}

export function AllyRoster(_props: PieceProps) {
  return (
    <Show when={pregame()} fallback={<Empty title="Your team" />}>
      {(view) => (
        <ol class={styles.roster}>
          <Index each={view().allies}>
            {(ally, index) => (
              <li
                class={styles.ally}
                data-part="ally"
                style={{ "--stagger": index }}
                data-self={ally().is_self ? "" : undefined}
                data-state={ally().selection || "none"}
              >
                <span class={styles.allyIcon}>
                  <Art src={ally().agent_icon} fallback={<span class={styles.allyEmpty}>{ally().agent_name?.slice(0, 2) ?? "?"}</span>} />
                </span>
                <span class={styles.allyText}>
                  <span class={styles.allyAgent}>
                    {ally().agent_name ?? "Picking"}
                    <Show when={ally().selection === "locked"}>
                      <span class={styles.lockedTag}>Locked</span>
                    </Show>
                  </span>
                  <span class={styles.allyName} data-sensitive>
                    {ally().game_name || (ally().is_self ? "You" : "Teammate")}
                  </span>
                </span>
                <span class={styles.allyRank} style={{ "--rank": rankColor(ally().rank.color) }}>
                  <Art src={ally().rank.icon} alt={ally().rank.name} fallback={ally().rank.tier > 0 ? ally().rank.name : ""} />
                </span>
              </li>
            )}
          </Index>
        </ol>
      )}
    </Show>
  );
}

export function AgentPicker(_props: PieceProps) {
  const [hovered, setHovered] = createSignal<string | null>(null);
  const isLocked = () => pregame()?.is_own_locked ?? false;
  const own = () => pregame()?.own_agent_uuid ?? null;
  const taken = createMemo(() => new Set(pregame()?.taken_agents ?? []));
  const focus = () => agentByUuid(hovered() ?? own());
  return (
    <Show when={pregame()} fallback={<Empty title="Agent picker">Your roster appears in agent select.</Empty>}>
      <div class={styles.picker} data-part="screen-board">
        <div class={styles.spotlight} data-part="spotlight" style={{ "--portrait": focus()?.portrait ? `url("${focus()?.portrait}")` : "none" }}>
          <span class={styles.spotlightRole}>{focus()?.role ?? ""}</span>
          <span class={styles.spotlightName}>{focus()?.display_name ?? "Choose your agent"}</span>
        </div>
        <div class={styles.grid}>
          <For each={[...ROLE_ORDER, ""]}>
            {(role) => (
              <For each={catalog().agents.filter((agent) => (role ? agent.role === role : !(ROLE_ORDER as readonly string[]).includes(agent.role)))}>
                {(agent) => (
                  <button
                    class={styles.agent}
                    data-part="agent-tile"
                    data-role={role}
                    data-own={own() === agent.uuid ? "" : undefined}
                    disabled={isLocked() || taken().has(agent.uuid)}
                    title={agent.display_name}
                    onMouseEnter={() => {
                      setHovered(agent.uuid);
                      playCue("hover");
                    }}
                    onMouseLeave={() => setHovered(null)}
                    onClick={() => void pregameActions.select(agent.uuid)}
                  >
                    <Art src={agent.icon} fallback={<span>{agent.display_name.slice(0, 2)}</span>} />
                  </button>
                )}
              </For>
            )}
          </For>
        </div>
        <div class={styles.actions}>
          <button
            class={`${controls.button} ${controls.ghost} ${controls.small}`}
            disabled={isLocked()}
            title="Leave this match (dodge penalty applies)"
            onClick={() => void pregameActions.dodge()}
          >
            <DoorOpen size={14} /> Dodge
          </button>
          <div class={styles.lockGlow}>
            <button
              class={styles.lock}
              data-part="lock"
              data-locked={isLocked() ? "" : undefined}
              disabled={isLocked() || !own()}
              onMouseEnter={() => playCue("hover")}
              onClick={() => {
                const uuid = own();
                if (uuid) void pregameActions.lock(uuid);
              }}
            >
              {isLocked() ? "Locked in" : "Lock in"}
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
}
