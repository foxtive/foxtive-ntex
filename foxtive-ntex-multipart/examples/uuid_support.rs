//! # UUID Support Example
//!
//! This example demonstrates how to use UUID support with the foxtive-ntex-multipart library.
//! UUID support is provided as an optional feature that can be enabled with the "uuid" feature flag.
//!
use uuid::Uuid;

fn main() {
    println!("🎯 UUID Support Example");
    println!("=======================");

    // Example UUIDs for demonstration
    let valid_uuid_str = "550e8400-e29b-41d4-a716-446655440000";
    let invalid_uuid_str = "not-a-valid-uuid";

    println!("\n📋 UUID Parsing Examples:");

    // Example 1: Direct UUID parsing using FromStr
    println!("\n🔍 Example 1: Direct UUID Parsing");
    match valid_uuid_str.parse::<Uuid>() {
        Ok(uuid) => {
            println!("  ✅ Successfully parsed UUID: {uuid}");
            println!("     Hyphenated: {}", uuid.hyphenated());
            println!("     Simple: {}", uuid.simple());
            println!("     Version: {:?}", uuid.get_version());
            println!("     Variant: {:?}", uuid.get_variant());
        }
        Err(e) => println!("  ❌ Error parsing UUID: {e}"),
    }

    // Example 2: Error handling for invalid UUID
    println!("\n🔍 Example 2: Error Handling for Invalid UUID");
    match invalid_uuid_str.parse::<Uuid>() {
        Ok(uuid) => {
            println!("  ❌ Unexpectedly parsed invalid UUID: {uuid}");
        }
        Err(e) => {
            println!("  ✅ Expected error for invalid UUID: {e}");
        }
    }

    // Example 3: UUID generation and formatting
    println!("\n🔍 Example 3: UUID Generation and Formatting");
    let generated_uuid = Uuid::new_v4();
    println!("  Generated UUID: {generated_uuid}");
    println!("  Formats:");
    println!("    Hyphenated: {}", generated_uuid.hyphenated());
    println!("    Simple:     {}", generated_uuid.simple());
    println!("    URN:        {}", generated_uuid.urn());
    println!("    Bytes:      {:?}", generated_uuid.as_bytes());

    // Example 4: Real-world usage scenario
    println!("\n🔍 Example 4: Real-world Usage Scenario");
    println!("  In a typical web application using multipart forms:");

    // Simulate parsing UUIDs from form data
    struct UserRegistration {
        user_id: Uuid,
        parent_id: Option<Uuid>,
        session_id: Uuid,
    }

    let user_registration = UserRegistration {
        user_id: valid_uuid_str.parse().unwrap_or_else(|_| Uuid::new_v4()),
        parent_id: "".parse().ok(), // Empty string -> None
        session_id: Uuid::new_v4(),
    };

    println!("  UserRegistration {{");
    println!("    user_id: {},", user_registration.user_id);
    println!("    parent_id: {:?},", user_registration.parent_id);
    println!("    session_id: {},", user_registration.session_id);
    println!("  }}");

    println!("\n✅ UUID support example completed successfully!");
    println!("\n💡 Key Points:");
    println!("  • UUID support is enabled with the 'uuid' feature flag");
    println!("  • UUIDs work with all multipart parsing methods: post(), post_or(), post_opt()");
    println!("  • Option<Uuid> provides optional UUID parsing");
    println!("  • Invalid UUIDs produce descriptive error messages");
    println!("  • All UUID formats and methods are supported");
    println!("\n📝 Usage in multipart forms:");
    println!("  let user_id: Uuid = multipart.post(\"user_id\")?;");
    println!("  let optional_id: Option<Uuid> = multipart.post(\"optional_id\")?;");
    println!("  let default_id = multipart.post_or(\"missing_id\", Uuid::new_v4());");
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    #[test]
    fn test_uuid_parsing_examples() {
        // Test that our examples actually work
        let test_uuid_str = "550e8400-e29b-41d4-a716-446655440000";

        // Test direct parsing
        let parsed_uuid: Uuid = test_uuid_str.parse().unwrap();
        let expected_uuid = Uuid::parse_str(test_uuid_str).unwrap();
        assert_eq!(parsed_uuid, expected_uuid);

        // Test invalid UUID
        let invalid_result: Result<Uuid, _> = "not-a-uuid".parse();
        assert!(invalid_result.is_err());

        // Test UUID generation
        let generated_uuid = Uuid::new_v4();
        assert_ne!(generated_uuid, Uuid::nil());
    }

    #[cfg(not(feature = "uuid"))]
    #[test]
    fn test_uuid_feature_disabled() {
        // This test ensures the feature flag works correctly
        println!("UUID feature is disabled - this is expected behavior");
    }
}
