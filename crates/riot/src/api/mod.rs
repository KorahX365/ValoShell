//! Typed Riot endpoints: request builders + response DTOs. No I/O here; callers send the
//! `Request` through `RiotHttp` (mutations) or `RateGate` (paced reads).

mod codec;
pub mod glz;
pub mod local;
pub mod pd;
pub mod shared;
