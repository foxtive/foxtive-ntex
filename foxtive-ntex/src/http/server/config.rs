use crate::http::kernel::Route;
use crate::http::shutdown::{ShutdownConfig, ShutdownRegistry};
use crate::http::Method;
use crate::FoxtiveNtexState;
use foxtive::prelude::AppResult;
use foxtive::setup::trace::Tracing;
use foxtive::setup::FoxtiveSetup;
use ntex::time::Seconds;
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

    pub(crate) app: String,
    pub(crate) foxtive_setup: FoxtiveSetup,

    pub(crate) tracing: Option<Tracing>,

    #[cfg(feature = "static")]
    pub(crate) static_config: StaticFileConfig,

    /// whether the app bootstrap has started
    pub(crate) has_started_bootstrap: bool,

    pub(crate) allowed_origins: Vec<String>,

    pub(crate) allowed_methods: Vec<Method>,

    pub(crate) route_factory: Arc<dyn Fn() -> Vec<Route> + Send + Sync>,

    pub(crate) on_shutdown: Option<ShutdownSignalHandler>,

    pub(crate) shutdown_signal: Option<Pin<Box<dyn Future<Output = ()> + Send>>>,

    pub(crate) shutdown_config: Option<ShutdownConfig>,

    pub(crate) shutdown_registry: ShutdownRegistry,

    pub(crate) custom_state_builder: Option<CustomStateBuilderFn>,
}

