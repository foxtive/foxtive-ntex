mod config;

#[cfg(feature = "static")]
pub use config::StaticFileConfig;
pub use config::{BodyConfig, ServerBuilder};
#[allow(deprecated)]
pub use config::JsonConfig;

use crate::FoxtiveNtexState;
use crate::http::kernel::{ntex_default_service, register_routes, setup_cors, setup_logger};
use crate::setup::{FoxtiveNtexSetup, make_ntex_state};
use foxtive::Error;
use foxtive::prelude::AppResult;
use foxtive::setup::load_environment_variables;
use foxtive::setup::trace::Tracing;
use ntex::io::IoConfig;
use ntex::{SharedCfg, web};
use std::future::Future;
use tracing::{debug, error};

pub fn init_bootstrap(service: &str, config: Tracing) -> AppResult<()> {
    foxtive::setup::trace::init_tracing(config)?;
    load_environment_variables(service);
    Ok(())
}

pub async fn start_ntex_server<Callback, Fut>(
    builder: ServerBuilder,
    callback: Callback,
) -> AppResult<()>
where
    Callback: FnOnce(FoxtiveNtexState) -> Fut + Copy + Send + 'static,
    Fut: Future<Output = AppResult<()>> + Send + 'static,
{
    if !builder.has_started_bootstrap {
        let t_config = builder.tracing.unwrap_or_default();
        debug!("Starting bootstrap");
        init_bootstrap(&builder.app, t_config).expect("failed to init bootstrap: ");
    }

    debug!("Creating Foxtive-Ntex state");
    let body_config = builder.body_config.unwrap_or_default();
    let custom_state_builder = builder.custom_state_builder;
    let app_state = make_ntex_state(FoxtiveNtexSetup {
        allowed_origins: builder.allowed_origins,
        allowed_methods: builder.allowed_methods,
        foxtive_setup: builder.foxtive_setup,
        body_config: body_config.clone(),
        custom_state_builder,
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

    let route_factory = builder.route_factory;
    let ntex_json_config = web::types::JsonConfig::default().limit(body_config.json_limit);

    let shared_config = SharedCfg::new("WEB").add(
        IoConfig::new()
            .set_keepalive_timeout(builder.keep_alive)
            .set_connect_timeout(builder.client_timeout)
            .set_disconnect_timeout(builder.client_disconnect),
    );

    let server = web::HttpServer::new(async move || {
        let routes = route_factory();

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
                    &builder.static_config.path,
                    &builder.static_config.dir,
                ));
            }
        }

        app
    })
    .config(shared_config)
    .backlog(builder.backlog)
    .workers(builder.workers)
    .maxconn(builder.max_connections)
    .maxconnrate(builder.max_connections_rate)
    // .keep_alive(config.keep_alive)
    .bind((builder.host, builder.port))?
    .run();

    // clone server handle
    let srv = server.clone();

    // use provided shutdown signal or default
    let shutdown_signal = builder
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
    if let Some(on_shutdown) = builder.on_shutdown {
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
