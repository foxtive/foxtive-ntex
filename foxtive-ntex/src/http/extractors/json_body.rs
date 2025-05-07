use crate::error::HttpError;
use foxtive::prelude::{AppMessage, AppResult};
use log::{debug, error};
use ntex::http::Payload;
use ntex::util::BytesMut;
use ntex::web::{FromRequest, HttpRequest};
use serde::de::DeserializeOwned;

pub struct JsonBody {
    json: String,
}

impl JsonBody {
    #[deprecated(since = "0.9.0", note = "Use the 'body' method instead")]
    /// Returns the raw JSON string.
    ///
    /// # Deprecated
    /// This method is deprecated. Use [`body()`] instead.
    pub fn raw(&self) -> &String {
        &self.json
    }

    /// Returns a reference to the underlying JSON string.
    ///
    /// # Example
    /// ```
    /// use foxtive_ntex::http::extractors::json_body::JsonBody;
    ///
    /// let json_body = JsonBody::from("{\"key\": \"value\"}");
    /// assert_eq!(json_body.body(), "{\"key\": \"value\"}");
    /// ```
    pub fn body(&self) -> &String {
        &self.json
    }

    /// Consumes the `JsonBody`, returning the inner JSON string.
    ///
    /// # Example
    /// ```
    /// use foxtive_ntex::http::extractors::json_body::JsonBody;
    ///
    /// let json_body = JsonBody::from("{\"key\": \"value\"}");
    /// let json = json_body.into_body();
    /// assert_eq!(json, "{\"key\": \"value\"}");
    /// ```
    pub fn into_body(self) -> String {
        self.json
    }

    /// Deserializes the JSON string to the specified type.
    ///
    /// Returns an application result containing the deserialized value or an error if deserialization fails.
    ///
    /// # Errors
    /// Return an error if the JSON string cannot be deserialized to the target type.
    pub fn deserialize<T: DeserializeOwned>(&self) -> AppResult<T> {
        serde_json::from_str::<T>(&self.json).map_err(|e| {
            error!("Error deserializing JSON: {:?}", e);
            HttpError::AppMessage(AppMessage::WarningMessageString(e.to_string())).into_app_error()
        })
    }

    /// Parses and returns the JSON string as a [`serde_json::Value`].
    ///
    /// # Errors
    /// Return an error if the string is not valid JSON.
    pub fn json_value(&self) -> AppResult<serde_json::Value> {
        Ok(serde_json::from_str(&self.json)?)
    }
}

impl From<String> for JsonBody {
    /// Creates a `JsonBody` from a `String`.
    ///
    /// # Example
    /// ```
    /// use foxtive_ntex::http::extractors::json_body::JsonBody;
    ///
    /// let json_str = "{\"key\": \"value\"}".to_string();
    /// let json_body = JsonBody::from(json_str);
    /// ```
    fn from(json: String) -> Self {
        JsonBody { json }
    }
}

impl From<&str> for JsonBody {
    /// Creates a `JsonBody` from a `&str`.
    ///
    /// # Example
    /// ```
    /// use foxtive_ntex::http::extractors::json_body::JsonBody;
    ///
    /// let json_body = JsonBody::from("{\"key\": \"value\"}");
    /// ```
    fn from(json: &str) -> Self {
        JsonBody {
            json: json.to_string(),
        }
    }
}

impl<Err> FromRequest<Err> for JsonBody {
    type Error = HttpError;

    async fn from_request(
        _req: &HttpRequest,
        payload: &mut Payload,
    ) -> Result<JsonBody, Self::Error> {
        let mut bytes = BytesMut::new();
        while let Some(item) = ntex::util::stream_recv(payload).await {
            bytes.extend_from_slice(&item?);
        }

        let raw = String::from_utf8(bytes.to_vec())?;
        debug!("[json-body] {}", raw);
        Ok(JsonBody { json: raw })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntex::http::StatusCode;
    use ntex::web::WebResponseError;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::collections::HashMap;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestStruct {
        field1: String,
        field2: i32,
    }

    #[test]
    fn test_raw() {
        let json_str = r#"{"field1": "value1", "field2": 42}"#.to_string();
        let json_body = JsonBody {
            json: json_str.clone(),
        };

        assert_eq!(json_body.body(), &json_str);
    }

    #[test]
    fn test_deserialize_success() {
        let json_str = r#"{"field1": "value1", "field2": 42}"#.to_string();
        let json_body = JsonBody { json: json_str };

        let result: AppResult<TestStruct> = json_body.deserialize();
        assert!(result.is_ok());

        let deserialized = result.unwrap();
        let expected = TestStruct {
            field1: "value1".to_string(),
            field2: 42,
        };

        assert_eq!(deserialized, expected);
    }

    #[test]
    fn test_deserialize_failure() {
        let json_str = r#"{"field1": "value1", "field2": "invalid_int"}"#.to_string();
        let json_body = JsonBody { json: json_str };

        let result: AppResult<TestStruct> = json_body.deserialize();
        assert!(result.is_err());
        let error = result.unwrap_err().downcast::<HttpError>().unwrap();

        assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(
            error.to_string(),
            "invalid type: string \"invalid_int\", expected i32 at line 1 column 44"
        );
    }

    #[test]
    fn test_json_value_success() {
        let json_str = r#"{"field1": "value1", "field2": 42}"#.to_string();
        let json_body = JsonBody { json: json_str };

        let result = json_body.json_value();
        assert!(result.is_ok());

        let json_value = result.unwrap();
        let expected = json!({
            "field1": "value1",
            "field2": 42
        });

        let parsed_json: serde_json::Value = serde_json::from_str(&json_body.json).unwrap();
        assert_eq!(json_value, expected);
        assert_eq!(json_value, parsed_json);
    }

    #[test]
    fn test_json_value_failure() {
        let json_str = "not_a_json".to_string();
        let json_body = JsonBody { json: json_str };

        let result = json_body.json_value();
        assert!(result.is_err());
    }

    #[test]
    fn test_json_value_string_as_value() {
        let json_str = "\"just_a_string\"".to_string();
        let json_body = JsonBody {
            json: json_str.clone(),
        };

        let result = json_body.json_value();
        assert!(result.is_ok());

        let json_value = result.unwrap();

        let expected = serde_json::Value::String("just_a_string".to_string());

        assert_eq!(json_value, expected);
    }

    #[test]
    fn test_deserialize_to_map() {
        let json_str = r#"{"key1": "value1", "key2": "value2"}"#.to_string();
        let json_body = JsonBody { json: json_str };

        let result: AppResult<HashMap<String, String>> = json_body.deserialize();
        assert!(result.is_ok());

        let deserialized = result.unwrap();
        let mut expected = HashMap::new();
        expected.insert("key1".to_string(), "value1".to_string());
        expected.insert("key2".to_string(), "value2".to_string());

        assert_eq!(deserialized, expected);
    }
}
