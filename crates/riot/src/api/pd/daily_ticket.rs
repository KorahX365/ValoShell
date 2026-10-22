//! Daily ticket reads. `renew` changes server state and is intentionally absent.

use domain::ids::Puuid;
use serde_json::Value;

use crate::http::{Host, Request};

/// `GET /daily-ticket/v1/{puuid}`.
///
/// UNVERIFIED on 13.08: path seen in client logs, body shape unknown.
pub fn get_daily_ticket(puuid: &Puuid) -> Request {
    Request::get(Host::Pd, format!("/daily-ticket/v1/{puuid}"))
}

/// `GET /daily-ticket/v1/queue-config`.
///
/// UNVERIFIED on 13.08: path seen in client logs, body shape unknown.
pub fn get_daily_ticket_queue_config() -> Request {
    Request::get(Host::Pd, "/daily-ticket/v1/queue-config")
}

/// Daily-ticket bodies stay raw until their shape is verified.
pub type DailyTicket = Value;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticket_requests_use_the_logged_paths() {
        assert_eq!(
            get_daily_ticket(&Puuid::new("abc")).path,
            "/daily-ticket/v1/abc"
        );
        assert_eq!(
            get_daily_ticket_queue_config().path,
            "/daily-ticket/v1/queue-config"
        );
    }
}
