use crate::http::Method;
use crate::FOXTIVE_NTEX;
use foxtive::setup::FoxtiveSetup;
use state::FoxtiveNtexState;

pub mod state;

pub struct FoxtiveNtexSetup {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<Method>,
    pub foxtive_setup: FoxtiveSetup,
}

pub async fn make_ntex_state(setup: FoxtiveNtexSetup) -> FoxtiveNtexState {
    let app = create_app_state(&setup).await;

    foxtive::setup::make_state(setup.foxtive_setup).await;

    FOXTIVE_NTEX
        .set(app.clone())
        .expect("failed to set up foxtive-ntex");

    app
}

async fn create_app_state(setup: &FoxtiveNtexSetup) -> FoxtiveNtexState {
    FoxtiveNtexState {
        allowed_origins: setup.allowed_origins.clone(),
        allowed_methods: setup.allowed_methods.clone(),
    }
}
