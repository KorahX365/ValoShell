//! Image proxy: the UI asks for `asset-cache` urls; the first request downloads the original
//! to `cache/images/`, later ones read the file. Only valorant-api media hosts are fetched.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use super::AssetsError;

const IMAGES_DIR: &str = "images";
const ALLOWED_PREFIX: &str = "https://media.valorant-api.com/";
const SOURCE_PARAM: &str = "src";
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Windows webviews cannot load custom schemes directly; Tauri maps them to `http://<scheme>.localhost`.
#[cfg(windows)]
const URL_PREFIX: &str = "http://asset-cache.localhost/";
#[cfg(not(windows))]
const URL_PREFIX: &str = "asset-cache://localhost/";

const DOWNLOAD_ATTEMPTS: u32 = 3;
const RETRY_BACKOFF: Duration = Duration::from_millis(400);

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// A cached image ready to serve.
#[derive(Debug)]
pub struct CachedImage {
    pub bytes: Vec<u8>,
    pub content_type: &'static str,
}

/// App-protocol url for a valorant-api image. The UI puts this in `src`.
pub fn image_url(original: &str) -> String {
    format!(
        "{URL_PREFIX}{}?{SOURCE_PARAM}={}",
        url_hash(original),
        percent_encode(original)
    )
}

/// FNV-1a 64 as 16 hex digits: stable, dependency-free, plenty for a few thousand urls.
pub fn url_hash(url: &str) -> String {
    let hash = url.bytes().fold(FNV_OFFSET, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME)
    });
    format!("{hash:016x}")
}

/// The original url from a request uri such as `asset-cache://localhost/<hash>?src=<encoded>`.
pub fn source_from_request_uri(uri: &str) -> Option<String> {
    let query = uri.split_once('?')?.1.split('#').next()?;
    query
        .split('&')
        .find_map(|pair| pair.strip_prefix("src="))
        .and_then(percent_decode)
}

pub fn is_allowed_source(url: &str) -> bool {
    url.starts_with(ALLOWED_PREFIX)
}

/// Content type from the url's file extension; the API only serves a handful of media kinds.
pub fn content_type_for(url: &str) -> &'static str {
    match extension_of(url).as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ => "application/octet-stream",
    }
}

fn extension_of(url: &str) -> String {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    let name = path.rsplit('/').next().unwrap_or(path);
    match name.rsplit_once('.') {
        Some((_, extension)) if extension.len() <= 5 => extension.to_ascii_lowercase(),
        _ => "bin".to_owned(),
    }
}

pub(super) fn cache_file(cache_dir: &Path, url: &str) -> PathBuf {
    cache_dir
        .join(IMAGES_DIR)
        .join(format!("{}.{}", url_hash(url), extension_of(url)))
}

/// Serve from disk, downloading on the first request.
pub(super) async fn load(
    http: &reqwest::Client,
    cache_dir: &Path,
    url: &str,
) -> Result<CachedImage, AssetsError> {
    if !is_allowed_source(url) {
        return Err(AssetsError::ImageNotAllowed(url.to_owned()));
    }
    let path = cache_file(cache_dir, url);
    let content_type = content_type_for(url);
    if let Ok(bytes) = tokio::fs::read(&path).await {
        return Ok(CachedImage {
            bytes,
            content_type,
        });
    }
    let bytes = download_with_retries(http, url).await?;
    save(&path, &bytes).await;
    Ok(CachedImage {
        bytes,
        content_type,
    })
}

/// A dropped connection or a busy CDN should not leave a blank card for the rest of the
/// session, so transient failures are retried a few times with a short backoff.
async fn download_with_retries(http: &reqwest::Client, url: &str) -> Result<Vec<u8>, AssetsError> {
    let mut attempt = 0;
    loop {
        match download(http, url).await {
            Ok(bytes) => return Ok(bytes),
            Err((error, is_transient)) if is_transient && attempt + 1 < DOWNLOAD_ATTEMPTS => {
                tracing::debug!(%error, attempt, "asset_image_retry");
                attempt += 1;
                tokio::time::sleep(RETRY_BACKOFF * attempt).await;
            }
            Err((error, _)) => return Err(error),
        }
    }
}

/// One download; the flag says whether trying again could help (network or 5xx, not 404).
async fn download(http: &reqwest::Client, url: &str) -> Result<Vec<u8>, (AssetsError, bool)> {
    let request_error = |error: reqwest::Error| {
        let is_transient = error
            .status()
            .is_none_or(|status| status.is_server_error() || status.as_u16() == 429);
        let error = AssetsError::Request {
            endpoint: url.to_owned(),
            reason: error.to_string(),
        };
        (error, is_transient)
    };
    let response = http.get(url).send().await.map_err(request_error)?;
    let bytes = response
        .error_for_status()
        .map_err(request_error)?
        .bytes()
        .await
        .map_err(request_error)?;
    Ok(bytes.to_vec())
}

/// A failed cache write only costs a re-download, so it is logged, not returned.
async fn save(path: &Path, bytes: &[u8]) {
    let id = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = path.to_path_buf();
    let bytes = bytes.to_vec();
    let written = tokio::task::spawn_blocking(move || {
        // Unique temp name so two requests for one image never share a half-written file.
        let temp = path.with_extension(format!("{id}.part"));
        write_via(&temp, &path, &bytes)
    })
    .await;
    if !matches!(written, Ok(Ok(()))) {
        tracing::warn!("asset_image_cache_write_failed");
    }
}

fn write_via(temp: &Path, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(temp, bytes)?;
    std::fs::rename(temp, path)
}

fn percent_encode(raw: &str) -> String {
    let mut encoded = String::with_capacity(raw.len() * 2);
    for byte in raw.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            // Writing to a String cannot fail.
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}

fn percent_decode(encoded: &str) -> Option<String> {
    let bytes = encoded.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut position = 0;
    while position < bytes.len() {
        if bytes[position] == b'%' {
            let hex = encoded.get(position + 1..position + 3)?;
            decoded.push(u8::from_str_radix(hex, 16).ok()?);
            position += 3;
        } else {
            decoded.push(bytes[position]);
            position += 1;
        }
    }
    String::from_utf8(decoded).ok()
}
