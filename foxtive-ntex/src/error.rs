use crate::error::helpers::make_http_error_response;
use foxtive::prelude::AppMessage;
#[cfg(feature = "multipart")]
use foxtive_ntex_multipart::{ErrorMessage as MultipartErrorMessage, MultipartError};
use ntex::http::StatusCode;
use ntex::http::error::PayloadError;
use ntex::web::error::{BlockingError, JsonError};
use ntex::web::{HttpRequest, HttpResponse, WebResponseError};
use std::string::FromUtf8Error;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Error, Debug)]
pub enum HttpError {
    #[error("{0}")]
    Std(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("{0}")]
    AppMessage(#[from] AppMessage),
    #[error("Payload Error: {0}")]
    PayloadError(#[from] PayloadError),
    #[error("Join Error: {0}")]
    JoinError(#[from] JoinError),
    #[error("Utf8 Error: {0}")]
    Utf8Error(#[from] FromUtf8Error),
    #[error("Json Error: {0}")]
    JsonError(#[from] JsonError),
    #[cfg(feature = "validator")]
    #[error("Validation Error: {0}")]
    ValidationError(#[from] validator::ValidationErrors),
    #[cfg(feature = "multipart")]
    #[error("Multipart Error: {0}")]
    MultipartError(#[from] MultipartError),
}

impl From<Box<dyn std::error::Error + Send + Sync>> for HttpError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        HttpError::Std(error)
    }
}

impl From<BlockingError<AppMessage>> for HttpError {
    fn from(value: BlockingError<AppMessage>) -> Self {
        match value {
            BlockingError::Error(e) => HttpError::AppMessage(e),
            BlockingError::Canceled => {
                HttpError::AppMessage(AppMessage::internal_server_error("Internal Server Error"))
            }
        }
    }
}

impl WebResponseError for HttpError {
    fn status_code(&self) -> StatusCode {
        match self {
            HttpError::AppMessage(m) => m.status_code(),
            #[cfg(feature = "validator")]
            HttpError::ValidationError(_) => StatusCode::BAD_REQUEST,
            HttpError::PayloadError(_) | HttpError::JsonError(_) => StatusCode::BAD_REQUEST,
            #[cfg(feature = "multipart")]
            HttpError::MultipartError(err) => match err {
                MultipartError::ValidationError(err) => match err.error {
                    MultipartErrorMessage::InvalidFileExtension(_, _, _)
                    | MultipartErrorMessage::InvalidContentType(_, _, _) => {
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
    use crate::http::HttpError;
    use crate::http::responder::Responder;
    use crate::http::response::error::helpers::make_json_response;
    use foxtive::prelude::AppMessage;
    #[cfg(feature = "multipart")]
    use foxtive_ntex_multipart::MultipartError;
    use ntex::http::StatusCode;
    use ntex::web::HttpResponse;
    use tracing::error;

    pub(crate) fn make_http_error_response(err: &HttpError) -> HttpResponse {
        match err {
            HttpError::AppMessage(m) => {
                m.log();
                make_json_response(m.message(), m.status_code())
            }
            #[cfg(feature = "validator")]
            HttpError::ValidationError(e) => {
                Responder::send_msg(e.errors(), ResponseCode::BadRequest, "Validation Error")
            }
            HttpError::PayloadError(e) => {
                error!("Payload Error: {e}");
                Responder::send_msg(e.to_string(), ResponseCode::BadRequest, "Payload Error")
            }
            HttpError::JsonError(e) => {
                error!("Json Error: {e}");
                Responder::send_msg(
                    e.to_string(),
                    ResponseCode::BadRequest,
                    "Json Payload Error",
                )
            }
            #[cfg(feature = "multipart")]
            HttpError::MultipartError(err) => match err {
                MultipartError::ValidationError(val) => {
                    let msg = err.to_string();
                    Responder::send_msg(
                        serde_json::json!({"field": val.name, "error": msg}),
                        ResponseCode::BadRequest,
                        &msg,
                    )
                }
                _ => {
                    let msg = err.to_string();
                    Responder::send_msg(
                        serde_json::json!({"message": msg}),
                        ResponseCode::BadRequest,
                        &msg,
                    )
                }
            },
            _ => {
                error!("Error: {err}");
                make_json_response(
                    AppMessage::internal_server_error("Internal Server Error").message(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_message() {
        let error = HttpError::AppMessage(AppMessage::internal_server_error("Error"));
        let app_error = helpers::make_http_error_response(&error);
        assert_eq!(app_error.status(), 500);
    }

    #[test]
    fn test_std_error() {
        #[allow(clippy::io_other_error)]
        let error = HttpError::Std(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Test",
        )));
        let app_error = helpers::make_http_error_response(&error);
        assert_eq!(app_error.status(), 500);
    }

    #[test]
    fn test_payload_error() {
        let error = HttpError::PayloadError(PayloadError::Overflow);
        let app_error = helpers::make_http_error_response(&error);
        assert_eq!(app_error.status(), 400);
    }

    #[cfg(feature = "validator")]
    #[test]
    fn test_validation_error() {
        let error = HttpError::ValidationError(validator::ValidationErrors::new());
        let app_error = helpers::make_http_error_response(&error);
        assert_eq!(app_error.status(), 400);
    }

    #[cfg(feature = "multipart")]
    #[test]
    fn test_multipart_error() {
        use foxtive_ntex_multipart::InputError;

        let error = HttpError::MultipartError(MultipartError::ValidationError(InputError {
            error: MultipartErrorMessage::InvalidFileExtension(
                "image".to_string(),
                "invalid ext".to_string(),
                Some("mp4".to_string()),
            ),
            name: "image".to_string(),
        }));

        let app_error = helpers::make_http_error_response(&error);

        assert_eq!(app_error.status(), 400);
    }
}
