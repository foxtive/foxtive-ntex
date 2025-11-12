use foxtive::prelude::AppResult;
use ntex::web::{HttpRequest, HttpResponse, WebResponse};
use std::sync::Arc;

mod executor;

pub use executor::MiddlewareChain;

/// Result type for middleware execution
pub enum MiddlewareFlow<T> {
    /// Continue to next middleware/handler
    Continue(T),
    /// Stop execution and return this response immediately
    Respond(HttpRequest, HttpResponse),
}

#[foxtive::async_trait(?Send)]
pub trait BeforeMiddleware {
    async fn handle(&self, req: HttpRequest) -> AppResult<MiddlewareFlow<HttpRequest>>;
}

#[foxtive::async_trait(?Send)]
pub trait AfterMiddleware {
    async fn handle(&self, res: WebResponse) -> AppResult<WebResponse>;
}

#[derive(Clone)]
pub enum Middleware {
    Before(Arc<dyn BeforeMiddleware>),
    After(Arc<dyn AfterMiddleware>),
}

impl Middleware {
    pub fn before<M: BeforeMiddleware + 'static>(middleware: M) -> Self {
        Self::Before(Arc::new(middleware))
    }

    pub fn after<M: AfterMiddleware + 'static>(middleware: M) -> Self {
        Self::After(Arc::new(middleware))
    }

    pub fn middleware(self) -> MiddlewareChain {
        MiddlewareChain::single(self)
    }
}