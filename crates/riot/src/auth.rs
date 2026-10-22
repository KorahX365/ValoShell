//! Remote auth headers.
//!
//! The access token lasts about an hour, so headers are refreshed from the local entitlements
//! endpoint when Riot rejects the token. Requests failing together must cost ONE refresh: a
//! mutex plus a 30 s minimum gap guarantees it. The refresh swaps only the two token headers
//! and keeps the client version taken from the game log.

use serde::Deserialize;

/// `X-Riot-ClientPlatform`: base64 of a fixed Windows platform JSON, byte-exact as the client sends.
pub const CLIENT_PLATFORM: &str = "ew0KCSJwbGF0Zm9ybVR5cGUiOiAiUEMiLA0KCSJwbGF0Zm9ybU9TIjogIldpbmRvd3MiLA0KCSJwbGF0Zm9ybU9TVmVyc2lvbiI6ICIxMC4wLjE5MDQyLjEuMjU2LjY0Yml0IiwNCgkicGxhdGZvcm1DaGlwc2V0IjogIlVua25vd24iDQp9";

/// Path of the local endpoint that mints tokens.
pub const ENTITLEMENTS_PATH: &str = "/entitlements/v1/token";

/// Answer of `GET /entitlements/v1/token` on the local client.
#[derive(Debug, Clone, Deserialize)]
pub struct EntitlementTokens {
    /// The signed-in player's puuid.
    pub subject: String,
    #[serde(rename = "accessToken")]
    pub access_token: String,
    /// The entitlements JWT.
    pub token: String,
}

impl EntitlementTokens {
    /// Both tokens present; an empty one means the client is not signed in yet.
    pub fn is_complete(&self) -> bool {
        !self.access_token.is_empty() && !self.token.is_empty() && !self.subject.is_empty()
    }
}

/// The header set sent to pd/glz/shared. Replaced whole on refresh (readers never see a mix).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RemoteHeaders {
    pub authorization: String,
    pub entitlements_jwt: String,
    pub client_version: String,
}

impl RemoteHeaders {
    pub fn new(tokens: &EntitlementTokens, client_version: &str) -> Self {
        Self {
            authorization: format!("Bearer {}", tokens.access_token),
            entitlements_jwt: tokens.token.clone(),
            client_version: client_version.to_owned(),
        }
    }

    /// Fresh tokens, same client version.
    pub fn with_tokens(&self, tokens: &EntitlementTokens) -> Self {
        Self::new(tokens, &self.client_version)
    }
}

/// Client version in the log's shape from valorant-api.com's `/v1/version` data:
/// `{branch}-shipping-{buildVersion}-{4th part of version}`. Fallback only; trails patches.
pub fn client_version_from_api(branch: &str, build_version: &str, version: &str) -> Option<String> {
    let build_number = version.split('.').nth(3)?;
    Some(format!("{branch}-shipping-{build_version}-{build_number}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;

    fn tokens(access: &str) -> EntitlementTokens {
        EntitlementTokens {
            subject: "me".into(),
            access_token: access.into(),
            token: "jwt".into(),
        }
    }

    #[test]
    fn client_platform_decodes_to_the_windows_platform_json() {
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(CLIENT_PLATFORM)
            .expect("b64");
        let text = String::from_utf8(decoded).expect("utf8");
        assert!(text.starts_with("{\r\n\t\"platformType\": \"PC\""));
    }

    #[test]
    fn refresh_keeps_the_log_client_version() {
        let headers = RemoteHeaders::new(&tokens("old"), "release-13.08-shipping-19-5698904");
        let refreshed = headers.with_tokens(&tokens("new"));
        assert_eq!(refreshed.authorization, "Bearer new");
        assert_eq!(
            refreshed.client_version,
            "release-13.08-shipping-19-5698904"
        );
    }

    #[test]
    fn incomplete_tokens_are_detected() {
        assert!(!tokens("").is_complete());
        assert!(tokens("a").is_complete());
    }

    #[test]
    fn api_version_is_shaped_like_the_log_version() {
        let version = client_version_from_api("release-13.08", "19", "13.08.00.5698904");
        assert_eq!(
            version.as_deref(),
            Some("release-13.08-shipping-19-5698904")
        );
        assert_eq!(client_version_from_api("b", "1", "13.08"), None);
    }
}
