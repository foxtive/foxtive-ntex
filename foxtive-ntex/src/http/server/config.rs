use crate::http::Method;
use crate::http::kernel::Route;
use foxtive::setup::FoxtiveSetup;
use foxtive::setup::logger::TracingConfig;
use ntex::http::KeepAlive;
use ntex::time::Seconds;

#[cfg(feature = "static")]
pub struct StaticFileConfig {
    pub path: String,
    pub dir: String,
}

pub struct ServerConfig<TB>
where
    TB: FnOnce() -> Vec<Route> + Send + Copy + 'static,
{
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) workers: usize,

    pub(crate) max_connections: usize,

    pub(crate) max_connections_rate: usize,

    pub(crate) client_timeout: Seconds,

    pub(crate) client_disconnect: Seconds,

    pub(crate) keep_alive: KeepAlive,

    pub(crate) backlog: i32,

    pub(crate) app: String,
    pub(crate) foxtive_setup: FoxtiveSetup,

    pub(crate) tracing_config: Option<TracingConfig>,

    #[cfg(feature = "static")]
    pub(crate) static_config: StaticFileConfig,

    /// whether the app bootstrap has started
    pub(crate) has_started_bootstrap: bool,

    pub(crate) routes: Vec<Route>,

    /// list of allowed CORS origins
    pub(crate) allowed_origins: Vec<String>,

    /// list of allowed CORS origins
    pub(crate) allowed_methods: Vec<Method>,

    pub(crate) boot_thread: Option<TB>,
}

impl<TB> ServerConfig<TB>
where
    TB: FnOnce() -> Vec<Route> + Send + Copy + 'static,
{
    pub fn create(host: &str, port: u16, setup: FoxtiveSetup) -> ServerConfig<TB> {
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
            routes: vec![],
            allowed_origins: vec![],
            allowed_methods: vec![],
            boot_thread: None,
            tracing_config: None,
        }
    }

    #[cfg(feature = "static")]
    pub fn create_with_static(
        host: &str,
        port: u16,
        setup: FoxtiveSetup,
        config: StaticFileConfig,
    ) -> ServerConfig<TB> {
        Self::create(host, port, setup).static_config(config)
    }

    pub fn app(mut self, app: &str) -> Self {
        self.app = app.to_string();
        self
    }

    pub fn tracing_config(mut self, config: TracingConfig) -> Self {
        self.tracing_config = Some(config);
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

    pub fn boot_thread(mut self, boot_thread: TB) -> Self {
        self.boot_thread = Some(boot_thread);
        self
    }

    pub fn has_started_bootstrap(mut self, has_started_bootstrap: bool) -> Self {
        self.has_started_bootstrap = has_started_bootstrap;
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
