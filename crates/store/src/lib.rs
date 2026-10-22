//! Settings, saved menu layouts and the data folder they live in.

pub mod documents;
pub mod paths;
pub mod settings;

pub use paths::DataPaths;
pub use settings::{Settings, SettingsStore};
