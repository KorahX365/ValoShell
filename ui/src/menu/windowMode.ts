// Styles drawn as a desktop client (League) run in a centred window at the client's own sizes;
// every other style covers the screen. Follows the Studio theme, so switching style switches
// the window too.
import { createEffect, on } from "solid-js";
import { isTauri } from "~/ipc/runtime";
import { WINDOWED_SKINS } from "~/studio/skins";
import { doc } from "~/studio/store";

/** The League client's window sizes; the larger one when the screen has room for it. */
const SMALL = { width: 1280, height: 720 };
const LARGE = { width: 1600, height: 900 };
/** Screen width (logical px) from which the larger window is used. */
const LARGE_FROM_WIDTH = 1800;

async function applyWindowMode(isWindowed: boolean): Promise<void> {
  const { LogicalSize, currentMonitor, getCurrentWindow } = await import("@tauri-apps/api/window");
  const window = getCurrentWindow();
  if (!isWindowed) {
    await window.setFullscreen(true);
    return;
  }
  await window.setFullscreen(false);
  const monitor = await currentMonitor();
  const screenWidth = monitor ? monitor.size.width / monitor.scaleFactor : SMALL.width;
  const size = screenWidth >= LARGE_FROM_WIDTH ? LARGE : SMALL;
  await window.setSize(new LogicalSize(size.width, size.height));
  await window.center();
}

/** Keep the window's mode in step with the menu style (call once from the shell). */
export function useWindowMode(): void {
  if (!isTauri) return;
  createEffect(
    on(
      () => WINDOWED_SKINS.includes(doc.theme.skin),
      (isWindowed) => void applyWindowMode(isWindowed).catch(() => undefined),
    ),
  );
}
