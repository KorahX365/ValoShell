// Where the UI runs: inside the Tauri shell, or in a plain browser (demo backend).

/** True inside the desktop app; false in a browser tab (dev preview, demo). */
export const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
