use crate::FileInput;
use crate::file_validator::{ErrorMessage, InputError};
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
                match &err.error {
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
                            display_name,
                            count
                        )
                    }
                    ErrorMessage::TooManyFiles(field_name, count) => {
                        let display_name = field_name.replace("_", " ");
                        write!(
                            f,
                            "Too many files uploaded for field '{}'. Current count: {}, maximum allowed",
                            display_name,
                            count
                        )
                    }
                    ErrorMessage::InvalidFileExtension(field_name, file_name, ext) => {
                        let display_name = field_name.replace("_", " ");
                        match ext {
                            Some(extension) => write!(
                                f,
                                "Invalid file extension '.{}' for file '{}' in field '{}'",
                                extension,
                                file_name,
                                display_name
                            ),
                            None => write!(
                                f,
                                "Missing file extension for file '{}' in field '{}'",
                                file_name,
                                display_name
                            )
                        }
                    }
                    ErrorMessage::InvalidContentType(field_name, file_name, message) => {
                        let display_name = field_name.replace("_", " ");
                        write!(
                            f,
                            "Invalid content type for file '{}' in field '{}': {}",
                            file_name,
                            display_name,
                            message
                        )
                    }
                    ErrorMessage::MissingFileExtension(field_name, file_name, message) => {
                        let display_name = field_name.replace("_", " ");
                        write!(
                            f,
                            "Missing file extension for file '{}' in field '{}': {}",
                            file_name,
                            display_name,
                            message
                        )
                    }
                }
            }
        }
    }
}