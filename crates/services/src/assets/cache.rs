//! Disk cache: one JSON file per endpoint plus `manifest.json` holding the valorant-api
//! `manifestId` the files belong to. The manifest is written last, so a half-finished refresh
//! is retried on the next start.

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::endpoints::Endpoint;
use super::index::Documents;

const MANIFEST_FILE: &str = "manifest.json";
/// Refresh even when the manifest id is unchanged after this long.
pub(super) const MAX_AGE_SECS: u64 = 7 * 24 * 60 * 60;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct Manifest {
    pub manifest_id: String,
    /// Unix seconds of the download that completed the file set.
    pub fetched_at: u64,
}

impl Manifest {
    /// True when the cached set still matches the live version and is younger than a week.
    pub fn is_current(&self, live_manifest_id: &str, now: u64) -> bool {
        self.manifest_id == live_manifest_id && now.saturating_sub(self.fetched_at) < MAX_AGE_SECS
    }
}

pub(super) fn read_manifest(dir: &Path) -> Option<Manifest> {
    let body = std::fs::read_to_string(dir.join(MANIFEST_FILE)).ok()?;
    serde_json::from_str(&body).ok()
}

pub(super) fn write_manifest(dir: &Path, manifest: &Manifest) -> std::io::Result<()> {
    let body = serde_json::to_string(manifest).map_err(std::io::Error::other)?;
    write_atomic(&dir.join(MANIFEST_FILE), body.as_bytes())
}

/// Every endpoint file that exists. Blocking: call from `spawn_blocking`.
pub(super) fn read_documents(dir: &Path) -> Documents {
    Endpoint::ALL
        .into_iter()
        .filter_map(|endpoint| {
            let body = std::fs::read_to_string(dir.join(endpoint.file_name())).ok()?;
            Some((endpoint, body))
        })
        .collect()
}

/// True when every endpoint has its file, so an endpoint added in an update gets downloaded
/// even while the manifest id is unchanged.
pub(super) fn has_every_document(dir: &Path) -> bool {
    Endpoint::ALL
        .into_iter()
        .all(|endpoint| dir.join(endpoint.file_name()).is_file())
}

pub(super) fn write_document(dir: &Path, endpoint: Endpoint, body: &str) -> std::io::Result<()> {
    write_atomic(&dir.join(endpoint.file_name()), body.as_bytes())
}

/// Write to a temp file then rename, so a crash never leaves a half-written JSON behind.
pub(super) fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("tmp");
    std::fs::write(&temp, bytes)?;
    std::fs::rename(&temp, path)
}
