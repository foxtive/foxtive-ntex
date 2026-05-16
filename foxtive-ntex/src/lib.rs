use std::sync::OnceLock;

pub mod contracts;
pub mod enums;
mod error;
pub mod ext;
pub mod helpers;
pub mod http;
pub mod setup;

pub use setup::state::{AppState, FoxtiveNtexState};
pub use http::shutdown::{ShutdownConfig, ShutdownRegistry};
pub use http::server::ServerBuilder;

pub static FOXTIVE_NTEX: OnceLock<FoxtiveNtexState> = OnceLock::new();

pub use ext::app_state::{fox_state, FoxtiveNtexExt};
