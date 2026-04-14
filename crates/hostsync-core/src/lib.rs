pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod model;
pub mod crypto;
pub mod ssh_config;
pub mod storage;
#[cfg(feature = "network")]
pub mod http;
#[cfg(feature = "network")]
pub mod auth;
#[cfg(feature = "network")]
pub mod sync;
pub mod terminal;
pub mod ffi;
