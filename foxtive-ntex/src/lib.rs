use std::sync::OnceLock;

pub mod contracts;
pub mod enums;
mod error;
pub mod helpers;
pub mod http;
mod setup;

pub use setup::state::FoxtiveNtexState;

pub static FOXTIVE_NTEX: OnceLock<FoxtiveNtexState> = OnceLock::new();

pub use crate::helpers::once_lock::FoxtiveNtexExt;
