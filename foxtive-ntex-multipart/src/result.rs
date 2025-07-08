use crate::file_validator::{ErrorMessage, InputError};
use crate::FileInput;
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
            MultipartError::ValidationError(err) => {
                let field_name = err.name.clone().replace("_", " ");
                match err.error.clone() {
                    ErrorMessage::NoFiles => {
                        write!(f, "No files were uploaded for field: '{field_name}'")
                    }
                    ErrorMessage::FileTooSmall(size) => {
                        write!(
                            f,
                            "File size is too small for field '{field_name}'. Minimum size is {}",
                            FileInput::format_size(size)
                        )
                    }
                    ErrorMessage::FileTooLarge(size) => {
                        write!(
                            f,
                            "File size is too big for field '{field_name}'. Maximum size is {}",
                            FileInput::format_size(size)
                        )
                    }
                    ErrorMessage::TooFewFiles(count) => {
                        write!(
                            f,
                            "Too few files uploaded for field '{field_name}'. Minimum is {count}"
                        )
                    }
                    ErrorMessage::TooManyFiles(count) => {
                        write!(
                            f,
                            "Too many files uploaded for field '{field_name}'. Maximum is {count}"
                        )
                    }
                    ErrorMessage::InvalidFileExtension(ext) => {
                        write!(
                            f,
                            "Invalid file extension for field '{field_name}': .{}",
                            ext.clone().unwrap_or_default()
                        )
                    }
                    ErrorMessage::InvalidContentType(mime) => {
                        write!(f, "Invalid mime type: {mime}")
                    }
                    ErrorMessage::MissingFileExtension(mime) => {
                        write!(f, "Invalid file, file extension is required: {mime}")
                    }
                }
            }
        }
    }
}
