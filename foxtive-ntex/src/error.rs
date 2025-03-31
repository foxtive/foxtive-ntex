use crate::error::helpers::make_http_error_response;
use crate::http::response::anyhow::helpers::make_status_code;
use foxtive::prelude::AppMessage;
use foxtive::Error;
#[cfg(feature = "multipart")]
use foxtive_ntex_multipart::{ErrorMessage as MultipartErrorMessage, MultipartError};
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
    AppError(#[from] Error),
    #[error("{0}")]
    AppMessage(#[from] AppMessage),
    #[error("Payload Error: {0}")]
    PayloadError(#[from] PayloadError),
    #[error("Utf8 Error: {0}")]
    Utf8Error(#[from] FromUtf8Error),
    #[cfg(feature = "validator")]
    #[error("Validation Error: {0}")]
    ValidationError(#[from] validator::ValidationErrors),
    #[cfg(feature = "multipart")]
    #[error("Multipart Error: {0}")]
    MultipartError(#[from] MultipartError),
}

impl HttpError {
    pub fn into_app_error(self) -> foxtive::Error {
        foxtive::Error::from(self)
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
            #[cfg(feature = "validator")]
            HttpError::ValidationError(_) => StatusCode::BAD_REQUEST,
            HttpError::PayloadError(_) => StatusCode::BAD_REQUEST,
            #[cfg(feature = "multipart")]
            HttpError::MultipartError(err) => match err {
                MultipartError::ValidationError(err) => match err.error {
                    MultipartErrorMessage::InvalidFileExtension(_)
                    | MultipartErrorMessage::InvalidContentType(_) => {
                        StatusCode::UNSUPPORTED_MEDIA_TYPE
                    }
                    _ => StatusCode::BAD_REQUEST,
                },
                _ => StatusCode::BAD_REQUEST,
            },
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        make_http_error_response(self)
    }
}

pub(crate) mod helpers {
    use crate::enums::ResponseCode;
    use crate::helpers::responder::Responder;
    use crate::http::response::anyhow::helpers::make_response;
    use crate::http::HttpError;
    use foxtive::prelude::AppMessage;
    use log::error;
    use ntex::web::HttpResponse;

    pub(crate) fn make_http_error_response(err: &HttpError) -> HttpResponse {
        match err {
            HttpError::AppMessage(m) => make_response(&m.clone().ae()),
            HttpError::AppError(e) => make_response(e),
            #[cfg(feature = "validator")]
            HttpError::ValidationError(e) => {
                error!("Validation Error: {}", e);
                Responder::send_msg(e.errors(), ResponseCode::BadRequest, "Validation Error")
            }
            HttpError::PayloadError(e) => {
                error!("Payload Error: {}", e);
                Responder::send_msg(e.to_string(), ResponseCode::BadRequest, "Payload Error")
            }
            #[cfg(feature = "multipart")]
            HttpError::MultipartError(err) => {
                error!("Multipart Error: {}", err);
                Responder::send_msg(
                    err.to_string(),
                    ResponseCode::BadRequest,
                    "File Upload Error",
                )
            }
            _ => {
                error!("Error: {}", err);
                make_response(&foxtive::Error::from(AppMessage::InternalServerError))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foxtive::Error;

    #[test]
    fn test_app_error() {
        let error = HttpError::AppError(Error::from(AppMessage::InternalServerError));
        let app_error = make_http_error_response(&error);
        assert_eq!(app_error.status(), 500);
    }

    #[test]
    fn test_app_message() {
        let error = HttpError::AppMessage(AppMessage::InternalServerError);
        let app_error = make_http_error_response(&error);
        assert_eq!(app_error.status(), 500);
    }

    #[test]
    fn test_std_error() {
        let error = HttpError::Std(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Test",
        )));
        let app_error = make_http_error_response(&error);
        assert_eq!(app_error.status(), 500);
    }

    #[test]
    fn test_payload_error() {
        let error = HttpError::PayloadError(PayloadError::Overflow);
        let app_error = make_http_error_response(&error);
        assert_eq!(app_error.status(), 400);
    }

    #[cfg(feature = "validator")]
    #[test]
    fn test_validation_error() {
        let error = HttpError::ValidationError(validator::ValidationErrors::new());
        let app_error = make_http_error_response(&error);
        assert_eq!(app_error.status(), 400);
    }

    #[cfg(feature = "multipart")]
    #[test]
    fn test_multipart_error() {
        use foxtive_ntex_multipart::InputError;

        let error = HttpError::MultipartError(MultipartError::ValidationError(InputError {
            error: MultipartErrorMessage::InvalidFileExtension(Some("mp4".to_string())),
            name: "image".to_string(),
        }));

        let app_error = make_http_error_response(&error);

        assert_eq!(app_error.status(), 400);
    }
}
