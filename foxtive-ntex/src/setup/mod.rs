use foxtive::internal_server_error;
use crate::FOXTIVE_NTEX;
use crate::http::Method;
use crate::http::server::BodyConfig;
use foxtive::results::AppResult;
use foxtive::setup::FoxtiveSetup;
use state::{AppState, FoxtiveNtexState};
use std::any::Any;
use std::collections::HashMap;
use tracing::debug;

pub mod state;

type CustomStateBuilder = Box<dyn FnOnce() -> HashMap<String, Box<dyn Any + Send + Sync>> + Send>;

pub struct FoxtiveNtexSetup {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<Method>,
    pub foxtive_setup: FoxtiveSetup,
    pub body_config: BodyConfig,
    pub custom_state_builder: Option<CustomStateBuilder>,
}

pub async fn make_ntex_state(setup: FoxtiveNtexSetup) -> AppResult<FoxtiveNtexState> {
    let custom_state_builder = setup.custom_state_builder;
    let foxtive_setup = setup.foxtive_setup;
    let allowed_origins = setup.allowed_origins;
    let allowed_methods = setup.allowed_methods;
    let body_config = setup.body_config;
    
    let custom_state = custom_state_builder
        .map(|builder| builder())
        .unwrap_or_default();
    
    let app = create_app_state(allowed_origins, allowed_methods, body_config, custom_state);

    debug!("Creating Foxtive state");
    foxtive::setup::make_state(foxtive_setup).await?;

    FOXTIVE_NTEX.set(app.clone()).map_err(|_| {
        internal_server_error!("failed to set up foxtive-ntex")
    })?;

    Ok(app)
}

fn create_app_state(
    allowed_origins: Vec<String>,
    allowed_methods: Vec<Method>,
    body_config: BodyConfig,
    custom_state: HashMap<String, Box<dyn Any + Send + Sync>>,
) -> FoxtiveNtexState {
    AppState::new(
        allowed_origins,
        allowed_methods,
        body_config,
        custom_state,
    )
}
