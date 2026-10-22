//! Typed app settings in `settings.json`: every field has a default (a missing or older file
//! never breaks startup), writes are atomic (`.tmp` + rename), and services subscribe to
//! changes through a `watch` channel instead of polling.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use serde::{Deserialize, Serialize};
use tokio::sync::watch;
use ts_rs::TS;

use crate::paths::with_path;

/// Menu sounds start at this volume (and fall back to it from an unreadable value).
const DEFAULT_UI_VOLUME: f64 = 0.6;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct Settings {
    /// Last region the client connected with; a fallback when the game log has none.
    pub region: Option<String>,
    pub match_flow: MatchFlowSettings,
    pub lobby: LobbySettings,
    pub interface: InterfaceSettings,
    pub streamer: StreamerSettings,
}

/// When the menu gets out of the way of the game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct MatchFlowSettings {
    pub should_hide_on_lock: bool,
    pub should_hide_on_match_start: bool,
    pub should_return_in_menus: bool,
    pub should_minimize_game_in_menus: bool,
}

impl Default for MatchFlowSettings {
    fn default() -> Self {
        Self {
            should_hide_on_lock: true,
            should_hide_on_match_start: true,
            should_return_in_menus: true,
            should_minimize_game_in_menus: false,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct LobbySettings {
    /// Off: enemy ranks are never requested during agent select, where the game hides them.
    /// They are read once the match starts, as the in-game scoreboard shows them anyway.
    pub should_rank_enemies_in_agent_select: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum MotionPreference {
    /// Follow the OS reduced-motion setting.
    #[default]
    System,
    Full,
    Reduced,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct InterfaceSettings {
    pub theme: String,
    /// 0.0 (muted) to 1.0.
    pub ui_volume: f64,
    pub motion: MotionPreference,
    pub backdrop: String,
}

impl Default for InterfaceSettings {
    fn default() -> Self {
        Self {
            theme: "valorant-dark".to_owned(),
            ui_volume: DEFAULT_UI_VOLUME,
            motion: MotionPreference::System,
            backdrop: "default".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct StreamerSettings {
    pub is_enabled: bool,
    pub should_hide_own_name: bool,
}

impl Default for StreamerSettings {
    fn default() -> Self {
        Self {
            is_enabled: false,
            should_hide_own_name: true,
        }
    }
}

impl Settings {
    /// Clamp values a hand-edited or older file could carry out of range.
    pub fn normalized(mut self) -> Self {
        let volume = self.interface.ui_volume;
        self.interface.ui_volume = if volume.is_finite() {
            volume.clamp(0.0, 1.0)
        } else {
            DEFAULT_UI_VOLUME
        };
        self
    }
}

/// Shared settings handle: read anywhere, update atomically, subscribe to changes.
#[derive(Clone)]
pub struct SettingsStore {
    path: Arc<PathBuf>,
    sender: Arc<watch::Sender<Settings>>,
    /// One update at a time, so concurrent changes neither drop each other nor race on `.tmp`.
    writer: Arc<Mutex<()>>,
}

impl SettingsStore {
    /// Load from `path`. Missing file = defaults; unreadable file = defaults plus a backup of
    /// the bad file so nothing the user wrote is lost.
    pub fn load(path: PathBuf) -> Self {
        let settings = read_settings(&path).normalized();
        let (sender, _) = watch::channel(settings);
        Self {
            path: Arc::new(path),
            sender: Arc::new(sender),
            writer: Arc::new(Mutex::new(())),
        }
    }

    /// Current settings (cheap clone of a small struct).
    pub fn get(&self) -> Settings {
        self.sender.borrow().clone()
    }

    /// Receive every future change.
    pub fn subscribe(&self) -> watch::Receiver<Settings> {
        self.sender.subscribe()
    }

    /// Apply `change`, persist, then publish. Nothing is published if the write fails.
    pub fn update(&self, change: impl FnOnce(&mut Settings)) -> std::io::Result<Settings> {
        let _writing = self.writer.lock().unwrap_or_else(PoisonError::into_inner);
        let mut next = self.get();
        change(&mut next);
        let next = next.normalized();
        write_atomically(&self.path, &next)?;
        self.sender.send_replace(next.clone());
        Ok(next)
    }
}

fn read_settings(path: &Path) -> Settings {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Settings::default();
    };
    match serde_json::from_str(&text) {
        Ok(settings) => settings,
        Err(error) => {
            let backup = path.with_extension("json.corrupt");
            let _ = std::fs::copy(path, &backup);
            tracing::warn!(%error, backup = %backup.display(), "settings_unreadable_using_defaults");
            Settings::default()
        }
    }
}

fn write_atomically(path: &Path, settings: &Settings) -> std::io::Result<()> {
    let text = serde_json::to_string_pretty(settings).map_err(std::io::Error::other)?;
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, text).map_err(|error| with_path(error, &temporary))?;
    std::fs::rename(&temporary, path).map_err(|error| with_path(error, path))
}

#[cfg(test)]
mod tests;
