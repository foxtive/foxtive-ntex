use crate::contracts::ResponseCodeContract;
use crate::enums::ResponseCode;
use crate::helpers::responder::Responder;
use crate::http::HttpResult;
use crate::http::response::ext::StructResponseExt;
use ntex::web::HttpResponse;
use serde::Serialize;

impl<T: Serialize> StructResponseExt for T {
    fn into_response(self) -> HttpResponse {
        Responder::send(self, ResponseCode::Ok)
    }

    fn respond_code<C: ResponseCodeContract>(self, code: C, msg: &str) -> HttpResult {
        Ok(Responder::send_msg(self, code, msg))
    }

    fn respond_msg(self, msg: &str) -> HttpResult {
        Ok(Responder::send_msg(self, ResponseCode::Ok, msg))
    }

    fn respond(self) -> HttpResult {
        Ok(Responder::send(self, ResponseCode::Ok))
    }
}
