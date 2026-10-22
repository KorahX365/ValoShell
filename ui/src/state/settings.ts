// App settings (match flow, lobby, interface, streamer). Edits apply instantly and are
// saved by the backend; a failed save reverts to what is on disk.
import { createSignal } from "solid-js";
import type { Settings } from "~/bindings/Settings";
import { call } from "~/ipc/commands";
import { setUiVolume } from "~/menu/sound";
import { pushToast } from "./toasts";

const [settings, setSettingsSignal] = createSignal<Settings | null>(null);

function apply(next: Settings): void {
  setSettingsSignal(next);
  setUiVolume(next.interface.ui_volume);
  document.documentElement.toggleAttribute("data-streamer", next.streamer.is_enabled);
  const motion = next.interface.motion;
  document.documentElement.dataset["motion"] = motion;
}

void call("get_settings", {}).then(apply, () => undefined);

export { settings };

let saving: Promise<unknown> = Promise.resolve();

/** Change settings with a mutator; saves in order so fast toggles never race. */
export function updateSettings(change: (draft: Settings) => void): void {
  const current = settings();
  if (!current) return;
  const draft: Settings = structuredClone(current);
  change(draft);
  apply(draft);
  saving = saving.then(async () => {
    try {
      apply(await call("save_settings", { settings: draft }));
    } catch (error) {
      pushToast("negative", "Settings not saved", typeof error === "string" ? error : "");
      apply(await call("get_settings", {}));
    }
  });
}
