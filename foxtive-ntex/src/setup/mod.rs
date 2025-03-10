use state::FoxtiveNtexState;
use crate::http::Method;
use crate::FOXTIVE_WEB;
use foxtive::setup::FoxtiveSetup;

pub mod state;

pub struct FoxtiveNtexSetup {
    pub env_prefix: String,
    pub private_key: String,
    pub public_key: String,
    pub auth_iss_public_key: String,
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<Method>,
}

pub async fn make_ntex_state(setup: FoxtiveNtexSetup) -> FoxtiveNtexState {
    foxtive::setup::make_app_state(FoxtiveSetup {
        env_prefix: setup.env_prefix.clone(),
        private_key: setup.private_key.clone(),
        public_key: setup.public_key.clone(),
        auth_iss_public_key: setup.auth_iss_public_key.clone(),
    })
        .await;

    let app = create_app_state(setup).await;

    FOXTIVE_WEB
        .set(app.clone())
        .expect("failed to set up foxtive-ntex");

    app
}

async fn create_app_state(setup: FoxtiveNtexSetup) -> FoxtiveNtexState {
    FoxtiveNtexState {
        #[cfg(feature = "jwt")]
        auth_pat_prefix: std::env::var(format!("{}_AUTH_PAT_PREFIX", setup.env_prefix)).unwrap(),

        allowed_origins: setup.allowed_origins,
        allowed_methods: setup.allowed_methods,
    }
}
