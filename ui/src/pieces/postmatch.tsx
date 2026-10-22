// Post-match summary: the result over the map's splash art, the player's own numbers as
// tiles, the RR change of a ranked match and everyone's scoreboard line. Incognito players
// stay unnamed (the backend never sends their names).
import { For, Show, type JSX } from "solid-js";
import { useNavigate } from "@solidjs/router";
import type { MatchResult } from "~/bindings/MatchResult";
import type { PostMatchPlayer } from "~/bindings/PostMatchPlayer";
import type { PostMatchView } from "~/bindings/PostMatchView";
import type { PieceProps } from "~/studio/registry";
import { doc } from "~/studio/store";
import type { Skin } from "~/studio/types";
import { dismissPostMatch, postMatch } from "~/state/postMatch";
import { Art } from "~/ui/Art";
import controls from "~/ui/controls.module.css";
import { Empty } from "~/ui/controls";
import { rankColor } from "./home";
import styles from "./postmatch.module.css";

const RESULT_LABEL: Record<MatchResult, string> = { victory: "Victory", defeat: "Defeat", draw: "Draw" };

/** What each style's own game would shout at the end of a match. */
const STYLE_RESULT: Partial<Record<Skin, Record<MatchResult, string>>> = {
  cs16: { victory: "Counter-Terrorists Win!", defeat: "Terrorists Win!", draw: "Round Draw!" },
  csgo: { victory: "Match won", defeat: "Match lost", draw: "Match tied" },
  cs2: { victory: "You won", defeat: "You lost", draw: "Tie" },
  fortnite: { victory: "#1 Victory Royale", defeat: "Eliminated", draw: "Top 2" },
  minecraft: { victory: "Victory!", defeat: "You Died!", draw: "Stalemate" },
  halo: { victory: "Victory", defeat: "Defeat", draw: "Tie game" },
  terminal: { victory: "exit 0  # victory", defeat: "exit 1  # defeat", draw: "exit 2  # draw" },
};

/** The heading over the scoreboard, in the style's vocabulary. */
const STYLE_CONTEXT: Partial<Record<Skin, string>> = {
  halo: "Post-game carnage report",
  minecraft: "Statistics",
  terminal: "$ match --summary",
  fortnite: "Match stats",
};

function signed(value: number): string {
  return value > 0 ? `+${value}` : String(value);
}

function Tile(props: { label: string; index: number; accent?: boolean; children: JSX.Element }) {
  return (
    <div class={styles.tile} data-part="stat-tile" data-accent={props.accent ? "" : undefined} style={{ "--stagger": props.index }}>
      <span class={styles.tileValue}>{props.children}</span>
      <span class={styles.tileLabel}>{props.label}</span>
    </div>
  );
}

function OwnPanel(props: { view: PostMatchView }) {
  const stats = () => props.view.stats;
  const kd = () => (stats().deaths === 0 ? stats().kills : stats().kills / stats().deaths).toFixed(2);
  return (
    <aside class={styles.own} data-part="own-stats">
      <div class={styles.agent}>
        <span class={styles.agentIcon}>
          <Art src={props.view.agent_icon} fallback={<span>{props.view.agent_name?.slice(0, 2) ?? "?"}</span>} />
        </span>
        <span class={styles.agentText}>
          <span class="overline">Played</span>
          <span class={styles.agentName}>{props.view.agent_name ?? "Unknown agent"}</span>
        </span>
      </div>
      <div class={styles.tiles}>
        <Tile label="K / D / A" index={0} accent>
          <span class="num">
            {stats().kills}/{stats().deaths}/{stats().assists}
          </span>
        </Tile>
        <Tile label="ACS" index={1}>
          <span class="num">{Math.round(stats().acs)}</span>
        </Tile>
        <Tile label="K/D" index={2}>
          <span class="num">{kd()}</span>
        </Tile>
        <Tile label="ADR" index={3}>
          <span class="num">{stats().adr === null ? "—" : Math.round(stats().adr ?? 0)}</span>
        </Tile>
        <Tile label="Headshot" index={4}>
          <span class="num">{stats().headshot_percent === null ? "—" : `${Math.round(stats().headshot_percent ?? 0)}%`}</span>
        </Tile>
        <Tile label="First bloods" index={5}>
          <span class="num">{stats().first_bloods}</span>
        </Tile>
        <Tile label="First deaths" index={6}>
          <span class="num">{stats().first_deaths}</span>
        </Tile>
      </div>
    </aside>
  );
}

