//! Demonstrates programmatic server shutdown.
//!
//! Run:  cargo run --example programmatic_shutdown
//!
//! Test:
//!   curl http://localhost:3000/stop       (via ShutdownSignal)
//!   curl http://localhost:3000/app-stop   (via app.shutdown())

use foxtive::App;
use foxtive::Environment;
use foxtive::prelude::*;
use foxtive_ntex::http::response::ext::StructResponseExt;
use foxtive_ntex::http::{HttpResult, Method};
use foxtive_ntex::{AppState, ServerBuilder, ShutdownSignal};
use ntex::web::{self, HttpRequest, get};
use std::sync::Arc;

#[ntex::main]
async fn main() -> AppResult<()> {
    let app = App::builder("programmatic-shutdown", "PSHUT")
        .environment(Environment::Development)
        .app_key("demo-app-key")
        .on_shutdown(|app| {
            Box::pin(async move {
                println!("[hook] Running app shutdown hook...");
                let _ = app;
            })
        })
        .build()
        .await?;

    println!("Server starting on http://127.0.0.1:3000");
    println!("Endpoints:");
    println!("  GET /          - status");
    println!("  GET /stop      - stop via ShutdownSignal");
    println!("  GET /app-stop  - stop via app.shutdown()");
    println!();

    let app_for_server = app.clone();

    ServerBuilder::dev_mode("127.0.0.1", 3000, app_for_server)
        .allowed_origins(vec!["http://localhost:3000".to_string()])
        .allowed_methods(vec![Method::GET])
        .configure(|cfg| {
            cfg.service(index).service(stop).service(app_stop);
        })
        .register_shutdown_service("example-cleanup", 1, || async {
            println!("[cleanup] Running example cleanup service...");
        })
        .start(|_app| async move { Ok(()) })
        .await?;

    println!("Server has stopped. Goodbye!");
    Ok(())
}

#[get("/")]
async fn index(_req: HttpRequest) -> HttpResult {
    serde_json::json!({
        "status": "running",
        "endpoints": ["/stop", "/app-stop"]
    })
    .respond()
}

#[get("/stop")]
async fn stop(state: web::types::State<AppState>) -> HttpResult {
    println!("[/stop] Shutdown requested via ShutdownSignal");

    if let Some(signal) = state.get::<ShutdownSignal>("shutdown_signal") {
        if signal.trigger().await {
            println!("[/stop] Signal sent");
        }
    }

    serde_json::json!({
        "method": "ShutdownSignal",
        "status": "shutting_down"
    })
    .respond()
}

#[get("/app-stop")]
async fn app_stop(app: web::types::State<Arc<App>>) -> HttpResult {
    println!("[/app-stop] Shutdown requested via app.shutdown()");

    let app = app.clone();
    ntex::rt::spawn(async move {
        ntex::time::sleep(ntex::time::Millis(100)).await;
        app.shutdown().await;
    });

    serde_json::json!({
        "method": "app.shutdown()",
        "status": "shutting_down"
    })
    .respond()
}
