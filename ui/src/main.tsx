// Entry point of the main window.
import { render } from "solid-js/web";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { App } from "./App";
import { isTauri } from "./ipc/runtime";
import { bindTheme } from "./studio/theme";
import { loadStudio } from "./studio/store";
import "./state/uiSounds";
import "./theme/tokens.css";
import "./theme/base.css";

const root = document.getElementById("root");
if (!root) {
  throw new Error("Missing #root element in index.html");
}

bindTheme();
// The saved layout and theme load before the first paint, so nothing jumps.
void loadStudio().then(() => {
  render(() => <App />, root);
  // The window starts hidden so the first frame is already painted (no white flash).
  if (isTauri) requestAnimationFrame(() => void getCurrentWindow().show());
});