function playerName(player: PostMatchPlayer): string {
  if (player.game_name) return player.game_name;
  if (player.is_incognito) return "Hidden";
  return player.agent_name ?? "Player";
}

function Row(props: { player: PostMatchPlayer; index: number }) {
  return (
    <li
      class={styles.row}
      data-part="scoreboard-row"
      data-self={props.player.is_self ? "" : undefined}
      data-side={props.player.is_ally ? "ally" : "enemy"}
      style={{ "--stagger": props.index }}
    >
      <span class={`${styles.place} num`}>{props.index + 1}</span>
      <span class={styles.rowAgent}>
        <Art src={props.player.agent_icon} fallback={<span>{props.player.agent_name?.slice(0, 2) ?? ""}</span>} />
      </span>
      <span class={styles.rowWho}>
        <span class={styles.rowName} data-sensitive data-hidden={props.player.game_name ? undefined : ""}>
          {playerName(props.player)}
          <Show when={props.player.tag_line}>
            <small>#{props.player.tag_line}</small>
          </Show>
        </span>
        <span class={styles.rowMeta}>{props.player.agent_name ?? ""}</span>
      </span>
      <span class={`${styles.cell} num`}>{props.player.kills}</span>
      <span class={`${styles.cell} num`}>{props.player.deaths}</span>
      <span class={`${styles.cell} num`}>{props.player.assists}</span>
      <span class={`${styles.cell} ${styles.acs} num`}>{Math.round(props.player.acs)}</span>
    </li>
  );
}

function Scoreboard(props: { players: PostMatchPlayer[] }) {
  return (
    <section class={styles.board} data-part="scoreboard">
      <div class={styles.columns}>
        <span>#</span>
        <span />
        <span>Player</span>
        <span>K</span>
        <span>D</span>
        <span>A</span>
        <span>ACS</span>
      </div>
      <ol class={styles.rows}>
        <For each={props.players}>{(player, index) => <Row player={player} index={index()} />}</For>
      </ol>
    </section>
  );
}

function Banner(props: { view: PostMatchView; onContinue: () => void }) {
  return (
    <header class={styles.banner} data-part="post-match-banner">
      <span class={styles.context}>
        <Show when={STYLE_CONTEXT[doc.theme.skin]}>{(heading) => <>{heading()} · </>}</Show>
        {props.view.queue_label} · {props.view.map_name}
      </span>
      <h1 class={styles.result}>{(STYLE_RESULT[doc.theme.skin] ?? RESULT_LABEL)[props.view.result]}</h1>
      <div class={`${styles.score} num`}>
        <b data-own>{props.view.rounds_won}</b>
        <span>:</span>
        <b>{props.view.rounds_lost}</b>
      </div>
      <Show when={props.view.rr_change !== null}>
        <span
          class={styles.rr}
          data-part="rr-chip"
          data-down={(props.view.rr_change ?? 0) < 0 ? "" : undefined}
          style={{ "--rank": rankColor(props.view.rank_after?.color ?? null) }}
        >
          <Show when={props.view.rank_after?.icon}>{(icon) => <img src={icon()} alt="" />}</Show>
          <b class="num">{signed(props.view.rr_change ?? 0)} RR</b>
          <Show when={props.view.rank_after}>{(rank) => <span>{rank().name}</span>}</Show>
        </span>
      </Show>
      <Show when={props.view.is_ranked && props.view.rr_change === null}>
        <span class={styles.rrPending}>RR update pending…</span>
      </Show>
      <button class={`${controls.button} ${styles.continue}`} data-part="play" onClick={() => props.onContinue()}>
        Continue
      </button>
    </header>
  );
}

export function PostMatchPiece(_props: PieceProps) {
  const navigate = useNavigate();
  const close = () => {
    dismissPostMatch();
    navigate("/home");
  };
  return (
    <Show when={postMatch()} fallback={<Empty title="Match summary">Your result, stats and scoreboard appear here after a match.</Empty>}>
      {(view) => (
        <section
          class={styles.root}
          data-part="post-match"
          data-result={view().result}
          style={{ "--splash": view().map_splash ? `url("${view().map_splash}")` : "none" }}
        >
          <div class={styles.splash} aria-hidden="true" />
          <Banner view={view()} onContinue={close} />
          <div class={styles.body}>
            <OwnPanel view={view()} />
            <Scoreboard players={view().players} />
          </div>
        </section>
      )}
    </Show>
  );
}
