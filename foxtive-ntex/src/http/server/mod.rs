mod config;

#[cfg(feature = "static")]
pub use config::StaticFileConfig;
pub use config::{JsonConfig, ServerConfig};

use crate::http::kernel::{ntex_default_service, register_routes, setup_cors, setup_logger};
use crate::setup::{make_ntex_state, FoxtiveNtexSetup};
use crate::FoxtiveNtexState;
use foxtive::prelude::AppResult;
use foxtive::setup::load_environment_variables;
use foxtive::setup::trace::Tracing;
use foxtive::Error;
use ntex::web;
use std::future::Future;
use tracing::{debug, error};

pub fn init_bootstrap(service: &str, config: Tracing) -> AppResult<()> {
    foxtive::setup::trace::init_tracing(config)?;
    load_environment_variables(service);
    Ok(())
}

pub async fn start_ntex_server<Callback, Fut>(
    config: ServerConfig,
    callback: Callback,
) -> AppResult<()>
where
    Callback: FnOnce(FoxtiveNtexState) -> Fut + Copy + Send + 'static,
    Fut: Future<Output = AppResult<()>> + Send + 'static,
{
    if !config.has_started_bootstrap {
        let t_config = config.tracing.unwrap_or_default();
        debug!("Starting bootstrap");
        init_bootstrap(&config.app, t_config).expect("failed to init bootstrap: ");
    }

    debug!("Creating Foxtive-Ntex state");
    let json_config = config.json_config.unwrap_or_default();
    let app_state = make_ntex_state(FoxtiveNtexSetup {
        allowed_origins: config.allowed_origins,
        allowed_methods: config.allowed_methods,
        foxtive_setup: config.foxtive_setup,
        json_config: json_config.clone(),
    })
    .await?;

    debug!("Executing app bootstrap callback");
    match callback(app_state.clone()).await {
        Ok(_) => {}
        Err(err) => {
            error!("app bootstrap callback returned error: {err:?}");
            panic!("boostrap failed");
        }
    }

    let boot = config.boot_thread;
    let ntex_json_config = web::types::JsonConfig::default().limit(json_config.limit);

    web::HttpServer::new(move || {
        let routes = boot();

        let app = web::App::new()
            .state(ntex_json_config.clone())
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
    .backlog(config.backlog)
    .workers(config.workers)
    .maxconn(config.max_connections)
    .maxconnrate(config.max_connections_rate)
    .keep_alive(config.keep_alive)
    .bind((config.host, config.port))?
    .run()
    .await
    .map_err(Error::from)
}
