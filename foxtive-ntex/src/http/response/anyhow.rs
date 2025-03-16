use crate::error::HttpError;
use foxtive::prelude::AppMessage;
use foxtive::Error;
use ntex::http::error::BlockingError;
use ntex::http::StatusCode;
use ntex::web::{HttpRequest, HttpResponse, WebResponseError};
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

#[derive(Debug, Error)]
pub struct ResponseError {
    pub error: foxtive::Error,
}

impl ResponseError {
    pub fn new(error: foxtive::Error) -> Self {
        Self { error }
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

impl From<HttpError> for ResponseError {
    fn from(value: HttpError) -> Self {
        match value {
            HttpError::AppError(e) => ResponseError::new(e),
            HttpError::AppMessage(e) => ResponseError::new(e.ae()),
            HttpError::Std(e) => ResponseError::new(Error::from_boxed(e)),
            _ => ResponseError::new(foxtive::Error::from(value)),
        }
    }
}

impl From<foxtive::Error> for ResponseError {
    fn from(value: Error) -> Self {
        ResponseError::new(value)
    }
}

impl From<BlockingError<foxtive::Error>> for ResponseError {
    fn from(value: BlockingError<foxtive::Error>) -> Self {
        match value {
            BlockingError::Error(err) => ResponseError::new(err),
            BlockingError::Canceled => ResponseError::new(AppMessage::InternalServerError.ae()),
        }
    }
}

pub mod helpers {
    use foxtive::prelude::AppMessage;
    use ntex::http::body::Body;
    use ntex::http::error::BlockingError;
    use ntex::http::{header, StatusCode};
    use ntex::web::HttpResponse;

    pub fn make_status_code(err: &foxtive::Error) -> StatusCode {
        match err.downcast_ref::<AppMessage>() {
            Some(msg) => msg.status_code(),
            None => match err.downcast_ref::<BlockingError<AppMessage>>() {
                None => StatusCode::INTERNAL_SERVER_ERROR,
                Some(err) => match err {
                    BlockingError::Error(msg) => msg.status_code(),
                    BlockingError::Canceled => StatusCode::INTERNAL_SERVER_ERROR,
                },
            },
        }
    }

    pub fn make_response(err: &foxtive::Error) -> HttpResponse {
        let status = make_status_code(err);

        match err.downcast_ref::<AppMessage>() {
            Some(msg) => make_json_response(msg.message(), status),
            None => match err.downcast_ref::<BlockingError<AppMessage>>() {
                None => make_json_response(
                    AppMessage::InternalServerError.message(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ),
                Some(err) => match err {
                    BlockingError::Error(msg) => make_json_response(msg.message(), status),
                    BlockingError::Canceled => make_json_response(
                        AppMessage::InternalServerError.message(),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ),
                },
            },
        }
    }

    pub fn make_json_response(body: String, status: StatusCode) -> HttpResponse {
        let mut resp = HttpResponse::new(status);
        resp.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        resp.set_body(Body::from(body))
    }
}
