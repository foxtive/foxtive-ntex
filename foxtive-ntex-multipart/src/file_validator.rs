use crate::result::MultipartResult;
use crate::{FileInput, MultipartError};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct InputError {
    pub name: String,
    pub error: ErrorMessage,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ErrorMessage {
    NoFiles(String),
    FileTooSmall(String, String, usize),
    FileTooLarge(String, String, usize),
    TooFewFiles(String, usize),
    TooManyFiles(String, usize),
    InvalidFileExtension(String, String, Option<String>),
    InvalidContentType(String, String, String),
    MissingFileExtension(String, String, String),
}

#[derive(Debug, Clone, Default)]
pub struct Validator {
    rules: HashMap<String, FileRules>,
}

// Struct for File Validation Rules
#[derive(Debug, Default, Clone)]
pub struct FileRules {
    /// Whether field is required
    pub required: bool,

    /// Whether file extension is required
    pub extension_required: bool,

    /// Min file size in bytes
    pub min_size: Option<usize>,

    /// Max file size in bytes
    pub max_size: Option<usize>,

    /// Allowed file extensions (should be lowercase for consistent matching)
    pub allowed_extensions: Option<Vec<String>>,

    /// Allowed content types (should be lowercase for consistent matching)
    /// 
    /// # Security Note
    /// 
    /// Content-Type validation relies on the client-provided header, which can be
    /// easily spoofed. For production systems handling sensitive file uploads, consider
    /// implementing additional validation such as:
    /// - Magic byte checking (file signature validation)
    /// - File content inspection
    /// - Integration with antivirus scanning
    pub allowed_content_types: Option<Vec<String>>,

    /// Min number of files, this only works when validating through `Multipart` struct
    pub min_files: Option<usize>,

    /// Max number of files, this only works when validating through `Multipart` struct
    pub max_files: Option<usize>,
}

impl Validator {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_rule(mut self, field: &str, rules: FileRules) -> Self {
        self.rules.insert(field.to_string(), rules);
        self
    }

    pub fn validate(&self, files: &HashMap<String, Vec<FileInput>>) -> MultipartResult<()> {
        for (field_name, rules) in &self.rules {
            let files = files.get(field_name);
            Self::validate_files(field_name.clone(), files, rules)
                .map_err(MultipartError::ValidationError)?;
        }

        Ok(())
    }

    fn validate_files(
        field_name: String,
        files: Option<&Vec<FileInput>>,
        rules: &FileRules,
    ) -> Result<(), InputError> {
        if files.is_none() {
            if rules.required {
                return Err(InputError {
                    name: field_name.clone(),
                    error: ErrorMessage::NoFiles(field_name),
                });
            }

            return Ok(());
        }

        let files = files.unwrap();
        let file_count = files.len();

        // Validate required
        if rules.required && file_count == 0 {
            return Err(InputError {
                name: field_name.clone(),
                error: ErrorMessage::NoFiles(field_name),
            });
        }

        if file_count < rules.min_files.unwrap_or(0) {
            return Err(InputError {
                name: field_name.clone(),
                error: ErrorMessage::TooFewFiles(field_name, file_count),
            });
        }

        if file_count > rules.max_files.unwrap_or(usize::MAX) {
            return Err(InputError {
                name: field_name.clone(),
                error: ErrorMessage::TooManyFiles(field_name, file_count),
            });
        }

        for file in files {
            Self::validate_file(&field_name, rules.clone(), file)?;
        }

        // If all checks passed
        Ok(())
    }

    fn validate_file(
        field_name: &str,
        rule: FileRules,
        file: &FileInput,
    ) -> Result<(), InputError> {
        // Validate file extension
        if rule.extension_required && file.extension.is_none() {
            return Err(InputError {
                name: field_name.to_string(),
                error: ErrorMessage::MissingFileExtension(
                    field_name.to_string(),
                    file.file_name.clone(),
                    "Extension is required".to_string(),
                ),
            });
        }

        // Validate file size
        if let Some(min_size) = rule.min_size
            && file.size < min_size
        {
            return Err(InputError {
                name: field_name.to_string(),
                error: ErrorMessage::FileTooSmall(
                    field_name.to_string(),
                    file.file_name.clone(),
                    min_size,
                ),
            });
        }

        if let Some(max_size) = rule.max_size
            && file.size > max_size
        {
            return Err(InputError {
                name: field_name.to_string(),
                error: ErrorMessage::FileTooLarge(
                    field_name.to_string(),
                    file.file_name.clone(),
                    max_size,
                ),
            });
        }

        // Validate file extension
        if let Some(allowed_extensions) = &rule.allowed_extensions {
            if let Some(extension) = &file.extension {
                // Normalize both sides to lowercase for consistent comparison
                let ext_lower = extension.to_lowercase();
                if !allowed_extensions.iter().any(|e| e.to_lowercase() == ext_lower) {
                    return Err(InputError {
                        name: field_name.to_string(),
                        error: ErrorMessage::InvalidFileExtension(
                            field_name.to_string(),
                            file.file_name.clone(),
                            file.extension.clone(),
                        ),
                    });
                }
            } else {
                return Err(InputError {
                    name: field_name.to_string(),
                    error: ErrorMessage::MissingFileExtension(
                        field_name.to_string(),
                        file.file_name.clone(),
                        "File extension is missing but required".to_string(),
                    ),
                });
            }
        }

        // Validate content type
        if let Some(allowed_content_types) = &rule.allowed_content_types
            && !allowed_content_types.contains(&file.content_type.to_lowercase())
        {
            return Err(InputError {
                name: field_name.to_string(),
                error: ErrorMessage::InvalidContentType(
                    field_name.to_string(),
                    file.file_name.clone(),
                    format!(
                        "Invalid content type. Allowed content types are: {allowed_content_types:?}"
                    ),
                ),
            });
        }

        Ok(())
    }
}

impl Display for ErrorMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MultipartError;

    // Helper function to create a file input
    fn create_file_input(
        field_name: &str,
        file_name: &str,
        size: usize,
        extension: Option<&str>,
        content_type: &str,
    ) -> FileInput {
        FileInput {
            field_name: field_name.to_string(),
            file_name: file_name.to_string(),
            size,
            extension: extension.map(|e| e.to_string()),
            content_type: content_type.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_validate_required_files_missing() {
        let validator = Validator::new().add_rule(
            "file_field",
            FileRules {
                required: true,
                ..Default::default()
            },
        );

        let mut files = HashMap::new();
        files.insert("file_field".to_string(), vec![]);

        let result = validator.validate(&files);

        assert!(result.is_err());
        if let Err(MultipartError::ValidationError(InputError { error, .. })) = result {
            assert_eq!(error, ErrorMessage::NoFiles("file_field".to_string()));
        }
    }

    #[test]
    fn test_validate_required_files_present() {
        let validator = Validator::new().add_rule(
            "file_field",
            FileRules {
                required: true,
                ..Default::default()
            },
        );

        let mut files = HashMap::new();
        let file = create_file_input("file_field", "test.jpg", 500, Some("jpg"), "image/jpeg");
        files.insert("file_field".to_string(), vec![file]);

        let result = validator.validate(&files);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_file_size_too_small() {
        let validator = Validator::new().add_rule(
            "file_field",
            FileRules {
                min_size: Some(1024),
                ..Default::default()
            },
        );

        let mut files = HashMap::new();
        let file = create_file_input("file_field", "test.jpg", 500, Some("jpg"), "image/jpeg");
        files.insert("file_field".to_string(), vec![file]);

        let result = validator.validate(&files);

        assert!(result.is_err());
        if let Err(MultipartError::ValidationError(InputError { error, .. })) = result {
            assert_eq!(
                error,
                ErrorMessage::FileTooSmall("file_field".to_string(), "test.jpg".to_string(), 1024)
            );
        }
    }

    #[test]
    fn test_validate_file_size_ok() {
        let validator = Validator::new().add_rule(
            "file_field",
            FileRules {
                min_size: Some(100),
                max_size: Some(1024),
                ..Default::default()
            },
        );

        let mut files = HashMap::new();
        let file = create_file_input("file_field", "test.jpg", 500, Some("jpg"), "image/jpeg");
        files.insert("file_field".to_string(), vec![file]);

        let result = validator.validate(&files);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_file_extension_invalid() {
        let validator = Validator::new().add_rule(
            "file_field",
            FileRules {
                allowed_extensions: Some(vec!["jpg".to_string(), "png".to_string()]),
                ..Default::default()
            },
        );

        let mut files = HashMap::new();
        let file = create_file_input("file_field", "test.txt", 500, Some("txt"), "image/jpeg");
        files.insert("file_field".to_string(), vec![file]);

        let result = validator.validate(&files);

        assert!(result.is_err());
        if let Err(MultipartError::ValidationError(InputError { error, .. })) = result {
            assert_eq!(
                error,
                ErrorMessage::InvalidFileExtension(
                    "file_field".to_string(),
                    "test.txt".to_string(),
                    Some("txt".to_string())
                )
            );
        }
    }

    #[test]
    fn test_validate_file_extension_valid() {
        let validator = Validator::new().add_rule(
            "file_field",
            FileRules {
                allowed_extensions: Some(vec!["jpg".to_string(), "png".to_string()]),
                ..Default::default()
            },
        );

        let mut files = HashMap::new();
        let file = create_file_input("file_field", "test.jpg", 500, Some("jpg"), "image/jpeg");
        files.insert("file_field".to_string(), vec![file]);

        let result = validator.validate(&files);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_content_type_invalid() {
        let validator = Validator::new().add_rule(
            "file_field",
            FileRules {
                allowed_content_types: Some(vec![
                    "image/jpeg".to_string(),
                    "image/png".to_string(),
                ]),
                ..Default::default()
            },
        );

        let mut files = HashMap::new();
        let file = create_file_input(
            "file_field",
            "test.jpg",
            500,
            Some("jpg"),
            "application/pdf",
        );
        files.insert("file_field".to_string(), vec![file]);

        let result = validator.validate(&files);

        assert!(result.is_err());
        if let Err(MultipartError::ValidationError(InputError { error, .. })) = result {
            assert_eq!(
                error,
                ErrorMessage::InvalidContentType(
                    "file_field".to_string(),
                    "test.jpg".to_string(),
                    "Invalid content type. Allowed content types are: [\"image/jpeg\", \"image/png\"]".to_string()
                )
            );
        }
    }

    #[test]
    fn test_validate_file_count_too_few() {
        let validator = Validator::new().add_rule(
            "file_field",
            FileRules {
                min_files: Some(2),
                ..Default::default()
            },
        );

        let mut files = HashMap::new();
        let file = create_file_input("file_field", "test.jpg", 500, Some("jpg"), "image/jpeg");
        files.insert("file_field".to_string(), vec![file]);

        let result = validator.validate(&files);

        assert!(result.is_err());
        if let Err(MultipartError::ValidationError(InputError { error, .. })) = result {
            assert_eq!(
                error,
                ErrorMessage::TooFewFiles("file_field".to_string(), 1)
            );
        }
    }

    #[test]
    fn test_validate_file_count_too_many() {
        let validator = Validator::new().add_rule(
            "file_field",
            FileRules {
                max_files: Some(1),
                ..Default::default()
            },
        );

        let mut files = HashMap::new();
        let file1 = create_file_input("file_field", "test1.jpg", 500, Some("jpg"), "image/jpeg");
        let file2 = create_file_input("file_field", "test2.jpg", 500, Some("jpg"), "image/jpeg");
        files.insert("file_field".to_string(), vec![file1, file2]);

        let result = validator.validate(&files);

        assert!(result.is_err());
        if let Err(MultipartError::ValidationError(InputError { error, .. })) = result {
            assert_eq!(
                error,
                ErrorMessage::TooManyFiles("file_field".to_string(), 2)
            );
        }
    }

    #[test]
    fn test_validate_file_count_ok() {
        let validator = Validator::new().add_rule(
            "file_field",
            FileRules {
                max_files: Some(2),
                min_files: Some(1),
                ..Default::default()
            },
        );

        let mut files = HashMap::new();
        let file1 = create_file_input("file_field", "test1.jpg", 500, Some("jpg"), "image/jpeg");
        let file2 = create_file_input("file_field", "test2.jpg", 500, Some("jpg"), "image/jpeg");
        files.insert("file_field".to_string(), vec![file1, file2]);

        let result = validator.validate(&files);

        assert!(result.is_ok());
    }
}
