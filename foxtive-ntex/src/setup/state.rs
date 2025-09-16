use crate::http::Method;
use std::fmt::{Debug, Formatter};
use crate::http::server::JsonConfig;

#[derive(Clone)]
pub struct FoxtiveNtexState {
    /// list of allowed origins
    pub allowed_origins: Vec<String>,

    /// list of allowed methods
    pub allowed_methods: Vec<Method>,

    /// Json body config
    pub json_config: JsonConfig,
}

impl Debug for FoxtiveNtexState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("application state")
    }
}
