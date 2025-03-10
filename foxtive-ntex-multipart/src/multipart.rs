use std::collections::HashMap;
use std::convert::Infallible;
use std::path::Path;

use crate::content_disposition::ContentDisposition;
use crate::data_input::DataInput;
use crate::file_input::FileInput;
use crate::file_validator::Validator;
use crate::result::{MultipartError, MultipartResult};
use futures::StreamExt;
use ntex::http::Payload;
use ntex::web::{FromRequest, HttpRequest};
use ntex_multipart::Multipart as NtexMultipart;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub struct Multipart {
    multipart: NtexMultipart,
    file_inputs: HashMap<String, Vec<FileInput>>, // Store multiple files for the same field
    data_inputs: HashMap<String, Vec<DataInput>>, // Store multiple data entries for the same field
}

impl<Err> FromRequest<Err> for Multipart {
    type Error = Infallible;

    async fn from_request(
        req: &HttpRequest,
        payload: &mut Payload,
    ) -> Result<Multipart, Infallible> {
        let multipart = NtexMultipart::new(req.headers(), payload.take());
        Ok(Multipart::new(multipart).await)
    }
}

impl Multipart {
    pub async fn new(multipart: NtexMultipart) -> Multipart {
        Self {
            multipart,
            file_inputs: Default::default(),
            data_inputs: Default::default(),
        }
    }

    pub async fn process(&mut self) -> Result<&mut Multipart, MultipartError> {
        while let Some(item) = self.multipart.next().await {
            let mut field = item.map_err(MultipartError::NtexError)?;

            if let Some(content_disposition) = field.headers().get("content-disposition") {
                let content_disposition = content_disposition.to_str().ok();
                if let Some(content_disposition) = content_disposition {
                    let content_disposition = ContentDisposition::create(content_disposition);

                    if !content_disposition.has_name_field() {
                        continue;
                    }

                    // Process form fields (non-file fields)
                    if !content_disposition.is_file_field() {
                        let value = self.collect_data_field_value(&mut field).await;
                        let field_name =
                            content_disposition.get_variable("name").unwrap_or_default();

                        // Insert or append to the data_inputs array for this field
                        self.data_inputs
                            .entry(field_name.to_string())
                            .or_default()
                            .push(DataInput {
                                value,
                                name: field_name.to_string(),
                            });

                        continue;
                    }

                    // Process file fields
                    let mut info = FileInput::create(field.headers(), content_disposition)?;
                    let mut total_size = 0;
                    let mut bytes = Vec::new();

                    // Collect all file chunks
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        total_size += data.len();
                        bytes.push(data);
                    }

                    info.size = total_size;
                    info.bytes = bytes;

                    // Insert or append file input to the corresponding field
                    self.file_inputs
                        .entry(info.field_name.clone())
                        .or_default()
                        .push(info);
                }
            }
        }

        Ok(self)
    }

    async fn collect_data_field_value(&self, field: &mut ntex_multipart::Field) -> String {
        let mut value = String::new();
        while let Some(chunk) = field.next().await {
            if let Ok(chunk_data) = chunk {
                value.push_str(&String::from_utf8_lossy(&chunk_data));
            }
        }

        value
    }

    pub async fn save_file(file_input: &FileInput, path: impl AsRef<Path>) -> MultipartResult<()> {
        let mut file = File::create(path).await?;

        // Write all bytes in a single batch
        for byte in &file_input.bytes {
            file.write_all(byte).await?;
        }

        file.flush().await?;
        Ok(())
    }

    /// Get all data inputs
    pub fn all_data(&self) -> &HashMap<String, Vec<DataInput>> {
        &self.data_inputs
    }

    /// Get a data input for a given field
    pub fn data(&self, field: &str) -> Option<&Vec<DataInput>> {
        self.data_inputs.get(field)
    }

    /// Get the first data input for a given field
    pub fn first_data(&self, field: &str) -> Option<&DataInput> {
        self.data_inputs
            .get(field)
            .and_then(|inputs| inputs.first())
    }

    /// Get the first data input for a given field.
    /// Returns an error if the field is not found
    pub fn first_data_required(&self, field: &str) -> MultipartResult<&DataInput> {
        self.data_inputs
            .get(field)
            .and_then(|inputs| inputs.first())
            .ok_or(MultipartError::MissingDataField(field.to_string()))
    }

    /// Get all files
    pub fn all_files(&self) -> &HashMap<String, Vec<FileInput>> {
        &self.file_inputs
    }

    /// Get all files for a given field
    pub fn files(&self, field: &str) -> Option<&Vec<FileInput>> {
        self.file_inputs.get(field)
    }

    /// Get the first file for a given field
    pub fn first_file(&self, field: &str) -> Option<&FileInput> {
        self.file_inputs.get(field).and_then(|files| files.first())
    }

    /// Check if a field has any files
    pub fn has_file(&self, field: &str) -> bool {
        self.file_inputs.contains_key(field)
    }

    /// Validate all files against the provided rules
    pub async fn validate(&mut self, validator: Validator) -> MultipartResult<&mut Multipart> {
        self.process().await?;
        validator.validate(&self.file_inputs).map(|_| self)
    }
}

#[cfg(test)]
mod test {
    use crate::data_input::DataInput;
    use crate::file_input::FileInput;
    use crate::file_validator::Validator;
    use crate::{FileRules, Multipart};
    use ntex::http::{HeaderMap, Payload};
    use ntex::util::Bytes;
    use ntex_multipart::Multipart as NtexMultipart;
    use tokio::fs;