impl ServerBuilder {
    pub fn create(host: &str, port: u16, setup: FoxtiveSetup) -> ServerBuilder {
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
            app: "foxtive".to_string(),
            foxtive_setup: setup,
            #[cfg(feature = "static")]
            static_config: StaticFileConfig::default(),
            has_started_bootstrap: false,
            allowed_origins: vec![],
            allowed_methods: vec![],
            route_factory: Arc::new(Vec::new),
            tracing: None,
            body_config: None,
            on_shutdown: None,
            shutdown_signal: None,
            shutdown_config: None,
            shutdown_registry: ShutdownRegistry::new(),
            custom_state_builder: None,
        }
    }

    #[cfg(feature = "static")]
    pub fn create_with_static(
        host: &str,
        port: u16,
        setup: FoxtiveSetup,
        config: StaticFileConfig,
    ) -> ServerBuilder {
        Self::create(host, port, setup).static_config(config)
    }

    pub fn app(mut self, app: &str) -> Self {
        self.app = app.to_string();
        self
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

    #[deprecated(since = "0.32.0", note = "Use route_factory instead")]
    pub fn boot_thread<F: Fn() -> Vec<Route> + Send + Sync + 'static>(self, factory: F) -> Self {
        self.route_factory(factory)
    }

    pub fn has_started_bootstrap(mut self, has_started_bootstrap: bool) -> Self {
        self.has_started_bootstrap = has_started_bootstrap;
        self
    }

    pub fn body_config(mut self, body_config: BodyConfig) -> Self {
        self.body_config = Some(body_config);
        self
    }

    #[deprecated(since = "0.31.0", note = "Use body_config instead")]
    pub fn json_config(self, config: BodyConfig) -> Self {
        self.body_config(config)
    }

    pub fn custom_state_builder(
        mut self,
        builder: CustomStateBuilderFn,
    ) -> Self {
        self.custom_state_builder = Some(builder);
        self
    }

    /// Sets a custom shutdown handler to be called when the application is shutting down.
    ///
    /// This method allows you to provide a future that will be awaited during shutdown.
    /// It is typically used to perform cleanup tasks like closing database connections,
    /// flushing logs, or other async teardown operations.
    ///
    /// **Note:** If a custom `shutdown_signal` is also provided using [`shutdown_signal`],
    /// that will take precedence over this handler, and this `on_shutdown` handler will
    /// **not** be executed.
    ///
    pub fn on_shutdown<F>(mut self, func: F) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.on_shutdown = Some(Box::pin(func));
        self
    }

    /// Sets a custom shutdown signal handler that determines when the application should begin shutting down.
    ///
    /// This method allows you to provide a future that, when resolved, triggers the application shutdown.
    /// It is typically used to listen for signals like `Ctrl+C` or system termination requests (`SIGTERM`).
    ///
    /// If this shutdown signal is provided, it will override any handler set using [`on_shutdown`].
    pub fn shutdown_signal<F>(mut self, func: F) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.shutdown_signal = Some(Box::pin(func));
        self
    }

    /// Validate the server configuration before startup.
    ///
    /// This method checks for common configuration errors and warns about potentially
    /// problematic settings. It returns an error if critical issues are found.
    ///
    /// # Validation Rules
    /// - Port must not be 0
    /// - Workers must be at least 1
    /// - Backlog must not be negative
    /// - Timeout values are checked for reasonable ranges (warnings only)
    ///
    /// # Example
    /// ```rust,ignore
    /// use foxtive_ntex::http::server::ServerConfig;
    /// // FoxtiveSetup must be created with your application's setup logic
    /// // let config = ServerConfig::validate_example();
    /// ```
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
    ///
    /// This is a convenience method that creates a configuration optimized for
    /// local development with relaxed timeouts and single worker.
    ///
    /// # Defaults
    /// - Workers: 1 (easier debugging)
    /// - Client timeout: 60 seconds
    /// - Keep-alive: 60 seconds
    /// - Max connections: 1000
    /// - Backlog: 256
    ///
    /// # Example
    /// ```rust,ignore
    /// use foxtive_ntex::http::server::ServerConfig;
    /// // FoxtiveSetup must be created with your application's setup logic
    /// // let config = ServerConfig::dev_mode("127.0.0.1", 3000, setup);
    /// ```
    pub fn dev_mode(host: &str, port: u16, setup: FoxtiveSetup) -> Self {
        Self::create(host, port, setup)
            .workers(1)
            .client_timeout(60)
            .keep_alive(Seconds(60))
            .max_conn(1000)
            .backlog(256)
    }

    /// Create a server configuration optimized for production deployment.
    ///
    /// This configuration uses conservative settings suitable for most production
    /// workloads with good performance and resource management.
    ///
    /// # Defaults
    /// - Workers: Auto-detected (number of CPU cores)
    /// - Client timeout: 15 seconds
    /// - Keep-alive: 30 seconds
    /// - Max connections: 25,000
    /// - Backlog: 2048
    ///
    /// # Example
    /// ```rust,ignore
    /// use foxtive_ntex::http::server::ServerConfig;
    /// // FoxtiveSetup must be created with your application's setup logic
    /// // let config = ServerConfig::production_mode("0.0.0.0", 8080, setup);
    /// ```
    pub fn production_mode(host: &str, port: u16, setup: FoxtiveSetup) -> Self {
        let workers = num_cpus::get();
        Self::create(host, port, setup)
            .workers(workers)
            .client_timeout(15)
            .keep_alive(ntex::time::Seconds(30))
            .max_conn(25_000)
            .backlog(2048)
    }

    /// Create a server configuration optimized for high-performance scenarios.
    ///
    /// This configuration maximizes throughput and concurrent connections,
    /// suitable for high-traffic APIs or microservices.
    ///
    /// # Defaults
    /// - Workers: 2x CPU cores (for I/O-bound workloads)
    /// - Client timeout: 5 seconds
    /// - Keep-alive: 10 seconds
    /// - Max connections: 50,000
    /// - Max connection rate: 512
    /// - Backlog: 4096
    ///
    /// # Warning
    /// This configuration uses more resources. Monitor your system to ensure
    /// it can handle the increased load.
    ///
    /// # Example
    /// ```rust,ignore
    /// use foxtive_ntex::http::server::ServerConfig;
    /// // FoxtiveSetup must be created with your application's setup logic
    /// // let config = ServerConfig::high_performance_mode("0.0.0.0", 8080, setup);
    /// ```
    pub fn high_performance_mode(host: &str, port: u16, setup: FoxtiveSetup) -> Self {
        let workers = num_cpus::get() * 2;
        Self::create(host, port, setup)
            .workers(workers)
            .client_timeout(5)
            .keep_alive(Seconds(10))
            .max_conn(50_000)
            .max_conn_rate(512)
            .backlog(4096)
    }

    /// Configure shutdown behavior with timeout and cleanup coordination.
    ///
    /// This method sets up coordinated shutdown for all registered services.
    /// Services are shut down in priority order with per-service timeouts.
    ///
    /// # Arguments
    /// * `config` - Shutdown configuration with timeout settings
    ///
    /// # Example
    /// ```rust,ignore
    /// use foxtive_ntex::http::server::{ServerConfig, ShutdownConfig};
    /// // FoxtiveSetup must be created with your application's setup logic
    /// // let config = ServerConfig::create("127.0.0.1", 8080, setup)
    /// //     .shutdown_config(ShutdownConfig::new(30));
    /// ```
    pub fn shutdown_config(mut self, config: ShutdownConfig) -> Self {
        self.shutdown_config = Some(config);
        self
    }

    /// Register a service for graceful shutdown cleanup.
    ///
    /// Services are shut down in priority order (lower priority number first).
    /// Each service has a timeout to prevent one slow service from blocking others.
    ///
    /// # Arguments
    /// * `name` - Name of the service (for logging)
    /// * `priority` - Shutdown priority (lower = shutdown first)
    ///   - 0-10: Critical infrastructure (databases, message queues)
    ///   - 11-50: Application services (caches, connection pools)
    ///   - 51-100: Auxiliary services (loggers, metrics)
    /// * `cleanup` - Async cleanup function to execute
    ///
    /// # Example
    /// ```rust,ignore
    /// use foxtive_ntex::http::server::ServerBuilder;
    /// // FoxtiveSetup must be created with your application's setup logic
    /// // let config = ServerBuilder::create("127.0.0.1", 8080, setup)
    /// //     .register_shutdown_service("database", 1, || async {
    /// //         println!("Database closed");
    /// //     });
    /// ```
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
    /// This method validates the configuration, sets up the ntex server,
    /// and starts listening for incoming requests.
    ///
    /// # Arguments
    /// * `callback` - Optional async function that runs after state creation but before server starts.
    ///   Useful for database migrations, cache warming, etc.
    ///
    /// # Example
    /// ```rust,ignore
    /// use foxtive_ntex::http::server::ServerBuilder;
    /// use foxtive::setup::FoxtiveSetup;
    ///
    /// let foxtive = FoxtiveSetup::default();
    ///
    /// ServerBuilder::dev_mode("127.0.0.1", 3000, foxtive)
    ///     .on_shutdown(async {
    ///         println!("Server shutting down gracefully");
    ///     })
    ///     .start(|state| async move {
    ///         // Bootstrap code here (e.g., database migrations)
    ///         println!("Server starting...");
    ///         Ok(())
    ///     })
    ///     .await?;
    /// ```
    pub async fn start<Callback, Fut>(self, callback: Callback) -> AppResult<()>
    where
        Callback: FnOnce(FoxtiveNtexState) -> Fut + Copy + Send + 'static,
        Fut: Future<Output = AppResult<()>> + Send + 'static,
    {
        super::start_ntex_server(self, callback).await
    }

    /// Start the HTTP server without a bootstrap callback.
    ///
    /// This is a convenience method for simple servers that don't need
    /// initialization logic before starting.
    ///
    /// # Example
    /// ```rust,ignore
    /// use foxtive_ntex::http::server::ServerBuilder;
    /// use foxtive::setup::FoxtiveSetup;
    ///
    /// let foxtive = FoxtiveSetup::default();
    ///
    /// ServerBuilder::production_mode("0.0.0.0", 8080, foxtive)
    ///     .run()
    ///     .await?;
    /// ```
    pub async fn run(self) -> AppResult<()> {
        self.start(|_state| async { Ok(()) }).await
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
