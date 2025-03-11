use foxtive::prelude::{AppMessage, AppResult};
use ntex::http::error::BlockingError;
use ntex::web::ServiceConfig;

pub mod extractors;
pub mod kernel;
pub mod middlewares;
pub mod response;
pub mod server;

pub use ntex::http::Method;
pub use ntex_cors::Cors;
use crate::http::response::anyhow::ResponseError;

pub type HttpHandler = fn(cfg: &mut ServiceConfig);

pub type HttpResult = Result<ntex::web::HttpResponse, ResponseError>;

pub trait IntoAppResult<T> {
    fn into_app_result(self) -> AppResult<T>;
}

impl<T> IntoAppResult<T> for Result<AppResult<T>, BlockingError<AppMessage>> {
    fn into_app_result(self) -> AppResult<T> {
        match self {
            Ok(res) => res,
            Err(err) => Err(err.into()),
        }
    }
}

impl<T> IntoAppResult<T> for Result<T, BlockingError<AppMessage>> {
    fn into_app_result(self) -> AppResult<T> {
        match self {
            Ok(res) => Ok(res),
            Err(err) => Err(err.into()),
        }
    }
}
