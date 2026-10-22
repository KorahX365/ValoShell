//! Layout and theme documents (the live editor's saves) and the in-game overlay switches.

use std::path::{Path, PathBuf};

use serde_json::Value;
use store::DataPaths;
use store::documents::{delete_document, list_documents, load_document, save_document};
use tauri::{AppHandle, State};

type CommandResult<T = ()> = Result<T, String>;

/// Folders documents are kept in (managed state, set at startup).
pub struct DocumentDirs {
    layouts: PathBuf,
    themes: PathBuf,
}

impl DocumentDirs {
    pub fn new(paths: &DataPaths) -> Self {
        Self {
            layouts: paths.layouts_dir(),
            themes: paths.themes_dir(),
        }
    }

    fn of(&self, kind: &str) -> CommandResult<&Path> {
        match kind {
            "layouts" => Ok(&self.layouts),
            "themes" => Ok(&self.themes),
            other => Err(format!("Unknown document kind \"{other}\".")),
        }
    }
}

#[tauri::command]
pub fn list_ui_documents(
    dirs: State<'_, DocumentDirs>,
    kind: String,
) -> CommandResult<Vec<String>> {
    list_documents(dirs.of(&kind)?).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn load_ui_document(
    dirs: State<'_, DocumentDirs>,
    kind: String,
    name: String,
) -> CommandResult<Option<Value>> {
    load_document(dirs.of(&kind)?, &name).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_ui_document(
    dirs: State<'_, DocumentDirs>,
    kind: String,
    name: String,
    document: Value,
) -> CommandResult {
    save_document(dirs.of(&kind)?, &name, &document).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_ui_document(
    dirs: State<'_, DocumentDirs>,
    kind: String,
    name: String,
) -> CommandResult {
    delete_document(dirs.of(&kind)?, &name).map_err(|error| error.to_string())
}

/// The UI says whether the in-game layout has anything to show.
#[tauri::command]
pub fn set_overlay_enabled(app: AppHandle, is_enabled: bool) {
    crate::overlay::set_enabled(&app, is_enabled);
}

/// Show the in-game overlay now (editing / filming) regardless of the match.
#[tauri::command]
pub fn set_overlay_preview(app: AppHandle, is_visible: bool) {
    crate::overlay::set_preview(&app, is_visible);
}
