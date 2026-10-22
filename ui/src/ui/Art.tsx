// An image that never shows a blank box or the browser's broken-image icon: the fallback
// (usually initials) stays until the image has loaded, and comes back if it fails. An image
// that already loaded once shows at once on the next screen, and a failed one is retried a
// couple of times before the fallback stays, so one dropped download doesn't blank a card.
import { Show, createEffect, createSignal, on, onCleanup, type JSX } from "solid-js";

const RETRY_DELAYS_MS = [600, 2000];
const loaded = new Set<string>();
/** Preloads still downloading, so repeated party and friend updates don't request them again. */
const pending = new Set<string>();

/** The url with a retry marker, so the webview asks again instead of reusing the failure. */
function withAttempt(src: string, attempt: number): string {
  if (attempt === 0) return src;
  return `${src}${src.includes("?") ? "&" : "?"}retry=${attempt}`;
}

/** Start downloading images before the screen that shows them mounts. */
export function preloadArt(sources: readonly (string | null | undefined)[]) {
  for (const src of sources) {
    if (!src || loaded.has(src) || pending.has(src)) continue;
    pending.add(src);
    const image = new Image();
    image.decoding = "async";
    image.onload = () => {
      pending.delete(src);
      loaded.add(src);
    };
    image.onerror = () => pending.delete(src);
    image.src = src;
  }
}

export function Art(props: { src: string | null | undefined; fallback: JSX.Element; class?: string | undefined; alt?: string }) {
  const initial = () => (props.src && loaded.has(props.src) ? "loaded" : "loading");
  const [state, setState] = createSignal<"loading" | "loaded" | "broken">(initial());
  const [attempt, setAttempt] = createSignal(0);
  let timer: number | undefined;
  createEffect(
    on(
      () => props.src,
      () => {
        window.clearTimeout(timer);
        setAttempt(0);
        setState(initial());
      },
      { defer: true },
    ),
  );
  onCleanup(() => window.clearTimeout(timer));
  const fail = () => {
    const delay = RETRY_DELAYS_MS[attempt()];
    if (delay === undefined) {
      setState("broken");
      return;
    }
    timer = window.setTimeout(() => setAttempt((count) => count + 1), delay);
  };
  const finish = () => {
    if (props.src) loaded.add(props.src);
    setState("loaded");
  };
  return (
    <>
      <Show when={state() !== "loaded"}>{props.fallback}</Show>
      <Show when={props.src && state() !== "broken"}>
        <img
          ref={(image) => queueMicrotask(() => image.complete && image.naturalWidth > 0 && finish())}
          class={props.class}
          src={withAttempt(props.src ?? "", attempt())}
          alt={props.alt ?? ""}
          draggable={false}
          decoding="async"
          style={{ display: state() === "loaded" ? undefined : "none" }}
          onLoad={finish}
          onError={fail}
        />
      </Show>
    </>
  );
}
