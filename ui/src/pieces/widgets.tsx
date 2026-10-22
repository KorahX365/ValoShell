// Free widgets anyone can drop on any stage: text, clock, image/GIF, video, web embed
// (TikTok, YouTube, Twitch, Spotify, any site) and the live match score.
import { Match, Show, Switch, createResource, createSignal, onCleanup } from "solid-js";
import { call } from "~/ipc/commands";
import { isTauri } from "~/ipc/runtime";
import { blockedReason, embedSource, embedUrl, isShortLink } from "~/studio/embeds";
import type { PieceProps } from "~/studio/registry";
import { game } from "~/state/session";
import styles from "./widgets.module.css";

function text(props: PieceProps, key: string, fallback = ""): string {
  const value = props.piece.props[key];
  return typeof value === "string" ? value : fallback;
}

function number(props: PieceProps, key: string, fallback: number): number {
  const value = props.piece.props[key];
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function flag(props: PieceProps, key: string, fallback = false): boolean {
  const value = props.piece.props[key];
  return typeof value === "boolean" ? value : fallback;
}

const FONT_VARS: Record<string, string> = {
  display: "var(--font-display)",
  text: "var(--font-text)",
  mono: "var(--font-mono)",
};

export function TextWidget(props: PieceProps) {
  return (
    <div
      class={styles.text}
      style={{
        "font-size": `${number(props, "size", 48)}px`,
        "font-weight": text(props, "weight", "800"),
        "text-align": text(props, "align", "left") as "left" | "center" | "right",
        "justify-content": { left: "flex-start", center: "center", right: "flex-end" }[text(props, "align", "left")] ?? "flex-start",
        color: text(props, "color") || "var(--text-1)",
        "font-family": FONT_VARS[text(props, "font", "display")] ?? "var(--font-display)",
        "text-transform": flag(props, "isUppercase", true) ? "uppercase" : "none",
        "text-shadow": flag(props, "hasShadow", true) ? "0 4px 24px rgb(0 0 0 / 0.55)" : "none",
      }}
    >
      <span>{text(props, "text", "Your text")}</span>
    </div>
  );
}

function useNow(intervalMs: number): () => Date {
  const [now, setNow] = createSignal(new Date());
  const timer = window.setInterval(() => setNow(new Date()), intervalMs);
  onCleanup(() => window.clearInterval(timer));
  return now;
}

export function ClockWidget(props: PieceProps) {
  const now = useNow(1000);
  const time = () => {
    const options: Intl.DateTimeFormatOptions = {
      hour: "2-digit",
      minute: "2-digit",
      hour12: text(props, "format", "24h") === "12h",
    };
    if (flag(props, "hasSeconds")) options.second = "2-digit";
    return now().toLocaleTimeString([], options);
  };
  const date = () => now().toLocaleDateString([], { weekday: "long", day: "numeric", month: "long" });
  return (
    <div class={styles.clock} style={{ "font-size": `${number(props, "size", 56)}px` }}>
      <span class={`${styles.clockTime} num`}>{time()}</span>
      <Show when={flag(props, "hasDate", true)}>
        <span class={styles.clockDate}>{date()}</span>
      </Show>
    </div>
  );
}

export function ImageWidget(props: PieceProps) {
  const [isBroken, setBroken] = createSignal(false);
  return (
    <Show when={text(props, "url") && !isBroken()} fallback={<Placeholder label="Image or GIF" hint="Paste a link in the inspector" />}>
      <img
        class={styles.media}
        src={text(props, "url")}
        alt=""
        draggable={false}
        style={{ "object-fit": text(props, "fit", "cover") as "cover" | "contain" }}
        onError={() => setBroken(true)}
      />
    </Show>
  );
}

export function VideoWidget(props: PieceProps) {
  return (
    <Show when={text(props, "url")} fallback={<Placeholder label="Video" hint="Paste an .mp4 or .webm link" />}>
      <video
        class={styles.media}
        src={text(props, "url")}
        autoplay
        playsinline
        muted={flag(props, "isMuted", true)}
        loop={flag(props, "isLooping", true)}
        style={{ "object-fit": text(props, "fit", "cover") as "cover" | "contain" }}
      />
    </Show>
  );
}

export function WebWidget(props: PieceProps) {
  const raw = () => text(props, "url");
  // Short links are followed by the app first; everything else maps straight to a player url.
  const [resolved] = createResource(
    () => (isShortLink(raw()) ? raw() : null),
    (link) => call("resolve_embed_link", { url: link }).catch(() => null),
  );
  const target = () => (isShortLink(raw()) ? (resolved() ?? "") : raw());
  const source = () => embedUrl(target());
  const blocked = () => blockedReason(target());
  return (
    <Show
      when={raw()}
      fallback={<Placeholder label="Web embed" hint="TikTok, YouTube, Twitch, Spotify or any site" />}
    >
      <Switch>
        <Match when={isShortLink(raw()) && resolved.loading}>
          <Placeholder label={embedSource(raw())} hint="Opening the short link…" />
        </Match>
        <Match when={isShortLink(raw()) && !resolved()}>
          <Placeholder label={embedSource(raw())} hint="Could not follow that short link. Paste the full video link instead." />
        </Match>
        <Match when={blocked()}>
          {(reason) => (
            <div class={styles.placeholder}>
              <span class={styles.placeholderLabel}>{embedSource(target())}</span>
              <span class={styles.placeholderHint}>{reason()}</span>
              <button class={styles.openButton} onClick={() => void openExternal(target())}>
                Open in browser
              </button>
            </div>
          )}
        </Match>
        <Match when={source()}>
          <div class={styles.webFrame}>
            <iframe
              class={styles.web}
              src={source()}
              title={embedSource(target())}
              allow="autoplay; encrypted-media; picture-in-picture; clipboard-write"
              referrerpolicy="strict-origin-when-cross-origin"
              style={{ "pointer-events": props.isEditing || !flag(props, "isInteractive", true) ? "none" : "auto" }}
            />
          </div>
        </Match>
      </Switch>
    </Show>
  );
}

/** Open a link in the browser (the system one inside the app). */
export async function openExternal(url: string): Promise<void> {
  if (!isTauri) {
    window.open(url, "_blank", "noopener");
    return;
  }
  const { openUrl } = await import("@tauri-apps/plugin-opener");
  await openUrl(url).catch(() => undefined);
}

export function MatchScoreWidget(props: PieceProps) {
  const snapshot = () => game();
  const isLive = () => snapshot()?.phase === "ingame" || snapshot()?.phase === "pregame";
  return (
    <div class={styles.score} data-compact={flag(props, "isCompact") ? "" : undefined}>
      <Switch fallback={<span class={styles.scoreIdle}>Match score</span>}>
        <Match when={isLive()}>
          <span class={styles.scoreMap}>{snapshot()?.map_name || "Live"}</span>
          <span class={`${styles.scoreValue} num`}>
            <span class={styles.ally}>{snapshot()?.ally_score ?? 0}</span>
            <span class={styles.scoreDash}>:</span>
            <span class={styles.enemy}>{snapshot()?.enemy_score ?? 0}</span>
          </span>
          <Show when={snapshot()?.side}>
            <span class={styles.scoreSide}>{snapshot()?.side === "attack" ? "Attacking" : "Defending"}</span>
          </Show>
        </Match>
      </Switch>
    </div>
  );
}

function Placeholder(props: { label: string; hint: string }) {
  return (
    <div class={styles.placeholder}>
      <span class={styles.placeholderLabel}>{props.label}</span>
      <span class={styles.placeholderHint}>{props.hint}</span>
    </div>
  );
}
