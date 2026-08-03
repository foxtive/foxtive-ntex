use crate::http::kernel::Route;
use crate::http::shutdown::{ShutdownConfig, ShutdownRegistry};
use crate::http::Method;
use foxtive::prelude::AppResult;
use foxtive::setup::trace::Tracing;
use foxtive::App;
use ntex::http::header;
use ntex::time::Seconds;
use ntex::web::ServiceConfig;
use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type ShutdownSignalHandler = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// Custom state builder function type
type CustomStateBuilderFn = Box<dyn FnOnce() -> HashMap<String, Box<dyn Any + Send + Sync>> + Send>;

/// Configuration for serving static files.
///
/// This struct defines the mapping between URL paths and filesystem directories
/// for static file serving. Only available when the `static` feature is enabled.
///
/// # Example
/// ```rust
/// use foxtive_ntex::http::server::StaticFileConfig;
///
/// let config = StaticFileConfig {
///     path: "/assets".to_string(),
///     dir: "./public".to_string(),
/// };
///
/// // This would serve files from "./public" directory at "/assets/*" URL path
/// // e.g., "./public/style.css" would be accessible at "/assets/style.css"
/// ```
///
/// # Security Notes
/// - The `dir` path should be carefully validated to prevent directory traversal attacks
/// - Consider using absolute paths or canonicalized paths for the `dir` field
/// - Ensure proper file permissions are set on the served directory
#[cfg(feature = "static")]
pub struct StaticFileConfig {
    /// The URL path prefix where static files will be served.
    ///
    /// This defines the base route under which static files are accessible.
    /// Should start with "/" (e.g., "/static", "/assets", "/public").
    pub path: String,

    /// The filesystem directory path containing the static files to serve.
    ///
    /// This can be either a relative path (relative to the application's working directory)
    /// or an absolute path. All files within this directory and its subdirectories
    /// will be served under the configured URL path.
    pub dir: String,
}

/// Configuration for HTTP request body parsing.
///
/// This struct controls how different body types (JSON, string, bytes) are processed,
/// including size limits for each type.
///
/// # Default Settings
/// - JSON limit: 51,000 bytes (50 KB)
/// - String limit: 51,000 bytes (50 KB)
/// - Byte limit: 51,000 bytes (50 KB)
///
/// # Example
/// ```rust
/// use foxtive_ntex::http::server::BodyConfig;
///
/// // Use default configuration
/// let config = BodyConfig::default();
///
/// // Custom limits for different body types
/// let config = BodyConfig::default()
///     .json_limit(1024 * 1024)      // 1 MB for JSON
///     .string_limit(512 * 1024)     // 512 KB for strings
///     .byte_limit(2 * 1024 * 1024); // 2 MB for bytes
/// ```
#[derive(Clone, Debug)]
pub struct BodyConfig {
    pub(crate) json_limit: usize,
    pub(crate) string_limit: usize,
    pub(crate) byte_limit: usize,
}

impl BodyConfig {
    pub fn json_limit(mut self, limit: usize) -> Self {
        self.json_limit = limit;
        self
    }

    pub fn string_limit(mut self, limit: usize) -> Self {
        self.string_limit = limit;
        self
    }

    pub fn byte_limit(mut self, limit: usize) -> Self {
        self.byte_limit = limit;
        self
    }
}

impl Default for BodyConfig {
    fn default() -> Self {
        Self {
            json_limit: 51_000,
            string_limit: 51_000,
            byte_limit: 51_000,
        }
    }
}

#[deprecated(since = "0.31.0", note = "Use BodyConfig instead")]
pub type JsonConfig = BodyConfig;

