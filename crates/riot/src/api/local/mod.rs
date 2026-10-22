//! Endpoints on the local Riot Client host (`https://127.0.0.1:{port}`)
//! Pure request builders and response DTOs; sending is the caller's job. Never implemented on
//! purpose: `PUT chat/v2/me` (overwrites the game's presence blob) and `Session_*` calls.

pub mod chat;
pub mod events;
pub mod friends;
pub mod session;
pub mod status;

/// Append `?key=value&...` for the present values, percent-encoding each value.
pub(crate) fn with_query(path: &str, pairs: &[(&str, Option<&str>)]) -> String {
    let query: Vec<String> = pairs
        .iter()
        .filter_map(|(key, value)| Some(format!("{key}={}", percent_encode((*value)?))))
        .collect();
    if query.is_empty() {
        return path.to_owned();
    }
    format!("{path}?{}", query.join("&"))
}

/// Encode everything outside the RFC 3986 unreserved set (conversation ids contain `@`).
fn percent_encode(raw: &str) -> String {
    let mut encoded = String::with_capacity(raw.len());
    for byte in raw.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(char::from(byte));
            }
            other => encoded.push_str(&format!("%{other:02X}")),
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_skips_absent_values() {
        let path = with_query("/x", &[("a", None), ("b", Some("1"))]);

        assert_eq!(path, "/x?b=1");
    }

    #[test]
    fn query_without_values_leaves_path_untouched() {
        assert_eq!(with_query("/x", &[("a", None)]), "/x");
    }

    #[test]
    fn query_encodes_conversation_id_separators() {
        let path = with_query("/x", &[("cid", Some("ab@ares-parties.eu1.pvp.net"))]);

        assert_eq!(path, "/x?cid=ab%40ares-parties.eu1.pvp.net");
    }
}
