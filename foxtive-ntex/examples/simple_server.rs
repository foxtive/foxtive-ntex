use foxtive::App;
use foxtive::Environment;
use foxtive::prelude::*;
use foxtive_ntex::ServerBuilder;
use foxtive_ntex::http::response::ext::StructResponseExt;
use foxtive_ntex::http::{HttpResult, JsonBody, Method};
use ntex::web::{HttpRequest, get, post};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct EchoRequest {
    message: String,
}

#[derive(Debug, Serialize)]
struct EchoResponse {
    received: String,
    timestamp: u64,
}

#[ntex::main]
async fn main() -> AppResult<()> {
    let app = App::builder("simple-server", "FOXTIVE")
        .environment(Environment::Development)
        .app_key("demo-app-key")
        .private_key("demo-private-key")
        .public_key("demo-public-key")
        .build()
        .await?;

    println!("Server starting on http://127.0.0.1:3000");

    ServerBuilder::dev_mode("0.0.0.0", 3000, app)
        .allowed_origins(vec!["http://localhost:3000".to_string()])
        .allowed_methods(vec![Method::GET, Method::POST])
        .body_config(foxtive_ntex::http::server::BodyConfig::default().json_limit(1024 * 1024))
        .configure(|cfg| {
            cfg.service(root_handler)
                .service(health_handler)
                .service(echo_handler);
        })
        .start(|_app| async move { Ok(()) })
        .await?;

    Ok(())
}

#[get("/")]
async fn root_handler(_req: HttpRequest) -> HttpResult {
    serde_json::json!({
        "message": "Welcome to Foxtive!",
        "version": "1.0.0",
        "endpoints": [
            "GET /",
            "GET /health",
            "POST /echo"
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

#[post("/echo")]
async fn echo_handler(body: JsonBody<EchoRequest>) -> HttpResult {
    let response = EchoResponse {
        received: body.message.clone(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    response.respond()
}
