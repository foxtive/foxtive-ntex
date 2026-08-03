use crate::error::HttpError;
use crate::http::server::BodyConfig;
use foxtive::prelude::AppMessage;
use ntex::http::Payload;
use ntex::util::BytesMut;
use ntex::web::{FromRequest, HttpRequest};
use tracing::debug;

/// Extractor for reading the request body as a plain UTF-8 string.
///
/// # Example
/// ```
/// use foxtive_ntex::http::extractors::StringBody;
///
/// async fn handler(body: StringBody) -> String {
///     format!("Received: {}", body.body())
/// }
/// ```
pub struct StringBody {
    body: String,
}

impl StringBody {
    /// Returns a reference to the underlying string body.
    pub fn body(&self) -> &String {
        &self.body
    }

    /// Consumes the `StringBody`, returning the inner string.
    pub fn into_body(self) -> String {
        self.body
    }

    /// Returns the length of the string body in bytes.
    pub fn len(&self) -> usize {
        self.body.len()
    }

    /// Returns true if the string body is empty.
    pub fn is_empty(&self) -> bool {
        self.body.is_empty()
    }

    /// Tries to parse the body to a specific type that implements `FromStr`.
    /// Returns an application-level result or an error if parsing fails.
    pub fn parse<T: std::str::FromStr>(&self) -> Result<T, AppMessage>
    where
        <T as std::str::FromStr>::Err: ToString,
    {
        self.body
            .parse::<T>()
            .map_err(|e| AppMessage::invalid(e.to_string()))
    }
}

impl From<String> for StringBody {
    fn from(body: String) -> Self {
        Self { body }
    }
}

impl From<&str> for StringBody {
    fn from(body: &str) -> Self {
        Self {
            body: body.to_owned(),
        }
    }
}

impl<Err> FromRequest<Err> for StringBody {
    type Error = HttpError;

    async fn from_request(req: &HttpRequest, payload: &mut Payload) -> Result<Self, Self::Error> {
        let max_size = req
            .app_state::<BodyConfig>()
            .map(|c| c.string_limit)
            .unwrap_or(262_144);
        let mut bytes = BytesMut::new();
        let mut total_size = 0;

        while let Some(chunk) = ntex::util::stream_recv(payload).await {
            let chunk = chunk?;
            total_size += chunk.len();

            if total_size > max_size {
                return Err(HttpError::AppMessage(AppMessage::invalid(
                    "String body exceeded maximum size",
                )));
            }

            bytes.extend_from_slice(&chunk);
        }

        let raw = String::from_utf8(bytes.to_vec())?;
        debug!("[string-body] {raw}");
        Ok(Self { body: raw })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntex::http::StatusCode;

    #[test]
    fn test_body_and_into_body() {
        let data = "hello string body".to_string();
        let sb = StringBody::from(data.clone());
        assert_eq!(sb.body(), &data);

        let sb = StringBody::from(&data[..]);
        assert_eq!(sb.body(), &data);

        let moved = sb.into_body();
        assert_eq!(moved, data);
    }

    #[test]
    fn test_len_and_is_empty() {
        let empty = StringBody::from("");
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);

        let s = StringBody::from("abcde");
        assert!(!s.is_empty());
        assert_eq!(s.len(), 5);
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_parse_success() {
        let s = StringBody::from("42");
        let val: i32 = s.parse().unwrap();
        assert_eq!(val, 42);

        let s = StringBody::from("3.1415");
        let val: f64 = s.parse().unwrap();
        assert!((val - 3.1415).abs() < 1e-6);
    }

    #[test]
    fn test_parse_failure() {
        let s = StringBody::from("not_a_number");
        let result = s.parse::<i32>();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
        // Message should include 'invalid digit' for i32::FromStr
        assert!(err.message().to_lowercase().contains("invalid"));
    }

    #[test]
    fn test_deprecated_raw() {
        let data = "raw string body".to_string();
        let sb = StringBody::from(data.clone());
        assert_eq!(sb.body(), &data);
    }
}
