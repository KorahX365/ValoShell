// VALORANT (current client) Career page, in the client's pixels on a ValCanvas: the sub-tabs,
// your card and title, the agents you played most, the match history (every mode, result
// tinted, MVP and placement called out) and the competitive rank with RR, plus Premier.
import { For, Show, createMemo, createSignal, onMount } from "solid-js";
import { useLocation } from "@solidjs/router";
import { ChevronDown, Copy, Download, Info } from "lucide-solid";
import type { MatchRow } from "~/bindings/MatchRow";
import type { PieceProps } from "~/studio/registry";
import { playCue } from "~/menu/sound";
import { agentByUpperName } from "~/state/catalog";
import { party } from "~/state/party";
import { ownRiotId } from "~/state/session";
import { pushToast } from "~/state/toasts";
import { careerView, historyView } from "~/state/views";
import { Art } from "~/ui/Art";
import { colorVars, textProp } from "./current";
import { ValProgress } from "./currentProgress";
import { ValCanvas } from "./valCanvas";
import { CompetitiveGlyph, DeathmatchGlyph, SpikeGlyph } from "./valIcons";
import { queueName, valLang } from "./valText";
import styles from "./currentCareer.module.css";

const PAGE_BACKGROUND = "radial-gradient(ellipse at 50% 30%, #1d2b3a, #172230 60%, #142030)";
const TABS_ES = ["Trayectoria", "Resumen de rango", "Legado de rango", "Calificación en el mapa", "Méritos"];
const TABS_EN = ["Career", "Rank overview", "Rank legacy", "Map rating", "Merits"];
const MASTERY_SHOWN = 3;

const say = (en: string, es: string) => (valLang() === "es" ? es : en);

/** The career screen shows the battlepass page when opened as its "pass" view. */
export function ValCareer(props: PieceProps) {
  const location = useLocation();
  const isPass = () => new URLSearchParams(location.search).get("view") === "pass";
  return (
    <Show when={!isPass()} fallback={<div class={styles.pass}><ValProgress {...props} /></div>}>
      <CareerPage {...props} />
    </Show>
  );
}

function CareerPage(props: PieceProps) {
  const [tab, setTab] = createSignal(0);
  const [mode, setMode] = createSignal("all");
  onMount(() => {
    if (!careerView.value() && !careerView.isLoading()) void careerView.load();
    if (!historyView.value() && !historyView.isLoading()) void historyView.load();
  });
  const self = () => party()?.members.find((member) => member.is_self);
  const rows = () => (historyView.value()?.rows ?? []).filter((row) => mode() === "all" || row.queue_id === mode());
  const modes = createMemo(() => [...new Set((historyView.value()?.rows ?? []).map((row) => row.queue_id))]);
  const tabs = () => (valLang() === "es" ? TABS_ES : TABS_EN);
  return (
    <ValCanvas style={{ ...colorVars(props, ["win", "loss", "gold"]), background: PAGE_BACKGROUND }}>
      <div class={styles.page}>
        <span class={styles.crumb}>
          <i />
          {tabs()[0]}
        </span>
        <nav class={styles.tabs}>
          <For each={tabs()}>
            {(label, index) => (
              <button
                class={styles.tab}
                data-active={tab() === index() ? "" : undefined}
                onMouseEnter={() => playCue("hover")}
                onClick={() => {
                  playCue("tab");
                  setTab(index());
                }}
              >
                {label}
              </button>
            )}
          </For>
        </nav>
        <header class={styles.identity}>
          <Art src={self()?.card_small} class={styles.identityCard} fallback={<span class={styles.identityCard} />} />
          <b data-sensitive>{self()?.game_name || ownRiotId()?.name || "—"}</b>
          <button
            class={styles.copy}
            title={say("Copy Riot ID", "Copiar Riot ID")}
            onClick={() => {
              const id = ownRiotId();
              if (!id) return;
              void navigator.clipboard?.writeText(`${id.name}#${id.tag}`).catch(() => undefined);
              pushToast("positive", say("Riot ID copied", "Riot ID copiado"), `${id.name}#${id.tag}`);
            }}
          >
            <Copy size={16} strokeWidth={2.4} />
          </button>
          <small>{self()?.title ?? ""}</small>
        </header>
        <h3 class={styles.heading} data-col="1">
          {say("Agent mastery", "Maestría de agente")}
        </h3>
        <h3 class={styles.heading} data-col="2">
          {say("Match history", "Historial de partidas")}
        </h3>
        <h3 class={styles.heading} data-col="3">
          {queueName("competitive", "Competitive")}
          <Info size={16} />
        </h3>
        <Mastery rows={historyView.value()?.rows ?? []} />
        <section class={styles.history}>
          <div class={styles.historyTools}>
            <label class={styles.modeFilter}>
              <select value={mode()} onChange={(event) => setMode(event.currentTarget.value)}>
                <option value="all">{say("All modes", "Todos los modos")}</option>
                <For each={modes()}>{(id) => <option value={id}>{queueName(id, id)}</option>}</For>
              </select>
              <ChevronDown size={16} />
            </label>
            <span class={styles.replays}>
              <i />
              {say("Show only replay downloads", "Mostrar solo descargas de repeticiones")}
            </span>
          </div>
          <div class={styles.rows}>
            <Show when={rows().length > 0} fallback={<p class={styles.empty}>{historyView.isLoading() ? say("Loading matches…", "Cargando partidas…") : (historyView.error() ?? say("No matches yet.", "Aún no hay partidas."))}</p>}>
              <For each={rows()}>{(row) => <HistoryRow row={row} />}</For>
            </Show>
          </div>
        </section>
        <RankCard />
        <h3 class={styles.heading} data-col="premier">
          {say("Premier portal", "Portal de Premier")}
        </h3>
        <section class={styles.premier}>
          <b>{textProp(props, "premierTeam", "premierNone")}</b>
          <small>{stringProp(props, "premierTag")}</small>
          <PremierMark />
        </section>
      </div>
    </ValCanvas>
  );
}

