// Minecraft "Statistics": the title over three tab buttons (General / Maps / Ranks, after the
// game's General / Items / Mobs), a grey bevelled panel of pixel rows with item icons, the green
// experience bar for RR, and Done.
import { For, Match, Show, Switch, createSignal } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { playCue } from "~/menu/sound";
import { rankColor } from "../home";
import { division, nextRankName, signed, useCountUp, type RecordProps } from "./shared";
import buttons from "../styled/minecraft.module.css";
import styles from "./minecraft.module.css";

type Tab = "general" | "maps" | "ranks";

const TABS: readonly { id: Tab; label: string }[] = [
  { id: "general", label: "General" },
  { id: "maps", label: "Maps" },
  { id: "ranks", label: "Ranks" },
];

/** A 16x16 pixel icon from rows of palette letters ("." = transparent). */
function Pixels(props: { rows: readonly string[]; palette: Record<string, string> }) {
  const cells = () =>
    props.rows.flatMap((row, y) =>
      [...row].flatMap((key, x) => {
        const fill = props.palette[key];
        return fill ? [{ x, y, fill }] : [];
      }),
    );
  return (
    <svg class={styles.icon} viewBox="0 0 16 16" shape-rendering="crispEdges" aria-hidden="true">
      <For each={cells()}>{(cell) => <rect x={cell.x} y={cell.y} width="1" height="1" fill={cell.fill} />}</For>
    </svg>
  );
}

// The map item: parchment with a brown frame and a bit of terrain.
const MAP_ICON = [
  "................",
  ".bbbbbbbbbbbbbb.",
  ".bppppppppppppb.",
  ".bpgggppppwwppb.",
  ".bpggggpppwwwpb.",
  ".bppgggpppwwwpb.",
  ".bpppgpppppwwpb.",
  ".bpppppprrppppb.",
  ".bppppprrrrpppb.",
  ".bpgpppprrpppgb.",
  ".bpggppppppggpb.",
  ".bpgggpwwwpgggb.",
  ".bppggpwwwppggb.",
  ".bppppppppppppb.",
  ".bbbbbbbbbbbbbb.",
  "................",
];
const MAP_PALETTE = { b: "#6b4a2b", p: "#e9dcae", g: "#7fae4a", w: "#4f86c6", r: "#b23a2a" };

// A cut gem, tinted with the rank's colour.
const GEM_ICON = [
  "................",
  "................",
  "....kkkkkkkk....",
  "...khhllllmmk...",
  "..khhlllllmmmk..",
  ".kkkkkkkkkkkkkk.",
  ".khhlllllllmmmk.",
  "..khllllllmmmk..",
  "...khlllllmmk...",
  "....khlllmmk....",
  ".....khllmk.....",
  "......khmk......",
  ".......kk.......",
  "................",
  "................",
  "................",
];

// Sword for games, gold ingot for wins, bone for losses, clock for streak, emerald for RR.
const SWORD_ICON = [
  "...........kkk..",
  "..........kwwk..",
  ".........kwwk...",
  "........kwwk....",
  ".......kwwk.....",
  "......kwwk......",
  "..kk.kwwk.......",
  "..kwkwwk........",
  "...kwwk.........",
  "...kbwk.........",
  "..kbkkwk........",
  ".kbk..kk........",
  "kbk.............",
  "kk..............",
  "................",
  "................",
];
const SWORD_PALETTE = { k: "#1d1d1d", w: "#e6e6e6", b: "#6b4a2b" };

const INGOT_ICON = [
  "................",
  "................",
  "................",
  "................",
  ".....kkkkkkkk...",
  "....kwyyyyyyok..",
  "...kwyyyyyyook..",
  "..kwyyyyyyoook..",
  ".kkkkkkkkkkook..",
  ".kyyyyyyyyyook..",
  ".kyyyyyyyyyok...",
  ".kooooooooook...",
  ".kkkkkkkkkkk....",
  "................",
  "................",
  "................",
];
const INGOT_PALETTE = { k: "#3a2a00", w: "#fffbd0", y: "#f5d742", o: "#c99a1b" };

const BONE_ICON = [
  "................",
  "...........ww...",
  "..........wwww..",
  "..........wwwg..",
  ".........wwgg...",
  "........wwg.....",
  ".......wwg......",
  "......wwg.......",
  ".....wwg........",
  "....wwg.........",
  "...wwg..........",
  ".wwwg...........",
  "wwwg............",
  ".wg.............",
  "................",
  "................",
];
const BONE_PALETTE = { w: "#f1efe2", g: "#b8b49c" };

const EMERALD_ICON = [
  "................",
  "......kkkk......",
  ".....kwllgk.....",
  "....kwlllggk....",
  "...kwllllgggk...",
  "...kwllllgggk...",
  "...klllllgggk...",
  "...klllllgggk...",
  "...kllllggggk...",
  "....klllgggk....",
  ".....kllggk.....",
  "......kkkk......",
  "................",
  "................",
  "................",
  "................",
];
const EMERALD_PALETTE = { k: "#0b3b1d", w: "#b8ffcf", l: "#41d36b", g: "#1c9a45" };

function gemPalette(color: string): Record<string, string> {
  return {
    k: "#1d1d1d",
    h: `color-mix(in srgb, ${color} 35%, #ffffff)`,
    l: color,
    m: `color-mix(in srgb, ${color} 60%, #000000)`,
  };
}

