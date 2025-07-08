use std::collections::HashMap;
use std::convert::Infallible;
use std::path::Path;

use crate::content_disposition::ContentDisposition;
use crate::contract::PostParseable;
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
    pub(crate) multipart: NtexMultipart,
    pub(crate) file_inputs: HashMap<String, Vec<FileInput>>, // Store multiple files for the same field
    pub(crate) data_inputs: HashMap<String, Vec<DataInput>>, // Store multiple data entries for the same field
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

    /// Get a parsed value of the specified type from a form field
    /// Usage: post::<i32>("price"), post::<String>("name"), post::<bool>("is_active")
    /// For Option types: post::<Option<i32>>("price") - returns None for missing/empty fields
    pub fn post<T>(&self, field: &str) -> MultipartResult<T>
    where
        T: PostParseable,
    {
        T::parse_from_multipart(self, field)
    }

    /// Get a parsed value of the specified type from a form field with a default fallback
    /// Usage: post_or::<i32>("price", 0), post_or::<String>("name", "default".to_string())
    pub fn post_or<T>(&self, field: &str, default: T) -> T
    where
        T: PostParseable,
    {
        self.post(field).unwrap_or(default)
    }

    /// Get an optional parsed value of the specified type from a form field
    /// Usage: post_opt::<i32>("price"), post_opt::<String>("name")
    pub fn post_opt<T>(&self, field: &str) -> Option<T>
    where
        T: PostParseable,
    {
        self.post(field).ok()
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

    /// Add test data to multipart instance (for testing purposes only)
    #[cfg(test)]
    pub fn add_test_data(&mut self, field: &str, value: &str) {
        self.data_inputs
            .entry(field.to_string())
            .or_default()
            .push(DataInput {
                name: field.to_string(),
                value: value.to_string(),
            });
    }
}
