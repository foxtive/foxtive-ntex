//! Custom state injection example
//!
//! Run: cargo run --example custom_state
//! Test: curl http://localhost:3002/

use foxtive::prelude::*;
use foxtive::Environment;
use foxtive_ntex::app_state_ext;
use foxtive_ntex::http::kernel::{Controller, Route};
use foxtive_ntex::http::response::ext::StructResponseExt;
use foxtive_ntex::http::{HttpResult, Method};
use foxtive_ntex::{fox_state, AppState, ServerBuilder};
use ntex::web::get;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct DatabasePool {
    name: String,
    max_connections: u32,
}

impl DatabasePool {
    fn new(name: &str, max_connections: u32) -> Self {
        Self {
            name: name.to_string(),
            max_connections,
        }
    }
}

#[derive(Debug, Clone)]
struct CacheService {
    ttl_seconds: u64,
    data: Arc<Mutex<HashMap<String, String>>>,
}

impl CacheService {
    fn new(ttl_seconds: u64) -> Self {
        Self {
            ttl_seconds,
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn get(&self, key: &str) -> Option<String> {
        let data = self.data.lock().await;
        data.get(key).cloned()
    }

    async fn set(&self, key: String, value: String) {
        let mut data = self.data.lock().await;
        data.insert(key, value);
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct AppConfig {
    app_name: String,
    version: String,
    debug_mode: bool,
}

app_state_ext! {
    pub trait CustomStateExt {
        fn db_pool(&self) -> Option<&DatabasePool> { "db_pool" }
        fn cache_service(&self) -> Option<&CacheService> { "cache_service" }
        fn app_config(&self) -> Option<&AppConfig> { "app_config" }
    }
}

#[ntex::main]
async fn main() -> AppResult<()> {
    let db_pool = DatabasePool::new("postgres://localhost/mydb", 10);
    let cache = CacheService::new(300);
    let config = AppConfig {
        app_name: "Custom State Demo".to_string(),
        version: "1.0.0".to_string(),
        debug_mode: true,
    };

    use foxtive_ntex::http::server::BodyConfig;
    let _state = AppState::builder(BodyConfig::default())
        .with_allowed_origin("http://localhost:3002")
        .with_allowed_method(Method::GET)
        .with_value("db_pool", db_pool.clone())
        .with_value("cache_service", cache.clone())
        .with_value("app_config", config.clone())
        .build();

    let route_factory = || {
        vec![Route {
            controllers: vec![Controller::new("", |cfg| {
                cfg.service(root_handler)
                    .service(health_handler)
                    .service(cache_demo_handler);
            })],
            prefix: "/".to_string(),
            middlewares: vec![],
        }]
    };

    let foxtive = foxtive::setup::FoxtiveSetup {
        env_prefix: "CUSTOM_STATE".to_string(),
        private_key: "demo-private-key".to_string(),
        public_key: "demo-public-key".to_string(),
        app_key: "demo-app-key".to_string(),
        app_code: "custom-state".to_string(),
        app_name: "custom-state-demo".to_string(),
        env: Environment::Development,
        #[cfg(feature = "jwt")]
        jwt_iss_public_key: "".to_string(),
        #[cfg(feature = "jwt")]
        jwt_token_lifetime: 0,
        #[cfg(feature = "database")]
        db_config: foxtive::database::DbConfig::create("sqlite::memory:"),
    };

    let db_pool_clone = db_pool.clone();
    let cache_clone = cache.clone();
    let config_clone = config.clone();

    println!("📡 Server starting on http://127.0.0.1:3002");

    ServerBuilder::dev_mode("127.0.0.1", 3002, foxtive)
        .allowed_origins(vec!["http://localhost:3002".to_string()])
        .allowed_methods(vec![Method::GET])
        .route_factory(route_factory)
        .custom_state_builder(Box::new(move || {
            let mut map: HashMap<String, Box<dyn std::any::Any + Send + Sync>> = HashMap::new();
            map.insert("db_pool".to_string(), Box::new(db_pool_clone));
            map.insert("cache_service".to_string(), Box::new(cache_clone));
            map.insert("app_config".to_string(), Box::new(config_clone));
            map
        }))
        .start(|_state| async move { Ok(()) })
        .await?;

    Ok(())
}

#[get("/")]
async fn root_handler() -> HttpResult {
    let config = fox_state::<AppConfig>()
        .map_err(|e| foxtive_ntex::http::HttpError::from(foxtive::internal_server_error!("{}", e)))?;

    serde_json::json!({
        "message": "Custom State Demo",
        "app": format!("{} v{}", config.app_name, config.version),
        "endpoints": ["GET /", "GET /health", "GET /cache/test"]
    })
    .respond()
}

#[get("/health")]
async fn health_handler() -> HttpResult {
    let _db = fox_state::<DatabasePool>()
        .map_err(|e| foxtive_ntex::http::HttpError::from(foxtive::internal_server_error!("{}", e)))?;

    serde_json::json!({
        "status": "healthy",
        "database": "connected",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    })
    .respond()
}

#[get("/cache/test")]
async fn cache_demo_handler() -> HttpResult {
    let cache = fox_state::<CacheService>()
        .map_err(|e| foxtive_ntex::http::HttpError::from(foxtive::internal_server_error!("{}", e)))?;

    cache.set("test_key".to_string(), "test_value".to_string()).await;
    let value = cache.get("test_key").await;

    serde_json::json!({
        "message": "Cache operation successful",
        "key": "test_key",
        "value": value,
        "ttl_seconds": cache.ttl_seconds
    })
    .respond()
}
