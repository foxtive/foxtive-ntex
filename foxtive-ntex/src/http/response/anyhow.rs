use foxtive::prelude::AppMessage;
use ntex::http::body::Body;
use ntex::http::{header, StatusCode};
use ntex::web::{HttpRequest, HttpResponse, WebResponseError};
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub struct ResponseError {
    pub error: foxtive::Error,
}

impl Display for ResponseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl WebResponseError for ResponseError {
    fn status_code(&self) -> StatusCode {
        match self.error.downcast_ref::<AppMessage>() {
            Some(msg) => msg.status_code(),
            None => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        let respond = |msg| {
            let mut resp = HttpResponse::new(self.status_code());
            resp.headers_mut().insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("application/json"),
            );

            resp.set_body(Body::from(msg))
        };

        match self.error.downcast_ref::<AppMessage>() {
            Some(msg) => respond(msg.message()),
            None => respond("Internal Server Error".to_string()),
        }
    }
}
