// Single subscription to backend `AppEvent`s; stores register handlers by event type.
// In the browser the demo backend dispatches events through the same table.
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { AppEvent } from "~/bindings/AppEvent";
import { isTauri } from "./runtime";

const APP_EVENT = "app-event";

type EventType = AppEvent["type"];
type DataOf<T extends EventType> = Extract<AppEvent, { type: T }>["data"];
type Handler<T extends EventType> = (data: DataOf<T>) => void;

const handlers = new Map<EventType, Set<(data: unknown) => void>>();
let unlisten: Promise<UnlistenFn> | undefined;

/** Deliver one event to every handler of its type. */
export function dispatchAppEvent(event: AppEvent): void {
  handlers.get(event.type)?.forEach((run) => run(event.data));
}

/** Run `handler` for every backend event of `type`. Returns an unsubscribe function. */
export function onAppEvent<T extends EventType>(type: T, handler: Handler<T>): () => void {
  if (isTauri) {
    unlisten ??= listen<AppEvent>(APP_EVENT, ({ payload }) => dispatchAppEvent(payload));
  }
  const set = handlers.get(type) ?? new Set();
  set.add(handler as (data: unknown) => void);
  handlers.set(type, set);
  return () => set.delete(handler as (data: unknown) => void);
}
