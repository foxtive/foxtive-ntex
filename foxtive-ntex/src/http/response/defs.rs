use crate::contracts::ResponseCodeContract;
use crate::http::HttpResult;
use ntex::web::HttpResponse;

pub trait ResultResponse {
    fn send_result<C: ResponseCodeContract>(self, code: C) -> HttpResult;

    fn send_result_msg<C: ResponseCodeContract>(self, code: C, msg: &str) -> HttpResult;
}

pub trait Responder {
    fn respond_code<C: ResponseCodeContract>(self, msg: &str, code: C) -> HttpResult;

    fn respond_msg(self, suc: &str) -> HttpResult;

    fn respond(self) -> HttpResult;
}

pub trait StructResponse: Sized {
    fn into_response(self) -> HttpResponse;

    fn respond_code<C: ResponseCodeContract>(self, code: C, msg: &str) -> HttpResult;

    fn respond_msg(self, msg: &str) -> HttpResult;

    fn respond(self) -> HttpResult;
}

pub trait OptionResultResponse<T> {
    fn is_empty(&self) -> bool;

    fn is_error(&self) -> bool;

    fn is_error_or_empty(&self) -> bool;

    fn send_response<C: ResponseCodeContract>(self, code: C, msg: &str) -> HttpResult;
}
