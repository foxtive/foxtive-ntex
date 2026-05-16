use crate::file_validator::InputError;
use foxtive::StatusCode;
use foxtive::helpers::time::current_timestamp;
use ntex::web::{HttpRequest, HttpResponse, WebResponseError};
use serde_json::json;
use std::io::Error;
use thiserror::Error;

pub type MultipartResult<T> = Result<T, MultipartError>;

#[derive(Debug, Error)]
pub enum MultipartError {
    #[error("No file was uploaded")]
    NoFile,
    #[error("IO error: {0}")]
    IoError(Error),
    #[error("Invalid content type: {0}")]
    NoContentType(String),
    #[error("Failed to parse post data: {0}")]
    ParseError(String),
    #[error("Data field '{0}' is required")]
    MissingDataField(String),
    #[error("Invalid content disposition: {0}")]
    InvalidContentDisposition(String),
    #[error("Multipart error: {0}")]
    NtexError(NtexMultipartError),
    #[error("Validation error: {}", .0.error)]
    ValidationError(InputError),
    #[error("File '{filename}' in field '{field}' is too large: {size} bytes (max: {max_size} bytes)")]
    FileTooLarge {
        field: String,
        filename: String,
        size: usize,
        max_size: usize,
    },
    #[error("Payload too large for field '{field}': {size} bytes (max: {max_size} bytes)")]
    PayloadTooLarge {
        field: String,
        size: usize,
        max_size: usize,
    },
}

#[derive(Debug, Error)]
pub enum NtexMultipartError {
    #[error("No Content-Type header found")]
    NoContentType,
    #[error("Cannot parse Content-Type header")]
    ParseContentType,
    #[error("Incompatible Content-Type")]
    IncompatibleContentType,
    #[error("Multipart boundary not found")]
    Boundary,
    #[error("Content-Disposition missing")]
    ContentDispositionMissing,
    #[error("Content-Disposition name missing")]
    ContentDispositionNameMissing,
    #[error("Nested multipart not supported")]
    Nested,
    #[error("Multipart stream incomplete")]
    Incomplete,
    #[error("Decode error: {0}")]
    Decode(String),
    #[error("Payload error: {0}")]
    Payload(String),
    #[error("Not consumed")]
    NotConsumed,
    #[error("Field error: {name}: {message}")]
    Field { name: String, message: String },  // ← only this one loses fidelity
    #[error("Duplicate field: {0}")]
    DuplicateField(String),
    #[error("Missing field: {0}")]
    MissingField(String),
    #[error("Unknown field: {0}")]
    UnknownField(String),
    #[error("Blocking error: {0}")]
    Blocking(String),
}

impl From<ntex_multipart::MultipartError> for NtexMultipartError {
    fn from(e: ntex_multipart::MultipartError) -> Self {
        use ntex_multipart::MultipartError as E;
        match e {
            E::NoContentType => Self::NoContentType,
            E::ParseContentType => Self::ParseContentType,
            E::IncompatibleContentType => Self::IncompatibleContentType,
            E::Boundary => Self::Boundary,
            E::ContentDispositionMissing => Self::ContentDispositionMissing,
            E::ContentDispositionNameMissing => Self::ContentDispositionNameMissing,
            E::Nested => Self::Nested,
            E::Incomplete => Self::Incomplete,
            E::Decode(e) => Self::Decode(e.to_string()),
            E::Payload(e) => Self::Payload(e.to_string()),
            E::NotConsumed => Self::NotConsumed,
            E::Field { name, source } => Self::Field { name, message: source.to_string() },
            E::DuplicateField(s) => Self::DuplicateField(s),
            E::MissingField(s) => Self::MissingField(s),
            E::UnknownField(s) => Self::UnknownField(s),
            E::Blocking(e) => Self::Blocking(e.to_string()),
        }
    }
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
            | MultipartError::ValidationError(_)
            | MultipartError::FileTooLarge { .. }
            | MultipartError::PayloadTooLarge { .. } => StatusCode::BAD_REQUEST,
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
            MultipartError::FileTooLarge { field, filename, size, max_size } => {
                let message = format!(
                    "File '{}' in field '{}' exceeds maximum size limit ({} > {} bytes)",
                    filename, field, size, max_size
                );
                send_response(self.status_code(), &message)
            }
            MultipartError::PayloadTooLarge { field, size, max_size } => {
                let message = format!(
                    "Request payload for field '{}' exceeds maximum size limit ({} > {} bytes)",
                    field, size, max_size
                );
                send_response(self.status_code(), &message)
            }
        }
    }
}

fn send_response(status_code: StatusCode, message: &str) -> HttpResponse {
    let data: Option<String> = None;
    let code = match status_code {
        StatusCode::BAD_REQUEST => "004",
        _ => "010",
    };

    HttpResponse::build(status_code).json(&json!({
        "success": false,
        "message": message,
        "code": code,
        "timestamp": current_timestamp(),
        "data": data
    }))
}
