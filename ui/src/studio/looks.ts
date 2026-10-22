// Looks: named snapshots of the whole Studio document (theme and every stage), saved as
// documents in `themes/`. Applying one is a normal edit, so it can be undone. A look can
// also travel as text (copy/paste) to share a setup.
import { createSignal } from "solid-js";
import { unwrap } from "solid-js/store";
import { call, errorText } from "~/ipc/commands";
import { pushToast } from "~/state/toasts";
import { normalizeDoc } from "./defaults";
import { doc, mutate, setSelectedId } from "./store";
import type { StudioDoc } from "./types";
import { animateChange } from "./transitions";

const KIND = "themes";
/** Marks clipboard text as a ValoShell look so random text is never applied. */
const SHARE_PREFIX = "valoshell-look:";

const [lookNames, setLookNames] = createSignal<string[]>([]);
export { lookNames };

export async function refreshLooks(): Promise<void> {
  try {
    setLookNames(await call("list_ui_documents", { kind: KIND }));
  } catch (error) {
    pushToast("negative", "Looks unavailable", errorText(error));
  }
}

export async function saveLook(name: string): Promise<boolean> {
  const trimmed = name.trim();
  if (!trimmed) return false;
  try {
    await call("save_ui_document", { kind: KIND, name: trimmed, document: structuredClone(unwrap(doc)) });
    await refreshLooks();
    pushToast("positive", "Look saved", trimmed);
    return true;
  } catch (error) {
    pushToast("negative", "Look not saved", errorText(error));
    return false;
  }
}

function applyDoc(next: StudioDoc): void {
  animateChange(
    () =>
      mutate((draft) => {
        draft.theme = next.theme;
        draft.stages = next.stages;
      }),
    "build",
  );
  setSelectedId(null);
}

export async function applyLook(name: string): Promise<void> {
  try {
    const saved = await call("load_ui_document", { kind: KIND, name });
    if (!saved) throw new Error("It no longer exists.");
    applyDoc(normalizeDoc(saved));
  } catch (error) {
    pushToast("negative", "Look not applied", errorText(error));
  }
}

export async function deleteLook(name: string): Promise<void> {
  try {
    await call("delete_ui_document", { kind: KIND, name });
    await refreshLooks();
  } catch (error) {
    pushToast("negative", "Look not deleted", errorText(error));
  }
}

function toBase64(text: string): string {
  let binary = "";
  for (const byte of new TextEncoder().encode(text)) binary += String.fromCharCode(byte);
  return btoa(binary);
}

function fromBase64(encoded: string): string {
  const bytes = Uint8Array.from(atob(encoded), (char) => char.charCodeAt(0));
  return new TextDecoder().decode(bytes);
}

/** Copy the current setup to the clipboard as shareable text. */
export async function copyLook(): Promise<void> {
  const text = SHARE_PREFIX + toBase64(JSON.stringify(unwrap(doc)));
  try {
    await navigator.clipboard.writeText(text);
    pushToast("positive", "Look copied", "Paste it in another ValoShell to use it.");
  } catch {
    pushToast("negative", "Clipboard unavailable", "");
  }
}

/** Apply a look from shared text (from `copyLook`). */
export function pasteLook(text: string): boolean {
  const trimmed = text.trim();
  if (!trimmed.startsWith(SHARE_PREFIX)) return false;
  try {
    const json = fromBase64(trimmed.slice(SHARE_PREFIX.length));
    applyDoc(normalizeDoc(JSON.parse(json)));
    pushToast("positive", "Look applied", "Ctrl+Z to undo.");
    return true;
  } catch {
    pushToast("negative", "That text is not a ValoShell look", "");
    return false;
  }
}