pub struct ServerBuilder {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) workers: usize,

    pub(crate) max_connections: usize,

    pub(crate) max_connections_rate: usize,

    pub(crate) client_timeout: Seconds,

    pub(crate) client_disconnect: Seconds,

    pub(crate) keep_alive: Seconds,

    pub(crate) backlog: i32,

    pub(crate) body_config: Option<BodyConfig>,

    pub(crate) app: Arc<App>,

    pub(crate) tracing: Option<Tracing>,

    #[cfg(feature = "static")]
    pub(crate) static_config: StaticFileConfig,

    /// whether the app bootstrap has started
    pub(crate) has_started_bootstrap: bool,

    pub(crate) allowed_origins: Vec<String>,

    pub(crate) allowed_methods: Vec<Method>,

    pub(crate) route_factory: Arc<dyn Fn() -> Vec<Route> + Send + Sync>,

    #[allow(clippy::type_complexity)]
    /// Complete replacement for route_factory — receives raw `ServiceConfig`.
    /// When set, `route_factory` is ignored.
    pub(crate) configure_fn: Option<Arc<dyn Fn(&mut ServiceConfig) + Send + Sync>>,

    #[allow(clippy::type_complexity)]
    /// Additive raw ntex configure callbacks, called after route registration.
    pub(crate) raw_configures: Vec<Arc<dyn Fn(&mut ServiceConfig) + Send + Sync>>,

    pub(crate) on_shutdown: Option<ShutdownSignalHandler>,

    pub(crate) shutdown_signal: Option<Pin<Box<dyn Future<Output = ()> + Send>>>,

    pub(crate) shutdown_config: Option<ShutdownConfig>,

    pub(crate) shutdown_registry: ShutdownRegistry,

    pub(crate) custom_state_builder: Option<CustomStateBuilderFn>,

    /// Additional CORS allowed headers beyond the defaults
    /// (Authorization, Accept, Content-Type).
    pub(crate) allowed_cors_headers: Vec<header::HeaderName>,

    /// Paths excluded from the access logger.
    pub(crate) logger_exclude_paths: Vec<String>,

    /// Optional path at which the built-in health check endpoint is registered.
    pub(crate) health_check_path: Option<String>,

    /// OpenSSL TLS acceptor (requires the `openssl` feature).
    #[cfg(feature = "openssl")]
    pub(crate) tls_openssl: Option<openssl::ssl::SslAcceptor>,

    /// Rustls TLS configuration (requires the `rustls` feature).
    #[cfg(feature = "rustls")]
    pub(crate) tls_rustls: Option<std::sync::Arc<rustls::ServerConfig>>,
}

impl ServerBuilder {
    pub fn create(host: &str, port: u16, app: Arc<App>) -> ServerBuilder {
        ServerBuilder {
            host: host.to_string(),
            port,
            workers: 2,
            max_connections: 25_000,
            max_connections_rate: 256,
            client_timeout: Seconds(3),
            client_disconnect: Seconds(5),
            keep_alive: Seconds(5),
            backlog: 2048,
            app,
            #[cfg(feature = "static")]
            static_config: StaticFileConfig::default(),
            has_started_bootstrap: false,
            allowed_origins: vec![],
            allowed_methods: vec![],
            route_factory: Arc::new(Vec::new),
            configure_fn: None,
            raw_configures: vec![],
            tracing: None,
            body_config: None,
            on_shutdown: None,
            shutdown_signal: None,
            shutdown_config: None,
            shutdown_registry: ShutdownRegistry::new(),
            custom_state_builder: None,
            allowed_cors_headers: vec![],
            logger_exclude_paths: vec![
                "/favicon.ico".into(),
                "/system/health-check".into(),
                "/api/v1/admin/health-check".into(),
            ],
            health_check_path: None,
            #[cfg(feature = "openssl")]
            tls_openssl: None,
            #[cfg(feature = "rustls")]
            tls_rustls: None,
        }
    }

    #[cfg(feature = "static")]
    pub fn create_with_static(
        host: &str,
        port: u16,
        app: Arc<App>,
        config: StaticFileConfig,
    ) -> ServerBuilder {
        Self::create(host, port, app).static_config(config)
    }

    pub fn tracing(mut self, config: Tracing) -> Self {
        self.tracing = Some(config);
        self
    }

    /// Set number of workers to start.
    ///
    /// By default http server uses 2
    pub fn workers(mut self, workers: usize) -> Self {
        self.workers = workers;
        self
    }

