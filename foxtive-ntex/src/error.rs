use crate::http::response::anyhow::helpers::{make_response, make_status_code};
use foxtive::prelude::AppMessage;
use foxtive::Error;
use ntex::http::error::PayloadError;
use ntex::http::StatusCode;
use ntex::web::error::BlockingError;
use ntex::web::{HttpRequest, HttpResponse, WebResponseError};
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HttpError {
    #[error("{0}")]
    Std(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("{0}")]
    AppError(Error),
    #[error("{0}")]
    AppMessage(AppMessage),
    #[error("Payload Error: {0}")]
    PayloadError(#[from] PayloadError),
    #[error("Utf8 Error: {0}")]
    Utf8Error(#[from] FromUtf8Error),
}

impl HttpError {
    pub fn into_app_error(self) -> foxtive::Error {
        foxtive::Error::from(self)
    }
}

impl From<AppMessage> for HttpError {
    fn from(error: AppMessage) -> Self {
        HttpError::AppMessage(error)
    }
}

impl From<Error> for HttpError {
    fn from(value: Error) -> Self {
        HttpError::AppError(value)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for HttpError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        HttpError::Std(error)
    }
}

impl From<BlockingError<Error>> for HttpError {
    fn from(value: BlockingError<Error>) -> Self {
        match value {
            BlockingError::Error(e) => HttpError::AppError(e),
            BlockingError::Canceled => HttpError::AppMessage(AppMessage::InternalServerError),
        }
    }
}

impl WebResponseError for HttpError {
    fn status_code(&self) -> StatusCode {
        match self {
            HttpError::AppMessage(m) => m.status_code(),
            HttpError::AppError(e) => make_status_code(e),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        match self {
            HttpError::AppMessage(m) => make_response(&m.clone().ae()),
            HttpError::AppError(e) => make_response(e),
            _ => make_response(&foxtive::Error::from(AppMessage::InternalServerError)),
        }
    }
}
