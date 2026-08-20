mod config;

#[allow(deprecated)]
pub use config::JsonConfig;
#[cfg(feature = "static")]
pub use config::StaticFileConfig;
pub use config::{BodyConfig, ServerBuilder};

use crate::http::kernel::{ntex_default_service, register_routes, setup_cors, setup_logger};
use crate::http::shutdown::ShutdownSignal;
use crate::setup::{CustomStateBuilder, NtexSetup, build_app_state};
use foxtive::App;
use foxtive::prelude::{AppMessage, AppResult};
use foxtive::setup::load_environment_variables;
use foxtive::setup::trace::Tracing;
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

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let (app_shutdown_tx, app_shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let user_custom_state_builder = builder.custom_state_builder;
    let custom_state_builder: Option<CustomStateBuilder> = Some(Box::new(move || {
        let mut state = user_custom_state_builder.map(|b| b()).unwrap_or_default();
        state.insert(
            "shutdown_signal".to_string(),
            Box::new(ShutdownSignal::new(shutdown_tx)),
        );
        state
    }));

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

    {
        let app_for_bridge = app.clone();
        let bridge_tx = app_shutdown_tx;
        ntex::rt::spawn(async move {
            loop {
                if app_for_bridge.is_shutting_down() {
                    let _ = bridge_tx.send(());
                    break;
                }
                ntex::time::sleep(ntex::time::Millis(50)).await;
            }
        });
    }

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

    let srv = server.clone();

    let srv_for_signal = srv.clone();
    ntex::rt::spawn(async move {
        tokio::select! {
            _ = shutdown_rx => {
                debug!("Programmatic shutdown signal received");
            }
            _ = app_shutdown_rx => {
                debug!("app.shutdown() triggered server stop");
            }
        }
        srv_for_signal.stop(true).await;
    });

    let shutdown_signal = builder
        .shutdown_signal
        .unwrap_or_else(default_shutdown_signal);

    ntex::rt::spawn(async move {
        shutdown_signal.await;

        debug!("Shutdown signal received");
        srv.stop(true).await;
    });

    server.await.map_err(|e| AppMessage::Infrastructure {
        message: "Server error".to_string(),
        source: Some(Box::new(e)),
    })?;

    if let Some(on_shutdown) = builder.on_shutdown {
        debug!("Running shutdown handler");
        on_shutdown.await;
    }

    let mut registry = builder.shutdown_registry;
    if !registry.is_empty() {
        let timeout = builder.shutdown_config.map(|c| c.timeout);
        debug!(
            "Running {} shutdown services via ShutdownRegistry",
            registry.len()
        );
        registry.shutdown_all(timeout).await;
    }

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
