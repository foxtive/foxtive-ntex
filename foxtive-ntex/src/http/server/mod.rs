mod config;

#[cfg(feature = "static")]
pub use config::StaticFileConfig;
pub use config::{BodyConfig, ServerBuilder};
#[allow(deprecated)]
pub use config::JsonConfig;

use crate::http::kernel::{ntex_default_service, register_routes, setup_cors, setup_logger};
use crate::setup::{NtexSetup, build_app_state};
use foxtive::prelude::AppResult;
use foxtive::setup::load_environment_variables;
use foxtive::setup::trace::Tracing;
use foxtive::App;
use ntex::io::IoConfig;
use ntex::{SharedCfg, web};
use std::future::Future;
use std::sync::Arc;
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
    Callback: FnOnce(Arc<App>) -> Fut + Copy + Send + 'static,
    Fut: Future<Output = AppResult<()>> + Send + 'static,
{
    let app = builder.app.clone();

    if !builder.has_started_bootstrap {
        let t_config = builder.tracing.unwrap_or_default();
        debug!("Starting bootstrap");
        init_bootstrap(app.app_name(), t_config)?;
    }

    debug!("Creating ntex app state");
    let body_config = builder.body_config.clone().unwrap_or_default();
    let custom_state_builder = builder.custom_state_builder;
    let app_state = build_app_state(NtexSetup {
        allowed_origins: builder.allowed_origins,
        allowed_methods: builder.allowed_methods,
        body_config: body_config.clone(),
        custom_state_builder,
    });

    debug!("Executing app bootstrap callback");
    if let Err(err) = callback(app.clone()).await {
        error!("app bootstrap callback returned error: {err:?}");
        app.shutdown().await;
        return Err(err);
    }

    // Run foxtive startup hooks
    app.run_startup_hooks().await?;

    let route_factory = builder.route_factory;
    let configure_fn = builder.configure_fn;
    let raw_configures = builder.raw_configures;
    let health_check_path = builder.health_check_path;
    let ntex_json_config = web::types::JsonConfig::default().limit(body_config.json_limit);

    let shared_config = SharedCfg::new("WEB").add(
        IoConfig::new()
            .set_keepalive_timeout(builder.keep_alive)
            .set_connect_timeout(builder.client_timeout)
            .set_disconnect_timeout(builder.client_disconnect),
    );

    let app_for_server = app.clone();
    let server = web::HttpServer::new(async move || {
        let app = web::App::new()
            .state(app_for_server.clone())
            .state(ntex_json_config.clone())
            .state(app_state.clone())
            .state(body_config.clone())
            .configure(|cfg| {
                // configure() takes full control; otherwise fall back to route_factory
                if let Some(ref f) = configure_fn {
                    f(cfg);
                } else {
                    let routes = route_factory();
                    register_routes(cfg, routes, health_check_path.as_deref());
                }
                // additive raw configure callbacks
                for f in &raw_configures {
                    f(cfg);
                }
            })
            .middleware(setup_logger(&builder.logger_exclude_paths))
            .middleware(
                setup_cors(
                    app_state.allowed_origins.clone(),
                    app_state.allowed_methods.clone(),
                    &builder.allowed_cors_headers,
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
    server.await.map_err(|e| foxtive::prelude::AppMessage::Infrastructure {
        message: "Server error".to_string(),
        source: Some(Box::new(e)),
    })?;

    // AFTER server fully stops, run cleanup handler and foxtive shutdown hooks
    if let Some(on_shutdown) = builder.on_shutdown {
        debug!("Running shutdown handler");
        on_shutdown.await;
    }

    // Run registered shutdown services (ShutdownRegistry)
    let mut registry = builder.shutdown_registry;
    if !registry.is_empty() {
        let timeout = builder.shutdown_config.map(|c| c.timeout);
        debug!("Running {} shutdown services via ShutdownRegistry", registry.len());
        registry.shutdown_all(timeout).await;
    }

    // Run foxtive shutdown hooks
    app.shutdown().await;

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
            use tokio::signal::unix::{SignalKind, signal};
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
