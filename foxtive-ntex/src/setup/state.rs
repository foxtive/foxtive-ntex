use crate::http::Method;
use crate::http::server::BodyConfig;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

/// Type-safe container for ntex-specific application state
#[derive(Clone)]
pub struct AppState {
    /// Built-in CORS configuration
    pub allowed_origins: Vec<String>,

    /// Built-in allowed HTTP methods
    pub allowed_methods: Vec<Method>,

    /// Built-in HTTP body configuration
    pub body_config: BodyConfig,

    /// Custom state storage - type-erased but type-safe access
    /// Immutable after initialization for thread safety
    custom_state: Arc<HashMap<String, Box<dyn Any + Send + Sync>>>,
}

impl AppState {
    pub fn new(
        allowed_origins: Vec<String>,
        allowed_methods: Vec<Method>,
        body_config: BodyConfig,
        custom_state: HashMap<String, Box<dyn Any + Send + Sync>>,
    ) -> Self {
        Self {
            allowed_origins,
            allowed_methods,
            body_config,
            custom_state: Arc::new(custom_state),
        }
    }

    pub fn builder(body_config: BodyConfig) -> AppStateBuilder {
        AppStateBuilder::new(vec![], vec![], body_config)
    }

    /// Get a reference to a value from the custom state storage
    pub fn get<T: Send + Sync + 'static>(&self, key: &str) -> Option<&T> {
        self.custom_state
            .get(key)
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.custom_state.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.custom_state.len()
    }

    pub fn is_empty(&self) -> bool {
        self.custom_state.is_empty()
    }

    pub fn iter_custom_state(&self) -> impl Iterator<Item = (&String, &Box<dyn Any + Send + Sync>)> {
        self.custom_state.iter()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(
            Vec::new(),
            Vec::new(),
            BodyConfig::default(),
            HashMap::new(),
        )
    }
}

impl Debug for AppState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("allowed_origins", &self.allowed_origins)
            .field("allowed_methods", &self.allowed_methods)
            .field("body_config", &self.body_config)
            .field("custom_state_count", &self.custom_state.len())
            .finish()
    }
}

/// Builder for constructing AppState with custom values
pub struct AppStateBuilder {
    allowed_origins: Vec<String>,
    allowed_methods: Vec<Method>,
    body_config: BodyConfig,
    custom_state: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl AppStateBuilder {
    fn new(
        allowed_origins: Vec<String>,
        allowed_methods: Vec<Method>,
        body_config: BodyConfig,
    ) -> Self {
        Self {
            allowed_origins,
            allowed_methods,
            body_config,
            custom_state: HashMap::new(),
        }
    }

    /// Add a custom value to the state
    pub fn with_value<T: Send + Sync + 'static>(mut self, key: &str, value: T) -> Self {
        self.custom_state.insert(key.to_string(), Box::new(value));
        self
    }

    /// Add an allowed CORS origin
    pub fn with_allowed_origin(mut self, origin: impl Into<String>) -> Self {
        self.allowed_origins.push(origin.into());
        self
    }

    /// Add multiple allowed CORS origins
    pub fn with_allowed_origins(mut self, origins: Vec<String>) -> Self {
        self.allowed_origins.extend(origins);
        self
    }

    /// Add an allowed HTTP method
    pub fn with_allowed_method(mut self, method: Method) -> Self {
        self.allowed_methods.push(method);
        self
    }

    /// Add multiple allowed HTTP methods
    pub fn with_allowed_methods(mut self, methods: Vec<Method>) -> Self {
        self.allowed_methods.extend(methods);
        self
    }

    pub fn with_body_config(mut self, config: BodyConfig) -> Self {
        self.body_config = config;
        self
    }

    pub fn build(self) -> AppState {
        AppState::new(
            self.allowed_origins,
            self.allowed_methods,
            self.body_config,
            self.custom_state,
        )
    }
}

