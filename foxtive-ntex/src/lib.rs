use std::sync::OnceLock;

pub mod contracts;
pub mod enums;
pub mod helpers;
pub mod http;
mod setup;

pub use setup::state::FoxtiveNtexState;

pub static FOXTIVE_WEB: OnceLock<FoxtiveNtexState> = OnceLock::new();

pub mod prelude {
    pub use crate::helpers::once_lock::WebOnceLockHelper;
    pub use crate::http::HttpResult;
    pub use crate::setup::state::FoxtiveNtexState;
    pub use crate::FOXTIVE_WEB;
}