function stringProp(props: PieceProps, key: string): string {
  const value = props.piece.props[key];
  return typeof value === "string" ? value : "";
}

function HistoryRow(props: { row: MatchRow }) {
  const won = () => props.row.result === "victory";
  const rankIcon = () => (props.row.queue_id === "competitive" ? careerView.value()?.updates.find((update) => update.match_id === props.row.match_id)?.rank_after.icon : null);
  const verdict = () => {
    if (props.row.placement !== null) return say(`Place #${props.row.placement}`, `Puesto #${props.row.placement}`);
    if (props.row.result === "draw") return say("Draw", "Empate");
    return won() ? say("Victory", "Victoria") : say("Defeat", "Derrota");
  };
  const mvp = () => {
    if (props.row.mvp === "match") return say("Match MVP", "MVP de la partida");
    if (props.row.mvp === "team") return say("Team MVP", "MVP del equipo");
    return "";
  };
  return (
    <article class={styles.row} data-won={won() ? "" : undefined} onMouseEnter={() => playCue("hover")}>
      <Show when={props.row.map_splash}>{(splash) => <img class={styles.rowMap} src={splash()} alt="" draggable={false} />}</Show>
      <Art src={props.row.agent_icon} class={styles.rowAgent} fallback={<span class={styles.rowAgent} />} />
      <span class={styles.rowMode}>
        <Show
          when={rankIcon()}
          fallback={<ModeGlyph queue={props.row.queue_id} />}
        >
          {(icon) => <Art src={icon()} class={styles.rowRank} fallback={<CompetitiveGlyph size={36} />} />}
        </Show>
        <Show when={props.row.rr_change !== null}>
          <small>
            {(props.row.rr_change ?? 0) > 0 ? "+" : ""}
            {props.row.rr_change}
          </small>
        </Show>
      </span>
      <span class={styles.rowStats}>
        <small>{props.row.map_name}</small>
        <b>
          {props.row.kills} / {props.row.deaths} / {props.row.assists}
        </b>
        <Show when={props.row.score > 0}>
          <em>
            {say("Score", "Puntuación")} {props.row.score}
          </em>
        </Show>
      </span>
      <span class={styles.rowResult}>
        <b>{verdict()}</b>
        <Show when={props.row.placement === null}>
          <small>
            {props.row.rounds_won}-{props.row.rounds_lost}
          </small>
        </Show>
      </span>
      <Show when={mvp()}>
        <span class={styles.rowMvp}>{mvp()}</span>
      </Show>
      <button class={styles.rowDownload} title={say("Download replay", "Descargar repetición")} disabled={props.row.placement !== null}>
        <Download size={20} strokeWidth={2.2} />
      </button>
    </article>
  );
}

