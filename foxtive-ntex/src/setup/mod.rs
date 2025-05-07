use crate::http::Method;
use crate::FOXTIVE_NTEX;
use foxtive::setup::FoxtiveSetup;
use state::FoxtiveNtexState;

pub mod state;

pub struct FoxtiveNtexSetup {
    pub env_prefix: String,
    pub private_key: String,
    pub public_key: String,
    pub auth_iss_public_key: String,
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<Method>,
    #[cfg(feature = "cache")]
    pub cache_driver_setup: foxtive::setup::CacheDriverSetup,
}

pub async fn make_ntex_state(setup: FoxtiveNtexSetup) -> FoxtiveNtexState {
    let app = create_app_state(&setup).await;

    foxtive::setup::make_app_state(FoxtiveSetup {
        env_prefix: setup.env_prefix,
        private_key: setup.private_key,
        public_key: setup.public_key,
        auth_iss_public_key: setup.auth_iss_public_key,
        #[cfg(feature = "cache")]
        cache_driver_setup: setup.cache_driver_setup,
    })
    .await;

    FOXTIVE_NTEX
        .set(app.clone())
        .expect("failed to set up foxtive-ntex");

    app
}

async fn create_app_state(setup: &FoxtiveNtexSetup) -> FoxtiveNtexState {
    FoxtiveNtexState {
        #[cfg(feature = "jwt")]
        jwt_secret: std::env::var(format!("{}_JWT_SECRET", setup.env_prefix)).unwrap(),

        allowed_origins: setup.allowed_origins.clone(),
        allowed_methods: setup.allowed_methods.clone(),
    }
}
