// Transient notifications (action errors, confirmations). Newest last; each fades out.
import { createSignal } from "solid-js";

export type ToastTone = "info" | "positive" | "negative" | "accent";

export interface Toast {
  id: number;
  tone: ToastTone;
  title: string;
  body: string;
}

const LIFETIME_MS = 4200;
const MAX_VISIBLE = 4;

const [toasts, setToasts] = createSignal<Toast[]>([]);
let nextId = 1;
const keyed = new Map<string, number>();

export { toasts };

export function dismissToast(id: number): void {
  setToasts((list) => list.filter((toast) => toast.id !== id));
}

/** A toast with a `key` takes the place of the last one with that key, so stepping through
 *  styles shows one toast rather than a stack. */
export function pushToast(tone: ToastTone, title: string, body = "", key?: string): void {
  const toast: Toast = { id: nextId++, tone, title, body };
  const replaced = key ? keyed.get(key) : undefined;
  if (key) keyed.set(key, toast.id);
  setToasts((list) => [...list.filter((entry) => entry.id !== replaced), toast].slice(-MAX_VISIBLE));
  window.setTimeout(() => dismissToast(toast.id), LIFETIME_MS);
}

/** Run an action; a failure becomes a toast instead of an unhandled rejection. */
export async function attempt(action: () => Promise<unknown>, failureTitle: string): Promise<boolean> {
  try {
    await action();
    return true;
  } catch (error) {
    pushToast("negative", failureTitle, typeof error === "string" ? error : error instanceof Error ? error.message : "");
    return false;
  }
}
