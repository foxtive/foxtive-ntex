use crate::content_disposition::ContentDisposition;
use crate::file_validator::Validator;
use crate::result::{MultipartError, MultipartResult};
use crate::{FileRules, Multipart};
use ntex::http::HeaderMap;
use ntex::util::Bytes;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Default, Clone)]
pub struct FileInput {
    pub file_name: String,
    pub field_name: String,
    pub size: usize, // Size in bytes
    pub content_type: String,
    pub bytes: Vec<Bytes>,
    pub extension: Option<String>,
    pub content_disposition: ContentDisposition,
}

impl FileInput {
    // Create a new FileInput instance from headers and content disposition
    pub fn create(headers: &HeaderMap, cd: ContentDisposition) -> MultipartResult<Self> {
        let content_type = Self::get_content_type(headers)?;

        let variables = cd.get_variables();
        let field = variables.get("name").cloned().unwrap();
        let name = variables.get("filename").cloned().unwrap();

        let binding = name.clone();
        let split_name: Vec<&str> = binding.split('.').collect();

        Ok(Self {
            content_type,
            size: 0,
            bytes: vec![],
            file_name: name,
            field_name: field,
            extension: split_name.last().map(|e| e.to_string()),
            content_disposition: cd,
        })
    }

    // Save the file to the specified path
    pub async fn save(&self, path: impl AsRef<Path>) -> MultipartResult<()> {
        Multipart::save_file(self, path).await
    }

    pub fn validate(&self, rules: FileRules) -> MultipartResult<()> {
        let mut files = HashMap::new();
        files.insert(self.field_name.clone(), vec![self.clone()]);

        Validator::new()
            .add_rule(&self.field_name, rules)
            .validate(&files)
    }

    /// Calculate the file size from bytes collected
    pub fn calculate_size(&self) -> usize {
        self.bytes.iter().map(|b| b.len()).sum()
    }

    /// Get the human-readable file size (e.g., "1.2 MB", "300 KB")
    pub fn human_size(&self) -> String {
        let size_in_bytes = self.calculate_size();
        Self::format_size(size_in_bytes)
    }

    // Get the content type from headers
    fn get_content_type(headers: &HeaderMap) -> MultipartResult<String> {
        match headers.get("content-type") {
            None => Err(MultipartError::NoContentType(
                "Empty content type".to_string(),
            )),
            Some(header) => header
                .to_str()
                .map(|v| v.to_string())
                .map_err(|err| MultipartError::NoContentType(err.to_string())),
        }
    }

    // Helper function to format size in bytes to a human-readable string
    pub fn format_size(size_in_bytes: usize) -> String {
        const KILOBYTE: usize = 1024;
        const MEGABYTE: usize = KILOBYTE * 1024;
        const GIGABYTE: usize = MEGABYTE * 1024;

        if size_in_bytes >= GIGABYTE {
            format!("{:.2} GB", size_in_bytes as f64 / GIGABYTE as f64)
        } else if size_in_bytes >= MEGABYTE {
            format!("{:.2} MB", size_in_bytes as f64 / MEGABYTE as f64)
        } else if size_in_bytes >= KILOBYTE {
            format!("{:.2} KB", size_in_bytes as f64 / KILOBYTE as f64)
        } else {
            format!("{} bytes", size_in_bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test for `human_size`
    #[test]
    fn test_human_readable_size() {
        let file_input = FileInput {
            size: 1048576,                                  // 1 MB in bytes
            bytes: vec![Bytes::from_static(&[0; 1048576])], // Mock 1MB data
            ..Default::default()
        };

        // Test for 1 MB
        assert_eq!(file_input.human_size(), "1.00 MB");

        let file_input = FileInput {
            size: 1572864,                                  // 1.5 MB in bytes
            bytes: vec![Bytes::from_static(&[0; 1572864])], // Mock 1MB data
            ..Default::default()
        };

        // Test for 1.5 MB
        assert_eq!(file_input.human_size(), "1.50 MB");

        let file_input = FileInput {
            size: 102400,                                  // 100 KB in bytes
            bytes: vec![Bytes::from_static(&[0; 102400])], // Mock 100KB data
            ..Default::default()
        };

        // Test for 100 KB (100.00 KB)
        assert_eq!(file_input.human_size(), "100.00 KB");

        let file_input = FileInput {
            size: 1014,                                  // 1234 bytes
            bytes: vec![Bytes::from_static(&[0; 1014])], // Mock 1014 bytes
            ..Default::default()
        };

        // Test for bytes (1014 bytes)
        assert_eq!(file_input.human_size(), "1014 bytes");
    }

    // Test for `calculate_size`
    #[test]
    fn test_calculate_size() {
        let file_input = FileInput {
            bytes: vec![
                Bytes::from_static(&[0; 1024]), // 1 KB
                Bytes::from_static(&[0; 2048]), // 2 KB
                Bytes::from_static(&[0; 4096]), // 4 KB
            ],
            ..Default::default()
        };

        assert_eq!(file_input.calculate_size(), 1024 + 2048 + 4096);
    }
}