export function MinecraftRecord(props: RecordProps) {
  const navigate = useNavigate();
  const [tab, setTab] = createSignal<Tab>("general");
  const rr = useCountUp(() => props.view.rr, 200, 900);
  const color = () => rankColor(props.view.rank.color);
  const general = () => [
    { icon: SWORD_ICON, palette: SWORD_PALETTE, label: "Ranked Games Played", value: `${props.view.games}` },
    { icon: INGOT_ICON, palette: INGOT_PALETTE, label: "Games Won", value: `${props.view.wins}` },
    { icon: BONE_ICON, palette: BONE_PALETTE, label: "Games Lost", value: `${props.stats.losses}` },
    { icon: INGOT_ICON, palette: INGOT_PALETTE, label: "Win Ratio", value: `${props.stats.winRate}%` },
    { icon: EMERALD_ICON, palette: EMERALD_PALETTE, label: `RR From Last ${props.stats.recent} Games`, value: signed(props.stats.net) },
    { icon: EMERALD_ICON, palette: EMERALD_PALETTE, label: "Best Game", value: signed(props.stats.best) },
    {
      icon: SWORD_ICON,
      palette: SWORD_PALETTE,
      label: props.stats.streak?.isWin === false ? "Losing Streak" : "Win Streak",
      value: `${props.stats.streak?.length ?? 0}`,
    },
    { icon: GEM_ICON, palette: gemPalette(color()), label: "Act Peak", value: props.view.peak?.name ?? "None" },
  ];
  return (
    <section class={styles.screen}>
      <h2 class={styles.title}>Statistics</h2>
      <div class={styles.tabs}>
        <For each={TABS}>
          {(option) => (
            <button
              class={`${buttons.button} ${styles.tabButton}`}
              disabled={tab() === option.id}
              onMouseEnter={() => playCue("hover")}
              onClick={() => {
                playCue("confirm");
                setTab(option.id);
              }}
            >
              <span>{option.label}</span>
            </button>
          )}
        </For>
      </div>

      <div class={styles.panel}>
        <div class={styles.rankLine}>
          <Pixels rows={GEM_ICON} palette={gemPalette(color())} />
          <span class={styles.rankName}>{props.view.rank.name}</span>
          <span class={styles.rankNext}>
            <Show when={props.view.placements_left === 0} fallback={`${props.view.placements_left} placements left`}>
              {Math.max(0, 100 - props.view.rr)} RR to {nextRankName(props.view.rank.tier)}
            </Show>
          </span>
        </div>
        <div class={styles.xp}>
          <span class={styles.level}>{rr()}</span>
          <div class={styles.xpBar}>
            <span class={styles.xpFill} style={{ width: `${Math.min(100, props.view.rr)}%` }} />
          </div>
        </div>

        <div class={styles.list}>
          <Switch>
            <Match when={tab() === "general"}>
              <For each={general()}>
                {(row, index) => (
                  <div class={styles.row} style={{ "--stagger": index() }}>
                    <Pixels rows={row.icon} palette={row.palette} />
                    <span class={styles.label}>{row.label}</span>
                    <span class={styles.value}>{row.value}</span>
                  </div>
                )}
              </For>
            </Match>
            <Match when={tab() === "maps"}>
              <div class={`${styles.row} ${styles.head} ${styles.mapRow}`}>
                <span />
                <span>Map</span>
                <span>Played</span>
                <span>Won</span>
                <span>Lost</span>
                <span>RR</span>
              </div>
              <For each={props.stats.maps}>
                {(map, index) => (
                  <div class={`${styles.row} ${styles.mapRow}`} style={{ "--stagger": index() }}>
                    <span class={styles.slot}>
                      <Pixels rows={MAP_ICON} palette={MAP_PALETTE} />
                    </span>
                    <span class={styles.label}>{map.name}</span>
                    <span>{map.games}</span>
                    <span>{map.wins}</span>
                    <span>{map.games - map.wins}</span>
                    <span data-sign={map.net < 0 ? "neg" : "pos"}>
                      {signed(map.net)}
                    </span>
                  </div>
                )}
              </For>
            </Match>
            <Match when={tab() === "ranks"}>
              <For each={props.stats.ranks}>
                {(line, index) => (
                  <div class={`${styles.row} ${styles.mobRow}`} style={{ "--stagger": index() }}>
                    <span class={styles.slot}>
                      <Pixels rows={GEM_ICON} palette={gemPalette(rankColor(line.rank.color))} />
                    </span>
                    <span class={styles.mobText}>
                      <span class={styles.label}>{line.rank.name}</span>
                      <span class={styles.mobLine}>
                        You won {line.wins} {line.wins === 1 ? "game" : "games"} at {line.rank.name}
                      </span>
                      <span class={styles.mobLine}>
                        {line.rank.name} beat you {line.losses} {line.losses === 1 ? "time" : "times"}
                      </span>
                    </span>
                    <span class={styles.tierPips}>
                      <For each={[1, 2, 3]}>
                        {(pip) => <i data-on={pip <= Math.max(1, division(line.rank.tier)) ? "" : undefined} />}
                      </For>
                    </span>
                  </div>
                )}
              </For>
              <Show when={props.stats.ranks.length === 0}>
                <div class={styles.row}>
                  <span class={styles.label}>No ranked games yet</span>
                </div>
              </Show>
            </Match>
          </Switch>
        </div>
      </div>

      <div class={styles.done}>
        <button class={buttons.button} disabled={props.isEditing} onClick={() => navigate("/home")}>
          <span>Done</span>
        </button>
      </div>
    </section>
  );
}
