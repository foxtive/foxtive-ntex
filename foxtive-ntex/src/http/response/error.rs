use crate::error::HttpError;
use foxtive::prelude::AppMessage;
use ntex::http::StatusCode;
use ntex::http::error::BlockingError;
use ntex::web::{HttpRequest, HttpResponse, WebResponseError};
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug, Error)]
pub struct ResponseError {
    pub error: HttpError,
}

impl ResponseError {
    pub fn new(error: impl Into<HttpError>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

impl Display for ResponseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl WebResponseError for ResponseError {
    fn status_code(&self) -> StatusCode {
        helpers::make_status_code(&self.error)
    }

    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        helpers::make_response(&self.error)
    }
}

impl From<AppMessage> for ResponseError {
    fn from(value: AppMessage) -> Self {
        ResponseError::new(HttpError::AppMessage(value))
    }
}

impl From<BlockingError<AppMessage>> for ResponseError {
    fn from(value: BlockingError<AppMessage>) -> Self {
        match value {
            BlockingError::Error(err) => ResponseError::new(HttpError::AppMessage(err)),
            BlockingError::Canceled => ResponseError::new(HttpError::AppMessage(
                AppMessage::internal_server_error("Internal Server Error"),
            )),
        }
    }
}

pub mod helpers {
    use crate::contracts::ResponseCodeContract;
    use crate::enums::ResponseCode;
    use crate::error::HttpError;
    use crate::http::responder::Responder;
    use ntex::http::StatusCode;
    use ntex::web::HttpResponse;

    pub fn make_status_code(err: &HttpError) -> StatusCode {
        use ntex::web::WebResponseError;
        err.status_code()
    }

    pub fn make_response(err: &HttpError) -> HttpResponse {
        crate::error::helpers::make_http_error_response(err)
    }

    pub fn make_json_response(body: impl Into<String>, status: StatusCode) -> HttpResponse {
        let code = ResponseCode::from_status(status);
        let body = body.into();
        Responder::message(&body, code)
    }
}
