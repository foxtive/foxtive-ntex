use crate::http::kernel::Route;
use crate::http::Method;
use foxtive::setup::trace::Tracing;
use foxtive::setup::FoxtiveSetup;
use ntex::http::KeepAlive;
use ntex::time::Seconds;
use std::sync::Arc;

/// Configuration for serving static files.
///
/// This struct defines the mapping between URL paths and filesystem directories
/// for static file serving. Only available when the `static` feature is enabled.
///
/// # Example
/// ```rust
/// use foxtive_ntex::http::StaticFileConfig;
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

/// Configuration for JSON request body parsing.
///
/// This struct controls how JSON payloads are processed, including size limits
/// and content-type validation.
///
/// # Default Settings
/// - Maximum body size: 512,000 bytes (500 KB)
/// - Content-type validation: None (accepts any content-type)
///
/// # Example
/// ```rust
/// use foxtive_ntex::http::JsonConfig;
///
/// // Use default configuration (50 KB limit)
/// let config = JsonConfig::default();
///
/// // Custom size limit
/// let config = JsonConfig {
///     limit: 1024 * 1024, // 1 MB
///     content_type: None,
/// };
/// ```
#[derive(Clone)]
pub struct JsonConfig {
    /// Maximum allowed size for JSON request bodies in bytes.
    ///
    /// Requests exceeding this limit will be rejected with a payload too large error.
    /// Default: 51,200 bytes (50 KB)
    pub(crate) limit: usize,
}

pub struct ServerConfig {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) workers: usize,

    pub(crate) max_connections: usize,

    pub(crate) max_connections_rate: usize,

    pub(crate) client_timeout: Seconds,

    pub(crate) client_disconnect: Seconds,

    pub(crate) keep_alive: KeepAlive,

    pub(crate) backlog: i32,

    pub(crate) json_config: Option<JsonConfig>,

    pub(crate) app: String,
    pub(crate) foxtive_setup: FoxtiveSetup,

    pub(crate) tracing: Option<Tracing>,

    #[cfg(feature = "static")]
    pub(crate) static_config: StaticFileConfig,

    /// whether the app bootstrap has started
    pub(crate) has_started_bootstrap: bool,

    /// list of allowed CORS origins
    pub(crate) allowed_origins: Vec<String>,

    /// list of allowed CORS origins
    pub(crate) allowed_methods: Vec<Method>,

    pub(crate) boot_thread: Arc<dyn Fn() -> Vec<Route> + Send + Sync>,
}

impl ServerConfig {
    pub fn create(host: &str, port: u16, setup: FoxtiveSetup) -> ServerConfig {
        ServerConfig {
            host: host.to_string(),
            port,
            workers: 2,
            max_connections: 25_000,
            max_connections_rate: 256,
            client_timeout: Seconds(3),
            client_disconnect: Seconds(5),
            keep_alive: KeepAlive::Timeout(Seconds(5)),
            backlog: 2048,
            app: "foxtive".to_string(),
            foxtive_setup: setup,
            #[cfg(feature = "static")]
            static_config: StaticFileConfig::default(),
            has_started_bootstrap: false,
            allowed_origins: vec![],
            allowed_methods: vec![],
            boot_thread: Arc::new(Vec::new),
            tracing: None,
            json_config: None,
        }
    }

    #[cfg(feature = "static")]
    pub fn create_with_static(
        host: &str,
        port: u16,
        setup: FoxtiveSetup,
        config: StaticFileConfig,
    ) -> ServerConfig {
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
    pub fn keep_alive(mut self, keep_alive: KeepAlive) -> Self {
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

    pub fn boot_thread(mut self, boot_thread: Arc<dyn Fn() -> Vec<Route> + Send + Sync>) -> Self {
        self.boot_thread = boot_thread;
        self
    }

    pub fn has_started_bootstrap(mut self, has_started_bootstrap: bool) -> Self {
        self.has_started_bootstrap = has_started_bootstrap;
        self
    }

    pub fn json_config(mut self, json_config: JsonConfig) -> Self {
        self.json_config = Some(json_config);
        self
    }
}

impl JsonConfig {
    /// Change max size of payload. By default max size is 50Kb
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
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

impl Default for JsonConfig {
    fn default() -> Self {
        JsonConfig {
            limit: 51_000, // 50 KB
        }
    }
}
