//! In-game overlay: a transparent, click-through, always-on-top window over the game showing
//! the pieces the user placed on the "In game" layout. Visible during a match when that
//! layout has pieces, or on demand (preview) while editing or filming. Works over windowed
//! and borderless VALORANT; exclusive fullscreen draws over every other window.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};

pub const OVERLAY_WINDOW: &str = "overlay";

/// Why the overlay may be shown; it is visible when enabled and (in a match or previewing).
#[derive(Default)]
pub struct OverlayState {
    is_enabled: AtomicBool,
    is_in_match: AtomicBool,
    is_preview: AtomicBool,
}

/// Create the hidden overlay window. Called once at startup.
pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let window =
        WebviewWindowBuilder::new(app, OVERLAY_WINDOW, WebviewUrl::App("overlay.html".into()))
            .title("ValoShell overlay")
            .transparent(true)
            .decorations(false)
            .shadow(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            // Never take keyboard focus from the game, even when shown mid-match.
            .focusable(false)
            .focused(false)
            .visible(false)
            .build()?;
    cover_primary_monitor(&window);
    app.manage(OverlayState::default());
    Ok(())
}

fn cover_primary_monitor(window: &tauri::WebviewWindow) {
    let Ok(Some(monitor)) = window.primary_monitor() else {
        return;
    };
    let position = *monitor.position();
    let size = *monitor.size();
    let _ = window.set_position(PhysicalPosition::new(position.x, position.y));
    let _ = window.set_size(PhysicalSize::new(size.width, size.height));
}

/// The match flow hid (true) or showed (false) the menu.
pub fn set_in_match(app: &AppHandle, is_in_match: bool) {
    update(app, |state| {
        state.is_in_match.store(is_in_match, Ordering::Relaxed)
    });
}

/// The UI reports whether the in-game layout has pieces.
pub fn set_enabled(app: &AppHandle, is_enabled: bool) {
    update(app, |state| {
        state.is_enabled.store(is_enabled, Ordering::Relaxed)
    });
}

/// Force the overlay visible (editor preview, filming) or back to automatic.
pub fn set_preview(app: &AppHandle, is_preview: bool) {
    update(app, |state| {
        state.is_preview.store(is_preview, Ordering::Relaxed)
    });
}

/// Flip the preview (global hotkey).
pub fn toggle_preview(app: &AppHandle) {
    update(app, |state| {
        state.is_preview.fetch_xor(true, Ordering::Relaxed);
    });
}

fn update(app: &AppHandle, change: impl FnOnce(&OverlayState)) {
    let Some(state) = app.try_state::<OverlayState>() else {
        return;
    };
    change(&state);
    let is_visible = state.is_preview.load(Ordering::Relaxed)
        || (state.is_enabled.load(Ordering::Relaxed) && state.is_in_match.load(Ordering::Relaxed));
    apply(app, is_visible);
}

fn apply(app: &AppHandle, is_visible: bool) {
    let Some(window) = app.get_webview_window(OVERLAY_WINDOW) else {
        return;
    };
    let result = if is_visible {
        cover_primary_monitor(&window);
        window
            .show()
            .and_then(|()| window.set_always_on_top(true))
            .and_then(|()| window.set_ignore_cursor_events(true))
    } else {
        window.hide()
    };
    if let Err(error) = result {
        tracing::warn!(%error, is_visible, "overlay_visibility_failed");
    }
}
