//! Custom state injection example
//!
//! Run: cargo run --example custom_state
//! Test: curl http://localhost:3002/

use foxtive::App;
use foxtive::Environment;
use foxtive::prelude::*;
use foxtive_ntex::ServerBuilder;
use foxtive_ntex::http::response::ext::StructResponseExt;
use foxtive_ntex::http::{HttpResult, Method};
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

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct CacheService {
    ttl_seconds: u64,
    data: Arc<Mutex<HashMap<String, String>>>,
}

#[allow(dead_code)]
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

#[ntex::main]
async fn main() -> AppResult<()> {
    let db_pool = DatabasePool::new("postgres://localhost/mydb", 10);
    let cache = CacheService::new(300);
    let config = AppConfig {
        app_name: "Custom State Demo".to_string(),
        version: "1.0.0".to_string(),
        debug_mode: true,
    };

    let app = App::builder("custom-state-demo", "CUSTOM_STATE")
        .environment(Environment::Development)
        .app_key("demo-app-key")
        .private_key("demo-private-key")
        .public_key("demo-public-key")
        .register(db_pool.clone())
        .register(cache.clone())
        .register(config.clone())
        .build()
        .await?;

    println!("Server starting on http://127.0.0.1:3002");

    ServerBuilder::dev_mode("127.0.0.1", 3002, app)
        .allowed_origins(vec!["http://localhost:3002".to_string()])
        .allowed_methods(vec![Method::GET])
        .configure(|cfg| {
            cfg.service(root_handler)
                .service(health_handler)
                .service(cache_demo_handler);
        })
        .start(|_app| async move { Ok(()) })
        .await?;

    Ok(())
}

#[get("/")]
async fn root_handler() -> HttpResult {
    serde_json::json!({
        "message": "Custom State Demo",
        "app": "Custom State Demo v1.0.0",
        "endpoints": ["GET /", "GET /health", "GET /cache/test"]
    })
    .respond()
}

#[get("/health")]
async fn health_handler() -> HttpResult {
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
    serde_json::json!({
        "message": "Cache operation successful",
        "ttl_seconds": 300
    })
    .respond()
}
