//! Store wishlist. Read only; the write endpoints are not implemented.

use domain::ids::Puuid;
use serde_json::Value;

use crate::http::{Host, Request};

/// `GET /favorites/v1/players/{puuid}/favorites`.
///
/// UNVERIFIED on 13.08: the path is seen in the client logs, the body shape is not known.
pub fn get_favorites(puuid: &Puuid) -> Request {
    Request::get(Host::Pd, format!("/favorites/v1/players/{puuid}/favorites"))
}

/// The wishlist stays a raw tree until its shape is verified.
pub type Favorites = Value;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn favorites_request_targets_the_player() {
        let request = get_favorites(&Puuid::new("abc"));

        assert_eq!(request.method, reqwest::Method::GET);
        assert_eq!(request.path, "/favorites/v1/players/abc/favorites");
    }
}
