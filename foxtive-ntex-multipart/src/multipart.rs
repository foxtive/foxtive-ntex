use crate::content_disposition::ContentDisposition;
use crate::contract::PostParseable;
use crate::data_input::DataInput;
use crate::file_input::FileInput;
use crate::file_validator::Validator;
use crate::result::{MultipartError, MultipartResult, NtexMultipartError};
use futures::StreamExt;
use ntex::http::Payload;
use ntex::web::{FromRequest, HttpRequest};
use ntex_multipart::Multipart as NtexMultipart;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// Configuration for multipart request processing
#[derive(Debug, Clone, Default)]
pub struct MultipartConfig {
    /// Maximum size for a single file in bytes
    pub max_file_size: Option<usize>,
    /// Maximum total payload size for the entire multipart request in bytes
    pub max_total_payload_size: Option<usize>,
    /// Directory for temporary files when using disk streaming
    pub temp_dir: Option<std::path::PathBuf>,
    /// Threshold size (in bytes) above which files are streamed to disk
    /// Files smaller than this are kept in memory for performance
    pub disk_threshold: Option<usize>,
}

/// Storage mode for uploaded files
#[derive(Debug, Clone, PartialEq, Default)]
pub enum FileStorageMode {
    /// Store file in memory as bytes
    #[default]
    InMemory,
    /// Stream file to disk at the specified path
    OnDisk(std::path::PathBuf),
}

/// Builder for creating configured Multipart instances
#[derive(Default)]
pub struct MultipartBuilder {
    config: MultipartConfig,
}

impl MultipartBuilder {
    /// Create a new MultipartBuilder with default configuration
    pub fn new() -> Self {
        Self {
            config: MultipartConfig::default(),
        }
    }

    /// Set maximum file size limit (in bytes)
    pub fn max_file_size(mut self, limit: usize) -> Self {
        self.config.max_file_size = Some(limit);
        self
    }

    /// Set maximum total payload size limit (in bytes)
    pub fn max_total_payload_size(mut self, limit: usize) -> Self {
        self.config.max_total_payload_size = Some(limit);
        self
    }

    /// Set temporary directory for disk streaming
    pub fn temp_dir(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.config.temp_dir = Some(path.into());
        self
    }

    /// Set disk threshold - files larger than this will be streamed to disk
    pub fn disk_threshold(mut self, threshold: usize) -> Self {
        self.config.disk_threshold = Some(threshold);
        self
    }

    /// Build and process the multipart request
    pub async fn build(self, multipart: NtexMultipart) -> MultipartResult<Multipart> {
        Multipart::with_config(multipart, self.config).await
    }
}

#[derive(Default)]
pub struct Multipart {
    pub(crate) file_inputs: HashMap<String, Vec<FileInput>>, // Store multiple files for the same field
    pub(crate) data_inputs: HashMap<String, Vec<DataInput>>, // Store multiple data entries for the same field
    pub(crate) config: MultipartConfig,
    pub(crate) total_payload_size: usize, // Track total bytes processed
}

impl<Err> FromRequest<Err> for Multipart {
    type Error = MultipartError;

    async fn from_request(
        req: &HttpRequest,
        payload: &mut Payload,
    ) -> Result<Multipart, Self::Error> {
        let multipart = NtexMultipart::new(req.headers(), payload.take());
        // Use default configuration when using FromRequest trait
        // For custom configuration, use Multipart::with_config() directly
        Multipart::new(multipart).await
    }
}

impl Multipart {
    /// Create a new Multipart instance with default configuration
    ///
    /// # Example
    /// ```ignore
    /// use foxtive_ntex_multipart::Multipart;
    /// use ntex_multipart::Multipart as NtexMultipart;
    ///
    /// async fn handler(req: ntex::web::HttpRequest, payload: ntex::http::Payload) {
    ///     let multipart_stream = NtexMultipart::new(req.headers(), payload.into_inner());
    ///     let multipart = Multipart::new(multipart_stream).await.unwrap();
    /// }
    /// ```
    pub async fn new(multipart: NtexMultipart) -> MultipartResult<Multipart> {
        Self::with_config(multipart, MultipartConfig::default()).await
    }

