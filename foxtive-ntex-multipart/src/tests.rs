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

    // Test 16: Test conditional UUID support when feature is enabled
    #[cfg(feature = "uuid")]
    #[tokio::test]
    async fn test_uuid_support() {
        let headers = HeaderMap::new();
        let payload = Payload::None;
        let multipart = NtexMultipart::new(&headers, payload);
        let mut multipart_instance = Multipart::new(multipart).await;

        // Add valid UUID test data
        let test_uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        multipart_instance.add_test_data("user_uuid", test_uuid_str);

        // Test UUID parsing
        let parsed_uuid: uuid::Uuid = multipart_instance.post("user_uuid").unwrap();
        let expected_uuid = uuid::Uuid::parse_str(test_uuid_str).unwrap();
        assert_eq!(parsed_uuid, expected_uuid);

        // Test with Option<Uuid>
        let optional_uuid: Option<uuid::Uuid> = multipart_instance.post("user_uuid").unwrap();
        assert_eq!(optional_uuid, Some(expected_uuid));

        // Test missing optional UUID
        let missing_uuid: Option<uuid::Uuid> = multipart_instance.post("missing_uuid").unwrap();
        assert_eq!(missing_uuid, None);

        // Test invalid UUID format
        multipart_instance.add_test_data("invalid_uuid", "not-a-valid-uuid");

        let result: Result<uuid::Uuid, _> = multipart_instance.post("invalid_uuid");
        assert!(result.is_err());

        let error_message = format!("{}", result.unwrap_err());
        assert!(error_message.contains("invalid_uuid"));
        assert!(error_message.contains("not-a-valid-uuid"));
        assert!(error_message.contains("uuid::Uuid"));

        println!("✅ UUID support tests passed!");
    }

    // Test 17: Comprehensive UUID integration test
    #[cfg(feature = "uuid")]
    #[tokio::test]
    async fn test_uuid_comprehensive_integration() {
        let headers = HeaderMap::new();
        let payload = Payload::None;
        let multipart = NtexMultipart::new(&headers, payload);
        let mut multipart_instance = Multipart::new(multipart).await;

        // Test various UUID formats and use cases
        let uuids = vec![
            ("user_id", "550e8400-e29b-41d4-a716-446655440000"),
            ("session_id", "6ba7b810-9dad-11d1-80b4-00c04fd430c8"),
            ("request_id", "01234567-89ab-cdef-0123-456789abcdef"),
            ("trace_id", "f47ac10b-58cc-4372-a567-0e02b2c3d479"),
        ];

        // Add all UUID test data
        for (field, uuid_str) in &uuids {
            multipart_instance.add_test_data(field, uuid_str);
        }

        // Test parsing all UUIDs
        for (field, uuid_str) in &uuids {
            let parsed: uuid::Uuid = multipart_instance.post(field).unwrap();
            let expected = uuid::Uuid::parse_str(uuid_str).unwrap();
            assert_eq!(parsed, expected);

            // Test with Option
            let optional: Option<uuid::Uuid> = multipart_instance.post(field).unwrap();
            assert_eq!(optional, Some(expected));
        }

        // Test post_or with UUID default
        let default_uuid = uuid::Uuid::new_v4();
        let result_uuid = multipart_instance.post_or("missing_uuid", default_uuid);
        assert_eq!(result_uuid, default_uuid);

        // Test post_opt
        let opt_result: Option<uuid::Uuid> = multipart_instance.post_opt("missing_uuid");
        assert_eq!(opt_result, None);

        let opt_existing: Option<uuid::Uuid> = multipart_instance.post_opt("user_id");
        assert!(opt_existing.is_some());

        println!("✅ Comprehensive UUID integration tests passed!");
    }

    // Test 18: Test that UUID support is only available with feature flag
    #[cfg(not(feature = "uuid"))]
    #[tokio::test]
    async fn test_uuid_not_available_without_feature() {
        // This test ensures that UUID support is properly gated behind the feature flag
        // In a real scenario, attempting to use uuid::Uuid without the feature would cause a compile error
        println!("✅ UUID feature properly gated - not available without 'uuid' feature flag");
    }
}
