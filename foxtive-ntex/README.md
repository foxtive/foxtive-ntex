# Foxtive-Ntex

Build web services with [Foxtive](https://github.com/foxtive/foxtive) and [ntex](https://ntex.rs). You register your services in a DI container, wire up routes, and start the server. No global state, no hidden magic.

## Install

```toml
[dependencies]
foxtive-ntex = "1.0"
```

## Quick Start

```rust
use foxtive::prelude::*;
use foxtive::{App, Environment};
use foxtive_ntex::http::response::ext::StructResponseExt;
use foxtive_ntex::http::{HttpResult, Method};
use foxtive_ntex::ServerBuilder;
use ntex::web::{get, HttpRequest};

#[get("/")]
async fn hello(_req: HttpRequest) -> HttpResult {
    serde_json::json!({ "message": "Hello, World!" }).respond()
}

#[ntex::main]
async fn main() -> AppResult<()> {
    let app = App::builder("my-app", "MYAPP")
        .environment(Environment::Development)
        .app_key("your-app-key")
        .private_key("your-private-key")
        .public_key("your-public-key")
        .build()
        .await?;

    ServerBuilder::dev_mode("127.0.0.1", 3000, app)
        .allowed_origins(vec!["http://localhost:3000".to_string()])
        .allowed_methods(vec![Method::GET])
        .configure(|cfg| {
            cfg.service(hello);
        })
        .start(|_app| async move { Ok(()) })
        .await?;

    Ok(())
}
```

```bash
cargo run
# curl http://localhost:3000/
```

## Dependency Injection

`App` is your DI container. Register things at startup, pull them out in handlers.

```rust
use foxtive::App;
use foxtive_ntex::ext::request::RequestExt;
use std::sync::Arc;

struct DatabasePool { /* ... */ }
struct CacheService { /* ... */ }

let app = App::builder("my-app", "MYAPP")
    .register(DatabasePool::new())
    .register(CacheService::new())
    .build()
    .await?;

// In a handler:
#[get("/users")]
async fn list_users(req: HttpRequest) -> HttpResult {
    let db: Arc<DatabasePool> = req.service::<DatabasePool>()?;
    let cache: Arc<CacheService> = req.service::<CacheService>()?;
    // ...
}
```

That's it. No global state. `Arc<T>` everywhere, cheap to clone.

## Routing

You get two options. Pick whichever fits.

**Structured routes** — group controllers under prefixes, attach middleware per group:

```rust
use foxtive_ntex::http::kernel::{Controller, Route};

ServerBuilder::create("127.0.0.1", 3000, app)
    .route_factory(|| {
        vec![Route {
            prefix: "/api/v1".to_string(),
            middlewares: vec![Middleware::before(AuthMiddleware)],
            controllers: vec![
                Controller::new("/users", |cfg| {
                    cfg.service(list_users);
                    cfg.service(create_user);
                }),
            ],
        }]
    })
    .start(|_app| async move { Ok(()) })
    .await?;
```

**Raw ntex config** — when you need full control, skip the abstraction:

```rust
ServerBuilder::create("127.0.0.1", 3000, app)
    .configure(|cfg| {
        cfg.service(
            web::scope("/api/v1")
                .middleware(auth_middleware)
                .service(users_handler)
                .service(posts_handler),
        );
    })
    .start(|_app| async move { Ok(()) })
    .await?;
```

`raw_configure()` adds routes on top of either approach — useful for one-off endpoints.

## Middleware

Implement `BeforeMiddleware` to run logic before the handler, `AfterMiddleware` for after:

```rust
use foxtive_ntex::http::middlewares::{BeforeMiddleware, Middleware, MiddlewareFlow};
use ntex::web::HttpRequest;

struct AuthMiddleware;

#[async_trait(?Send)]
impl BeforeMiddleware for AuthMiddleware {
    async fn handle(&self, req: HttpRequest) -> AppResult<MiddlewareFlow<HttpRequest>> {
        // Check token, modify request, or return early
        Ok(MiddlewareFlow::Continue(req))
    }
}
```

Attach to a route group:

```rust
let route = Route {
    prefix: "/api".to_string(),
    middlewares: vec![Middleware::before(AuthMiddleware)],
    controllers: vec![Controller::new("/protected", |cfg| { /* ... */ })],
};
```

## Extractors

JSON, string, and byte bodies come built-in with configurable size limits:

```rust
use foxtive_ntex::http::JsonBody;

#[post("/users")]
async fn create_user(body: JsonBody<CreateUser>) -> HttpResult {
    // body.name, body.email via Deref
    let user = body.into_inner(); // or take ownership
}
```

Size limits:

```rust
use foxtive_ntex::http::server::BodyConfig;

let config = BodyConfig::default()
    .json_limit(1024 * 1024)      // 1 MB
    .string_limit(512 * 1024)     // 512 KB
    .byte_limit(2 * 1024 * 1024); // 2 MB
```

Other extractors: `StringBody`, `ByteBody`, `ClientInfo`, `JwtAuthToken` (behind `jwt` feature).

## Server Presets

Three presets, override anything:

```rust
// Development — 1 worker, relaxed timeouts
ServerBuilder::dev_mode("127.0.0.1", 3000, app)

// Production — workers = CPU cores, tighter timeouts
ServerBuilder::production_mode("0.0.0.0", 8080, app)

// High throughput — 2x CPU cores, aggressive limits
ServerBuilder::high_performance_mode("0.0.0.0", 8080, app)
```

## Graceful Shutdown

Register cleanup handlers, they run in priority order (lower = first) with per-service timeouts:

```rust
ServerBuilder::dev_mode("127.0.0.1", 3000, app)
    .shutdown_config(foxtive_ntex::ShutdownConfig::new(30))
    .register_shutdown_service("database", 1, move || {
        let pool = db_pool.clone();
        async move { pool.close().await }
    })
    .register_shutdown_service("cache", 2, move || {
        let cache = cache.clone();
        async move { cache.flush().await }
    })
    .start(|_app| async move { Ok(()) })
    .await?;
```

The `start()` callback gives you `Arc<App>` so you can run migrations, warm caches, etc. before accepting connections:

```rust
.start(|app| async move {
    let db = app.require::<DatabasePool>()?;
    // run migrations...
    Ok(())
})
```

## Health Checks

Turn it on:

```rust
ServerBuilder::dev_mode("127.0.0.1", 3000, app)
    .health_check_path("/system/health-check")
    .start(|_app| async move { Ok(()) })
    .await?;
```

Returns 200 + JSON report when healthy, 503 when something's down.

## Features

```toml
foxtive-ntex = { version = "1.0", features = ["jwt", "multipart", "validator"] }
```

| Feature | What it does |
|---------|-------------|
| `jwt` | JWT token extraction and validation |
| `multipart` | File uploads via foxtive-ntex-multipart |
| `validator` | Request validation with the `validator` crate |
| `static` | Serve static files from a directory |
| `database` | Database connection pooling (via foxtive) |
| `strum` | Enum utilities |
| `openssl` | TLS via OpenSSL |
| `rustls` | TLS via Rustls |

## Examples

```bash
cargo run --example simple_server     # basic JSON API
cargo run --example rest_api_server   # REST API with shutdown handling
cargo run --example custom_state      # DI with custom services
```

## Migration Guide

### 0.31 → 1.0

This is the DI release. The big changes:

- `ServerBuilder` now takes `Arc<App>` instead of `FoxtiveSetup`. Build your app with `App::builder().build().await?`.
- Register services with `App::builder().register(service)` — no more `fox_state` or global accessors.
- In handlers, use `req.service::<T>()` to get `Arc<T>`.
- The `start()` callback receives `Arc<App>`.
- New: `configure()` for raw ntex routing, `raw_configure()` for additive routes, `health_check_path()` for built-in health checks.

### 0.30 → 0.31

- `JsonConfig` → `BodyConfig` (old name deprecated)
- `boot_thread()` → `route_factory()` (old name deprecated)
- `JsonBody<T>` no longer stores raw JSON after deserialization

### 0.29 → 0.30

- `DeJsonBody` removed — use `JsonBody<T>`
- `JsonBody` implements `Deref` for direct field access
- Added `route_factory_arc()`

## License

MIT
