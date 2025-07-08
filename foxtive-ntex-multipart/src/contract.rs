use std::str::FromStr;
use crate::{Multipart, MultipartError};
use crate::result::MultipartResult;

/// Trait for types that can be parsed from multipart form data
pub trait PostParseable: Sized {
    fn parse_from_multipart(multipart: &Multipart, field: &str) -> MultipartResult<Self>;
}

/// Trait for types that can be parsed from multipart form data.
///
/// This trait acts as a bridge between `std::str::FromStr` and `PostParseable`, allowing
/// any type that implements `FromStr` to be used in multipart parsing while avoiding
/// trait conflicts with `Option<T>`.
///
/// ## For Library Users
///
/// You don't need to implement this trait directly. Instead, use the
/// `impl_post_parseable_for_custom_type!` macro which automatically implements
/// both this trait and the sealed trait for your custom types.
///
/// ## For Library Maintainers
///
/// This trait uses the sealed trait pattern to control which types can implement
/// `PostParseable`. All standard library types that implement `FromStr` are
/// automatically supported, and users can add support for their custom types
/// via the provided macro.
pub trait PostParseableFromStr: Sized + sealed::Sealed {
    /// Parse a value from multipart data using FromStr
    ///
    /// This method handles the standard parsing logic:
    /// 1. Extracts the field value from multipart data
    /// 2. Trims whitespace
    /// 3. Handles empty values (returns error)
    /// 4. Attempts to parse using `FromStr`
    /// 5. Provides detailed error messages on failure
    fn parse_from_multipart_str(multipart: &Multipart, field: &str) -> MultipartResult<Self>;
}

/// Sealed module to control which types can implement PostParseableFromStr
///
/// This module implements the sealed trait pattern to prevent external crates
/// from implementing `PostParseableFromStr` for arbitrary types, which could
/// cause trait conflicts. Only types that are explicitly allowed (via the macro
/// or internal implementations) can implement `PostParseableFromStr`.
pub mod sealed {
    /// Sealed trait to control trait implementation
    ///
    /// This trait is automatically implemented for all standard library types
    /// that implement `FromStr`. Users can add support for their custom types
    /// using the `impl_post_parseable_for_custom_type!` macro.
    pub trait Sealed {}
}



/// Blanket implementation of PostParseable for all types that implement PostParseableFromStr
/// This provides the bridge between PostParseableFromStr and PostParseable
impl<T> PostParseable for T
where
    T: PostParseableFromStr,
{
    fn parse_from_multipart(multipart: &Multipart, field: &str) -> MultipartResult<Self> {
        Self::parse_from_multipart_str(multipart, field)
    }
}

/// Special implementation for Option<T> - returns None for missing or empty fields
impl<T> PostParseable for Option<T>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    fn parse_from_multipart(multipart: &Multipart, field: &str) -> MultipartResult<Self> {
        // Check if field exists
        if let Some(data_input) = multipart.first_data(field) {
            let value = data_input.value.trim();

            // Return None for empty values
            if value.is_empty() {
                return Ok(None);
            }

            // Try to parse the value
            match value.parse::<T>() {
                Ok(parsed_value) => Ok(Some(parsed_value)),
                Err(e) => Err(MultipartError::ParseError(format!(
                    "Failed to parse field '{}' with value '{}' as {}: {}",
                    field,
                    value,
                    std::any::type_name::<T>(),
                    e
                ))),
            }
        } else {
            // Field doesn't exist, return None
            Ok(None)
        }
    }
}