    /// Set the maximum number of pending connections.
    ///
    /// This refers to the number of clients that can be waiting to be served.
    /// Exceeding this number results in the client getting an error when
    /// attempting to connect. It should only affect servers under significant
    /// load.
    ///
    /// Generally set in the 64-2048 range. Default value is 2048.
    ///
    /// This method should be called before `bind()` method call.
    pub fn backlog(mut self, backlog: i32) -> Self {
        self.backlog = backlog;
        self
    }

    /// Set server keep-alive setting.
    ///
    /// By default keep alive is set to a 5 seconds.
    pub fn keep_alive(mut self, keep_alive: Seconds) -> Self {
        self.keep_alive = keep_alive;
        self
    }

    /// Set request read timeout in seconds.
    ///
    /// Defines a timeout for reading client request headers. If a client does not transmit
    /// the entire set headers within this time, the request is terminated with
    /// the 408 (Request Time-out) error.
    ///
    /// To disable timeout set value to 0.
    ///
    /// By default client timeout is set to 3 seconds.
    pub fn client_timeout(mut self, timeout: u16) -> Self {
        self.client_timeout = Seconds(timeout);
        self
    }

    /// Set server connection disconnect timeout in seconds.
    ///
    /// Defines a timeout for shutdown connection. If a shutdown procedure does not complete
    /// within this time, the request is dropped.
    ///
    /// To disable timeout set value to 0.
    ///
    /// By default client timeout is set to 5 seconds.
    pub fn client_disconnect(mut self, timeout: u16) -> Self {
        self.client_disconnect = Seconds(timeout);
        self
    }

    /// Sets the maximum per-worker number of concurrent connections.
    ///
    /// All socket listeners will stop accepting connections when this limit is reached
    /// for each worker.
    ///
    /// By default max connections is set to a 25k.
    pub fn max_conn(mut self, max: usize) -> Self {
        self.max_connections = max;
        self
    }

    /// Sets the maximum per-worker concurrent connection establish process.
    ///
    /// All listeners will stop accepting connections when this limit is reached. It
    /// can be used to limit the global SSL CPU usage.
    ///
    /// By default max connections is set to a 256.
    pub fn max_conn_rate(mut self, max: usize) -> Self {
        self.max_connections_rate = max;
        self
    }

    pub fn allowed_origins(mut self, allowed_origins: Vec<String>) -> Self {
        self.allowed_origins = allowed_origins;
        self
    }

    pub fn allowed_methods(mut self, allowed_methods: Vec<Method>) -> Self {
        self.allowed_methods = allowed_methods;
        self
    }

    #[cfg(feature = "static")]
    pub fn static_config(mut self, static_config: StaticFileConfig) -> Self {
        self.static_config = static_config;
        self
    }

