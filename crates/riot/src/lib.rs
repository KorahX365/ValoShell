//! HTTP and websocket access to the local Riot Client and Riot remote hosts.

pub mod api;
pub mod auth;
pub mod error;
pub mod gate;
pub mod http;
pub mod lockfile;
pub mod session;
pub mod shooter_log;
pub mod tls;
pub mod ws;

pub use error::{RiotError, RiotResult};
pub use gate::{CachePolicy, Fetched, RateGate};
pub use http::{Host, Reply, Request, RiotHttp};
