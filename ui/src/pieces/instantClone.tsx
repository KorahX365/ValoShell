// InstantClone stream delay controls: what is on air, how full the buffer is, one button per
// preset delay and a cut back to live. Polls InstantClone on this machine once a second. The
// layout follows the box it is given unless one is picked in Studio.
import { For, Show, createSignal, onCleanup } from "solid-js";
import type { InstantCloneAction } from "~/bindings/InstantCloneAction";
import type { InstantCloneState } from "~/bindings/InstantCloneState";
import { call, errorText } from "~/ipc/commands";
import type { PieceProps } from "~/studio/registry";
import { useWidgetLayout, widgetChoice } from "./widgetLooks";
import looks from "./widgetLooks.module.css";
import styles from "./instantClone.module.css";

const POLL_MS = 1000;
const DEFAULT_PORT = 7799;
const DEFAULT_PRESETS = "15, 30, 60";
const MAX_PRESETS = 4;
const PHASE_LABELS: Record<string, string> = {
  idle: "Live",
  preparing: "Buffering",
  ready: "Armed",
  active: "Delayed",
};

/** "15, 30, 60" -> [15, 30, 60]; anything that is not a positive number is skipped. */
function presetSeconds(raw: unknown): number[] {
  const text = typeof raw === "string" ? raw : DEFAULT_PRESETS;
  return text
    .split(",")
    .map((part) => Number(part.trim()))
    .filter((seconds) => Number.isFinite(seconds) && seconds > 0)
    .slice(0, MAX_PRESETS);
}

/** The port field is free text; anything that is not a valid port falls back to InstantClone's. */
function port(raw: unknown): number {
  const value = Number(raw);
  return Number.isInteger(value) && value > 0 && value <= 65535 ? value : DEFAULT_PORT;
}

function seconds(ms: number): string {
  return `${Math.round(ms / 1000)}s`;
}

/** Buffer progress 0-1 while preparing; full otherwise once armed or on air. */
function fillRatio(state: InstantCloneState): number {
  if (state.phase === "preparing" && state.armed_delay_ms > 0) {
    return Math.min(1, state.buffer_fill_ms / state.armed_delay_ms);
  }
  return state.phase === "idle" ? 0 : 1;
}

export function InstantCloneWidget(props: PieceProps) {
  const target = () => ({
    port: port(props.piece.props.port),
    token: typeof props.piece.props.token === "string" ? props.piece.props.token : "",
  });
  const [state, setState] = createSignal<InstantCloneState | null>(null);
  const [error, setError] = createSignal("");
  const box = useWidgetLayout(props);

  const refresh = async () => setState(await call("instant_clone_state", { target: target() }).catch(() => null));
  void refresh();
  const timer = setInterval(() => void refresh(), POLL_MS);
  onCleanup(() => clearInterval(timer));

  const send = async (action: InstantCloneAction, delayMs = 0) => {
    setError("");
    try {
      await call("instant_clone_send", { target: target(), action, delayMs });
    } catch (failure) {
      setError(errorText(failure));
    }
    await refresh();
  };

  const phase = () => state()?.phase ?? "offline";
  const detail = (current: InstantCloneState) => {
    if (current.phase === "active") return seconds(current.current_delay_ms);
    if (current.phase === "preparing") return `${seconds(current.buffer_fill_ms)} / ${seconds(current.armed_delay_ms)}`;
    if (current.phase === "ready") return seconds(current.armed_delay_ms);
    return "No delay";
  };

  return (
    <div class={looks.frame} ref={box.ref}>
      <div
        class={`${looks.surface} ${styles.widget}`}
        data-phase={phase()}
        data-look={widgetChoice(props, "look")}
        data-layout={box.layout()}
        data-editing={props.isEditing ? "" : undefined}
      >
        <Show
          when={state()}
          fallback={
            <div class={styles.offline}>
              <span class={styles.dot} />
              <span class={styles.title}>Stream delay</span>
              <span class={styles.hint}>InstantClone is not running</span>
            </div>
          }
        >
          {(current) => (
            <>
              <div class={styles.status}>
                <span class={styles.dot} />
                <span class={styles.title}>{PHASE_LABELS[current().phase] ?? current().phase}</span>
                <span class={`${styles.delay} num`}>{detail(current())}</span>
              </div>
              <i class={styles.meter} style={{ "--fill": fillRatio(current()) }} aria-hidden="true" />
              <div class={styles.actions}>
                <For each={presetSeconds(props.piece.props.presets)}>
                  {(preset) => (
                    <button
                      class={styles.preset}
                      data-on={current().armed_delay_ms === preset * 1000 ? "" : undefined}
                      onClick={() => void send("delay", preset * 1000)}
                    >
                      {preset}s
                    </button>
                  )}
                </For>
                <Show
                  when={current().phase === "active"}
                  fallback={
                    <button class={styles.cut} disabled={current().phase === "idle"} onClick={() => void send("disarm")}>
                      Cancel
                    </button>
                  }
                >
                  <button class={styles.cut} onClick={() => void send("cut")}>
                    Cut
                  </button>
                </Show>
              </div>
              <Show when={error()}>{(message) => <span class={styles.error}>{message()}</span>}</Show>
            </>
          )}
        </Show>
      </div>
    </div>
  );
}
