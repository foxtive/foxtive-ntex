use crate::http::Method;
use crate::http::server::BodyConfig;
use std::any::Any;
use std::collections::HashMap;

pub mod state;

/// Custom state builder function type
type CustomStateBuilder = Box<dyn FnOnce() -> HashMap<String, Box<dyn Any + Send + Sync>> + Send>;

/// Internal structure used during server bootstrapping
pub struct NtexSetup {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<Method>,
    pub body_config: BodyConfig,
    pub custom_state_builder: Option<CustomStateBuilder>,
}

impl NtexSetup {
    pub fn new() -> Self {
        Self {
            allowed_origins: Vec::new(),
            allowed_methods: Vec::new(),
            body_config: BodyConfig::default(),
            custom_state_builder: None,
        }
    }
}

impl Default for NtexSetup {
    fn default() -> Self {
        Self::new()
    }
}

/// Build the ntex-specific AppState from setup parameters
pub fn build_app_state(setup: NtexSetup) -> state::AppState {
    let custom_state = setup
        .custom_state_builder
        .map(|builder| builder())
        .unwrap_or_default();

    state::AppState::new(
        setup.allowed_origins,
        setup.allowed_methods,
        setup.body_config,
        custom_state,
    )
}
