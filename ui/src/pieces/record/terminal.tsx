// Terminal: `career --history` and `career --chart` printed in a phosphor pane. An ASCII table
// of the latest ranked games, a block sparkline of RR and a diverging bar chart per game, then
// a prompt with a blinking cursor. Lines arrive one after another like a slow tty.
import { For, createEffect, createMemo, on, onCleanup } from "solid-js";
import { isMotionReduced } from "~/studio/spring";
import type { RrUpdate } from "~/bindings/RrUpdate";
import { ladder, relativeDate } from "../career";
import { identity, nextRankName, outcome, signed, type RecordProps } from "./shared";
import styles from "./terminal.module.css";

type Kind = "prompt" | "head" | "rule" | "win" | "loss" | "dim" | "text" | "blank";

interface Line {
  kind: Kind;
  text: string;
  /** For prompts: the command typed after the prompt. */
  command?: string;
}

const PROMPT = "vl@valorant:~$ ";
/** Delay between printed lines, in ms (matches the stylesheet's stagger). */
const LINE_MS = 34;

function cell(text: string, width: number, align: "left" | "right" = "left"): string {
  const clipped = text.length > width ? `${text.slice(0, width - 1)}~` : text;
  return align === "left" ? clipped.padEnd(width) : clipped.padStart(width);
}

/** A small ASCII column chart of the ladder points, oldest left, `rows` lines tall. */
function trend(points: number[], rows = 4): Line[] {
  if (points.length === 0) return [];
  const min = Math.min(...points);
  const span = Math.max(...points) - min || 1;
  const heights = points.map((point) => 1 + Math.round(((point - min) / span) * (rows - 1)));
  return Array.from({ length: rows }, (_, index): Line => {
    const level = rows - index;
    const label = index === 0 ? "  rr trend " : index === rows - 1 ? "  (old>new)" : "           ";
    const columns = heights.map((height) => (height > level ? "## " : height === level ? "__ " : "   ")).join("");
    return { kind: "text", text: `${label} ${level === 1 ? "|" : ":"}${columns}` };
  });
}

function table(updates: readonly RrUpdate[]): Line[] {
  const widths = [3, 10, 11, 7, 5, 14];
  const rule = `+${widths.map((width) => "-".repeat(width + 2)).join("+")}+`;
  const row = (cells: string[], aligns: ("left" | "right")[]) =>
    `| ${cells.map((text, index) => cell(text, widths[index] ?? 4, aligns[index] ?? "left")).join(" | ")} |`;
  const aligns: ("left" | "right")[] = ["right", "left", "left", "left", "right", "left"];
  return [
    { kind: "rule", text: rule },
    { kind: "head", text: row(["#", "MAP", "PLAYED", "RESULT", "RR", "RANK AFTER"], aligns) },
    { kind: "rule", text: rule },
    ...updates.map(
      (update, index): Line => ({
        kind: outcome(update) === "loss" ? "loss" : "win",
        text: row(
          [
            `${index + 1}`,
            update.map_name,
            relativeDate(update.started_at_ms).toLowerCase(),
            { win: "WIN", loss: "LOSS", draw: "DRAW" }[outcome(update)],
            signed(update.rr_earned),
            `${update.rank_after.name}${update.is_promotion ? " ^" : update.is_demotion ? " v" : ""}`,
          ],
          aligns,
        ),
      }),
    ),
    { kind: "rule", text: rule },
  ];
}

/** One bar per game, oldest first, losses to the left of the axis and wins to the right. */
function bars(updates: readonly RrUpdate[]): Line[] {
  const width = 18;
  const peak = Math.max(1, ...updates.map((update) => Math.abs(update.rr_earned)));
  return [...updates].reverse().map((update): Line => {
    const length = Math.max(1, Math.round((Math.abs(update.rr_earned) / peak) * width));
    const left = update.rr_earned < 0 ? "#".repeat(length).padStart(width) : " ".repeat(width);
    const right = update.rr_earned > 0 ? "#".repeat(length) : "";
    return {
      kind: update.rr_earned < 0 ? "loss" : "win",
      text: `  ${cell(update.map_name, 8)} ${cell(signed(update.rr_earned), 4, "right")} ${left}|${right}`,
    };
  });
}

