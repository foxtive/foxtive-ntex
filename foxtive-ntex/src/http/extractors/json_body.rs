use crate::error::HttpError;
use crate::{FOXTIVE_NTEX, FoxtiveNtexExt};
use foxtive::prelude::AppMessage;
use ntex::http::Payload;
use ntex::util::BytesMut;
use ntex::web::{FromRequest, HttpRequest};
use serde::de::DeserializeOwned;
use std::ops;

pub struct JsonBody<T: DeserializeOwned>(T);

impl<T: DeserializeOwned> JsonBody<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: DeserializeOwned> ops::Deref for JsonBody<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: DeserializeOwned> ops::DerefMut for JsonBody<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: DeserializeOwned, Err> FromRequest<Err> for JsonBody<T> {
    type Error = HttpError;

    async fn from_request(
        _req: &HttpRequest,
        payload: &mut Payload,
    ) -> Result<JsonBody<T>, Self::Error> {
        let max_size = FOXTIVE_NTEX.app().body_config.json_limit;
        let mut bytes = BytesMut::new();
        let mut total_size = 0;

        while let Some(item) = ntex::util::stream_recv(payload).await {
            let chunk = item?;
            total_size += chunk.len();

            if total_size > max_size {
                return Err(HttpError::AppMessage(AppMessage::invalid(
                    "JSON body exceeded maximum size",
                )));
            }

            bytes.extend_from_slice(&chunk);
        }

        let json = String::from_utf8(bytes.to_vec())?;
        let value = serde_json::from_str::<T>(&json)
            .map_err(|e| HttpError::AppMessage(AppMessage::invalid(e.to_string())))?;

        Ok(JsonBody(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestStruct {
        field1: String,
        field2: i32,
    }

    #[test]
    fn test_deref() {
        let value = TestStruct {
            field1: "value1".to_string(),
            field2: 42,
        };
        let json_body = JsonBody(value.clone());

        assert_eq!(*json_body, value);
        assert_eq!(json_body.field1, "value1");
        assert_eq!(json_body.field2, 42);
    }

    #[test]
    fn test_deref_mut() {
        let value = TestStruct {
            field1: "value1".to_string(),
            field2: 42,
        };
        let mut json_body = JsonBody(value);

        json_body.field2 = 100;
        assert_eq!(json_body.field2, 100);
    }

    #[test]
    fn test_into_inner() {
        let value = TestStruct {
            field1: "value1".to_string(),
            field2: 42,
        };
        let json_body = JsonBody(value.clone());

        let extracted = json_body.into_inner();
        assert_eq!(extracted, value);
    }
}
