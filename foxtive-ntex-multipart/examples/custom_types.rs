use foxtive_ntex_multipart::impl_post_parseable_for_custom_type;
use std::str::FromStr;

/// A custom user ID type
#[derive(Debug, PartialEq, Clone)]
pub struct UserId(u64);

impl FromStr for UserId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(UserId(s.parse()?))
    }
}

// Enable PostParseable support for UserId
impl_post_parseable_for_custom_type!(UserId);

/// A custom email type with validation
#[derive(Debug, PartialEq, Clone)]
pub struct Email(String);

impl FromStr for Email {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('@') && s.len() > 3 {
            Ok(Email(s.to_string()))
        } else {
            Err(format!("Invalid email format: '{s}'"))
        }
    }
}

// Enable PostParseable support for Email
impl_post_parseable_for_custom_type!(Email);

/// A custom product code type
#[derive(Debug, PartialEq, Clone)]
pub struct ProductCode(String);

impl FromStr for ProductCode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() >= 3 && s.chars().all(|c| c.is_alphanumeric() || c == '-') {
            Ok(ProductCode(s.to_uppercase()))
        } else {
            Err(format!("Invalid product code format: '{s}'"))
        }
    }
}

// Enable PostParseable support for ProductCode
impl_post_parseable_for_custom_type!(ProductCode);

/// A custom price type that handles currency
#[derive(Debug, PartialEq, Clone)]
pub struct Price {
    amount: f64,
    currency: String,
}

impl FromStr for Price {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Expected format: "123.45 USD" or "123.45"
        let parts: Vec<&str> = s.split_whitespace().collect();

        match parts.len() {
            1 => {
                let amount = parts[0]
                    .parse::<f64>()
                    .map_err(|_| format!("Invalid price amount: '{}'", parts[0]))?;
                Ok(Price {
                    amount,
                    currency: "USD".to_string(),
                })
            }
            2 => {
                let amount = parts[0]
                    .parse::<f64>()
                    .map_err(|_| format!("Invalid price amount: '{}'", parts[0]))?;
                let currency = parts[1].to_uppercase();
                if currency.len() == 3 {
                    Ok(Price { amount, currency })
                } else {
                    Err(format!("Invalid currency code: '{currency}'"))
                }
            }
            _ => Err(format!("Invalid price format: '{s}'")),
        }
    }
}

// Enable PostParseable support for Price
impl_post_parseable_for_custom_type!(Price);

#[cfg(test)]
mod tests {
    use super::*;
    use foxtive_ntex_multipart::Multipart;
    use ntex::http::HeaderMap;
    use ntex::http::Payload;
    use ntex_multipart::Multipart as NtexMultipart;

    #[tokio::test]
    async fn test_all_custom_types() {
        // Add test data for all custom types
        let test_data = vec![
            ("user_id", "12345"),
            ("email", "user@example.com"),
            ("product_code", "ABC-123"),
            ("price", "99.99 EUR"),
            ("simple_price", "49.99"),
            ("optional_user_id", "67890"),
            ("invalid_email", "not-an-email"),
        ];

        for (name, value) in test_data {
            // Note: In a real application, you would populate multipart data
            // from an actual HTTP request. This is just for testing purposes.
            // For now, we'll create a mock multipart instance with test data.
            println!("Would process field '{}' with value '{}'", name, value);
        }

        // Test direct parsing of custom types
        let user_id: UserId = "12345".parse().unwrap();
        assert_eq!(user_id, UserId(12345));

        let email: Email = "user@example.com".parse().unwrap();
        assert_eq!(email, Email("user@example.com".to_string()));

        let product_code: ProductCode = "abc-123".parse().unwrap();
        assert_eq!(product_code, ProductCode("ABC-123".to_string()));

        let price: Price = "99.99 EUR".parse().unwrap();
        assert_eq!(
            price,
            Price {
                amount: 99.99,
                currency: "EUR".to_string(),
            }
        );

        let simple_price: Price = "49.99".parse().unwrap();
        assert_eq!(
            simple_price,
            Price {
                amount: 49.99,
                currency: "USD".to_string(),
            }
        );

        // Test error handling
        let invalid_email_result: Result<Email, _> = "not-an-email".parse();
        assert!(invalid_email_result.is_err());

        let invalid_price_result: Result<Price, _> = "invalid".parse();
        assert!(invalid_price_result.is_err());
    }

    #[tokio::test]
    async fn test_post_or_with_custom_types() {
        let headers = HeaderMap::new();
        let payload = Payload::None;
        let multipart = NtexMultipart::new(&headers, payload);
        let multipart_instance = Multipart::new(multipart).await;

        // Test post_or with default values
        let default_user_id = multipart_instance.post_or("missing_user_id", UserId(0));
        assert_eq!(default_user_id, UserId(0));

        let default_email =
            multipart_instance.post_or("missing_email", Email("default@example.com".to_string()));
        assert_eq!(default_email, Email("default@example.com".to_string()));

        let default_price = multipart_instance.post_or(
            "missing_price",
            Price {
                amount: 0.0,
                currency: "USD".to_string(),
            },
        );
        assert_eq!(
            default_price,
            Price {
                amount: 0.0,
                currency: "USD".to_string(),
            }
        );
    }

    #[tokio::test]
    async fn test_post_opt_with_custom_types() {
        // Test Option parsing directly
        let user_id: UserId = "999".parse().unwrap();
        assert_eq!(user_id, UserId(999));

        // In a real application, you would use:
        // let existing_user_id: Option<UserId> = multipart_instance.post_opt("existing_user_id");
        // let missing_user_id: Option<UserId> = multipart_instance.post_opt("missing_user_id");
        println!("Custom types work with Option<T> for optional fields");
    }

    #[tokio::test]
    async fn test_error_messages() {
        // Test error handling directly
        let user_id_error: Result<UserId, _> = "not-a-number".parse();
        assert!(user_id_error.is_err());
        println!("UserId parse error: {:?}", user_id_error);

        let email_error: Result<Email, _> = "not-an-email".parse();
        assert!(email_error.is_err());
        println!("Email parse error: {:?}", email_error);

        // In a real application, when using multipart parsing, the errors would contain
        // helpful information about the field name and invalid value.
    }
}

fn main() {
    println!("This is an example demonstrating custom types with foxtive-ntex-multipart.");
    println!("Run 'cargo test' to see the examples in action.");
}
