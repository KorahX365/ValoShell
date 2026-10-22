//! One Riot Client session: everything that depends on the lockfile, rebuilt
//! whenever the Riot Client restarts. Never reused across launches.

use std::sync::Arc;

use domain::ids::Puuid;
use domain::region::Region;
use serde::Deserialize;

use crate::api::local::session::{ChatSession, get_chat_session};
use crate::auth::{ENTITLEMENTS_PATH, EntitlementTokens, RemoteHeaders, client_version_from_api};
use crate::error::{RiotError, RiotResult};
use crate::gate::RateGate;
use crate::http::{Host, Request, RiotHttp};
use crate::lockfile::{Lockfile, read_lockfile};
use crate::shooter_log::read_log_facts;

const VERSION_API_URL: &str = "https://valorant-api.com/v1/version";

/// A connected session.
pub struct RiotSession {
    pub lockfile: Lockfile,
    pub puuid: Puuid,
    /// `Name#Tag`, when the chat session reports it.
    pub riot_id: Option<String>,
    pub http: Arc<RiotHttp>,
    pub gate: Arc<RateGate>,
}

impl RiotSession {
    /// Connect to the running Riot Client. `fallback_region` is the last region from settings,
    /// used only when the game log does not name one.
    pub async fn connect(fallback_region: Option<&str>) -> RiotResult<Self> {
        let lockfile = read_lockfile().await?;
        let facts = read_log_facts().await;
        let region = Region::resolve(&facts.region)
            .or_else(|| fallback_region.and_then(Region::resolve))
            .ok_or(RiotError::RegionUnknown)?;
        let http = RiotHttp::new(&lockfile, region, RemoteHeaders::default())?;
        let tokens = exchange_tokens(&http).await?;
        let client_version = running_client_version(facts.client_version).await;
        http.replace_headers(RemoteHeaders::new(&tokens, &client_version));
        let riot_id = read_riot_id(&http).await;
        tracing::info!(region = %http.region().region, %client_version, "riot_session_connected");
        Ok(Self {
            lockfile,
            puuid: Puuid::new(&tokens.subject),
            riot_id,
            http: Arc::new(http),
            gate: Arc::new(RateGate::default()),
        })
    }
}

async fn exchange_tokens(http: &RiotHttp) -> RiotResult<EntitlementTokens> {
    let reply = http
        .send(&Request::get(Host::Local, ENTITLEMENTS_PATH))
        .await;
    if !reply.is_success() {
        return Err(RiotError::Handshake {
            status: reply.status,
        });
    }
    let tokens: EntitlementTokens = reply.json(ENTITLEMENTS_PATH)?;
    if !tokens.is_complete() {
        return Err(RiotError::Handshake {
            status: reply.status,
        });
    }
    Ok(tokens)
}

/// The log's build when present; valorant-api.com's (which trails patches) otherwise.
async fn running_client_version(from_log: String) -> String {
    if !from_log.is_empty() {
        return from_log;
    }
    match fetch_api_version().await {
        Some(version) => version,
        None => {
            tracing::warn!("riot_client_version_unknown");
            String::new()
        }
    }
}

#[derive(Deserialize)]
struct VersionEnvelope {
    data: VersionData,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VersionData {
    branch: String,
    build_version: String,
    version: String,
}

async fn fetch_api_version() -> Option<String> {
    crate::tls::install_crypto_provider();
    let response = reqwest::Client::new()
        .get(VERSION_API_URL)
        .timeout(crate::http::REQUEST_TIMEOUT)
        .send()
        .await
        .ok()?;
    let envelope: VersionEnvelope = response.json().await.ok()?;
    let data = envelope.data;
    client_version_from_api(&data.branch, &data.build_version, &data.version)
}

/// Display name only; a failure here never blocks connecting.
async fn read_riot_id(http: &RiotHttp) -> Option<String> {
    let session: ChatSession = http.send_json(&get_chat_session()).await.ok()?;
    session.riot_id()
}
