//! Real windows for the match flow: the menu (a Tauri window) and the game window, found by
//! its process image name so other Unreal games can never be mistaken for VALORANT.

use services::match_flow::WindowPort;
use tauri::{AppHandle, Manager};

use crate::MAIN_WINDOW;

pub struct AppWindows {
    app: AppHandle,
}

impl AppWindows {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    fn menu(&self) -> Option<tauri::WebviewWindow> {
        self.app.get_webview_window(MAIN_WINDOW)
    }
}

impl WindowPort for AppWindows {
    fn show_menu(&self) {
        if let Some(window) = self.menu()
            && let Err(error) = window.show().and_then(|()| window.set_focus())
        {
            tracing::warn!(%error, "show_menu_failed");
        }
        crate::overlay::set_in_match(&self.app, false);
    }

    fn hide_menu(&self) {
        if let Some(window) = self.menu()
            && let Err(error) = window.hide()
        {
            tracing::warn!(%error, "hide_menu_failed");
        }
        crate::overlay::set_in_match(&self.app, true);
    }

    fn focus_game(&self) {
        game::focus();
    }

    fn minimize_game(&self) {
        game::minimize();
    }
}

#[cfg(not(windows))]
mod game {
    /// VALORANT only runs on Windows; elsewhere (development builds) there is nothing to focus.
    pub fn focus() {
        tracing::debug!("game_window_not_supported_on_this_os");
    }

    pub fn minimize() {}
}

#[cfg(windows)]
mod game {
    // Win32 calls are FFI and need `unsafe`; each block is a single documented API call.
    #![allow(unsafe_code)]

    use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM};
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        QueryFullProcessImageNameW,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, IsIconic, IsWindowVisible, SW_MINIMIZE, SW_RESTORE,
        SetForegroundWindow, ShowWindow,
    };
    use windows::core::{BOOL, PWSTR};

    const GAME_EXECUTABLE: &str = "valorant-win64-shipping.exe";

    pub fn focus() {
        let Some(game) = find_game_window() else {
            tracing::debug!("game_window_not_found");
            return;
        };
        // SAFETY: `game` is a live top-level window handle returned by EnumWindows.
        unsafe {
            if IsIconic(game).as_bool() {
                let _ = ShowWindow(game, SW_RESTORE);
            }
            // Allowed: we hide our own foreground window first, so Windows lets us hand focus on.
            let _ = SetForegroundWindow(game);
        }
    }

    pub fn minimize() {
        if let Some(game) = find_game_window() {
            // SAFETY: as above.
            let _ = unsafe { ShowWindow(game, SW_MINIMIZE) };
        }
    }

    /// The first visible top-level window owned by the VALORANT game process.
    fn find_game_window() -> Option<HWND> {
        let mut found = HWND::default();
        // SAFETY: the callback only writes through the pointer we pass, which outlives the call.
        let _ = unsafe { EnumWindows(Some(visit_window), LPARAM(&raw mut found as isize)) };
        (!found.is_invalid()).then_some(found)
    }

    unsafe extern "system" fn visit_window(window: HWND, found: LPARAM) -> BOOL {
        // SAFETY: plain queries on a handle EnumWindows just gave us.
        let is_visible = unsafe { IsWindowVisible(window) }.as_bool();
        if is_visible && is_game_process(window) {
            // SAFETY: `found` is the `&mut HWND` passed by `find_game_window`.
            unsafe { *(found.0 as *mut HWND) = window };
            return BOOL(0);
        }
        BOOL(1)
    }

    fn is_game_process(window: HWND) -> bool {
        let mut process_id = 0u32;
        // SAFETY: writes the owning process id into a local.
        unsafe { GetWindowThreadProcessId(window, Some(&raw mut process_id)) };
        // SAFETY: limited query rights; the handle is closed below.
        let Ok(process) =
            (unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) })
        else {
            return false;
        };
        let mut buffer = [0u16; 512];
        let mut length = buffer.len() as u32;
        // SAFETY: buffer and length describe the same local array.
        let is_read = unsafe {
            QueryFullProcessImageNameW(
                process,
                PROCESS_NAME_WIN32,
                PWSTR(buffer.as_mut_ptr()),
                &raw mut length,
            )
        }
        .is_ok();
        // SAFETY: closing the handle we opened.
        let _ = unsafe { CloseHandle(process) };
        let path = String::from_utf16_lossy(&buffer[..length as usize]).to_lowercase();
        is_read && path.ends_with(GAME_EXECUTABLE)
    }
}