    /// Set the route factory function.
    ///
    /// This function is called once per worker to create route definitions.
    pub fn route_factory<F: Fn() -> Vec<Route> + Send + Sync + 'static>(mut self, factory: F) -> Self {
        self.route_factory = Arc::new(factory);
        self
    }

    /// Set the route factory using an existing `Arc`.
    ///
    /// Useful for sharing the same factory across multiple server configurations.
    pub fn route_factory_arc(mut self, factory: Arc<dyn Fn() -> Vec<Route> + Send + Sync>) -> Self {
        self.route_factory = factory;
        self
    }
    
    /// Configure routes using ntex's native [`ServiceConfig`] API directly.
    ///
    /// This is a **complete replacement** for [`route_factory`](Self::route_factory).
    /// When set, the route factory is ignored and this function receives full
    /// control over route registration, giving you access to all ntex features
    /// (scopes, resources, guards, custom error handlers, per-scope data, etc.).
    ///
    /// # Example
    /// ```rust,ignore
    /// ServerBuilder::create("0.0.0.0", 3000, app)
    ///     .configure(|cfg| {
    ///         cfg.service(
    ///             web::scope("/api/v1")
    ///                 .middleware(auth_middleware)
    ///                 .service(users_handler)
    ///                 .service(posts_handler),
    ///         );
    ///         cfg.service(
    ///             web::scope("/public")
    ///                 .service(health_handler),
    ///         );
    ///     })
    ///     .start(|_| async { Ok(()) })
    ///     .await
    /// ```
    pub fn configure<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut ServiceConfig) + Send + Sync + 'static,
    {
        self.configure_fn = Some(Arc::new(f));
        self
    }
    
    /// Register an additional raw ntex configure callback.
    ///
    /// Unlike [`configure`](Self::configure), this is **additive** — callbacks
    /// are executed *after* the route factory (or `configure`) has registered
    /// routes.  Multiple callbacks can be registered and they execute in the
    /// order they were added.
    ///
    /// This is the escape hatch for mixing the framework's `Route`/`Controller`
    /// system with raw ntex configuration (guards, custom error handlers,
    /// resource-level settings, etc.).
    ///
    /// # Example
    /// ```rust,ignore
    /// ServerBuilder::create("0.0.0.0", 3000, app)
    ///     .route_factory(my_routes)
    ///     .raw_configure(|cfg| {
    ///         cfg.service(
    ///             web::resource("/special")
    ///                 .guard(guard::Header("content-type", "application/json"))
    ///                 .to(special_handler),
    ///         );
    ///     })
    ///     .start(|_| async { Ok(()) })
    ///     .await
    /// ```
    pub fn raw_configure<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut ServiceConfig) + Send + Sync + 'static,
    {
        self.raw_configures.push(Arc::new(f));
        self
    }

    pub fn has_started_bootstrap(mut self, has_started_bootstrap: bool) -> Self {
        self.has_started_bootstrap = has_started_bootstrap;
        self
    }

    pub fn body_config(mut self, body_config: BodyConfig) -> Self {
        self.body_config = Some(body_config);
        self
    }

    pub fn custom_state_builder(
        mut self,
        builder: CustomStateBuilderFn,
    ) -> Self {
        self.custom_state_builder = Some(builder);
        self
    }

    /// Add extra header names to the CORS allowed-headers list.
    ///
    /// These are merged with the built-in defaults (Authorization, Accept,
    /// Content-Type).
    pub fn allowed_cors_headers(mut self, headers: Vec<header::HeaderName>) -> Self {
        self.allowed_cors_headers = headers;
        self
    }

    /// Set the paths that should be excluded from the access logger.
    ///
    /// Overrides the default exclusion list (`/favicon.ico`,
    /// `/system/health-check`, `/api/v1/admin/health-check`).
    pub fn logger_exclude_paths(mut self, paths: Vec<String>) -> Self {
        self.logger_exclude_paths = paths;
        self
    }

    /// Register a built-in health check endpoint at the given path.
    ///
    /// The handler calls [`App::check_health()`](foxtive::App::check_health)
    /// and returns a JSON [`HealthReport`](foxtive::health::HealthReport).
    pub fn health_check_path(mut self, path: impl Into<String>) -> Self {
        self.health_check_path = Some(path.into());
        self
    }

    /// Configure an OpenSSL TLS acceptor for the server.
    #[cfg(feature = "openssl")]
    pub fn tls_openssl(mut self, acceptor: openssl::ssl::SslAcceptor) -> Self {
        self.tls_openssl = Some(acceptor);
        self
    }

    /// Configure a Rustls TLS configuration for the server.
    #[cfg(feature = "rustls")]
    pub fn tls_rustls(mut self, config: std::sync::Arc<rustls::ServerConfig>) -> Self {
        self.tls_rustls = Some(config);
        self
    }

    /// Sets a custom shutdown handler to be called when the application is shutting down.
    pub fn on_shutdown<F>(mut self, func: F) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.on_shutdown = Some(Box::pin(func));
        self
    }

    /// Sets a custom shutdown signal handler that determines when the application should begin shutting down.
    pub fn shutdown_signal<F>(mut self, func: F) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.shutdown_signal = Some(Box::pin(func));
        self
    }

    /// Validate the server configuration before startup.
    pub fn validate(&self) -> foxtive::results::AppResult<()> {
        use foxtive::internal_server_error;

        // Critical validations
        if self.port == 0 {
            return Err(internal_server_error!("Port cannot be 0"));
        }

        if self.workers == 0 {
            return Err(internal_server_error!("Workers must be at least 1"));
        }

        if self.backlog < 0 {
            return Err(internal_server_error!("Backlog cannot be negative"));
        }

        if self.max_connections == 0 {
            return Err(internal_server_error!("Max connections must be at least 1"));
        }

        if self.max_connections_rate == 0 {
            return Err(internal_server_error!(
                "Max connection rate must be at least 1"
            ));
        }

        // Warnings for potentially problematic settings
        if self.client_timeout.0 > 300 {
            tracing::warn!(
                "Client timeout is very high: {} seconds. Consider reducing for better resource management.",
                self.client_timeout.0
            );
        }

        if self.keep_alive.0 > 300 {
            tracing::warn!(
                "Keep-alive timeout is very high: {} seconds. This may cause resource exhaustion under load.",
                self.keep_alive.0
            );
        }

        if self.workers > num_cpus::get() * 2 {
            tracing::warn!(
                "Worker count ({}) is more than 2x the available CPU cores ({}). This may degrade performance.",
                self.workers,
                num_cpus::get()
            );
        }

        if self.backlog > 10000 {
            tracing::warn!(
                "Backlog ({}) is very high. This may cause memory issues under extreme load.",
                self.backlog
            );
        }

        Ok(())
    }

    /// Create a server configuration with smart defaults for development.
    pub fn dev_mode(host: &str, port: u16, app: Arc<App>) -> Self {
        Self::create(host, port, app)
            .workers(1)
            .client_timeout(60)
            .keep_alive(Seconds(60))
            .max_conn(1000)
            .backlog(256)
    }

    /// Create a server configuration optimized for production deployment.
    pub fn production_mode(host: &str, port: u16, app: Arc<App>) -> Self {
        let workers = num_cpus::get();
        Self::create(host, port, app)
            .workers(workers)
            .client_timeout(15)
            .keep_alive(ntex::time::Seconds(30))
            .max_conn(25_000)
            .backlog(2048)
    }

    /// Create a server configuration optimized for high-performance scenarios.
    pub fn high_performance_mode(host: &str, port: u16, app: Arc<App>) -> Self {
        let workers = num_cpus::get() * 2;
        Self::create(host, port, app)
            .workers(workers)
            .client_timeout(5)
            .keep_alive(Seconds(10))
            .max_conn(50_000)
            .max_conn_rate(512)
            .backlog(4096)
    }

    /// Configure shutdown behavior with timeout and cleanup coordination.
    pub fn shutdown_config(mut self, config: ShutdownConfig) -> Self {
        self.shutdown_config = Some(config);
        self
    }

    /// Register a service for graceful shutdown cleanup.
    pub fn register_shutdown_service<F, Fut>(mut self, name: &str, priority: u8, cleanup: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.shutdown_registry.register(name, priority, cleanup);
        self
    }

    /// Start the HTTP server with an optional bootstrap callback.
    ///
    /// The callback receives `Arc<App>` for accessing the DI container.
    pub async fn start<Callback, Fut>(self, callback: Callback) -> AppResult<()>
    where
        Callback: FnOnce(Arc<App>) -> Fut + Copy + Send + 'static,
        Fut: Future<Output = AppResult<()>> + Send + 'static,
    {
        super::start_ntex_server(self, callback).await
    }

    /// Start the HTTP server without a bootstrap callback.
    pub async fn run(self) -> AppResult<()> {
        self.start(|_app| async { Ok(()) }).await
    }
}

#[cfg(feature = "static")]
impl Default for StaticFileConfig {
    fn default() -> Self {
        Self {
            path: "static".to_string(),
            dir: "./static".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_config_limits() {
        let config = BodyConfig::default()
            .json_limit(1024 * 1024)
            .string_limit(512 * 1024)
            .byte_limit(2 * 1024 * 1024);
        
        assert_eq!(config.json_limit, 1_048_576);
        assert_eq!(config.string_limit, 524_288);
        assert_eq!(config.byte_limit, 2_097_152);
    }
}