export function TerminalRecord(props: RecordProps) {
  const lines = createMemo((): Line[] => {
    const view = props.view;
    const stats = props.stats;
    const me = identity();
    const filled = Math.round((Math.min(100, view.rr) / 100) * 24);
    const streak = stats.streak ? `${stats.streak.isWin ? "W" : "L"}${stats.streak.length}` : "-";
    return [
      { kind: "prompt", text: PROMPT, command: "career --history" },
      { kind: "text", text: `  player   ${me.name}${me.tag ? `#${me.tag}` : ""}${me.level !== null ? `  (lvl ${me.level})` : ""}` },
      view.placements_left > 0
        ? { kind: "text", text: `  rank     placements: ${view.placements_left} games left` }
        : {
            kind: "text",
            text: `  rank     ${cell(view.rank.name, 12)} [${"#".repeat(filled)}${".".repeat(24 - filled)}] ${view.rr}/100 RR`,
          },
      { kind: "dim", text: `  next     ${nextRankName(view.rank.tier)} in ${Math.max(0, 100 - view.rr)} RR${view.peak ? `   peak ${view.peak.name}` : ""}` },
      {
        kind: "text",
        text: `  record   ${view.wins}W ${stats.losses}L  ${stats.winRate}% win   last ${stats.recent}: ${signed(stats.net)} RR   streak ${streak}`,
      },
      { kind: "blank", text: "" },
      ...table(view.updates),
      { kind: "dim", text: `  ${view.updates.length} rows   ^ promoted   v demoted` },
      { kind: "blank", text: "" },
      { kind: "prompt", text: PROMPT, command: "career --chart" },
      ...trend(ladder(view.updates)),
      { kind: "blank", text: "" },
      ...bars(view.updates),
    ];
  });
  // Like a real tty, the pane follows the output: once it overflows, it scrolls with each new
  // line so the prompt and cursor end up in view.
  let screen: HTMLDivElement | undefined;
  let frame = 0;
  createEffect(
    on(lines, (printed) => {
      cancelAnimationFrame(frame);
      const start = performance.now();
      const follow = (now: number) => {
        if (!screen) return;
        const shown = isMotionReduced() ? printed.length : Math.floor((now - start) / LINE_MS);
        const last = screen.children[Math.min(shown, screen.children.length - 1)] as HTMLElement | undefined;
        if (last) screen.scrollTop = Math.max(0, last.offsetTop + last.offsetHeight + 8 - screen.clientHeight);
        if (shown < printed.length) frame = requestAnimationFrame(follow);
      };
      frame = requestAnimationFrame(follow);
    }),
  );
  onCleanup(() => cancelAnimationFrame(frame));
  return (
    <section class={styles.term}>
      <header class={styles.bar}>
        <span class={styles.dots}>
          <i />
          <i />
          <i />
        </span>
        <span>career: ~/valorant</span>
        <span class={styles.barRight}>{props.view.rank.name.toLowerCase()} · {props.view.rr} rr</span>
      </header>
      <div class={styles.screen} ref={screen}>
        <For each={lines()}>
          {(line, index) => (
            <div class={styles.line} data-kind={line.kind} style={{ "--stagger": index() }}>
              {line.kind === "prompt" ? (
                <>
                  <span class={styles.prompt}>{line.text}</span>
                  <span class={styles.typed} style={{ "--chars": line.command?.length ?? 0 }}>
                    {line.command}
                  </span>
                </>
              ) : (
                line.text || " "
              )}
            </div>
          )}
        </For>
        <div class={styles.line} data-kind="prompt" style={{ "--stagger": lines().length }}>
          <span class={styles.prompt}>{PROMPT}</span>
          <span class={styles.cursor} />
        </div>
      </div>
      <footer class={styles.status}>
        <span>[vl] 0:career* 1:matches-</span>
        <span>{props.view.updates.length} games · {signed(props.stats.net)} rr</span>
      </footer>
    </section>
  );
}
