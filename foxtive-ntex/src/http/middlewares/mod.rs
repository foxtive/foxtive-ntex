use crate::http::middlewares::executor::MiddlewareExecutor;
use foxtive::prelude::AppResult;
use ntex::web::{HttpRequest, WebResponse};
use std::sync::Arc;

mod executor;

#[foxtive::async_trait(?Send)]
pub trait BeforeMiddleware {
    async fn handle(&self, req: HttpRequest) -> AppResult<HttpRequest>;
}

#[foxtive::async_trait(?Send)]
pub trait AfterMiddleware {
    async fn handle(&self, res: WebResponse) -> AppResult<WebResponse>;
}

#[derive(Clone)]
pub enum Middleware {
    /// Before middleware, called before the request is handled by the handler
    Before(Arc<dyn BeforeMiddleware>),
    /// After middleware, called after the request is handled by the handler
    After(Arc<dyn AfterMiddleware>),
}

impl Middleware {
    pub fn middleware(&self) -> MiddlewareExecutor {
        MiddlewareExecutor::new(self.clone())
    }
}
