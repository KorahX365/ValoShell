//! Static game assets from valorant-api.com: names, art urls and videos
//! for agents, maps, cosmetics, ranks and more. Served from the disk cache or the bundled
//! fallback immediately; refreshed in the background when the API's manifest id changes.
//! Images are not downloaded up front: see `images` and the app's `asset-cache` protocol.

mod cache;
mod endpoints;
mod images;
mod index;
mod lookup;
mod records;
mod refresh;

#[cfg(test)]
pub(crate) mod fixtures;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use arc_swap::ArcSwap;
use tokio_util::sync::CancellationToken;

use crate::event::{AppEvent, Bus};

pub use endpoints::Endpoint;
pub use images::{CachedImage, image_url, source_from_request_uri};
pub use index::{AssetIndex, AssetIndexSummary, AssetSource, AssetsStatus, ItemInfo, ItemKind};
pub use records::*;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
/// A stalled connect fails fast so the image retry gets a fresh one.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(8);
/// How often a long-running app re-checks the manifest id.
const RECHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);

#[derive(Debug, thiserror::Error)]
pub enum AssetsError {
    #[error("Could not start the asset downloader: {0}")]
    Client(String),
    #[error("Could not download {endpoint}: {reason}")]
    Request { endpoint: String, reason: String },
    #[error("valorant-api.com answered {endpoint} with an unexpected response.")]
    BadResponse { endpoint: String },
    #[error("Could not save the asset cache: {0}")]
    Cache(#[from] std::io::Error),
    #[error("Refusing to download an image from {0}: only media.valorant-api.com is allowed.")]
    ImageNotAllowed(String),
}

struct Inner {
    index: ArcSwap<AssetIndex>,
    cache_dir: PathBuf,
    http: reqwest::Client,
    bus: Bus,
}

/// Cheap to clone; every clone shares one index. Lookups never block or touch the disk.
#[derive(Clone)]
pub struct AssetStore {
    inner: Arc<Inner>,
}

impl AssetStore {
    /// Starts with the bundled fallback tables. Call [`AssetStore::run`] to load the cache and refresh.
    pub fn new(cache_dir: impl Into<PathBuf>, bus: Bus) -> Result<Self, AssetsError> {
        riot::tls::install_crypto_provider();
        let http = reqwest::Client::builder()
            .user_agent(concat!("ValoShell/", env!("CARGO_PKG_VERSION")))
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .gzip(true)
            .build()
            .map_err(|error| AssetsError::Client(error.to_string()))?;
        Ok(Self {
            inner: Arc::new(Inner {
                index: ArcSwap::from_pointee(AssetIndex::bundled()),
                cache_dir: cache_dir.into(),
                http,
                bus,
            }),
        })
    }

    /// Background task: load the disk cache, then keep it fresh until `stop` fires.
    pub async fn run(self, stop: CancellationToken) {
        self.load_cache().await;
        loop {
            if let Err(error) = self.refresh_if_stale().await {
                tracing::warn!(%error, "assets_refresh_failed");
            }
            tokio::select! {
                () = stop.cancelled() => return,
                () = tokio::time::sleep(RECHECK_INTERVAL) => {}
            }
        }
    }

    /// The current index snapshot; hold it for several lookups that must agree.
    pub fn index(&self) -> Arc<AssetIndex> {
        self.inner.index.load_full()
    }

    pub fn status(&self) -> AssetsStatus {
        self.index().status()
    }

    pub fn summary(&self) -> AssetIndexSummary {
        self.index().summary()
    }

    pub fn cache_dir(&self) -> &Path {
        &self.inner.cache_dir
    }

    pub fn agent(&self, id: &domain::ids::AgentId) -> Option<Arc<Agent>> {
        self.index().agent(id.as_str())
    }

    pub fn map_by_internal_id(&self, internal_id: &str) -> Option<Arc<GameMap>> {
        self.index().map_by_internal_id(internal_id)
    }

    pub fn skin_level(&self, uuid: &str) -> Option<Arc<SkinLevel>> {
        self.index().skin_level(uuid)
    }

    pub fn tier(&self, tier: u32) -> Option<Arc<Tier>> {
        self.index().tier(tier)
    }

    pub fn currency(&self, uuid: &str) -> Option<Arc<Currency>> {
        self.index().currency(uuid)
    }

    /// Name and icon for an entitlement or store offer item.
    pub fn item_by_type_and_id(&self, item_type_id: &str, id: &str) -> Option<ItemInfo> {
        self.index().item_by_type_and_id(item_type_id, id)
    }

    /// Image bytes for an `asset-cache` request uri (downloads on the first request).
    pub async fn load_image(&self, request_uri: &str) -> Result<CachedImage, AssetsError> {
        let source = source_from_request_uri(request_uri)
            .ok_or_else(|| AssetsError::ImageNotAllowed(request_uri.to_owned()))?;
        images::load(&self.inner.http, &self.inner.cache_dir, &source).await
    }

    fn swap_index(&self, index: AssetIndex) {
        let status = index.status();
        self.inner.index.store(Arc::new(index));
        self.inner.bus.publish(AppEvent::Assets(status));
    }
}
