mod config;

pub use config::ServerConfig;
#[cfg(feature = "static")]
pub use config::StaticFileConfig;

use crate::FoxtiveNtexState;
use crate::http::kernel::{Route, ntex_default_service, register_routes, setup_cors, setup_logger};
use crate::setup::{FoxtiveNtexSetup, make_ntex_state};
use foxtive::prelude::AppResult;
use foxtive::setup::load_environment_variables;
use log::error;
use ntex::web;
use std::future::Future;
use foxtive::setup::logger::TracingConfig;

pub fn init_bootstrap(service: &str, config: TracingConfig) -> AppResult<()> {
    foxtive::setup::logger::init_tracing(config)?;
    load_environment_variables(service);
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
        let t_config = config.tracing_config.unwrap_or_default();
        init_bootstrap(&config.app, t_config).expect("failed to init bootstrap: ");
    }

    let app_state = make_ntex_state(FoxtiveNtexSetup {
        allowed_origins: config.allowed_origins,
        allowed_methods: config.allowed_methods,
        foxtive_setup: config.foxtive_setup,
    })
    .await;

    match callback(app_state.clone()).await {
        Ok(_) => {}
        Err(err) => {
            error!("app bootstrap callback returned error: {err:?}");
            panic!("boostrap failed");
        }
    }

    let boot = config.boot_thread;
    let alt_routes = config.routes;

    web::HttpServer::new(move || {
        let routes = match boot {
            None => alt_routes.clone(),
            Some(boot) => boot(),
        };

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
    .backlog(config.backlog)
    .workers(config.workers)
    .maxconn(config.max_connections)
    .maxconnrate(config.max_connections_rate)
    .keep_alive(config.keep_alive)
    .bind((config.host, config.port))?
    .run()
    .await
}
