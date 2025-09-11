use crate::FOXTIVE_NTEX;
use crate::http::Method;
use foxtive::prelude::AppMessage;
use foxtive::results::AppResult;
use foxtive::setup::FoxtiveSetup;
use state::FoxtiveNtexState;
use tracing::debug;

pub mod state;

pub struct FoxtiveNtexSetup {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<Method>,
    pub foxtive_setup: FoxtiveSetup,
}

pub async fn make_ntex_state(setup: FoxtiveNtexSetup) -> AppResult<FoxtiveNtexState> {
    let app = create_app_state(&setup).await;

    debug!("Creating Foxtive state");
    foxtive::setup::make_state(setup.foxtive_setup).await?;

    FOXTIVE_NTEX.set(app.clone()).map_err(|_| {
        AppMessage::InternalServerErrorMessage("failed to set up foxtive-ntex").ae()
    })?;

    Ok(app)
}

async fn create_app_state(setup: &FoxtiveNtexSetup) -> FoxtiveNtexState {
    FoxtiveNtexState {
        allowed_origins: setup.allowed_origins.clone(),
        allowed_methods: setup.allowed_methods.clone(),
    }
}