/** The mode mark the client shows when a match has no rank badge: the Spike for the round
    modes, the octagon for deathmatches. */
function ModeGlyph(props: { queue: string }) {
  if (props.queue === "competitive" || props.queue === "premier") return <CompetitiveGlyph size={36} />;
  if (props.queue === "deathmatch" || props.queue === "hurm") return <DeathmatchGlyph size={40} />;
  return <SpikeGlyph size={40} />;
}

/** The three agents you played most lately, as the client's mastery cards. */
function Mastery(props: { rows: readonly MatchRow[] }) {
  const top = createMemo(() => {
    const counts = new Map<string, number>();
    for (const row of props.rows) if (row.agent_name) counts.set(row.agent_name, (counts.get(row.agent_name) ?? 0) + 1);
    return [...counts.entries()].sort((a, b) => b[1] - a[1]).slice(0, MASTERY_SHOWN);
  });
  return (
    <div class={styles.mastery}>
      <For each={top()}>
        {([name, games]) => {
          const agent = () => agentByUpperName(name.toUpperCase());
          const tint = () => agent()?.colors[1] ?? "#3a4656";
          return (
            <article class={styles.masteryCard} style={{ "--tint": tint() }}>
              <span class={styles.masteryArt}>
                <Art src={agent()?.portrait} class={styles.masteryPortrait} fallback={<span />} />
              </span>
              <footer>
                <span class={styles.masteryLevel}>
                  <svg viewBox="0 0 50 50" aria-hidden="true">
                    <path d="M25 4 L31 10 M25 4 L19 10 M25 46 L31 40 M25 46 L19 40 M4 25 L10 19 M4 25 L10 31 M46 25 L40 19 M46 25 L40 31" />
                    <path d="M25 11 L39 25 L25 39 L11 25 Z" />
                  </svg>
                  <b>{games}</b>
                </span>
                <h4>{name}</h4>
              </footer>
            </article>
          );
        }}
      </For>
    </div>
  );
}

function RankCard() {
  const view = () => careerView.value();
  const isRanked = () => (view()?.rank.tier ?? 0) >= 3;
  return (
    <section class={styles.rank}>
      <i class={styles.rankFrame} aria-hidden="true" />
      <Show when={view()?.rank.icon}>{(icon) => <Art src={icon()} class={styles.rankIcon} fallback={<span />} />}</Show>
      <b class={styles.rankName}>{isRanked() ? view()?.rank.name : say("Unranked", "Sin clasificar")}</b>
      <Show when={isRanked()}>
        <span class={styles.rankBar}>
          <i style={{ "--rr": `${Math.min(100, view()?.rr ?? 0)}%` }} />
        </span>
        <small class={styles.rankRr}>{view()?.rr ?? 0}/100</small>
      </Show>
      <i class={styles.rankDash} aria-hidden="true" />
    </section>
  );
}

function PremierMark() {
  return (
    <svg class={styles.premierMark} viewBox="0 0 100 100" aria-hidden="true">
      <circle cx="50" cy="50" r="40" fill="#1b1d22" stroke="#e8e4dc" stroke-width="6" />
      <circle cx="50" cy="50" r="26" fill="none" stroke="#ff2b3a" stroke-width="7" />
      <path d="M50 6 V30 M50 70 V94 M6 50 H30 M70 50 H94" stroke="#ff2b3a" stroke-width="7" />
      <circle cx="50" cy="50" r="8" fill="#ff2b3a" />
    </svg>
  );
}
