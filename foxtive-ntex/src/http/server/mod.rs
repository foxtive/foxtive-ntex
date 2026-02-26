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
use ntex::io::IoConfig;
use ntex::{web, SharedCfg};
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

    let shared_config = SharedCfg::new("WEB").add(
        IoConfig::new()
            .set_keepalive_timeout(config.keep_alive)
            .set_connect_timeout(config.client_timeout)
            .set_disconnect_timeout(config.client_disconnect),
    );

    let server = web::HttpServer::new(async move || {
        let routes = boot();

        let app = web::App::new()
            .state(ntex_json_config.clone())
            .state(app_state.clone())
            .configure(|cfg| register_routes(cfg, routes))
            .middleware(setup_logger())
            .middleware(
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
    .config(shared_config)
    .backlog(config.backlog)
    .workers(config.workers)
    .maxconn(config.max_connections)
    .maxconnrate(config.max_connections_rate)
    // .keep_alive(config.keep_alive)
    .bind((config.host, config.port))?
    .run();

    // clone server handle
    let srv = server.clone();

    // use provided shutdown signal or default
    let shutdown_signal = config
        .shutdown_signal
        .unwrap_or_else(default_shutdown_signal);

    // spawn shutdown listener
    ntex::rt::spawn(async move {
        shutdown_signal.await;

        debug!("Shutdown signal received");

        // graceful stop
        srv.stop(true).await;
    });

    // await server
    server.await.map_err(Error::from)?;

    // AFTER server fully stops, run cleanup handler
    if let Some(on_shutdown) = config.on_shutdown {
        debug!("Running shutdown handler");
        on_shutdown.await;
    }

    Ok(())
}

use std::pin::Pin;

pub fn default_shutdown_signal() -> Pin<Box<dyn Future<Output = ()> + Send>> {
    Box::pin(async {
        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to listen for ctrl_c");
        };

        #[cfg(unix)]
        let terminate = async {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigterm =
                signal(SignalKind::terminate()).expect("failed to listen for SIGTERM");
            sigterm.recv().await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }
    })
}
