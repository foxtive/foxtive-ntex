//! REST API server with custom state and graceful shutdown
//!
//! Run: cargo run --example rest_api_server
//! Test: curl http://localhost:3000/

use foxtive::prelude::*;
use foxtive::Environment;
use foxtive_ntex::app_state_ext;
use foxtive_ntex::http::response::ext::StructResponseExt;
use foxtive_ntex::http::{HttpResult, Method};
use foxtive_ntex::ServerBuilder;
use ntex::web::{self, get, HttpRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct DatabasePool {
    connection_string: String,
    max_connections: u32,
    active_connections: Arc<Mutex<u32>>,
}

#[allow(dead_code)]
impl DatabasePool {
    fn new(connection_string: &str, max_connections: u32) -> Self {
        Self {
            connection_string: connection_string.to_string(),
            max_connections,
            active_connections: Arc::new(Mutex::new(0)),
        }
    }

    async fn get_connection(&self) -> Result<DbConnection, String> {
        let mut active = self.active_connections.lock().await;
        if *active >= self.max_connections {
            return Err("Connection pool exhausted".to_string());
        }
        *active += 1;
        Ok(DbConnection {
            id: *active,
            pool: self.clone(),
        })
    }

    async fn close(&self) {
        println!("Closing database pool: {}", self.connection_string);
        let mut active = self.active_connections.lock().await;
        *active = 0;
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct DbConnection {
    id: u32,
    pool: DatabasePool,
}

impl Drop for DbConnection {
    fn drop(&mut self) {
        println!("Connection {} dropped", self.id);
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct AppConfig {
    app_name: String,
    version: String,
    environment: String, // "development", "staging", "production"
    debug_mode: bool,
}

#[allow(dead_code)]
impl AppConfig {
    fn is_production(&self) -> bool {
        self.environment == "production"
    }
}

app_state_ext! {
    pub trait ApiStateExt {
        fn db_pool(&self) -> Option<&DatabasePool> { "db_pool" }
        fn app_config(&self) -> Option<&AppConfig> { "app_config" }
    }
}

#[ntex::main]
async fn main() -> AppResult<()> {
    let db_pool = DatabasePool::new("postgresql://localhost:5432/mydb", 10);

    let app = App::builder("rest-api-server", "rest-api")
        .environment(Environment::Development)
        .app_key("demo-app-key")
        .private_key("demo-private-key")
        .public_key("demo-public-key")
        .build()
        .await?;

    println!("Starting server on http://127.0.0.1:3000");
    println!("Press Ctrl+C to stop\n");

    ServerBuilder::dev_mode("127.0.0.1", 3000, app)
        .allowed_origins(vec![
            "http://localhost:3000".to_string(),
            "https://example.com".to_string(),
        ])
        .allowed_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .configure(|cfg| {
            cfg.service(root_handler)
                .service(health_handler)
                .service(users_handler);
        })
        // raw_configure lets you add routes alongside the main configure() block
        .raw_configure(|cfg| {
            cfg.service(
                web::resource("/api/ping").route(
                    web::get().to(|| async {
                        serde_json::json!({"msg": "pong"}).respond()
                    })
                ),
            );
        })
        .shutdown_config(foxtive_ntex::ShutdownConfig::new(30))
        .register_shutdown_service("database", 1, move || {
            let pool = db_pool.clone();
            async move {
                pool.close().await;
            }
        })
        .start(|_app| async move { Ok(()) })
        .await?;

    Ok(())
}


#[get("/")]
async fn root_handler(_req: HttpRequest) -> HttpResult {
    serde_json::json!({
        "message": "Welcome to Foxtive REST API!",
        "version": "1.0.0",
        "endpoints": [
            "GET /",
            "GET /health",
            "GET /users"
        ]
    })
    .respond()
}

#[get("/health")]
async fn health_handler() -> HttpResult {
    serde_json::json!({
        "status": "healthy",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    })
    .respond()
}

#[get("/users")]
async fn users_handler() -> HttpResult {
    // Demo endpoint - in real app, would query database
    serde_json::json!({
        "users": [
            {"id": 1, "name": "Alice", "email": "alice@example.com"},
            {"id": 2, "name": "Bob", "email": "bob@example.com"},
            {"id": 3, "name": "Charlie", "email": "charlie@example.com"}
        ],
        "total": 3
    })
    .respond()
}
