// Sound files on disk for the menu styles (installed games, the player's sounds folder).
// The backend opens each file to the `asset:` protocol, so the urls only work after this call.
import { convertFileSrc } from "@tauri-apps/api/core";
import { createSignal } from "solid-js";
import type { UiSound } from "~/bindings/UiSound";
import { call } from "~/ipc/commands";
import { isTauri } from "~/ipc/runtime";
import { setSoundFiles } from "~/menu/sound";

const [uiSounds, setUiSounds] = createSignal<UiSound[]>([]);

export { uiSounds };

/** Look for sound files again (after the player adds some to the folder). */
export async function reloadUiSounds(): Promise<void> {
  const found = await call("get_ui_sounds", {}).catch(() => [] as UiSound[]);
  // The folder's files come first and override a game's file for the same cue.
  const files = new Map<string, string>();
  for (const sound of found) {
    const key = `${sound.skin}:${sound.cue}`;
    if (!files.has(key)) files.set(key, isTauri ? convertFileSrc(sound.path) : sound.path);
  }
  setUiSounds(found);
  setSoundFiles(files);
}

export function openSoundsFolder(): void {
  void call("open_sounds_folder", {}).catch(() => undefined);
}

void reloadUiSounds();
