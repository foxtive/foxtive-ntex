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
use crate::contract::PostParseable;

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

    // Test 8: Test generic post method for different types
    #[tokio::test]
    async fn test_post_method_with_types() {
        let headers = HeaderMap::new();
        let payload = Payload::None;
        let multipart = NtexMultipart::new(&headers, payload);
        let mut multipart_instance = Multipart::new(multipart).await;

        // Adding various typed data
        multipart_instance
            .data_inputs
            .entry("price".to_string())
            .or_insert_with(Vec::new)
            .push(DataInput {
                name: "price".to_string(),
                value: "100".to_string(),
            });

        multipart_instance
            .data_inputs
            .entry("name".to_string())
            .or_insert_with(Vec::new)
            .push(DataInput {
                name: "name".to_string(),
                value: "John Doe".to_string(),
            });

        multipart_instance
            .data_inputs
            .entry("is_active".to_string())
            .or_insert_with(Vec::new)
            .push(DataInput {
                name: "is_active".to_string(),
                value: "true".to_string(),
            });

        multipart_instance
            .data_inputs
            .entry("rating".to_string())
            .or_insert_with(Vec::new)
            .push(DataInput {
                name: "rating".to_string(),
                value: "4.5".to_string(),
            });

        // Test parsing different types
        let price: i32 = multipart_instance.post("price").unwrap();
        assert_eq!(price, 100);

        let name: String = multipart_instance.post("name").unwrap();
        assert_eq!(name, "John Doe");

        let is_active: bool = multipart_instance.post("is_active").unwrap();
        assert!(is_active);

        let rating: f64 = multipart_instance.post("rating").unwrap();
        assert_eq!(rating, 4.5);
    }

    // Test 9: Test post_or method with default values
    #[tokio::test]
    async fn test_post_or_method() {
        let headers = HeaderMap::new();
        let payload = Payload::None;
        let multipart = NtexMultipart::new(&headers, payload);
        let multipart_instance = Multipart::new(multipart).await;

        // Test with missing field - should return default
        let default_price: i32 = multipart_instance.post_or("missing_price", 50);
        assert_eq!(default_price, 50);

        let default_name: String =
            multipart_instance.post_or("missing_name", "Default Name".to_string());
        assert_eq!(default_name, "Default Name");
    }

    // Test 10: Test post_opt method for optional values
    #[tokio::test]
    async fn test_post_opt_method() {
        let headers = HeaderMap::new();
        let payload = Payload::None;
        let multipart = NtexMultipart::new(&headers, payload);
        let mut multipart_instance = Multipart::new(multipart).await;

        // Add some data
        multipart_instance
            .data_inputs
            .entry("optional_price".to_string())
            .or_insert_with(Vec::new)
            .push(DataInput {
                name: "optional_price".to_string(),
                value: "200".to_string(),
            });

        // Test with existing field
        let price: Option<i32> = multipart_instance.post_opt("optional_price");
        assert_eq!(price, Some(200));

        // Test with missing field
        let missing_price: Option<i32> = multipart_instance.post_opt("missing_price");
        assert_eq!(missing_price, None);
    }

    // Test 11: Test post method error handling
    #[tokio::test]
    async fn test_post_method_error_handling() {
        let headers = HeaderMap::new();
        let payload = Payload::None;
        let multipart = NtexMultipart::new(&headers, payload);
        let mut multipart_instance = Multipart::new(multipart).await;

        // Add invalid data for parsing
        multipart_instance
            .data_inputs
            .entry("invalid_number".to_string())
            .or_insert_with(Vec::new)
            .push(DataInput {
                name: "invalid_number".to_string(),
                value: "not_a_number".to_string(),
            });

        // Test parsing invalid number
        let result: Result<i32, _> = multipart_instance.post("invalid_number");
        assert!(result.is_err());

        // Test missing field
        let result: Result<String, _> = multipart_instance.post("missing_field");
        assert!(result.is_err());

        // Test invalid Option<T> parsing - should return error for invalid values
        multipart_instance
            .data_inputs
            .entry("invalid_optional_number".to_string())
            .or_insert_with(Vec::new)
            .push(DataInput {
                name: "invalid_optional_number".to_string(),
                value: "not_a_number".to_string(),
            });

        let result: Result<Option<i32>, _> = multipart_instance.post("invalid_optional_number");
        assert!(result.is_err()); // Should error because value exists but can't be parsed
    }

    // Test 12: Test post method with Option<T> types
    #[tokio::test]
    async fn test_post_method_with_option_types() {
        let headers = HeaderMap::new();
        let payload = Payload::None;
        let multipart = NtexMultipart::new(&headers, payload);
        let mut multipart_instance = Multipart::new(multipart).await;

        // Add some data - some fields present, some missing, some empty
        multipart_instance
            .data_inputs
            .entry("existing_price".to_string())
            .or_insert_with(Vec::new)
            .push(DataInput {
                name: "existing_price".to_string(),
                value: "100".to_string(),
            });

        multipart_instance
            .data_inputs
            .entry("empty_field".to_string())
            .or_insert_with(Vec::new)
            .push(DataInput {
                name: "empty_field".to_string(),
                value: "".to_string(),
            });

        multipart_instance
            .data_inputs
            .entry("whitespace_field".to_string())
            .or_insert_with(Vec::new)
            .push(DataInput {
                name: "whitespace_field".to_string(),
                value: "   ".to_string(),
            });

        // Test with existing field - should return Some(value)
        let existing_price: Option<i32> = multipart_instance.post("existing_price").unwrap();
        assert_eq!(existing_price, Some(100));

        // Test with missing field - should return None
        let missing_price: Option<i32> = multipart_instance.post("missing_field").unwrap();
        assert_eq!(missing_price, None);

        // Test with empty field - should return None
        let empty_price: Option<i32> = multipart_instance.post("empty_field").unwrap();
        assert_eq!(empty_price, None);

        // Test with whitespace-only field - should return None
        let whitespace_price: Option<i32> = multipart_instance.post("whitespace_field").unwrap();
        assert_eq!(whitespace_price, None);

        // Test with Option<String>
        let existing_name: Option<String> = multipart_instance.post("existing_price").unwrap();
        assert_eq!(existing_name, Some("100".to_string()));

        let missing_name: Option<String> = multipart_instance.post("missing_field").unwrap();
        assert_eq!(missing_name, None);
    }

    // Test 13: Test comprehensive FromStr type support
    #[tokio::test]
    async fn test_comprehensive_fromstr_type_support() {
        let headers = HeaderMap::new();
        let payload = Payload::None;
        let multipart = NtexMultipart::new(&headers, payload);
        let mut multipart_instance = Multipart::new(multipart).await;

        // Add test data for various types
        let test_data = vec![
            ("test_u8", "255"),
            ("test_u16", "65535"),
            ("test_u32", "4294967295"),
            ("test_u64", "18446744073709551615"),
            ("test_i8", "-128"),
            ("test_i16", "-32768"),
            ("test_i32", "-2147483648"),
            ("test_i64", "-9223372036854775808"),
            ("test_f32", "3.14159"),
            ("test_f64", "2.718281828459045"),
            ("test_bool_true", "true"),
            ("test_bool_false", "false"),
            ("test_char", "x"),
            ("test_string", "Hello, World!"),
            ("test_ipv4", "192.168.1.1"),
            ("test_ipv6", "2001:0db8:85a3:0000:0000:8a2e:0370:7334"),
            ("test_socket_addr", "127.0.0.1:8080"),
        ];

        for (name, value) in test_data {
            multipart_instance
                .data_inputs
                .entry(name.to_string())
                .or_insert_with(Vec::new)
                .push(DataInput {
                    name: name.to_string(),
                    value: value.to_string(),
                });
        }

        // Test parsing various types
        let parsed_u8: u8 = multipart_instance.post("test_u8").unwrap();
        assert_eq!(parsed_u8, 255);

        let parsed_u16: u16 = multipart_instance.post("test_u16").unwrap();
        assert_eq!(parsed_u16, 65535);

        let parsed_u32: u32 = multipart_instance.post("test_u32").unwrap();
        assert_eq!(parsed_u32, 4294967295);

        let parsed_u64: u64 = multipart_instance.post("test_u64").unwrap();
        assert_eq!(parsed_u64, 18446744073709551615);

        let parsed_i8: i8 = multipart_instance.post("test_i8").unwrap();
        assert_eq!(parsed_i8, -128);

        let parsed_i16: i16 = multipart_instance.post("test_i16").unwrap();
        assert_eq!(parsed_i16, -32768);

        let parsed_i32: i32 = multipart_instance.post("test_i32").unwrap();
        assert_eq!(parsed_i32, -2147483648);

        let parsed_i64: i64 = multipart_instance.post("test_i64").unwrap();
        assert_eq!(parsed_i64, -9223372036854775808);

        let parsed_f32: f32 = multipart_instance.post("test_f32").unwrap();
        #[allow(clippy::approx_constant)]
        let pi = 3.14159;
        assert!((parsed_f32 - pi).abs() < f32::EPSILON);

        let parsed_f64: f64 = multipart_instance.post("test_f64").unwrap();
        assert!((parsed_f64 - std::f64::consts::E).abs() < f64::EPSILON);

        let parsed_bool_true: bool = multipart_instance.post("test_bool_true").unwrap();
        assert!(parsed_bool_true);

        let parsed_bool_false: bool = multipart_instance.post("test_bool_false").unwrap();
        assert!(!parsed_bool_false);

        let parsed_char: char = multipart_instance.post("test_char").unwrap();
        assert_eq!(parsed_char, 'x');

        let parsed_string: String = multipart_instance.post("test_string").unwrap();
        assert_eq!(parsed_string, "Hello, World!");

        let parsed_ipv4: std::net::Ipv4Addr = multipart_instance.post("test_ipv4").unwrap();
        assert_eq!(
            parsed_ipv4,
            "192.168.1.1".parse::<std::net::Ipv4Addr>().unwrap()
        );

        let parsed_ipv6: std::net::Ipv6Addr = multipart_instance.post("test_ipv6").unwrap();
        assert_eq!(
            parsed_ipv6,
            "2001:0db8:85a3:0000:0000:8a2e:0370:7334"
                .parse::<std::net::Ipv6Addr>()
                .unwrap()
        );

        let parsed_socket_addr: std::net::SocketAddr =
            multipart_instance.post("test_socket_addr").unwrap();
        assert_eq!(
            parsed_socket_addr,
            "127.0.0.1:8080".parse::<std::net::SocketAddr>().unwrap()
        );
    }

    // Test 14: Test custom type support using the macro
    #[tokio::test]
    async fn test_custom_type_support() {
        use crate::impl_post_parseable_for_custom_type;

        // Define a custom type that implements FromStr
        #[derive(Debug, PartialEq)]
        struct CustomId(u64);

        impl std::str::FromStr for CustomId {
            type Err = std::num::ParseIntError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(CustomId(s.parse()?))
            }
        }

        // Use the macro to enable PostParseable support
        impl_post_parseable_for_custom_type!(CustomId);

        // Create multipart instance
        let headers = HeaderMap::new();
        let payload = Payload::None;
        let multipart = NtexMultipart::new(&headers, payload);
        let mut multipart_instance = Multipart::new(multipart).await;

        // Add test data
        multipart_instance
            .data_inputs
            .entry("custom_id".to_string())
            .or_insert_with(Vec::new)
            .push(DataInput {
                name: "custom_id".to_string(),
                value: "12345".to_string(),
            });

        // Test parsing the custom type
        let parsed_id: CustomId = multipart_instance.post("custom_id").unwrap();
        assert_eq!(parsed_id, CustomId(12345));

        // Test with Option
        let optional_id: Option<CustomId> = multipart_instance.post("custom_id").unwrap();
        assert_eq!(optional_id, Some(CustomId(12345)));

        // Test missing field with Option
        let missing_id: Option<CustomId> = multipart_instance.post("missing_id").unwrap();
        assert_eq!(missing_id, None);

        // Test error handling
        multipart_instance
            .data_inputs
            .entry("invalid_id".to_string())
            .or_insert_with(Vec::new)
            .push(DataInput {
                name: "invalid_id".to_string(),
                value: "not_a_number".to_string(),
            });

        let result: Result<CustomId, _> = multipart_instance.post("invalid_id");
        assert!(result.is_err());
    }

    // Test 15: Real-world usage example demonstrating complete integration
    #[tokio::test]
    async fn test_real_world_integration() {
        use crate::impl_post_parseable_for_custom_type;

        // Define realistic custom types that a web application might use
        #[derive(Debug, PartialEq)]
        struct OrderId(String);

        impl std::str::FromStr for OrderId {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                if s.starts_with("ORD-") && s.len() == 10 {
                    Ok(OrderId(s.to_string()))
                } else {
                    Err(format!("Invalid order ID format: {s}"))
                }
            }
        }

        #[derive(Debug, PartialEq)]
        struct Money {
            cents: u64,
            currency: String,
        }

        impl std::str::FromStr for Money {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                // Parse formats like "USD 1234" (cents) or "$12.34"
                if let Some(amount_str) = s.strip_prefix('$') {
                    let dollars: f64 = amount_str
                        .parse()
                        .map_err(|_| format!("Invalid dollar amount: {amount_str}"))?;
                    Ok(Money {
                        cents: (dollars * 100.0) as u64,
                        currency: "USD".to_string(),
                    })
                } else if let Some(space_idx) = s.find(' ') {
                    let currency = &s[..space_idx];
                    let cents_str = &s[space_idx + 1..];
                    let cents: u64 = cents_str
                        .parse()
                        .map_err(|_| format!("Invalid cents amount: {cents_str}"))?;
                    Ok(Money {
                        cents,
                        currency: currency.to_string(),
                    })
                } else {
                    Err(format!("Invalid money format: {s}"))
                }
            }
        }

        // Enable multipart parsing for custom types
        impl_post_parseable_for_custom_type!(OrderId);
        impl_post_parseable_for_custom_type!(Money);

        // Create multipart instance
        let headers = HeaderMap::new();
        let payload = Payload::None;
        let multipart = NtexMultipart::new(&headers, payload);
        let mut multipart_instance = Multipart::new(multipart).await;

        // Simulate form data from a real e-commerce application
        let form_data = vec![
            ("order_id", "ORD-123456"),
            ("customer_name", "John Doe"),
            ("email", "john.doe@example.com"),
            ("product_count", "3"),
            ("total_amount", "$149.99"),
            ("discount_amount", "USD 1500"), // 15.00 in cents
            ("is_priority", "true"),
            ("shipping_weight", "2.5"),
            ("notes", "Please handle with care"),
        ];

        // Add form data to multipart
        for (name, value) in form_data {
            multipart_instance
                .data_inputs
                .entry(name.to_string())
                .or_insert_with(Vec::new)
                .push(DataInput {
                    name: name.to_string(),
                    value: value.to_string(),
                });
        }

        // Parse various field types - demonstrating the library's versatility

        // Custom types
        let order_id: OrderId = multipart_instance.post("order_id").unwrap();
        assert_eq!(order_id, OrderId("ORD-123456".to_string()));

        let total_amount: Money = multipart_instance.post("total_amount").unwrap();
        assert_eq!(
            total_amount,
            Money {
                cents: 14999,
                currency: "USD".to_string()
            }
        );

        let discount: Money = multipart_instance.post("discount_amount").unwrap();
        assert_eq!(
            discount,
            Money {
                cents: 1500,
                currency: "USD".to_string()
            }
        );

        // Standard types
        let customer_name: String = multipart_instance.post("customer_name").unwrap();
        assert_eq!(customer_name, "John Doe");

        let email: String = multipart_instance.post("email").unwrap();
        assert_eq!(email, "john.doe@example.com");

        let product_count: u32 = multipart_instance.post("product_count").unwrap();
        assert_eq!(product_count, 3);

        let is_priority: bool = multipart_instance.post("is_priority").unwrap();
        assert!(is_priority);

        let shipping_weight: f32 = multipart_instance.post("shipping_weight").unwrap();
        assert!((shipping_weight - 2.5).abs() < f32::EPSILON);

        let notes: String = multipart_instance.post("notes").unwrap();
        assert_eq!(notes, "Please handle with care");

        // Test optional fields
        let optional_field: Option<String> = multipart_instance.post("optional_field").unwrap();
        assert_eq!(optional_field, None);

        let optional_order_id: Option<OrderId> =
            multipart_instance.post("backup_order_id").unwrap();
        assert_eq!(optional_order_id, None);

        // Test default values
        let default_priority = multipart_instance.post_or("missing_priority", false);
        assert!(!default_priority);

        let default_amount = multipart_instance.post_or(
            "missing_amount",
            Money {
                cents: 0,
                currency: "USD".to_string(),
            },
        );
        assert_eq!(
            default_amount,
            Money {
                cents: 0,
                currency: "USD".to_string()
            }
        );

        // Test error handling with helpful messages
        multipart_instance
            .data_inputs
            .entry("invalid_order_id".to_string())
            .or_insert_with(Vec::new)
            .push(DataInput {
                name: "invalid_order_id".to_string(),
                value: "INVALID-ID".to_string(),
            });

        let error_result: Result<OrderId, _> = multipart_instance.post("invalid_order_id");
        assert!(error_result.is_err());

        let error_message = format!("{}", error_result.unwrap_err());
        assert!(error_message.contains("invalid_order_id"));
        assert!(error_message.contains("INVALID-ID"));

        println!("✅ All real-world integration tests passed!");
        println!("✅ Custom types: OrderId, Money");
        println!("✅ Standard types: String, u32, bool, f32");
        println!("✅ Optional fields with Option<T>");
        println!("✅ Default values with post_or");
        println!("✅ Error handling with descriptive messages");
    }
}