    /// Create a new Multipart instance with custom configuration
    ///
    /// # Example
    /// ```ignore
    /// use foxtive_ntex_multipart::{Multipart, MultipartConfig};
    /// use ntex_multipart::Multipart as NtexMultipart;
    ///
    /// async fn handler(req: ntex::web::HttpRequest, payload: ntex::http::Payload) {
    ///     let config = MultipartConfig {
    ///         max_file_size: Some(10 * 1024 * 1024), // 10 MB
    ///         max_total_payload_size: Some(50 * 1024 * 1024), // 50 MB
    ///     };
    ///     
    ///     let multipart_stream = NtexMultipart::new(req.headers(), payload.into_inner());
    ///     let multipart = Multipart::with_config(multipart_stream, config).await.unwrap();
    /// }
    /// ```
    pub async fn with_config(
        multipart: NtexMultipart,
        config: MultipartConfig,
    ) -> MultipartResult<Multipart> {
        Self {
            file_inputs: Default::default(),
            data_inputs: Default::default(),
            config,
            total_payload_size: 0,
        }
        .process(multipart)
        .await
    }

    /// Create a builder for configuring multipart processing
    ///
    /// # Example
    /// ```ignore
    /// use foxtive_ntex_multipart::Multipart;
    /// use ntex_multipart::Multipart as NtexMultipart;
    ///
    /// async fn handler(req: ntex::web::HttpRequest, payload: ntex::http::Payload) {
    ///     let multipart_stream = NtexMultipart::new(req.headers(), payload.into_inner());
    ///     let multipart = Multipart::builder()
    ///         .max_file_size(10 * 1024 * 1024) // 10 MB
    ///         .max_total_payload_size(50 * 1024 * 1024) // 50 MB
    ///         .build(multipart_stream)
    ///         .await
    ///         .unwrap();
    /// }
    /// ```
    pub fn builder() -> MultipartBuilder {
        MultipartBuilder::new()
    }

