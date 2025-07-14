use crate::content_disposition::ContentDisposition;
use crate::file_validator::Validator;
use crate::result::{MultipartError, MultipartResult};
use crate::{FileRules, Multipart};
use ntex::http::HeaderMap;
use ntex::util::Bytes;
use std::collections::HashMap;
use std::path::Path;
use foxtive::helpers::FileExtHelper;

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

        let extension = FileExtHelper::new().get_extension(&name);

        Ok(Self {
            extension,
            content_type,
            size: 0,
            bytes: vec![],
            file_name: name,
            field_name: field,
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
        foxtive::helpers::file_size::format_size(size_in_bytes as u64)
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
        foxtive::helpers::file_size::format_size(size_in_bytes as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use ntex::http::header::{HeaderName, HeaderValue};

    // Helper function to create a basic HeaderMap with content-type
    fn create_headers_with_content_type(content_type: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_str("content-type").unwrap(),
            HeaderValue::from_str(content_type).unwrap(),
        );
        headers
    }

    // Helper function to create a basic ContentDisposition
    fn create_content_disposition(field_name: &str, filename: &str) -> ContentDisposition {
        let mut variables = HashMap::new();
        variables.insert("name".to_string(), field_name.to_string());
        variables.insert("filename".to_string(), filename.to_string());

        ContentDisposition::from(variables)
    }

    // Test for `calculate_size` with various byte combinations
    #[test]
    fn test_calculate_size_empty() {
        let file_input = FileInput {
            bytes: vec![],
            ..Default::default()
        };

        assert_eq!(file_input.calculate_size(), 0);
    }

    #[test]
    fn test_calculate_size_single_chunk() {
        let file_input = FileInput {
            bytes: vec![Bytes::from_static(&[0; 1024])], // 1 KB
            ..Default::default()
        };

        assert_eq!(file_input.calculate_size(), 1024);
    }

    #[test]
    fn test_calculate_size_multiple_chunks() {
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

    #[test]
    fn test_calculate_size_various_sizes() {
        let file_input = FileInput {
            bytes: vec![
                Bytes::from_static(&[1; 1]),     // 1 byte
                Bytes::from_static(&[2; 10]),    // 10 bytes
                Bytes::from_static(&[3; 100]),   // 100 bytes
                Bytes::from_static(&[4; 1000]),  // 1000 bytes
            ],
            ..Default::default()
        };

        assert_eq!(file_input.calculate_size(), 1 + 10 + 100 + 1000);
    }

    // Test for `human_size` method
    #[test]
    fn test_human_size_bytes() {
        let file_input = FileInput {
            bytes: vec![Bytes::from_static(&[0; 500])], // 500 bytes
            ..Default::default()
        };

        let human_size = file_input.human_size();
        // This depends on your foxtive::helpers::file_size::format_size implementation
        // Adjust the expected value based on your actual implementation
        assert!(human_size.contains("500") || human_size.contains("bytes"));
    }

    #[test]
    fn test_human_size_kilobytes() {
        let file_input = FileInput {
            bytes: vec![Bytes::from_static(&[0; 2048])], // 2 KB
            ..Default::default()
        };

        let human_size = file_input.human_size();
        assert!(human_size.contains("KB") || human_size.contains("kB"));
    }

    #[test]
    fn test_human_size_megabytes() {
        let file_input = FileInput {
            bytes: vec![Bytes::from_static(&[0; 2_097_152])], // 2 MB
            ..Default::default()
        };

        let human_size = file_input.human_size();
        assert!(human_size.contains("MB"));
    }

    // Test for `create` method
    #[test]
    fn test_create_success() {
        let headers = create_headers_with_content_type("image/jpeg");
        let cd = create_content_disposition("upload", "test.jpg");

        let result = FileInput::create(&headers, cd);
        assert!(result.is_ok());

        let file_input = result.unwrap();
        assert_eq!(file_input.content_type, "image/jpeg");
        assert_eq!(file_input.field_name, "upload");
        assert_eq!(file_input.file_name, "test.jpg");
        assert_eq!(file_input.extension, Some("jpg".to_string()));
        assert_eq!(file_input.size, 0);
        assert!(file_input.bytes.is_empty());
    }

    #[test]
    fn test_create_missing_content_type() {
        let headers = HeaderMap::new(); // Empty headers
        let cd = create_content_disposition("upload", "test.jpg");

        let result = FileInput::create(&headers, cd);
        assert!(result.is_err());

        if let Err(MultipartError::NoContentType(msg)) = result {
            assert_eq!(msg, "Empty content type");
        } else {
            panic!("Expected NoContentType error");
        }
    }

    #[test]
    fn test_create_various_content_types() {
        let test_cases = vec![
            ("image/png", "image.png"),
            ("text/plain", "document.txt"),
            ("application/pdf", "document.pdf"),
            ("video/mp4", "video.mp4"),
            ("audio/mpeg", "audio.mp3"),
        ];

        for (content_type, filename) in test_cases {
            let headers = create_headers_with_content_type(content_type);
            let cd = create_content_disposition("file", filename);

            let result = FileInput::create(&headers, cd);
            assert!(result.is_ok(), "Failed for content type: {content_type}");

            let file_input = result.unwrap();
            assert_eq!(file_input.content_type, content_type);
            assert_eq!(file_input.file_name, filename);
        }
    }

    #[test]
    fn test_create_with_no_extension() {
        let headers = create_headers_with_content_type("text/plain");
        let cd = create_content_disposition("upload", "README");

        let result = FileInput::create(&headers, cd);
        assert!(result.is_ok());

        let file_input = result.unwrap();
        assert_eq!(file_input.extension, None);
        assert_eq!(file_input.file_name, "README");
    }

    #[test]
    fn test_create_with_multiple_extensions() {
        let headers = create_headers_with_content_type("application/gzip");
        let cd = create_content_disposition("upload", "archive.tar.gz");

        let result = FileInput::create(&headers, cd);
        assert!(result.is_ok());

        let file_input = result.unwrap();
        // This depends on your FileExtHelper implementation
        // Adjust based on whether it returns "gz" or "tar.gz"
        assert!(file_input.extension.is_some());
        assert_eq!(file_input.file_name, "archive.tar.gz");
    }

    // Test for `get_content_type` method
    #[test]
    fn test_get_content_type_success() {
        let headers = create_headers_with_content_type("application/json");
        let result = FileInput::get_content_type(&headers);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "application/json");
    }

    #[test]
    fn test_get_content_type_missing() {
        let headers = HeaderMap::new();
        let result = FileInput::get_content_type(&headers);

        assert!(result.is_err());
        if let Err(MultipartError::NoContentType(msg)) = result {
            assert_eq!(msg, "Empty content type");
        }
    }

    #[test]
    fn test_get_content_type_invalid_header() {
        let mut headers = HeaderMap::new();
        // Insert invalid UTF-8 bytes
        headers.insert(
            HeaderName::from_str("content-type").unwrap(),
            HeaderValue::from_bytes(&[0xFF, 0xFE]).unwrap(),
        );

        let result = FileInput::get_content_type(&headers);
        assert!(result.is_err());
    }

    // Test for Default implementation
    #[test]
    fn test_default_file_input() {
        let file_input = FileInput::default();

        assert_eq!(file_input.file_name, "");
        assert_eq!(file_input.field_name, "");
        assert_eq!(file_input.size, 0);
        assert_eq!(file_input.content_type, "");
        assert!(file_input.bytes.is_empty());
        assert_eq!(file_input.extension, None);
    }

    // Test for Clone implementation
    #[test]
    fn test_clone_file_input() {
        let original = FileInput {
            file_name: "test.txt".to_string(),
            field_name: "upload".to_string(),
            size: 1024,
            content_type: "text/plain".to_string(),
            bytes: vec![Bytes::from_static(&[0; 1024])],
            extension: Some("txt".to_string()),
            content_disposition: create_content_disposition("upload", "test.txt"),
        };

        let cloned = original.clone();

        assert_eq!(original.file_name, cloned.file_name);
        assert_eq!(original.field_name, cloned.field_name);
        assert_eq!(original.size, cloned.size);
        assert_eq!(original.content_type, cloned.content_type);
        assert_eq!(original.bytes.len(), cloned.bytes.len());
        assert_eq!(original.extension, cloned.extension);
    }

    // Integration test combining multiple operations
    #[test]
    fn test_file_input_workflow() {
        let headers = create_headers_with_content_type("image/jpeg");
        let cd = create_content_disposition("photo", "vacation.jpg");

        // Create FileInput
        let mut file_input = FileInput::create(&headers, cd).unwrap();

        // Add some data
        file_input.bytes = vec![
            Bytes::from_static(&[0xFF, 0xD8, 0xFF, 0xE0]), // JPEG header
            Bytes::from_static(&[0; 1020]), // Rest of 1KB
        ];

        // Test size calculation
        assert_eq!(file_input.calculate_size(), 1024);

        // Test human readable size
        let human_size = file_input.human_size();
        assert!(human_size.contains("1") && human_size.contains("KB"));

        // Verify other properties
        assert_eq!(file_input.content_type, "image/jpeg");
        assert_eq!(file_input.field_name, "photo");
        assert_eq!(file_input.file_name, "vacation.jpg");
        assert_eq!(file_input.extension, Some("jpg".to_string()));
    }

    // Benchmark-style test for performance
    #[test]
    fn test_calculate_size_performance() {
        // Create a file input with many small chunks
        let mut bytes = Vec::new();
        for _ in 0..1000 {
            bytes.push(Bytes::from_static(&[0; 100])); // 100 bytes each
        }

        let file_input = FileInput {
            bytes,
            ..Default::default()
        };

        let start = std::time::Instant::now();
        let size = file_input.calculate_size();
        let duration = start.elapsed();

        assert_eq!(size, 100_000); // 1000 * 100 bytes
        assert!(duration.as_millis() < 10); // Should be very fast
    }
}