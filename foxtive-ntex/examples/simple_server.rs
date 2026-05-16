use foxtive::prelude::*;
use foxtive::Environment;
use foxtive_ntex::http::kernel::{Controller, Route};
use foxtive_ntex::http::response::ext::StructResponseExt;
use foxtive_ntex::http::{HttpResult, JsonBody, Method};
use foxtive_ntex::{AppState, ServerBuilder};
use ntex::web::{get, post, HttpRequest};
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
    let _state = AppState::builder(
        foxtive_ntex::http::server::BodyConfig::default().json_limit(1024 * 1024),
    )
    .with_allowed_origin("http://localhost:3000")
    .with_allowed_method(Method::GET)
    .with_allowed_method(Method::POST)
    .build();

    let route_factory = || {
        vec![Route {
            controllers: vec![
                Controller::new("", |cfg| {
                    cfg.service(root_handler)
                        .service(health_handler)
                        .service(echo_handler);
                })
            ],
            prefix: "/".to_string(),
            middlewares: vec![],
        }]
    };

    let foxtive = foxtive::setup::FoxtiveSetup {
        env_prefix: "FOXTIVE".to_string(),
        private_key: "demo-private-key".to_string(),
        public_key: "demo-public-key".to_string(),
        app_key: "demo-app-key".to_string(),
        app_code: "demo".to_string(),
        app_name: "simple-server".to_string(),
        env: Environment::Development,
        #[cfg(feature = "jwt")]
        jwt_iss_public_key: "".to_string(),
        #[cfg(feature = "jwt")]
        jwt_token_lifetime: 0,
        #[cfg(feature = "database")]
        db_config: foxtive::database::DbConfig::create("sqlite::memory:"),
    };

    println!("📡 Server starting on http://127.0.0.1:3000");

    ServerBuilder::dev_mode("127.0.0.1", 3000, foxtive)
        .allowed_origins(vec!["http://localhost:3000".to_string()])
        .allowed_methods(vec![Method::GET, Method::POST])
        .route_factory(route_factory)
        .start(|_state| async move { Ok(()) })
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
