// The VALORANT style's settings screen, laid out like the client's: tabs across the top, rows
// of label + control in a centred column, Restore defaults and Close settings at the bottom.
// The crosshair tab draws a live preview. A last tab holds ValoShell's own settings.
import { For, Index, Match, Show, Switch, createSignal } from "solid-js";
import { Portal } from "solid-js/web";
import { ChevronDown } from "lucide-solid";
import type { MotionPreference } from "~/bindings/MotionPreference";
import { valLang } from "~/pieces/styled/valText";
import { settings, updateSettings } from "~/state/settings";
import { playCue } from "./sound";
import { CROSSHAIR_COLORS, type Row, TABS, type Words, resetValues, setValue, values } from "./valSettingsData";
import styles from "./ValSettings.module.css";

const say = (words: Words) => words[valLang() === "es" ? 1 : 0];

export function ValSettings(props: { onClose: () => void; onClassic: () => void }) {
  const [tab, setTab] = createSignal(TABS[0]?.id ?? "general");
  const current = () => TABS.find((entry) => entry.id === tab());
  const keys = () => (current()?.rows ?? []).flatMap((row) => ("key" in row && row.kind !== "app" ? [row.key] : []));
  return (
    <Portal>
      <div class={styles.screen} role="dialog" aria-label="Settings">
        <nav class={styles.tabs}>
          <For each={TABS}>
            {(entry) => (
              <button
                class={styles.tab}
                data-active={tab() === entry.id ? "" : undefined}
                onMouseEnter={() => playCue("hover")}
                onClick={() => {
                  playCue("tab");
                  setTab(entry.id);
                }}
              >
                {say(entry.label)}
              </button>
            )}
          </For>
          <button class={styles.tab} data-app="" onMouseEnter={() => playCue("hover")} onClick={() => props.onClassic()}>
            ValoShell
          </button>
        </nav>
        <div class={styles.body}>
          <Show when={tab() === "crosshair"}>
            <CrosshairPreview />
          </Show>
          <div class={styles.rows}>
            <For each={current()?.rows ?? []}>{(row) => <SettingRow row={row} />}</For>
          </div>
        </div>
        <footer class={styles.footer}>
          <button
            class={styles.restore}
            onMouseEnter={() => playCue("hover")}
            onClick={() => {
              playCue("back");
              resetValues(keys());
            }}
          >
            {say(["Restore defaults", "Restablecer valores predeterminados"])}
          </button>
          <button class={styles.close} onMouseEnter={() => playCue("hover")} onClick={() => props.onClose()}>
            {say(["Close settings", "Cerrar ajustes"])}
          </button>
        </footer>
      </div>
    </Portal>
  );
}

function SettingRow(props: { row: Row }) {
  return (
    <Switch>
      <Match when={props.row.kind === "section"}>
        <h3 class={styles.section}>{say(props.row.label)}</h3>
      </Match>
      <Match when={true}>
        <div class={styles.row}>
          <span class={styles.label}>{say(props.row.label)}</span>
          <span class={styles.control}>
            <Control row={props.row} />
          </span>
        </div>
      </Match>
    </Switch>
  );
}

function Control(props: { row: Row }) {
  const row = props.row;
  switch (row.kind) {
    case "toggle":
      return <OnOff value={values()[row.key] === true} onChange={(isOn) => setValue(row.key, isOn)} />;
    case "slider":
      return <Slider value={Number(values()[row.key] ?? 0)} min={row.min} max={row.max} step={row.step} onChange={(value) => setValue(row.key, value)} />;
    case "select":
      return <Dropdown value={Number(values()[row.key] ?? 0)} options={row.options} onChange={(index) => setValue(row.key, index)} />;
    case "bind":
      return <Bind value={String(values()[row.key] ?? "")} onChange={(key) => setValue(row.key, key)} />;
    case "app":
      return <AppControl which={row.key} />;
    default:
      return null;
  }
}

function OnOff(props: { value: boolean; onChange: (isOn: boolean) => void }) {
  const pick = (isOn: boolean) => {
    playCue("tab");
    props.onChange(isOn);
  };
  return (
    <span class={styles.onOff}>
      <button data-active={props.value ? "" : undefined} onClick={() => pick(true)}>
        {say(["On", "Activado"])}
      </button>
      <button data-active={props.value ? undefined : ""} onClick={() => pick(false)}>
        {say(["Off", "Desactivado"])}
      </button>
    </span>
  );
}

function Slider(props: { value: number; min: number; max: number; step: number; onChange: (value: number) => void }) {
  const digits = () => (props.step >= 1 ? 0 : props.step >= 0.01 ? 2 : 3);
  const fill = () => ((props.value - props.min) / (props.max - props.min)) * 100;
  return (
    <span class={styles.slider}>
      <input
        class={styles.number}
        value={props.value.toFixed(digits())}
        onChange={(event) => {
          const next = Number(event.currentTarget.value.replace(",", "."));
          if (Number.isFinite(next)) props.onChange(Math.min(props.max, Math.max(props.min, next)));
        }}
      />
      <input
        class={styles.track}
        type="range"
        min={props.min}
        max={props.max}
        step={props.step}
        value={props.value}
        style={{ "--fill": `${fill()}%` }}
        onInput={(event) => props.onChange(Number(event.currentTarget.value))}
      />
    </span>
  );
}

