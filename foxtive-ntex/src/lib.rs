use std::sync::OnceLock;

pub mod contracts;
pub mod enums;
mod error;
pub mod helpers;
pub mod http;
mod setup;
pub mod ext;

pub use setup::state::FoxtiveNtexState;

pub static FOXTIVE_NTEX: OnceLock<FoxtiveNtexState> = OnceLock::new();

pub use ext::app_state::FoxtiveNtexExt;