/// Macro to generate convenience methods for AppState access
///
/// This macro creates an extension trait with typed getter methods for your custom state.
///
/// # Example
/// ```rust,ignore
/// use foxtive_ntex::setup::state::app_state_ext;
///
/// // Define your custom state types
/// struct DatabasePool;
/// struct RedisClient;
///
/// // Generate extension trait
/// app_state_ext! {
///     pub trait MyAppStateExt {
///         fn db_pool(&self) -> Option<&DatabasePool> { "db_pool" }
///         fn redis_client(&self) -> Option<&RedisClient> { "redis_client" }
///     }
/// }
///
/// // Use in handlers
/// // fn my_handler(state: &AppState) {
/// //     if let Some(pool) = state.db_pool() {
/// //         // Use the database pool
/// //     }
/// // }
/// ```
#[macro_export]
macro_rules! app_state_ext {
    (
        $(#[$meta:meta])*
        $vis:vis trait $trait_name:ident {
            $(
                $(#[$method_meta:meta])*
                fn $method:ident(&self) -> Option<&$type:ty> { $key:expr }
            )+
        }
    ) => {
        $(#[$meta])*
        $vis trait $trait_name {
            $(
                $(#[$method_meta])*
                fn $method(&self) -> Option<&$type>;
            )+
        }

        impl $trait_name for $crate::setup::state::AppState {
            $(
                fn $method(&self) -> Option<&$type> {
                    self.get::<$type>($key)
                }
            )+
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test types for state storage
    #[derive(Debug, Clone, PartialEq)]
    struct TestDatabase {
        name: String,
        max_connections: u32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TestCache {
        ttl_seconds: u64,
    }

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert!(state.allowed_origins.is_empty());
        assert!(state.allowed_methods.is_empty());
        assert_eq!(state.body_config.json_limit, 51_000);
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
    }

    #[test]
    fn test_app_state_new_with_custom_state() {
        let mut custom_state: HashMap<String, Box<dyn Any + Send + Sync>> = HashMap::new();
        let db = TestDatabase {
            name: "test_db".to_string(),
            max_connections: 10,
        };
        custom_state.insert("db".to_string(), Box::new(db.clone()));

        let state = AppState::new(
            vec!["http://localhost".to_string()],
            vec![Method::GET],
            BodyConfig::default(),
            custom_state,
        );

        assert_eq!(state.allowed_origins, vec!["http://localhost"]);
        assert_eq!(state.allowed_methods, vec![Method::GET]);
        assert_eq!(state.len(), 1);
        assert!(!state.is_empty());

        let retrieved = state.get::<TestDatabase>("db");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &db);
    }

    #[test]
    fn test_app_state_get_nonexistent_key() {
        let state = AppState::default();
        let result = state.get::<String>("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_app_state_get_wrong_type() {
        let mut custom_state: HashMap<String, Box<dyn Any + Send + Sync>> = HashMap::new();
        custom_state.insert("number".to_string(), Box::new(42i32));

        let state = AppState::new(Vec::new(), Vec::new(), BodyConfig::default(), custom_state);

        // Try to get as wrong type
        let result = state.get::<String>("number");
        assert!(result.is_none());

        // Get as correct type
        let result = state.get::<i32>("number");
        assert!(result.is_some());
        assert_eq!(*result.unwrap(), 42);
    }

    #[test]
    fn test_app_state_contains_key() {
        let mut custom_state: HashMap<String, Box<dyn Any + Send + Sync>> = HashMap::new();
        custom_state.insert("key1".to_string(), Box::new("value1".to_string()));

        let state = AppState::new(Vec::new(), Vec::new(), BodyConfig::default(), custom_state);

        assert!(state.contains_key("key1"));
        assert!(!state.contains_key("key2"));
    }

    #[test]
    fn test_app_state_builder() {
        let db = TestDatabase {
            name: "prod_db".to_string(),
            max_connections: 100,
        };

        let cache = TestCache { ttl_seconds: 300 };

        let state = AppState::builder(BodyConfig::default().json_limit(1024 * 1024))
            .with_allowed_methods(vec![Method::GET, Method::POST])
            .with_allowed_origins(vec!["https://example.com".to_string()])
            .with_value("database", db.clone())
            .with_value("cache", cache.clone())
            .build();

        assert_eq!(state.allowed_origins, vec!["https://example.com"]);
        assert_eq!(state.allowed_methods.len(), 2);
        assert_eq!(state.body_config.json_limit, 1_048_576);
        assert_eq!(state.len(), 2);

        let retrieved_db = state.get::<TestDatabase>("database");
        assert!(retrieved_db.is_some());
        assert_eq!(retrieved_db.unwrap(), &db);

        let retrieved_cache = state.get::<TestCache>("cache");
        assert!(retrieved_cache.is_some());
        assert_eq!(retrieved_cache.unwrap(), &cache);
    }

    #[test]
    fn test_app_state_clone() {
        let mut custom_state: HashMap<String, Box<dyn Any + Send + Sync>> = HashMap::new();
        custom_state.insert("data".to_string(), Box::new(123i32));

        let state1 = AppState::new(Vec::new(), Vec::new(), BodyConfig::default(), custom_state);

        let state2 = state1.clone();

        // Both should have the same data
        assert_eq!(state1.len(), state2.len());
        assert_eq!(state1.get::<i32>("data"), state2.get::<i32>("data"));

        // Verify they share the same Arc (shallow clone)
        assert!(std::sync::Arc::ptr_eq(
            &state1.custom_state,
            &state2.custom_state
        ));
    }

    #[test]
    fn test_app_state_debug() {
        let state = AppState::default();
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("AppState"));
        assert!(debug_str.contains("custom_state_count"));
    }

    #[test]
    fn test_app_state_multiple_types() {
        let mut custom_state: HashMap<String, Box<dyn Any + Send + Sync>> = HashMap::new();
        custom_state.insert("string".to_string(), Box::new("hello".to_string()));
        custom_state.insert("number".to_string(), Box::new(42i32));
        custom_state.insert("float".to_string(), Box::new(std::f64::consts::PI));
        custom_state.insert("bool".to_string(), Box::new(true));

        let state = AppState::new(Vec::new(), Vec::new(), BodyConfig::default(), custom_state);

        assert_eq!(state.len(), 4);
        assert_eq!(state.get::<String>("string"), Some(&"hello".to_string()));
        assert_eq!(state.get::<i32>("number"), Some(&42));
        assert_eq!(state.get::<f64>("float"), Some(&std::f64::consts::PI));
        assert_eq!(state.get::<bool>("bool"), Some(&true));
    }

    #[test]
    fn test_app_state_overwrite_value() {
        let mut custom_state: HashMap<String, Box<dyn Any + Send + Sync>> = HashMap::new();
        custom_state.insert("counter".to_string(), Box::new(1i32));

        let state = AppState::new(Vec::new(), Vec::new(), BodyConfig::default(), custom_state);

        assert_eq!(state.get::<i32>("counter"), Some(&1));
    }

    #[test]
    fn test_macro_generated_extension_trait() {
        // Define test types
        #[derive(Debug, PartialEq, Clone)]
        struct MyDb {
            url: String,
        }

        #[derive(Debug, PartialEq, Clone)]
        struct MyCache {
            size: usize,
        }

        // Generate extension trait
        app_state_ext! {
            trait TestAppStateExt {
                fn my_db(&self) -> Option<&MyDb> { "my_db" }
                fn my_cache(&self) -> Option<&MyCache> { "my_cache" }
            }
        }

        let mut custom_state: HashMap<String, Box<dyn Any + Send + Sync>> = HashMap::new();
        let db = MyDb {
            url: "postgres://localhost".to_string(),
        };
        let cache = MyCache { size: 1024 };
        custom_state.insert("my_db".to_string(), Box::new(db.clone()));
        custom_state.insert("my_cache".to_string(), Box::new(cache.clone()));

        let state = AppState::new(Vec::new(), Vec::new(), BodyConfig::default(), custom_state);

        // Use generated methods
        assert_eq!(state.my_db(), Some(&db));
        assert_eq!(state.my_cache(), Some(&cache));
    }
}
