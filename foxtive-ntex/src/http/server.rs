use crate::http::kernel::{ntex_default_service, register_routes, setup_cors, setup_logger, Route};
use crate::http::Method;
use crate::setup::{make_ntex_state, FoxtiveNtexSetup};
use crate::FoxtiveNtexState;
use foxtive::env_logger::init_env_logger;
use foxtive::prelude::AppResult;
use foxtive::setup::{get_server_host_config, load_environment_variables};
use log::error;
use ntex::web;
use std::future::Future;

pub struct ServerConfig<TB>
where
    TB: FnOnce() -> Vec<Route> + Send + Copy + 'static,
{
    pub app: String,
    pub env_prefix: String,
    pub private_key: String,
    pub public_key: String,
    pub auth_iss_public_key: String,

    #[cfg(feature = "static")]
    pub static_config: StaticFileConfig,

    /// whether the app bootstrap has started
    pub has_started_bootstrap: bool,

    /// list of allowed CORS origins
    pub allowed_origins: Vec<String>,

    /// list of allowed CORS origins
    pub allowed_methods: Vec<Method>,

    pub boot_thread: TB,
}

#[cfg(feature = "static")]
pub struct StaticFileConfig {
    pub path: String,
    pub dir: String,
}

pub fn init_bootstrap(service: &str) -> AppResult<()> {
    load_environment_variables(service);
    init_env_logger();
    Ok(())
}

pub async fn start_ntex_server<Callback, Fut, TB>(
    config: ServerConfig<TB>,
    callback: Callback,
) -> std::io::Result<()>
where
    Callback: FnOnce(FoxtiveNtexState) -> Fut + Copy + Send + 'static,
    Fut: Future<Output = AppResult<()>> + Send + 'static,
    TB: FnOnce() -> Vec<Route> + Send + Copy + 'static,
{
    if !config.has_started_bootstrap {
        init_bootstrap(&config.app).expect("failed to init bootstrap: ");
    }

    let app_state = make_ntex_state(FoxtiveNtexSetup {
        public_key: config.public_key,
        private_key: config.private_key,
        env_prefix: config.env_prefix.clone(),
        auth_iss_public_key: config.auth_iss_public_key,
        allowed_origins: config.allowed_origins,
        allowed_methods: config.allowed_methods,
    })
    .await;

    let (host, port, workers) = get_server_host_config(&config.env_prefix);

    match callback(app_state.clone()).await {
        Ok(_) => {}
        Err(err) => {
            error!("app bootstrap callback returned error: {:?}", err);
            panic!("boostrap failed");
        }
    }

    let boot = config.boot_thread;
    web::HttpServer::new(move || {
        let routes = boot();
        let app = web::App::new()
            .state(app_state.clone())
            .configure(|cfg| register_routes(cfg, routes))
            .wrap(setup_logger())
            .wrap(
                setup_cors(
                    app_state.allowed_origins.clone(),
                    app_state.allowed_methods.clone(),
                )
                .finish(),
            )
            .default_service(ntex_default_service());

        if cfg!(feature = "static") {
            #[cfg(feature = "static")]
            {
                return app.service(ntex_files::Files::new(
                    &config.static_config.path,
                    &config.static_config.dir,
                ));
            }
        }

        app
    })
    .bind((host, port))?
    .workers(workers)
    .run()
    .await
}
