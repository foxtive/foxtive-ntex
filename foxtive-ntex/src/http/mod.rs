use foxtive::prelude::{AppMessage, AppResult};
use ntex::http::error::BlockingError;

pub mod extractors;
pub mod kernel;
pub mod middlewares;
pub mod response;
pub mod server;

use crate::enums::ResponseCode;
use crate::helpers::responder::Responder;
pub use ntex::http::Method;
use ntex::web::ServiceConfig;
pub use ntex_cors::Cors;

pub use crate::error::HttpError;

pub type HttpResult = Result<ntex::web::HttpResponse, HttpError>;

pub type HttpHandler = fn(cfg: &mut ServiceConfig);

pub trait IntoAppResult<T> {
    fn into_app_result(self) -> AppResult<T>;
}

pub trait IntoHttpResult {
    fn into_http_result(self) -> HttpResult;
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

impl IntoHttpResult for AppMessage {
    fn into_http_result(self) -> HttpResult {
        Err(self.into())
    }
}

impl IntoHttpResult for AppResult<AppMessage> {
    fn into_http_result(self) -> HttpResult {
        match self {
            Ok(res) => Ok(Responder::message(&res.message(), ResponseCode::Ok)),
            Err(err) => Err(HttpError::AppError(err)),
        }
    }
}
