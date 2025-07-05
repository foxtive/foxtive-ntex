use crate::error::HttpError;
use foxtive::prelude::AppMessage;
use log::debug;
use ntex::http::Payload;
use ntex::util::BytesMut;
use ntex::web::{FromRequest, HttpRequest};
use serde::de::DeserializeOwned;
use std::ops;

/// A wrapper struct that holds both the raw JSON string and its deserialized form.
///
/// This struct is useful when you need both the raw JSON string and the parsed
/// object, avoiding multiple deserialization operations.
pub struct DeJsonBody<T: DeserializeOwned>(String, T);

impl<T: DeserializeOwned> DeJsonBody<T> {
    /// Creates a new `DeJsonBody` instance by parsing the given JSON string.
    ///
    /// # Arguments
    /// * `json` - A string slice containing valid JSON
    ///
    /// # Returns
    /// * `AppResult<DeJsonBody<T>>` - Result containing the new instance or an error
    ///
    /// # Errors
    /// Returns an error if the JSON string cannot be deserialized into the target type T.
    pub fn new(json: String) -> Result<DeJsonBody<T>, HttpError> {
        let t = serde_json::from_str::<T>(&json)
            .map_err(|e| AppMessage::WarningMessageString(e.to_string()))?;

        Ok(DeJsonBody(json, t))
    }

    /// Returns a reference to the raw JSON string.
    ///
    /// # Example
    /// ```
    /// use foxtive_ntex::http::extractors::DeJsonBody;
    ///
    /// let json_str = "{\"field1\": \"value1\", \"field2\": 42}".to_string();
    /// let de_json_body = DeJsonBody::<serde_json::Value>::new(json_str.clone()).unwrap();
    /// assert_eq!(de_json_body.body(), &json_str);
    /// ```
    pub fn body(&self) -> &String {
        &self.0
    }

    /// Consumes the `JsonBody`, returning the inner JSON string.
    ///
    /// # Example
    /// ```
    /// use foxtive_ntex::http::extractors::DeJsonBody;
    /// let json_str = "{\"field1\": \"value1\", \"field2\": 42}".to_string();
    /// let de_json_body = DeJsonBody::<serde_json::Value>::new(json_str.clone()).unwrap();
    /// assert_eq!(de_json_body.into_body(), json_str);
    /// ```
    pub fn into_body(self) -> String {
        self.0
    }

    /// Returns a reference to the deserialized object.
    ///
    /// # Example
    /// ```
    /// use foxtive_ntex::http::extractors::DeJsonBody;
    /// let json_str = "{\"field1\": \"value1\", \"field2\": 42}".to_string();
    /// let manual_body = serde_json::from_str::<serde_json::Value>(&json_str).unwrap();
    /// let de_json_body = DeJsonBody::<serde_json::Value>::new(json_str.clone()).unwrap();
    /// assert_eq!(de_json_body.inner(), &manual_body);
    /// ```
    pub fn inner(&self) -> &T {
        &self.1
    }

    /// Consumes the `JsonBody`, returning the inner deserialized object.
    ///
    /// # Example
    /// ```
    /// use foxtive_ntex::http::extractors::DeJsonBody;
    /// let json_str = "{\"field1\": \"value1\", \"field2\": 42}".to_string();
    /// let manual_body = serde_json::from_str::<serde_json::Value>(&json_str).unwrap();
    /// let de_json_body = DeJsonBody::<serde_json::Value>::new(json_str.clone()).unwrap();
    /// assert_eq!(de_json_body.into_inner(), manual_body);
    /// ```
    pub fn into_inner(self) -> T {
        self.1
    }
}

impl<T: DeserializeOwned, Err> FromRequest<Err> for DeJsonBody<T> {
    type Error = HttpError;

    async fn from_request(
        _req: &HttpRequest,
        payload: &mut Payload,
    ) -> Result<DeJsonBody<T>, Self::Error> {
        let mut bytes = BytesMut::new();
        while let Some(item) = ntex::util::stream_recv(payload).await {
            bytes.extend_from_slice(&item?);
        }

        let raw = String::from_utf8(bytes.to_vec())?;

        debug!("[json-body] {raw}");

        Self::new(raw)
    }
}

impl<T: DeserializeOwned> ops::Deref for DeJsonBody<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.1
    }
}

impl<T: DeserializeOwned> ops::DerefMut for DeJsonBody<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntex::http::StatusCode;
    use ntex::web::WebResponseError;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestStruct {
        field1: String,
        field2: i32,
    }

    #[test]
    fn test_body() {
        let json_str = r#"{"field1": "value1", "field2": 42}"#.to_string();
        let de_json_body = DeJsonBody::<TestStruct>::new(json_str.clone()).unwrap();

        assert_eq!(de_json_body.body(), &json_str);
    }

    #[test]
    fn test_into_body() {
        let json_str = r#"{"field1": "value1", "field2": 42}"#.to_string();
        let de_json_body = DeJsonBody::<TestStruct>::new(json_str.clone()).unwrap();

        assert_eq!(de_json_body.into_body(), json_str);
    }

    #[test]
    fn test_inner() {
        let json_str = r#"{"field1": "value1", "field2": 42}"#.to_string();
        let de_json_body = DeJsonBody::<TestStruct>::new(json_str).unwrap();

        let expected = TestStruct {
            field1: "value1".to_string(),
            field2: 42,
        };

        assert_eq!(*de_json_body.inner(), expected);
    }

    #[test]
    fn test_into_inner() {
        let json_str = r#"{"field1": "value1", "field2": 42}"#.to_string();
        let de_json_body = DeJsonBody::<TestStruct>::new(json_str).unwrap();

        let expected = TestStruct {
            field1: "value1".to_string(),
            field2: 42,
        };

        assert_eq!(de_json_body.into_inner(), expected);
    }

    #[test]
    fn test_deserialize_success() {
        let json_str = r#"{"field1": "value1", "field2": 42}"#.to_string();
        let de_json_body = DeJsonBody::<TestStruct>::new(json_str).unwrap();

        let expected = TestStruct {
            field1: "value1".to_string(),
            field2: 42,
        };

        assert_eq!(*de_json_body.inner(), expected);
    }

    #[test]
    fn test_deserialize_failure() {
        let json_str = r#"{"field1": "value1", "field2": "invalid_int"}"#.to_string();
        let result = DeJsonBody::<TestStruct>::new(json_str);

        let error = match result {
            Err(ref err) => {
                assert!(result.is_err());
                err
            }
            Ok(_) => {
                panic!("Expected Err, got Ok(Val)");
            }
        };

        assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_deserialize_to_map() {
        let json_str = r#"{"key1": "value1", "key2": "value2"}"#.to_string();
        let de_json_body = DeJsonBody::<HashMap<String, String>>::new(json_str).unwrap();

        let expected = {
            let mut map = HashMap::new();
            map.insert("key1".to_string(), "value1".to_string());
            map.insert("key2".to_string(), "value2".to_string());
            map
        };

        assert_eq!(*de_json_body.inner(), expected);
    }
}
