// Applies the Studio theme to the document: semantic colour tokens, fonts, cut size and scale.
// Components only read tokens, so a theme change restyles everything in one style recalc.
import { createEffect, createRoot } from "solid-js";
import { cursorsFor } from "~/menu/cursors";
import { setSoundOptions } from "~/menu/sound";
import type { DisplayFont, Theme } from "./types";
import { doc } from "./store";

const DISPLAY_FONTS: Record<DisplayFont, string> = {
  saira: '"Saira Variable", "Bahnschrift", sans-serif',
  tall: '"Oswald Variable", "Bahnschrift Condensed", sans-serif',
  inter: '"Inter Variable", "Segoe UI", sans-serif',
  mono: '"JetBrains Mono Variable", Consolas, monospace',
  classic: 'Tahoma, Verdana, "Segoe UI", sans-serif',
  pixel: '"Pixelify Sans Variable", monospace',
  robco: '"Share Tech Mono", "JetBrains Mono Variable", monospace',
  beaufort: '"Beaufort for LOL", "Cinzel Variable", Georgia, serif',
};

/** `#rrggbb` -> `r g b` for `rgb(var(--x) / alpha)` style tokens. */
export function hexToRgbTriplet(hex: string): string {
  const match = /^#?([0-9a-f]{6})$/i.exec(hex.trim());
  const value = match?.[1] ? Number.parseInt(match[1], 16) : 0;
  return `${(value >> 16) & 255} ${(value >> 8) & 255} ${value & 255}`;
}

/** Readable text colour on top of `hex` (black on light accents such as Mono). */
export function onColor(hex: string): string {
  const [r = 0, g = 0, b = 0] = hexToRgbTriplet(hex).split(" ").map(Number);
  const luminance = (0.2126 * r + 0.7152 * g + 0.0722 * b) / 255;
  return luminance > 0.62 ? "#0b0b0e" : "#ffffff";
}

export function applyTheme(theme: Theme, root: HTMLElement = document.documentElement): void {
  const style = root.style;
  style.setProperty("--accent", theme.accent);
  style.setProperty("--accent-hover", `color-mix(in oklch, ${theme.accent} 86%, white)`);
  style.setProperty("--accent-pressed", `color-mix(in oklch, ${theme.accent} 84%, black)`);
  style.setProperty("--accent-gradient-end", theme.accent2);
  style.setProperty("--accent-2", theme.accent2);
  style.setProperty("--on-accent", onColor(theme.accent));
  style.setProperty("--ground", theme.ground);
  style.setProperty("--ground-rgb", hexToRgbTriplet(theme.ground));
  style.setProperty("--paper-100", theme.text);
  style.setProperty("--text-1", theme.text);
  style.setProperty("--text-rgb", hexToRgbTriplet(theme.text));
  style.setProperty("--text-2", `rgb(${hexToRgbTriplet(theme.text)} / 0.66)`);
  style.setProperty("--text-3", `rgb(${hexToRgbTriplet(theme.text)} / 0.46)`);
  style.setProperty("--font-display", DISPLAY_FONTS[theme.displayFont]);
  style.setProperty("--cut", `${theme.cut}px`);
  style.setProperty("--vignette", String(theme.vignette));
  style.setProperty("--grain", theme.isGrainy ? "0.07" : "0");
  root.dataset["skin"] = theme.skin;
  const cursors = cursorsFor(theme.skin, theme.accent);
  style.setProperty("--cursor", cursors.base);
  style.setProperty("--cursor-hot", cursors.hot);
  if (theme.fx.isCursorOn) root.dataset["cursor"] = "";
  else delete root.dataset["cursor"];
  setSoundOptions({ ...theme.sound, skin: theme.skin });
}

/** Keep the document styled from the live Studio theme. Call once per window. */
export function bindTheme(): void {
  createRoot(() => createEffect(() => applyTheme(doc.theme)));
}
