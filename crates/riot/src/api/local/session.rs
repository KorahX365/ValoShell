//! Own identity and the running game process.

use std::collections::HashMap;

use domain::ids::Puuid;
use serde::Deserialize;

use crate::http::{Host, Request};

/// Argument the Riot Client passes to the game naming the deployment region.
const ARES_DEPLOYMENT_FLAG: &str = "-ares-deployment=";
const VALORANT_PRODUCT_ID: &str = "valorant";

/// `GET /chat/v1/session`: own Riot ID, puuid and chat connection state.
pub fn get_chat_session() -> Request {
    Request::get(Host::Local, "/chat/v1/session")
}

/// `GET /product-session/v1/external-sessions`: running game processes by session id.
pub fn get_external_sessions() -> Request {
    Request::get(Host::Local, "/product-session/v1/external-sessions")
}

/// Own chat session; `state` is `disconnected|connecting|connected` (a guess in the spec).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ChatSession {
    pub federated: bool,
    pub game_name: String,
    pub game_tag: String,
    pub loaded: bool,
    pub name: String,
    pub pid: String,
    pub puuid: Option<Puuid>,
    pub region: String,
    pub resource: String,
    pub state: String,
}

impl ChatSession {
    /// Chat is signed in and ready (the Riot ID is only trustworthy then).
    pub fn is_connected(&self) -> bool {
        self.loaded && self.state == "connected"
    }

    /// `Name#TAG`, or `None` while the Riot ID is not loaded yet.
    pub fn riot_id(&self) -> Option<String> {
        (!self.game_name.is_empty()).then(|| format!("{}#{}", self.game_name, self.game_tag))
    }
}

/// How the Riot Client launched a product.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LaunchConfiguration {
    pub executable: String,
    pub arguments: Vec<String>,
    pub locale: String,
    pub working_directory: String,
}

/// One entry of the external-sessions map.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ExternalSession {
    pub product_id: String,
    pub patchline_full_name: String,
    pub patchline_id: String,
    pub phase: String,
    pub version: String,
    pub exit_code: Option<i64>,
    pub is_internal: bool,
    pub launch_configuration: LaunchConfiguration,
}

impl ExternalSession {
    /// This entry is the VALORANT game (not the Riot Client itself).
    pub fn is_valorant(&self) -> bool {
        self.product_id == VALORANT_PRODUCT_ID
    }

    /// The region from `-ares-deployment={region}`, when launched with it.
    pub fn ares_deployment(&self) -> Option<&str> {
        self.launch_configuration
            .arguments
            .iter()
            .find_map(|argument| argument.strip_prefix(ARES_DEPLOYMENT_FLAG))
            .filter(|region| !region.is_empty())
    }
}

/// The map `sessionId -> session` returned by the external-sessions endpoint.
pub type ExternalSessions = HashMap<String, ExternalSession>;

/// The VALORANT entry, if the game has been launched by this Riot Client.
pub fn find_valorant_session(sessions: &ExternalSessions) -> Option<&ExternalSession> {
    sessions.values().find(|session| session.is_valorant())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHAT_SESSION: &str = r#"{"federated":false,"game_name":"Tester","game_tag":"EUW",
        "loaded":true,"name":"Tester","pid":"abc@eu1.pvp.net","puuid":"AAAA-bbbb",
        "region":"eu1","resource":"RC-1","state":"connected"}"#;
    const EXTERNAL_SESSIONS: &str = r#"{
        "rc": {"productId":"riot_client","phase":"","launchConfiguration":{"arguments":[]}},
        "vl": {"productId":"valorant","patchlineId":"live","phase":"running","exitCode":null,
               "isInternal":false,"version":"13.08",
               "launchConfiguration":{"executable":"VALORANT.exe",
               "arguments":["-remoting-auth-token=x","-ares-deployment=eu"]}}}"#;

    #[test]
    fn builders_target_local_host() {
        let chat = get_chat_session();
        let sessions = get_external_sessions();

        assert_eq!(chat.host, Host::Local);
        assert_eq!(chat.method, reqwest::Method::GET);
        assert_eq!(chat.path, "/chat/v1/session");
        assert_eq!(sessions.host, Host::Local);
        assert_eq!(sessions.path, "/product-session/v1/external-sessions");
    }

    #[test]
    fn chat_session_decodes_identity() {
        let session: ChatSession = serde_json::from_str(CHAT_SESSION).expect("decodes");

        assert!(session.is_connected());
        assert_eq!(session.riot_id().as_deref(), Some("Tester#EUW"));
        assert_eq!(session.puuid.expect("puuid").as_str(), "AAAA-bbbb");
    }

    #[test]
    fn empty_chat_session_is_not_connected() {
        let session: ChatSession = serde_json::from_str("{}").expect("decodes");

        assert!(!session.is_connected());
        assert_eq!(session.riot_id(), None);
    }

    #[test]
    fn valorant_session_is_found_among_products() {
        let sessions: ExternalSessions = serde_json::from_str(EXTERNAL_SESSIONS).expect("decodes");

        let valorant = find_valorant_session(&sessions).expect("valorant entry");

        assert_eq!(valorant.patchline_id, "live");
        assert_eq!(valorant.ares_deployment(), Some("eu"));
        assert_eq!(valorant.exit_code, None);
    }

    #[test]
    fn riot_client_alone_means_valorant_is_not_running() {
        let sessions: ExternalSessions =
            serde_json::from_str(r#"{"rc":{"productId":"riot_client"}}"#).expect("decodes");

        assert!(find_valorant_session(&sessions).is_none());
        assert!(find_valorant_session(&ExternalSessions::new()).is_none());
    }
}
