// The Esc menu: settings for the match flow, lobby, sound, motion and streamer mode,
// plus Studio and Quit. Opens with Esc or the cog; closes with Esc or Resume.
import { Show, createSignal, onCleanup, onMount } from "solid-js";
import { Portal } from "solid-js/web";
import { LogOut, Paintbrush, Play } from "lucide-solid";
import type { MotionPreference } from "~/bindings/MotionPreference";
import { call } from "~/ipc/commands";
import { isTauri } from "~/ipc/runtime";
import { settings, updateSettings } from "~/state/settings";
import { doc, isEditing, setEditMode } from "~/studio/store";
import { Range, Segmented, Switch, controls } from "~/ui/controls";
import { playCue } from "./sound";
import { ValSettings } from "./ValSettings";
import styles from "./EscMenu.module.css";

const [isOpen, setOpen] = createSignal(false);
/** The VALORANT style shows the client's settings screen; its ValoShell tab comes back here. */
const [isClassic, setClassic] = createSignal(false);

export function openEscMenu(): void {
  playCue("confirm");
  setOpen(true);
}

export function closeEscMenu(): void {
  playCue("back");
  setOpen(false);
  setClassic(false);
}

export function EscMenu() {
  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key !== "Escape" || isEditing()) return;
    event.preventDefault();
    if (isOpen()) closeEscMenu();
    else openEscMenu();
  };
  onMount(() => window.addEventListener("keydown", onKeyDown));
  onCleanup(() => window.removeEventListener("keydown", onKeyDown));

  return (
    <Show
      when={!(doc.theme.skin === "current" && !isClassic())}
      fallback={
        <Show when={isOpen()}>
          <ValSettings
            onClose={closeEscMenu}
            onClassic={() => {
              playCue("tab");
              setClassic(true);
            }}
          />
        </Show>
      }
    >
    <Show when={isOpen() && settings()}>
      {(current) => (
        <Portal>
          <div class={styles.scrim} onPointerDown={(event) => event.target === event.currentTarget && closeEscMenu()}>
            <div class={styles.menu} role="dialog" aria-label="Settings">
              <header class={styles.header}>
                <h1 class={styles.title}>Settings</h1>
                <span class="overline">Esc to resume</span>
              </header>
              <div class={styles.columns}>
                <section class={styles.section}>
                  <h2 class={styles.heading}>Match flow</h2>
                  <Switch
                    title="Hide on lock-in"
                    hint="Get out of the way once your agent is locked"
                    isOn={current().match_flow.should_hide_on_lock}
                    onChange={(isOn) => updateSettings((draft) => void (draft.match_flow.should_hide_on_lock = isOn))}
                  />
                  <Switch
                    title="Hide when the match starts"
                    isOn={current().match_flow.should_hide_on_match_start}
                    onChange={(isOn) => updateSettings((draft) => void (draft.match_flow.should_hide_on_match_start = isOn))}
                  />
                  <Switch
                    title="Come back in the menus"
                    hint="Show ValoShell again after the match"
                    isOn={current().match_flow.should_return_in_menus}
                    onChange={(isOn) => updateSettings((draft) => void (draft.match_flow.should_return_in_menus = isOn))}
                  />
                  <Switch
                    title="Minimise VALORANT in the menus"
                    isOn={current().match_flow.should_minimize_game_in_menus}
                    onChange={(isOn) => updateSettings((draft) => void (draft.match_flow.should_minimize_game_in_menus = isOn))}
                  />
                </section>
                <section class={styles.section}>
                  <h2 class={styles.heading}>Lobby</h2>
                  <Switch
                    title="Enemy ranks in agent select"
                    hint="Off: never requested until the match starts, as the game hides them"
                    isOn={current().lobby.should_rank_enemies_in_agent_select}
                    onChange={(isOn) => updateSettings((draft) => void (draft.lobby.should_rank_enemies_in_agent_select = isOn))}
                  />
                </section>
                <section class={styles.section}>
                  <h2 class={styles.heading}>Interface</h2>
                  <Range
                    label="UI sounds"
                    value={current().interface.ui_volume}
                    min={0}
                    max={1}
                    step={0.05}
                    format={(value) => (value === 0 ? "Muted" : `${Math.round(value * 100)}%`)}
                    onInput={(value) => updateSettings((draft) => void (draft.interface.ui_volume = value))}
                  />
                  <div class={styles.field}>
                    <span class="overline">Motion</span>
                    <Segmented<MotionPreference>
                      value={current().interface.motion}
                      options={[
                        { value: "system", label: "System" },
                        { value: "full", label: "Full" },
                        { value: "reduced", label: "Reduced" },
                      ]}
                      onChange={(value) => updateSettings((draft) => void (draft.interface.motion = value))}
                    />
                  </div>
                  <Switch
                    title="Streamer mode"
                    hint="Blurs names, tags, codes and balances"
                    isOn={current().streamer.is_enabled}
                    onChange={(isOn) => updateSettings((draft) => void (draft.streamer.is_enabled = isOn))}
                  />
                  <button
                    class={`${controls.button} ${controls.ghost}`}
                    onClick={() => {
                      setOpen(false);
                      setEditMode(true);
                    }}
                  >
                    <Paintbrush size={15} /> Open Studio
                  </button>
                  <p class={styles.note}>
                    Ctrl+E edits any screen · Ctrl+Alt+V shows/hides ValoShell · Ctrl+Alt+O toggles the in-game overlay
                  </p>
                </section>
              </div>
              <footer class={styles.footer}>
                <button class={controls.button} onClick={closeEscMenu}>
                  <Play size={15} /> Resume
                </button>
                <Show when={isTauri}>
                  <button class={`${controls.button} ${controls.danger}`} onClick={() => void call("quit_app", {})}>
                    <LogOut size={15} /> Quit ValoShell
                  </button>
                </Show>
              </footer>
            </div>
          </div>
        </Portal>
      )}
    </Show>
    </Show>
  );
}
