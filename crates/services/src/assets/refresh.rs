//! Cache loading and the download pass. A pass runs only when the live `manifestId` differs
//! from the cached one or the cache is a week old; offline or failed passes keep what we have.

use std::time::{SystemTime, UNIX_EPOCH};

use super::cache::{self, Manifest};
use super::endpoints::Endpoint;
use super::index::{AssetIndex, AssetSource, Documents};
use super::records::{Envelope, Version};
use super::{AssetStore, AssetsError};

impl AssetStore {
    /// Replace the bundled fallback with the cached files, if any. Never touches the network.
    pub(super) async fn load_cache(&self) {
        let dir = self.inner.cache_dir.clone();
        let docs = tokio::task::spawn_blocking(move || cache::read_documents(&dir))
            .await
            .unwrap_or_default();
        if docs.is_empty() {
            return;
        }
        self.swap_index(AssetIndex::from_documents(&docs, AssetSource::Cache));
    }

    /// Check the live version and download everything when the cache is out of date.
    pub(super) async fn refresh_if_stale(&self) -> Result<(), AssetsError> {
        let version_body = self.fetch(Endpoint::Version).await?;
        let live = parse_version(&version_body)?;
        let cached = cache::read_manifest(&self.inner.cache_dir);
        let is_complete = cache::has_every_document(&self.inner.cache_dir);
        if is_complete
            && cached
                .as_ref()
                .is_some_and(|manifest| manifest.is_current(&live.manifest_id, now_secs()))
        {
            tracing::debug!(manifest_id = %live.manifest_id, "assets_cache_current");
            return Ok(());
        }
        tracing::info!(manifest_id = %live.manifest_id, "assets_refresh_started");
        let failures = self.download_all(&version_body).await;
        self.reload_from_disk().await;
        if failures == 0 {
            let manifest = Manifest {
                manifest_id: live.manifest_id,
                fetched_at: now_secs(),
            };
            cache::write_manifest(&self.inner.cache_dir, &manifest)?;
        }
        Ok(())
    }

    /// Fetch and store every endpoint; returns how many failed (the old file stays for those).
    async fn download_all(&self, version_body: &str) -> usize {
        let dir = self.inner.cache_dir.clone();
        let mut failures = 0;
        if let Err(error) = cache::write_document(&dir, Endpoint::Version, version_body) {
            tracing::warn!(%error, "assets_cache_write_failed");
            failures += 1;
        }
        for endpoint in Endpoint::ALL.into_iter().skip(1) {
            let saved = match self.fetch(endpoint).await {
                Ok(body) => cache::write_document(&dir, endpoint, &body).map_err(AssetsError::from),
                Err(error) => Err(error),
            };
            if let Err(error) = saved {
                tracing::warn!(?endpoint, %error, "assets_endpoint_failed");
                failures += 1;
            }
        }
        failures
    }

    async fn reload_from_disk(&self) {
        let dir = self.inner.cache_dir.clone();
        let docs: Documents = tokio::task::spawn_blocking(move || cache::read_documents(&dir))
            .await
            .unwrap_or_default();
        self.swap_index(AssetIndex::from_documents(&docs, AssetSource::Network));
    }

    /// One endpoint's body, checked to be a `status: 200` envelope before it can reach disk.
    async fn fetch(&self, endpoint: Endpoint) -> Result<String, AssetsError> {
        let name = endpoint.path().to_owned();
        let request_error = |error: reqwest::Error| AssetsError::Request {
            endpoint: name.clone(),
            reason: error.to_string(),
        };
        let body = self
            .inner
            .http
            .get(endpoint.url())
            .send()
            .await
            .map_err(request_error)?
            .error_for_status()
            .map_err(request_error)?
            .text()
            .await
            .map_err(request_error)?;
        match serde_json::from_str::<Envelope<serde::de::IgnoredAny>>(&body) {
            Ok(envelope) if envelope.status == 200 => Ok(body),
            _ => Err(AssetsError::BadResponse { endpoint: name }),
        }
    }
}

fn parse_version(body: &str) -> Result<Version, AssetsError> {
    serde_json::from_str::<Envelope<Version>>(body)
        .map(|envelope| envelope.data)
        .ok()
        .filter(|version| !version.manifest_id.is_empty())
        .ok_or_else(|| AssetsError::BadResponse {
            endpoint: Endpoint::Version.path().to_owned(),
        })
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}
