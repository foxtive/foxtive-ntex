use crate::error::HttpError;
use foxtive::prelude::{AppMessage, AppResult};
use jsonwebtoken::{DecodingKey, TokenData, Validation, decode};
use tracing::{debug, error};
use ntex::http::Payload;
use ntex::http::header;
use ntex::web::{FromRequest, HttpRequest};
use serde::de::DeserializeOwned;

#[derive(Clone, Debug, PartialEq)]
pub struct JwtAuthToken {
    token: String,
}

impl JwtAuthToken {
    /// Get the raw JWT string
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Consume into the inner token string
    pub fn into_token(self) -> String {
        self.token
    }

    /// Decode and verify the JWT, returning the claims as type `T`.
    /// Secret and validation should be passed explicitly.
    pub fn decode<T: DeserializeOwned>(
        &self,
        secret: &str,
        validation: &Validation,
    ) -> AppResult<T> {
        match decode::<T>(
            &self.token,
            &DecodingKey::from_secret(secret.as_bytes()),
            validation,
        ) {
            Ok(TokenData { claims, .. }) => Ok(claims),
            Err(e) => {
                error!("JWT decode error: {e:?}");
                Err(
                    HttpError::AppMessage(AppMessage::WarningMessageString(e.to_string()))
                        .into_app_error(),
                )
            }
        }
    }

    /// Utility: Check if the token seems to be present and nonempty
    pub fn is_empty(&self) -> bool {
        self.token.is_empty()
    }
}

impl From<String> for JwtAuthToken {
    fn from(token: String) -> Self {
        JwtAuthToken { token }
    }
}

impl From<&str> for JwtAuthToken {
    fn from(token: &str) -> Self {
        JwtAuthToken {
            token: token.to_string(),
        }
    }
}

impl<Err> FromRequest<Err> for JwtAuthToken {
    type Error = HttpError;

    async fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Result<Self, Self::Error> {
        let token = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|val| {
                val.strip_prefix("Bearer ")
                    .or_else(|| val.strip_prefix("bearer "))
                    .map(|s| s.trim())
            })
            .ok_or_else(|| {
                HttpError::AppMessage(AppMessage::WarningMessageString(
                    "Missing or malformed Authorization header".to_string(),
                ))
                .into_app_error()
            })?;

        debug!("[jwt-auth-token] extracted {token}");

        Ok(JwtAuthToken {
            token: token.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foxtive::helpers::jwt::Algorithm;
    use jsonwebtoken::{EncodingKey, Header, encode};
    use ntex::http::{Payload, header};
    use ntex::web::test::TestRequest;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
    struct TestClaims {
        sub: String,
        company: String,
        exp: usize,
    }

    fn create_jwt(secret: &str, claims: &TestClaims) -> String {
        encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }

    fn jwt_req_with_header(token: &str) -> HttpRequest {
        TestRequest::default()
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .to_http_request()
    }

    #[tokio::test]
    async fn test_extractor_success() {
        let claims = TestClaims {
            sub: "me".to_string(),
            company: "Acme".to_string(),
            exp: 2000000000,
        };
        let secret = "my-secret";
        let jwt = create_jwt(secret, &claims);

        let req = jwt_req_with_header(&jwt);
        let mut payload = Payload::None;

        let token = <JwtAuthToken as FromRequest<HttpError>>::from_request(&req, &mut payload)
            .await
            .unwrap();
        assert_eq!(token.token(), jwt);

        // Show decode utility
        let validation = Validation::new(Algorithm::HS256);
        let decoded: TestClaims = token.decode(secret, &validation).unwrap();
        assert_eq!(decoded, claims);
    }

    #[tokio::test]
    async fn test_extractor_missing_header() {
        let req = TestRequest::default().to_http_request();
        let mut payload = Payload::None;
        let token =
            <JwtAuthToken as FromRequest<HttpError>>::from_request(&req, &mut payload).await;
        assert!(token.is_err());
    }

    #[tokio::test]
    async fn test_extractor_bad_format() {
        let req = TestRequest::default()
            .header(header::AUTHORIZATION, "BAD")
            .to_http_request();
        let mut payload = Payload::None;
        let token =
            <JwtAuthToken as FromRequest<HttpError>>::from_request(&req, &mut payload).await;
        assert!(token.is_err());
    }

    #[test]
    fn test_utilities() {
        let token = JwtAuthToken::from("abc.def.ghi");
        assert_eq!(token.token(), "abc.def.ghi");
        assert!(!token.is_empty());
        assert_eq!(token.clone().into_token(), "abc.def.ghi".to_string());
    }
}
