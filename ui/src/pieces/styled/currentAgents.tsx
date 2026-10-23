// VALORANT (current client) Agents page, laid out in the client's pixels on a ValCanvas: the
// breadcrumb, the owned toggle, role filters and the icon grid on a navy column; the agent's
// portrait over their name stacked huge behind them on their own tint; and the mastery
// diamond, name, INFO / ability tabs (captioned with your keybinds), bio and role text.
import { For, Index, Show, createMemo, createSignal } from "solid-js";
import type { AgentCard } from "~/bindings/AgentCard";
import type { PieceProps } from "~/studio/registry";
import { values as gameSettings } from "~/menu/valSettingsData";
import { playCue } from "~/menu/sound";
import { isAgentOwned, masteryOf, refreshAgentProgress } from "~/state/agents";
import { ROLE_ORDER, catalog } from "~/state/catalog";
import { Art } from "~/ui/Art";
import { colorVars } from "./current";
import { ValCanvas } from "./valCanvas";
import { valLang, vt } from "./valText";
import styles from "./currentAgents.module.css";

type Role = (typeof ROLE_ORDER)[number];

const ROLE_TEXT: Record<Role, { en: [string, string]; es: [string, string] }> = {
  Duelist: {
    en: ["Duelist", "Duelists are self-sufficient fraggers who their team expects, through abilities and skills, to get high frags and seek out engagements first."],
    es: ["Duelista", "Los duelistas son autosuficientes y se espera que, mediante sus habilidades, consigan muchas eliminaciones y busquen el combate primero."],
  },
  Initiator: {
    en: ["Initiator", "Initiators challenge angles by setting up their team to enter contested ground and push defenders away."],
    es: ["Iniciador", "Los iniciadores ponen a prueba los ángulos preparando a su equipo para entrar en terreno disputado y apartar a los defensores."],
  },
  Controller: {
    en: ["Controller", "Controllers are experts in slicing up dangerous territory to set their team up for success."],
    es: ["Controlador", "Los controladores se especializan en dividir terrenos peligrosos en varias zonas para que su equipo pueda encargarse del resto."],
  },
  Sentinel: {
    en: ["Sentinel", "Sentinels are defensive experts who can lock down areas and watch flanks, both on attacker and defender rounds."],
    es: ["Centinela", "Los centinelas son expertos defensivos que pueden bloquear zonas y vigilar los flancos, tanto en ataque como en defensa."],
  },
};

/** The client's tab order, and which keybind captions each tab. */
const SLOTS: readonly [slot: string, bind: string][] = [
  ["Grenade", "ability1"],
  ["Ability1", "ability2"],
  ["Ability2", "signature"],
  ["Ultimate", "ultimate"],
];

const GHOST_LINES = 4;

function roleOf(agent: AgentCard): Role | null {
  return (ROLE_ORDER as readonly string[]).includes(agent.role) ? (agent.role as Role) : null;
}

/** The page tint from the agent's colours: dark at the top left, saturated bottom right. */
function tint(agent: AgentCard | undefined): Record<string, string> {
  const colors = agent?.colors ?? [];
  const lightness = (hex: string) => [1, 3, 5].reduce((sum, at) => sum + parseInt(hex.slice(at, at + 2), 16), 0);
  const sorted = [...colors].sort((a, b) => lightness(a) - lightness(b));
  const mid = sorted[Math.floor(sorted.length / 2)] ?? "#3a2a6a";
  return {
    "--tint-top": `color-mix(in srgb, ${mid} 32%, #0f1923)`,
    "--tint-bottom": `color-mix(in srgb, ${mid} 62%, #10061e)`,
  };
}

