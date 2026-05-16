# Foxtive-Ntex

A high-performance micro-framework built on [Foxtive](https://github.com/foxtive/foxtive) and [ntex](https://ntex.rs), designed for building production-ready Rust web services with minimal boilerplate.

## Features

- **High Performance**: Built on ntex, one of the fastest Rust web frameworks
- **Type Safety**: Leverage Rust's type system for safe request handling
- **Minimal Boilerplate**: Focus on business logic, not framework configuration
- **Extensible State Management**: Custom state injection with type-safe access
- **Graceful Shutdown**: Coordinated service cleanup with priority-based shutdown
- **Body Parsing**: Automatic JSON deserialization with configurable size limits
- **CORS Support**: Built-in CORS middleware with flexible configuration
- **JWT Authentication**: Optional JWT token extraction and validation
- **File Uploads**: Multipart form data support (optional feature)
- **Database Integration**: Optional database connection pooling
- **Validation**: Request validation with validator crate (optional feature)

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
foxtive-ntex = "0.30"
```

### Simple Server

```rust
use foxtive::prelude::*;
use foxtive::Environment;
use foxtive_ntex::http::{HttpResult, Method};
use foxtive_ntex::http::kernel::{Controller, Route};
use foxtive_ntex::http::response::ext::StructResponseExt;
use foxtive_ntex::{AppState, ServerBuilder};
use ntex::web::{get, HttpRequest};

#[get("/")]
async fn hello(_req: HttpRequest) -> HttpResult {
    serde_json::json!({ "message": "Hello, World!" }).respond()
}

#[ntex::main]
async fn main() -> AppResult<()> {
    let state = AppState::builder(
        foxtive_ntex::http::server::BodyConfig::default()
    )
    .with_allowed_origin("http://localhost:3000")
    .with_allowed_method(Method::GET)
    .build();

    let route_factory = || {
        vec![Route {
            controllers: vec![Controller::new("", |cfg| {
                cfg.service(hello);
            })],
            prefix: "/".to_string(),
            middlewares: vec![],
        }]
    };

    let foxtive = foxtive::setup::FoxtiveSetup {
        env_prefix: "APP".to_string(),
        private_key: "your-private-key".to_string(),
        public_key: "your-public-key".to_string(),
        app_key: "your-app-key".to_string(),
        app_code: "myapp".to_string(),
        app_name: "My Application".to_string(),
        env: Environment::Development,
    };

    ServerBuilder::dev_mode("127.0.0.1", 3000, foxtive)
        .route_factory(route_factory)
        .start(|_state| async move { Ok(()) })
        .await?;

    Ok(())
}
```

Run with:
```bash
cargo run
```

Test it:
```bash
curl http://localhost:3000/
```

## Examples

The repository includes several examples demonstrating different use cases:

### Simple Server
Basic HTTP server with JSON responses.

```bash
cargo run --example simple_server
```

### REST API Server
Production-ready REST API with custom state and graceful shutdown.

```bash
cargo run --example rest_api_server
```

### Custom State
Demonstrates custom state injection and type-safe access.

```bash
cargo run --example custom_state
```

## Core Concepts

### Server Configuration

Foxtive-Ntex provides three preset configurations:

- **`dev_mode()`**: Single worker, relaxed timeouts for development
- **`production_mode()`**: Optimized settings matching CPU cores
- **`high_performance_mode()`**: Maximum throughput for high-traffic APIs

```rust
ServerBuilder::dev_mode("127.0.0.1", 3000, foxtive)
    .workers(4)
    .max_conn(50_000)
    .keep_alive(ntex::time::Seconds(30))
    .route_factory(routes)
    .start(|_state| async move { Ok(()) })
    .await?;
```

### Custom State Management

Inject custom services into your application state:

```rust
use foxtive_ntex::app_state_ext;
use foxtive_ntex::fox_state;

struct DatabasePool { /* ... */ }
struct CacheService { /* ... */ }

// Generate type-safe accessors
app_state_ext! {
    pub trait AppStateExt {
        fn db_pool(&self) -> Option<&DatabasePool> { "db_pool" }
        fn cache(&self) -> Option<&CacheService> { "cache_service" }
    }
}

// In handlers
#[get("/users")]
async fn list_users() -> HttpResult {
    let db = fox_state::<DatabasePool>()?;
    // Use database pool...
}
```

### Request Body Parsing

Automatic JSON deserialization with size limits:

```rust
use foxtive_ntex::http::JsonBody;
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[post("/users")]
async fn create_user(body: JsonBody<CreateUser>) -> HttpResult {
    // Direct field access via Deref
    println!("Creating user: {} <{}>", body.name, body.email);
    
    // Or extract owned value
    let user = body.into_inner();
}
```

Configure body size limits:

```rust
use foxtive_ntex::http::server::BodyConfig;

let config = BodyConfig::default()
    .json_limit(1024 * 1024)      // 1 MB
    .string_limit(512 * 1024)     // 512 KB
    .byte_limit(2 * 1024 * 1024); // 2 MB
```

### Graceful Shutdown

Register services for coordinated cleanup:

```rust
ServerBuilder::dev_mode("127.0.0.1", 3000, foxtive)
    .shutdown_config(foxtive_ntex::ShutdownConfig::new(30))
    .register_shutdown_service("database", 1, || {
        let pool = db_pool.clone();
        async move {
            pool.close().await;
        }
    })
    .register_shutdown_service("cache", 2, || {
        let cache = cache.clone();
        async move {
            cache.flush().await;
        }
    })
    .start(|_state| async move { Ok(()) })
    .await?;
```

Services shut down in priority order (lower number = first).

### Bootstrap Callback

Run initialization logic before the server starts:

```rust
ServerBuilder::dev_mode("127.0.0.1", 3000, foxtive)
    .start(|state| async move {
        // Run database migrations
        // Warm up caches
        // Initialize services
        
        println!("Server bootstrap complete");
        Ok(())
    })
    .await?;
```

## Optional Features

Enable features in your `Cargo.toml`:

```toml
[dependencies]
foxtive-ntex = { version = "0.30", features = ["jwt", "multipart", "validator"] }
```

Available features:

- **`jwt`**: JWT authentication support
- **`multipart`**: File upload handling
- **`validator`**: Request validation
- **`static`**: Static file serving
- **`database`**: Database connection pooling
- **`strum`**: Enum utilities

## Architecture

Foxtive-Ntex is built on a layered architecture:

1. **ntex**: High-performance HTTP server foundation
2. **Foxtive**: Core utilities, environment management, error handling
3. **Foxtive-Ntex**: Web framework layer with routing, middleware, state management

This design provides the performance of ntex with the developer experience of a full-featured framework.

## Testing

Run the test suite:

```bash
cargo test
```

All tests pass with 123 unit tests covering core functionality.

## Migration Guide

### From 0.29 to 0.30

- `JsonConfig` renamed to `BodyConfig` (old name deprecated)
- Added separate limits for JSON, string, and byte bodies
- `boot_thread()` renamed to `route_factory()` (old name deprecated)
- `JsonBody<T>` now discards raw JSON after deserialization (memory optimization)

### From 0.28 to 0.29

- `DeJsonBody` removed; use `JsonBody<T>` with automatic deserialization
- `JsonBody` now implements `Deref` for direct field access
- Added `route_factory_arc()` for shared route factories

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.