function Dropdown(props: { value: number; options: readonly Words[]; onChange: (index: number) => void }) {
  const [isOpen, setOpen] = createSignal(false);
  return (
    <span class={styles.dropdown} onFocusOut={() => setOpen(false)}>
      <button
        class={styles.dropButton}
        data-open={isOpen() ? "" : undefined}
        onClick={() => {
          playCue("tab");
          setOpen(!isOpen());
        }}
      >
        <span>{say(props.options[props.value] ?? props.options[0] ?? ["", ""])}</span>
        <ChevronDown size={16} />
      </button>
      <Show when={isOpen()}>
        <span class={styles.dropList}>
          <Index each={props.options}>
            {(option, index) => (
              <button
                data-active={index === props.value ? "" : undefined}
                onMouseEnter={() => playCue("hover")}
                onMouseDown={(event) => event.preventDefault()}
                onClick={() => {
                  playCue("confirm");
                  props.onChange(index);
                  setOpen(false);
                }}
              >
                {say(option())}
              </button>
            )}
          </Index>
        </span>
      </Show>
    </span>
  );
}

function Bind(props: { value: string; onChange: (key: string) => void }) {
  const [isListening, setListening] = createSignal(false);
  const keyName = (event: KeyboardEvent) => {
    if (event.code === "Space") return "Space";
    if (event.key === "Shift" || event.key === "Control" || event.key === "Alt") return `${event.location === 2 ? "R" : "L"}-${event.key === "Control" ? "Ctrl" : event.key}`;
    return event.key.length === 1 ? event.key.toUpperCase() : event.key;
  };
  return (
    <span class={styles.binds}>
      <button
        class={styles.bind}
        data-listening={isListening() ? "" : undefined}
        onClick={() => setListening(true)}
        onBlur={() => setListening(false)}
        onKeyDown={(event) => {
          if (!isListening()) return;
          event.preventDefault();
          event.stopPropagation();
          if (event.key !== "Escape") props.onChange(keyName(event));
          playCue("confirm");
          setListening(false);
        }}
      >
        {isListening() ? "…" : props.value}
      </button>
      <button class={styles.bind} data-empty="" disabled>
        -
      </button>
    </span>
  );
}

/** ValoShell's real settings in the client's controls. */
function AppControl(props: { which: "uiVolume" | "motion" | "streamer" }) {
  const current = settings;
  return (
    <Show when={current()}>
      {(now) => (
        <Switch>
          <Match when={props.which === "uiVolume"}>
            <Slider
              value={Math.round(now().interface.ui_volume * 100)}
              min={0}
              max={100}
              step={1}
              onChange={(value) => updateSettings((draft) => void (draft.interface.ui_volume = value / 100))}
            />
          </Match>
          <Match when={props.which === "streamer"}>
            <OnOff value={now().streamer.is_enabled} onChange={(isOn) => updateSettings((draft) => void (draft.streamer.is_enabled = isOn))} />
          </Match>
          <Match when={props.which === "motion"}>
            <Dropdown
              value={MOTIONS.indexOf(now().interface.motion)}
              options={[
                ["System", "Sistema"],
                ["Full", "Completas"],
                ["Reduced", "Reducidas"],
              ]}
              onChange={(index) => updateSettings((draft) => void (draft.interface.motion = MOTIONS[index] ?? "system"))}
            />
          </Match>
        </Switch>
      )}
    </Show>
  );
}

const MOTIONS: readonly MotionPreference[] = ["system", "full", "reduced"];

/** The crosshair as the settings draw it, over a slice of a range wall. */
function CrosshairPreview() {
  const value = (key: string) => values()[key];
  const color = () => CROSSHAIR_COLORS[Number(value("xColor"))] ?? "#ffffff";
  const outline = () => (value("xOutlines") === true ? `0 0 0 ${Number(value("xOutlineThickness"))}px rgb(0 0 0 / ${Number(value("xOutlineOpacity"))})` : "none");
  const line = (length: number, thickness: number, offset: number, opacity: number, side: "t" | "b" | "l" | "r") => {
    const isVertical = side === "t" || side === "b";
    const along = side === "t" || side === "l" ? -(offset + length) : offset;
    return {
      width: `${isVertical ? thickness : length}px`,
      height: `${isVertical ? length : thickness}px`,
      transform: isVertical ? `translate(${-thickness / 2}px, ${along}px)` : `translate(${along}px, ${-thickness / 2}px)`,
      background: color(),
      opacity: String(opacity),
      "box-shadow": outline(),
    };
  };
  const SIDES = ["t", "b", "l", "r"] as const;
  return (
    <div class={styles.preview}>
      <div class={styles.previewCross} style={{ transform: "scale(2)" }}>
        <Show when={value("xDot") === true}>
          <i
            style={{
              width: `${Number(value("xDotThickness"))}px`,
              height: `${Number(value("xDotThickness"))}px`,
              transform: `translate(-50%, -50%)`,
              background: color(),
              opacity: String(value("xDotOpacity")),
              "box-shadow": outline(),
            }}
          />
        </Show>
        <Show when={value("xInner") === true}>
          <For each={SIDES}>
            {(side) => <i style={line(Number(value("xInnerLength")), Number(value("xInnerThickness")), Number(value("xInnerOffset")), Number(value("xInnerOpacity")), side)} />}
          </For>
        </Show>
        <Show when={value("xOuter") === true}>
          <For each={SIDES}>{(side) => <i style={line(Number(value("xOuterLength")), 2, Number(value("xOuterOffset")), 0.35, side)} />}</For>
        </Show>
      </div>
    </div>
  );
}
