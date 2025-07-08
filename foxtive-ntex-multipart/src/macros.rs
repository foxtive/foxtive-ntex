use crate::*;

/// Macro to implement PostParseableFromStr for specific types
/// This allows us to support all common FromStr types while excluding Option<T>
macro_rules! impl_post_parseable_from_str {
    ($($t:ty),*) => {
        $(
            impl sealed::Sealed for $t {}

            impl PostParseableFromStr for $t {
                fn parse_from_multipart_str(multipart: &Multipart, field: &str) -> MultipartResult<Self> {
                    let data_input = multipart.first_data_required(field)?;
                    let value = data_input.value.trim();

                    // Handle empty values
                    if value.is_empty() {
                        return Err(MultipartError::ParseError(format!(
                            "Field '{}' is empty and cannot be parsed as {}",
                            field,
                            std::any::type_name::<$t>()
                        )));
                    }

                    value.parse::<$t>().map_err(|e| {
                        MultipartError::ParseError(format!(
                            "Failed to parse field '{}' with value '{}' as {}: {}",
                            field,
                            value,
                            std::any::type_name::<$t>(),
                            e
                        ))
                    })
                }
            }
        )*
    };
}

// Implement for all standard library types that implement FromStr
impl_post_parseable_from_str!(
    // Integer types
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    // Floating point types
    f32,
    f64,
    // Other standard types
    bool,
    char,
    String,
    // Network types
    std::net::IpAddr,
    std::net::Ipv4Addr,
    std::net::Ipv6Addr,
    std::net::SocketAddr,
    std::net::SocketAddrV4,
    std::net::SocketAddrV6,
    // Path types
    std::path::PathBuf,
    // NonZero types
    std::num::NonZeroI8,
    std::num::NonZeroI16,
    std::num::NonZeroI32,
    std::num::NonZeroI64,
    std::num::NonZeroI128,
    std::num::NonZeroIsize,
    std::num::NonZeroU8,
    std::num::NonZeroU16,
    std::num::NonZeroU32,
    std::num::NonZeroU64,
    std::num::NonZeroU128,
    std::num::NonZeroUsize
);

// Uuid Support
#[cfg(feature = "uuid")]
impl_post_parseable_from_str!(uuid::Uuid);

/// Helper macro for users to implement PostParseableFromStr for their custom types
///
/// This macro allows users to easily add support for their custom types that implement FromStr.
/// It automatically implements both the sealed trait and PostParseableFromStr for the given type,
/// enabling it to work with the multipart parsing system.
///
/// ## Requirements
///
/// Your custom type must implement:
/// - `std::str::FromStr` - for parsing from strings
/// - `FromStr::Err` must implement `std::fmt::Display` - for error formatting
///
/// ## Usage
///
/// ```
/// use foxtive_ntex_multipart::impl_post_parseable_for_custom_type;
///
/// #[derive(Debug, PartialEq)]
/// struct UserId(u64);
///
/// impl std::str::FromStr for UserId {
///     type Err = std::num::ParseIntError;
///     fn from_str(s: &str) -> Result<Self, Self::Err> {
///         Ok(UserId(s.parse()?))
///     }
/// }
///
/// // This enables PostParseable support for UserId
/// impl_post_parseable_for_custom_type!(UserId);
///
/// // Now you can use UserId in multipart parsing:
/// // let user_id: UserId = multipart.post("user_id")?;
/// // let optional_id: Option<UserId> = multipart.post("optional_id")?;
/// ```
///
/// ## Advanced Example
///
/// ```
/// use foxtive_ntex_multipart::impl_post_parseable_for_custom_type;
/// use std::str::FromStr;
///
/// #[derive(Debug, PartialEq)]
/// struct Email(String);
///
/// impl FromStr for Email {
///     type Err = String;
///
///     fn from_str(s: &str) -> Result<Self, Self::Err> {
///         if s.contains('@') {
///             Ok(Email(s.to_string()))
///         } else {
///             Err(format!("Invalid email format: {}", s))
///         }
///     }
/// }
///
/// impl_post_parseable_for_custom_type!(Email);
/// ```
///
/// ## Error Handling
///
/// The macro automatically handles:
/// - Empty field values (returns ParseError)
/// - Missing fields (returns ParseError for required fields)
/// - Invalid format (uses your FromStr::Err for formatting)
/// - Proper error messages with field names and type information
///
/// ## Integration with Option<T>
///
/// Your custom types automatically work with Option<T> for optional fields:
/// - `multipart.post::<MyType>("field")` - required field, returns error if missing
/// - `multipart.post::<Option<MyType>>("field")` - optional field, returns None if missing
#[macro_export]
macro_rules! impl_post_parseable_for_custom_type {
    ($t:ty) => {
        impl $crate::sealed::Sealed for $t {}

        impl $crate::PostParseableFromStr for $t {
            fn parse_from_multipart_str(
                multipart: &$crate::Multipart,
                field: &str,
            ) -> $crate::MultipartResult<Self> {
                let data_input = multipart.first_data_required(field)?;
                let value = data_input.value.trim();

                // Handle empty values
                if value.is_empty() {
                    return Err($crate::MultipartError::ParseError(format!(
                        "Field '{}' is empty and cannot be parsed as {}",
                        field,
                        std::any::type_name::<$t>()
                    )));
                }

                value.parse::<$t>().map_err(|e| {
                    $crate::MultipartError::ParseError(format!(
                        "Failed to parse field '{}' with value '{}' as {}: {}",
                        field,
                        value,
                        std::any::type_name::<$t>(),
                        e
                    ))
                })
            }
        }
    };
}
