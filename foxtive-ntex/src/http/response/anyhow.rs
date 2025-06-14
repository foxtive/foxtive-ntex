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
    use crate::contracts::ResponseCodeContract;
    use crate::enums::ResponseCode;
    use crate::helpers::responder::Responder;
    use crate::http::HttpError;
    use foxtive::prelude::AppMessage;
    use log::error;
    use ntex::http::error::BlockingError;
    use ntex::http::StatusCode;
    use ntex::web::{HttpResponse, WebResponseError};

    pub fn make_status_code(err: &foxtive::Error) -> StatusCode {
        match err.downcast_ref::<AppMessage>() {
            Some(msg) => msg.status_code(),
            None => match err.downcast_ref::<BlockingError<AppMessage>>() {
                Some(err) => match err {
                    BlockingError::Error(msg) => msg.status_code(),
                    BlockingError::Canceled => StatusCode::INTERNAL_SERVER_ERROR,
                },
                None => match err.downcast_ref::<HttpError>() {
                    Some(err) => err.status_code(),
                    None => match err.downcast_ref::<serde_json::Error>() {
                        None => StatusCode::INTERNAL_SERVER_ERROR,
                        Some(err) => {
                            error!("Json-Error: {err}");
                            StatusCode::BAD_REQUEST
                        }
                    },
                },
            },
        }
    }

    pub fn make_response(err: &foxtive::Error) -> HttpResponse {
        let status = make_status_code(err);

        match err.downcast_ref::<AppMessage>() {
            Some(msg) => {
                msg.log();
                make_json_response(msg.message(), status)
            }
            None => match err.downcast_ref::<BlockingError<AppMessage>>() {
                Some(err) => match err {
                    BlockingError::Error(msg) => {
                        error!("Error: {msg}");
                        make_json_response(msg.message(), status)
                    }
                    BlockingError::Canceled => {
                        error!("Ntex Blocking Error");
                        make_json_response(
                            AppMessage::InternalServerError.message(),
                            StatusCode::INTERNAL_SERVER_ERROR,
                        )
                    }
                },
                None => match err.downcast_ref::<HttpError>() {
                    Some(err) => crate::error::helpers::make_http_error_response(err),
                    None => match err.downcast_ref::<serde_json::Error>() {
                        Some(err) => {
                            error!("Error: {err}");
                            // We can't send JSON error as a response, we don't know what may be leaked
                            make_json_response(
                                "Data processing error".to_string(),
                                StatusCode::BAD_REQUEST,
                            )
                        }
                        None => {
                            error!("Error: {err}");
                            make_json_response(
                                AppMessage::InternalServerError.message(),
                                StatusCode::INTERNAL_SERVER_ERROR,
                            )
                        }
                    },
                },
            },
        }
    }

    pub fn make_json_response(body: String, status: StatusCode) -> HttpResponse {
        let code = ResponseCode::from_status(status);
        Responder::message(&body, code)
    }
}
