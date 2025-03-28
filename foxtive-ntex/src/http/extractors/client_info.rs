use crate::helpers::request::RequestHelper;
use ntex::http::Payload;
use ntex::web::{FromRequest, HttpRequest};
use crate::error::HttpError;

pub struct ClientInfo {
    pub ip: Option<String>,
    pub ua: Option<String>,
}

impl ClientInfo {
    pub fn into_parts(self) -> (Option<String>, Option<String>) {
        (self.ip, self.ua)
    }
}

impl<Err> FromRequest<Err> for ClientInfo {
    type Error = HttpError;

    async fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Result<Self, Self::Error> {
        Ok(ClientInfo {
            ip: req.ip(),
            ua: req.user_agent(),
        })
    }
}