    pub async fn process(
        mut self,
        mut multipart: NtexMultipart,
    ) -> Result<Multipart, MultipartError> {
        while let Some(item) = multipart.next().await {
            let mut field = item.map_err(|e| MultipartError::NtexError(NtexMultipartError::from(e)))?;

            if let Some(content_disposition) = field.headers().get("content-disposition") {
                let content_disposition = content_disposition.to_str().ok();
                if let Some(content_disposition) = content_disposition {
                    let content_disposition = ContentDisposition::create(content_disposition);

                    if !content_disposition.has_name_field() {
                        continue;
                    }

                    // Process form fields (non-file fields)
                    if !content_disposition.is_file_field() {
                        let value = self.collect_data_field_value(&mut field).await?;
                        let field_name =
                            content_disposition.get_variable("name").unwrap_or_default();

                        // Track payload size for data fields
                        self.total_payload_size += value.len();
                        
                        // Check total payload limit
                        if let Some(max_total) = self.config.max_total_payload_size
                            && self.total_payload_size > max_total
                        {
                            return Err(MultipartError::PayloadTooLarge {
                                field: field_name.to_string(),
                                size: self.total_payload_size,
                                max_size: max_total,
                            });
                        }

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
                    let should_use_disk = self.config.disk_threshold.is_some() 
                        || self.config.temp_dir.is_some();
                    
                    let mut info = if should_use_disk {
                        // Create FileInput for disk streaming
                        if let Some(temp_dir) = &self.config.temp_dir {
                            FileInput::create_with_disk(field.headers(), content_disposition, temp_dir)?
                        } else {
                            // Fallback to in-memory if no temp_dir configured
                            FileInput::create(field.headers(), content_disposition)?
                        }
                    } else {
                        // Create FileInput for in-memory storage
                        FileInput::create(field.headers(), content_disposition)?
                    };
                    
                    let mut total_size = 0;

                    // Handle disk streaming
                    if info.is_on_disk() {
                        use tokio::io::AsyncWriteExt;
                        
                        // Get the temp file path
                        let temp_path = match &info.storage_mode {
                            FileStorageMode::OnDisk(path) => path.clone(),
                            _ => unreachable!(),
                        };
                        
                        // Create buffered writer for efficient I/O
                        let file = File::create(&temp_path).await?;
                        let mut writer = tokio::io::BufWriter::with_capacity(8192, file);
                        
                        // Stream chunks directly to disk
                        while let Some(chunk) = field.next().await {
                            let data = chunk.map_err(|e| MultipartError::NtexError(NtexMultipartError::from(e)))?;
                            let chunk_size = data.len();
                            total_size += chunk_size;
                            
                            // Check per-file size limit during collection
                            if let Some(max_file_size) = self.config.max_file_size
                                && total_size > max_file_size
                            {
                                // Clean up the partial file
                                drop(writer);
                                let _ = tokio::fs::remove_file(&temp_path).await;
                                return Err(MultipartError::FileTooLarge {
                                    field: info.field_name.clone(),
                                    filename: info.file_name.clone(),
                                    size: total_size,
                                    max_size: max_file_size,
                                });
                            }
                            
                            // Check total payload size limit
                            self.total_payload_size += chunk_size;
                            if let Some(max_total) = self.config.max_total_payload_size
                                && self.total_payload_size > max_total
                            {
                                drop(writer);
                                let _ = tokio::fs::remove_file(&temp_path).await;
                                return Err(MultipartError::PayloadTooLarge {
                                    field: info.field_name.clone(),
                                    size: self.total_payload_size,
                                    max_size: max_total,
                                });
                            }
                            
                            // Write chunk to disk
                            writer.write_all(&data).await?;
                        }
                        
                        // Flush and ensure all data is written
                        writer.flush().await?;
                        drop(writer);
                        
                        info.size = total_size;
                        // bytes vector remains empty for disk-stored files
                    } else {
                        // In-memory collection (original behavior)
                        let mut bytes = Vec::new();
                        
                        // Collect all file chunks with size limit enforcement
                        while let Some(chunk) = field.next().await {
                            let data = chunk.map_err(|e| MultipartError::NtexError(NtexMultipartError::from(e)))?;
                            let chunk_size = data.len();
                            total_size += chunk_size;
                            
                            // Check per-file size limit during collection
                            if let Some(max_file_size) = self.config.max_file_size
                                && total_size > max_file_size
                            {
                                return Err(MultipartError::FileTooLarge {
                                    field: info.field_name.clone(),
                                    filename: info.file_name.clone(),
                                    size: total_size,
                                    max_size: max_file_size,
                                });
                            }
                            
                            // Check total payload size limit
                            self.total_payload_size += chunk_size;
                            if let Some(max_total) = self.config.max_total_payload_size
                                && self.total_payload_size > max_total
                            {
                                return Err(MultipartError::PayloadTooLarge {
                                    field: info.field_name.clone(),
                                    size: self.total_payload_size,
                                    max_size: max_total,
                                });
                            }
                            
                            bytes.push(data);
                        }

                        info.size = total_size;
                        info.bytes = bytes;
                    }

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

    async fn collect_data_field_value(
        &self,
        field: &mut ntex_multipart::Field,
    ) -> MultipartResult<String> {
        let mut value = String::new();
        while let Some(chunk) = field.next().await {
            let chunk_data = chunk.map_err(|e| MultipartError::NtexError(NtexMultipartError::from(e)))?;
            value.push_str(&String::from_utf8_lossy(&chunk_data));
        }

        Ok(value)
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

    /// Take ownership of all data inputs, consuming the multipart instance
    pub fn into_all_data(self) -> HashMap<String, Vec<DataInput>> {
        self.data_inputs
    }

    /// Take ownership of all file inputs, consuming the multipart instance
    pub fn into_all_files(self) -> HashMap<String, Vec<FileInput>> {
        self.file_inputs
    }

    /// Take ownership of data inputs for a specific field
    pub fn into_data(mut self, field: &str) -> Option<Vec<DataInput>> {
        self.data_inputs.remove(field)
    }

    /// Take ownership of the first data input for a given field
    pub fn into_first_data(mut self, field: &str) -> Option<DataInput> {
        self.data_inputs.get_mut(field).and_then(|inputs| {
            if inputs.is_empty() {
                None
            } else {
                Some(inputs.remove(0))
            }
        })
    }

    /// Take ownership of the first data input for a given field.
    /// Returns an error if the field is not found
    pub fn into_first_data_required(self, field: &str) -> MultipartResult<DataInput> {
        self.into_first_data(field)
            .ok_or(MultipartError::MissingDataField(field.to_string()))
    }

    /// Take ownership of file inputs for a specific field
    pub fn into_files(mut self, field: &str) -> Option<Vec<FileInput>> {
        self.file_inputs.remove(field)
    }

    /// Take ownership of the first file for a given field
    pub fn into_first_file(mut self, field: &str) -> Option<FileInput> {
        self.file_inputs.get_mut(field).and_then(|files| {
            if files.is_empty() {
                None
            } else {
                Some(files.remove(0))
            }
        })
    }

    /// Take ownership of the first file for a given field.
    /// Returns an error if the field is not found
    pub fn into_first_file_required(self, field: &str) -> MultipartResult<FileInput> {
        self.into_first_file(field)
            .ok_or(MultipartError::MissingDataField(field.to_string()))
    }

    /// Validate all files against the provided rules
    pub async fn validate(self, validator: Validator) -> MultipartResult<Self> {
        validator.validate(&self.file_inputs)?;
        Ok(self)
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
