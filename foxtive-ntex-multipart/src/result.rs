use crate::file_validator::{ErrorMessage, InputError};
use crate::FileInput;
use foxtive::helpers::time::current_timestamp;
use foxtive::StatusCode;
use ntex::web::{HttpRequest, HttpResponse, WebResponseError};
use serde_json::json;
use std::fmt::{Display, Formatter};
use std::io::Error;
use thiserror::Error;

pub type MultipartResult<T> = Result<T, MultipartError>;

#[derive(Debug, Error)]
pub enum MultipartError {
    NoFile,
    IoError(Error),
    NoContentType(String),
    ParseError(String),
    MissingDataField(String),
    InvalidContentDisposition(String),
    NtexError(ntex_multipart::MultipartError),
    ValidationError(InputError),
}

impl From<Error> for MultipartError {
    fn from(value: Error) -> Self {
        MultipartError::IoError(value)
    }
}

impl WebResponseError for MultipartError {
    fn status_code(&self) -> StatusCode {
        match self {
            MultipartError::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            MultipartError::NoFile
            | MultipartError::NoContentType(_)
            | MultipartError::ParseError(_)
            | MultipartError::MissingDataField(_)
            | MultipartError::InvalidContentDisposition(_)
            | MultipartError::NtexError(_)
            | MultipartError::ValidationError(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        match self {
            MultipartError::NoFile => send_response(self.status_code(), "No file(s) were uploaded"),
            MultipartError::IoError(_) => {
                send_response(self.status_code(), "Internal Server Error")
            }
            MultipartError::NoContentType(err) => send_response(self.status_code(), err),
            MultipartError::ParseError(err) => send_response(self.status_code(), err),
            MultipartError::MissingDataField(err) => send_response(self.status_code(), err),
            MultipartError::InvalidContentDisposition(err) => {
                send_response(self.status_code(), err)
            }
            MultipartError::NtexError(err) => send_response(self.status_code(), &err.to_string()),
            MultipartError::ValidationError(err) => {
                send_response(self.status_code(), &err.error.to_string())
            }
        }
    }
}

fn send_response(status_code: StatusCode, message: &str) -> HttpResponse {
    let data: Option<String> = None;
    let code = match status_code {
        StatusCode::BAD_REQUEST => "004",
        StatusCode::INTERNAL_SERVER_ERROR => "010",
    };

    HttpResponse::build(status_code).json(&json!({
        "success": false,
        "message": message,
        "code": code,
        "timestamp": current_timestamp(),
        "data": data
    }))
}
impl Display for MultipartError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MultipartError::IoError(err) => {
                write!(f, "{err}")
            }
            MultipartError::NoFile => {
                write!(f, "No file was uploaded")
            }
            MultipartError::MissingDataField(ct) => {
                write!(f, "Data field '{ct}' is required")
            }
            MultipartError::NoContentType(ct) => {
                write!(f, "Invalid content type: {ct}")
            }
            MultipartError::ParseError(pe) => {
                write!(f, "Failed to parse post data: {pe}")
            }
            MultipartError::InvalidContentDisposition(err) => {
                write!(f, "Invalid content disposition: {err}")
            }
            MultipartError::NtexError(err) => {
                write!(f, "{err}")
            }
            MultipartError::ValidationError(err) => match &err.error {
                ErrorMessage::NoFiles(field_name) => {
                    let display_name = field_name.replace("_", " ");
                    write!(f, "No files were uploaded for field: '{display_name}'")
                }
                ErrorMessage::FileTooSmall(field_name, file_name, size) => {
                    let display_name = field_name.replace("_", " ");
                    write!(
                        f,
                        "File '{}' is too small for field '{}'. Minimum size is {}",
                        file_name,
                        display_name,
                        FileInput::format_size(*size)
                    )
                }
                ErrorMessage::FileTooLarge(field_name, file_name, size) => {
                    let display_name = field_name.replace("_", " ");
                    write!(
                        f,
                        "File '{}' is too large for field '{}'. Maximum size is {}",
                        file_name,
                        display_name,
                        FileInput::format_size(*size)
                    )
                }
                ErrorMessage::TooFewFiles(field_name, count) => {
                    let display_name = field_name.replace("_", " ");
                    write!(
                        f,
                        "Too few files uploaded for field '{}'. Current count: {}, minimum required",
                        display_name, count
                    )
                }
                ErrorMessage::TooManyFiles(field_name, count) => {
                    let display_name = field_name.replace("_", " ");
                    write!(
                        f,
                        "Too many files uploaded for field '{}'. Current count: {}, maximum allowed",
                        display_name, count
                    )
                }
                ErrorMessage::InvalidFileExtension(field_name, file_name, ext) => {
                    let display_name = field_name.replace("_", " ");
                    match ext {
                        Some(extension) => write!(
                            f,
                            "Invalid file extension '.{}' for file '{}' in field '{}'",
                            extension, file_name, display_name
                        ),
                        None => write!(
                            f,
                            "Missing file extension for file '{}' in field '{}'",
                            file_name, display_name
                        ),
                    }
                }
                ErrorMessage::InvalidContentType(field_name, file_name, message) => {
                    let display_name = field_name.replace("_", " ");
                    write!(
                        f,
                        "Invalid content type for file '{}' in field '{}': {}",
                        file_name, display_name, message
                    )
                }
                ErrorMessage::MissingFileExtension(field_name, file_name, message) => {
                    let display_name = field_name.replace("_", " ");
                    write!(
                        f,
                        "Missing file extension for file '{}' in field '{}': {}",
                        file_name, display_name, message
                    )
                }
            },
        }
    }
}