    // Test 1: Test creating a new multipart instance with no data
    #[tokio::test]
    async fn test_multipart_new() {
        let headers = HeaderMap::new();
        let payload = Payload::None;
        let multipart = NtexMultipart::new(&headers, payload);

        let multipart_instance = Multipart::new(multipart).await;

        assert!(multipart_instance.all_data().is_empty());
        assert!(multipart_instance.all_files().is_empty());
    }

    // Test 2: Test saving a file to disk
    #[tokio::test]
    async fn test_save_file() {
        let file_input = FileInput {
            field_name: "file".to_string(),
            file_name: "test.txt".to_string(),
            content_type: "text/plain".to_string(),
            size: 11,
            bytes: vec![Bytes::from("Hello World")],
            extension: None,
            content_disposition: Default::default(),
        };

        let path = "test_output.txt";
        let result = Multipart::save_file(&file_input, &path).await;

        assert!(result.is_ok());

        let content = fs::read_to_string(path).await.unwrap();
        assert_eq!(content, "Hello World");

        fs::remove_file(path).await.unwrap(); // Cleanup
    }

    // Test 3: Test adding multiple data fields and verifying the count
    #[tokio::test]
    async fn test_multiple_data_fields() {
        let headers = HeaderMap::new();
        let payload = Payload::None;
        let multipart = NtexMultipart::new(&headers, payload);
        let mut multipart_instance = Multipart::new(multipart).await;

        // Adding multiple data entries for the same field
        multipart_instance
            .data_inputs
            .entry("key1".to_string())
            .or_insert_with(Vec::new)
            .push(DataInput {
                name: "key1".to_string(),
                value: "value1".to_string(),
            });

        multipart_instance
            .data_inputs
            .entry("key1".to_string())
            .or_insert_with(Vec::new)
            .push(DataInput {
                name: "key1".to_string(),
                value: "value2".to_string(),
            });

        // Verify multiple data entries for the same field
        assert_eq!(multipart_instance.data("key1").unwrap().len(), 2);
    }

    // Test 4: Test adding multiple files for the same field
    #[tokio::test]
    async fn test_multiple_files() {
        let headers = HeaderMap::new();
        let payload = Payload::None;
        let multipart = NtexMultipart::new(&headers, payload);
        let mut multipart_instance = Multipart::new(multipart).await;

        // Adding multiple files for the same field
        multipart_instance
            .file_inputs
            .entry("file1".to_string())
            .or_insert_with(Vec::new)
            .push(FileInput {
                field_name: "file1".to_string(),
                file_name: "file1.txt".to_string(),
                content_type: "text/plain".to_string(),
                size: 11,
                bytes: vec![Bytes::from("File 1 Content")],
                extension: None,
                content_disposition: Default::default(),
            });

        multipart_instance
            .file_inputs
            .entry("file1".to_string())
            .or_insert_with(Vec::new)
            .push(FileInput {
                field_name: "file1".to_string(),
                file_name: "file2.txt".to_string(),
                content_type: "text/plain".to_string(),
                size: 12,
                bytes: vec![Bytes::from("File 2 Content")],
                extension: None,
                content_disposition: Default::default(),
            });

        // Verify multiple files for the same field
        assert_eq!(multipart_instance.files("file1").unwrap().len(), 2);
    }

    // Test 5: Test invalid validation when too few files are uploaded
    #[tokio::test]
    async fn test_validate_files_too_few() {
        let headers = HeaderMap::new();
        let payload = Payload::None;
        let multipart = NtexMultipart::new(&headers, payload);
        let mut multipart_instance = Multipart::new(multipart).await;

        // No files added, so validation should fail
        let validator = Validator::new().add_rule(
            "file1",
            FileRules {
                min_files: Some(1),
                max_files: Some(5),
                ..Default::default()
            },
        );

        let result = multipart_instance.validate(validator).await;

        assert!(result.is_err());
    }

    // Test 6: Test retrieval of the first file and data input
    #[tokio::test]
    async fn test_first_file_and_data_input() {
        let headers = HeaderMap::new();
        let payload = Payload::None;
        let multipart = NtexMultipart::new(&headers, payload);
        let mut multipart_instance = Multipart::new(multipart).await;

        // Adding data and files
        multipart_instance
            .data_inputs
            .entry("key1".to_string())
            .or_insert_with(Vec::new)
            .push(DataInput {
                name: "key1".to_string(),
                value: "value1".to_string(),
            });

        multipart_instance
            .file_inputs
            .entry("file1".to_string())
            .or_insert_with(Vec::new)
            .push(FileInput {
                field_name: "file1".to_string(),
                file_name: "file1.txt".to_string(),
                content_type: "text/plain".to_string(),
                size: 11,
                bytes: vec![Bytes::from("File 1 Content")],
                extension: None,
                content_disposition: Default::default(),
            });

        // Test first data input
        let first_data = multipart_instance.first_data("key1");
        assert_eq!(first_data.unwrap().value, "value1");

        // Test first file input
        let first_file = multipart_instance.first_file("file1");
        assert_eq!(first_file.unwrap().file_name, "file1.txt");
    }

    // Test 7: Test handling of empty file field
    #[tokio::test]
    async fn test_empty_file_field() {
        let headers = HeaderMap::new();
        let payload = Payload::None;
        let multipart = NtexMultipart::new(&headers, payload);
        let multipart_instance = Multipart::new(multipart).await;

        // Verify empty file field (no files should be found)
        assert!(multipart_instance.files("empty_file").is_none());
    }
}
