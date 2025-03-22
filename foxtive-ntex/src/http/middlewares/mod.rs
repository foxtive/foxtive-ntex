use crate::http::middlewares::executor::MiddlewareExecutor;
use foxtive::prelude::AppResult;
use ntex::web::{HttpRequest, WebResponse};
use std::future::Future;
use std::pin::Pin;

mod executor;

pub type BeforeMiddlewareHandler =
    fn(HttpRequest) -> Pin<Box<dyn Future<Output = AppResult<HttpRequest>>>>;

pub type AfterMiddlewareHandler =
    fn(WebResponse) -> Pin<Box<dyn Future<Output = AppResult<WebResponse>>>>;

#[derive(Clone)]
pub enum Middleware {
    /// Before middleware, called before the request is handled by the handler
    Before(BeforeMiddlewareHandler),
    /// After middleware, called after the request is handled by the handler
    After(AfterMiddlewareHandler),
}

impl Middleware {
    pub fn middleware(&self) -> MiddlewareExecutor {
        MiddlewareExecutor::new(self.clone())
    }
}