export function ValAgents(props: PieceProps) {
  const [role, setRole] = createSignal<Role | "all">("all");
  const [pickedId, setPickedId] = createSignal("");
  const [tab, setTab] = createSignal("info");
  const [isOwnedOnly, setOwnedOnly] = createSignal(false);
  const agents = createMemo(() => [...catalog().agents].sort((a, b) => a.display_name.localeCompare(b.display_name)));
  const shown = () =>
    agents().filter((agent) => (role() === "all" || agent.role === role()) && (!isOwnedOnly() || isAgentOwned(agent.uuid)));
  const picked = () => agents().find((agent) => agent.uuid === pickedId()) ?? agents()[0];
  const roleText = () => {
    const current = picked();
    const key = current ? roleOf(current) : null;
    return key ? ROLE_TEXT[key][valLang()] : null;
  };
  const ability = () => picked()?.abilities.find((entry) => entry.slot === tab());
  const pick = (agent: AgentCard) => {
    playCue("tab");
    setPickedId(agent.uuid);
    setTab("info");
  };
  refreshAgentProgress();
  return (
    <ValCanvas style={{ ...tint(picked()), ...colorVars(props, ["accent"]) }}>
      <div class={styles.page}>
        <Show when={picked()}>
          {(agent) => (
            <>
              <span class={styles.ghost} aria-hidden="true">
                <Index each={Array.from({ length: GHOST_LINES })}>{() => <span>{agent().display_name}</span>}</Index>
              </span>
              <Art src={agent().portrait} class={styles.portrait} fallback={<span />} />
            </>
          )}
        </Show>
        <div class={styles.column} />
        <span class={styles.crumb}>
          <i />
          {vt("agents")}
        </span>
        <button
          class={styles.owned}
          role="switch"
          aria-checked={isOwnedOnly()}
          title="Show only the agents you have unlocked"
          onClick={() => {
            playCue(isOwnedOnly() ? "back" : "confirm");
            setOwnedOnly(!isOwnedOnly());
          }}
        >
          <span>{vt("owned")}</span>
          <span class={styles.ownedToggle} data-on={isOwnedOnly() ? "" : undefined}>
            <i />
          </span>
        </button>
        <div class={styles.filters}>
          <button class={styles.filter} data-active={role() === "all" ? "" : undefined} onClick={() => setRole("all")}>
            <span class={styles.allDots} />
          </button>
          <For each={ROLE_ORDER}>
            {(entry) => {
              const icon = () => agents().find((agent) => agent.role === entry)?.role_icon ?? null;
              return (
                <button class={styles.filter} title={ROLE_TEXT[entry][valLang()][0]} data-active={role() === entry ? "" : undefined} onClick={() => setRole(entry)}>
                  <Art src={icon()} class={styles.filterIcon} fallback={<b>{entry[0]}</b>} />
                </button>
              );
            }}
          </For>
        </div>
        <div class={styles.grid}>
          <For each={shown()}>
            {(agent) => (
              <button
                class={styles.cell}
                data-active={picked()?.uuid === agent.uuid ? "" : undefined}
                data-unowned={isAgentOwned(agent.uuid) ? undefined : ""}
                title={isAgentOwned(agent.uuid) ? agent.display_name : `${agent.display_name} (not unlocked)`}
                onMouseEnter={() => playCue("hover")}
                onClick={() => pick(agent)}
              >
                <Art src={agent.icon} class={styles.cellIcon} fallback={<span>{agent.display_name}</span>} />
              </button>
            )}
          </For>
        </div>
        <Show when={picked()}>{(agent) => <AgentInfo agent={agent()} tab={tab()} setTab={setTab} roleText={roleText()} ability={ability()} />}</Show>
      </div>
    </ValCanvas>
  );
}

function AgentInfo(props: {
  agent: AgentCard;
  tab: string;
  setTab: (tab: string) => void;
  roleText: [string, string] | null;
  ability: AgentCard["abilities"][number] | undefined;
}) {
  const mastery = () => masteryOf(props.agent.uuid);
  const slots = () => SLOTS.flatMap(([slot, bind]) => {
    const entry = props.agent.abilities.find((ability) => ability.slot === slot);
    return entry ? [{ entry, key: String(gameSettings()[bind] ?? "") }] : [];
  });
  const choose = (tab: string) => {
    playCue("tab");
    props.setTab(tab);
  };
  return (
    <>
      <i class={styles.rail} aria-hidden="true" />
      <i class={styles.rule} aria-hidden="true" />
      <span class={styles.mastery} title={`Mastery level ${mastery().lifetime} (act level ${mastery().act})`}>
        <svg viewBox="0 0 70 68" aria-hidden="true">
          <path d="M35 3 L44 12 M35 3 L26 12 M35 65 L44 56 M35 65 L26 56 M3 34 L12 25 M3 34 L12 43 M67 34 L58 25 M67 34 L58 43" />
          <path d="M35 13 L56 34 L35 55 L14 34 Z" />
        </svg>
        <b>{mastery().lifetime}</b>
      </span>
      <h2 class={styles.name}>{props.agent.display_name}</h2>
      <nav class={styles.tabs}>
        <button class={styles.tab} data-active={props.tab === "info" ? "" : undefined} onMouseEnter={() => playCue("hover")} onClick={() => choose("info")}>
          <Art src={props.agent.role_icon} class={styles.tabIcon} fallback={<span />} />
          <b>INFO</b>
        </button>
        <For each={slots()}>
          {(slot) => (
            <button class={styles.tab} data-active={props.tab === slot.entry.slot ? "" : undefined} title={slot.entry.name} onMouseEnter={() => playCue("hover")} onClick={() => choose(slot.entry.slot)}>
              <Art src={slot.entry.icon} class={styles.tabIcon} fallback={<span />} />
              <b>{slot.key}</b>
            </button>
          )}
        </For>
      </nav>
      <i class={styles.tabRule} aria-hidden="true" />
      <div class={styles.text}>
        <Show
          when={props.tab !== "info" && props.ability}
          fallback={
            <>
              <Show when={props.agent.description}>
                <p class={styles.bio}>{props.agent.description}</p>
              </Show>
              <Show when={props.roleText}>
                {(text) => (
                  <>
                    <h3>{text()[0].toUpperCase()}</h3>
                    <p>{text()[1]}</p>
                  </>
                )}
              </Show>
            </>
          }
        >
          {(entry) => (
            <>
              <h3>{entry().name.toUpperCase()}</h3>
              <p>{entry().description}</p>
            </>
          )}
        </Show>
      </div>
    </>
  );
}
