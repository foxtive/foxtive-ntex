use crate::enums::ResponseCode;
use crate::http::Method;
use crate::http::middlewares::{Middleware, MiddlewareChain};
use crate::http::responder::Responder;
use ntex::http::header;
use ntex::web::ServiceConfig;
use ntex::web::middleware::Logger;
use ntex::{web, web::Route as NtexRoute};
use ntex_cors::Cors;
use tracing::info;

pub struct Controller {
    pub path: String,
    pub handler: fn(cfg: &mut ServiceConfig),
}

pub struct Route {
    pub prefix: String,
    pub middlewares: Vec<Middleware>,
    pub controllers: Vec<Controller>,
}

pub fn register_routes(config: &mut ServiceConfig, routes: Vec<Route>) {
    tracing::debug!("discovering routes...");

    for route in routes {
        for controller in route.controllers {
            let path = format!("{}{}", route.prefix, controller.path);
            let display_path = if path.is_empty() { "/" } else { &path };
            tracing::debug!("route group: {}", display_path);

            let path_str = if path.is_empty() { "/" } else { &path };
            let scope = web::scope(path_str);

            if !route.middlewares.is_empty() {
                config.service(
                    scope
                        .middleware(MiddlewareChain::new(route.middlewares.clone()))
                        .configure(controller.handler)
                );
            } else {
                config.service(scope.configure(controller.handler));
            }
        }
    }

    tracing::debug!("route discovery finished");
}

pub fn setup_logger() -> Logger {
    Logger::default()
        .exclude("/favicon.ico")
        .exclude("/system/health-check")
        .exclude("/api/v1/admin/health-check")
}

pub fn setup_cors(origins: Vec<String>, methods: Vec<Method>) -> Cors {
    let mut cors = Cors::new();

    // Handle wildcard separately from specific origins
    let has_wildcard = origins.iter().any(|o| o == "*");

    if has_wildcard {
        info!("registering cors origin: '*' (wildcard)");
        cors = cors.allowed_origin("*").send_wildcard();
    } else {
        for origin in origins {
            info!("registering cors origin: '{}'", origin);
            cors = cors.allowed_origin(&origin);
        }
    }

    let allowed_methods = if methods.is_empty() {
        vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ]
    } else {
        methods
    };

    cors.allowed_methods(allowed_methods)
        .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
        .allowed_header(header::CONTENT_TYPE)
        .max_age(3600)
}

pub fn ntex_default_service() -> NtexRoute {
    web::to(|| async {
        Responder::message("Requested Resource(s) Not Found", ResponseCode::NotFound)
    })
}