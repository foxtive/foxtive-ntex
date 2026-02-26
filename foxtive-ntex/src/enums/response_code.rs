use crate::contracts::ResponseCodeContract;
use ntex::http::StatusCode;
use std::borrow::Cow;
use tracing::error;

#[derive(Clone)]
pub enum ResponseCode {
    Ok,
    Created,
    Accepted,
    NoContent,
    BadRequest,
    Unauthorized,
    PaymentRequired,
    Forbidden,
    NotFound,
    Conflict,
    InternalServerError,
    ServiceUnavailable,
    NotImplemented,
    Unknown(StatusCode),
}

impl ResponseCodeContract for ResponseCode {
    fn code(&self) -> Cow<'static, str> {
        match self {
            ResponseCode::Ok => Cow::Borrowed("000"),
            ResponseCode::Created => Cow::Borrowed("001"),
            ResponseCode::Accepted => Cow::Borrowed("002"),
            ResponseCode::NoContent => Cow::Borrowed("003"),
            ResponseCode::BadRequest => Cow::Borrowed("004"),
            ResponseCode::Unauthorized => Cow::Borrowed("005"),
            ResponseCode::PaymentRequired => Cow::Borrowed("006"),
            ResponseCode::Forbidden => Cow::Borrowed("007"),
            ResponseCode::NotFound => Cow::Borrowed("008"),
            ResponseCode::Conflict => Cow::Borrowed("009"),
            ResponseCode::InternalServerError => Cow::Borrowed("010"),
            ResponseCode::ServiceUnavailable => Cow::Borrowed("011"),
            ResponseCode::NotImplemented => Cow::Borrowed("012"),
            ResponseCode::Unknown(status) => Cow::Owned(status.as_u16().to_string()),
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            ResponseCode::Ok => StatusCode::OK,
            ResponseCode::Created => StatusCode::CREATED,
            ResponseCode::Accepted => StatusCode::ACCEPTED,
            ResponseCode::NoContent => StatusCode::NO_CONTENT,
            ResponseCode::BadRequest => StatusCode::BAD_REQUEST,
            ResponseCode::Unauthorized => StatusCode::UNAUTHORIZED,
            ResponseCode::PaymentRequired => StatusCode::PAYMENT_REQUIRED,
            ResponseCode::Forbidden => StatusCode::FORBIDDEN,
            ResponseCode::NotFound => StatusCode::NOT_FOUND,
            ResponseCode::Conflict => StatusCode::CONFLICT,
            ResponseCode::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ResponseCode::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            ResponseCode::NotImplemented => StatusCode::NOT_IMPLEMENTED,
            ResponseCode::Unknown(status) => *status,
        }
    }

    fn from_code(code: &str) -> Self {
        match code {
            "000" => ResponseCode::Ok,
            "001" => ResponseCode::Created,
            "002" => ResponseCode::Accepted,
            "003" => ResponseCode::NoContent,
            "004" => ResponseCode::BadRequest,
            "005" => ResponseCode::Unauthorized,
            "006" => ResponseCode::PaymentRequired,
            "007" => ResponseCode::Forbidden,
            "008" => ResponseCode::NotFound,
            "009" => ResponseCode::Conflict,
            "010" => ResponseCode::InternalServerError,
            "011" => ResponseCode::ServiceUnavailable,
            "012" => ResponseCode::NotImplemented,
            _ => {
                error!("Unknown response code: {code}");
                ResponseCode::InternalServerError
            }
        }
    }

    fn from_status(status: StatusCode) -> Self {
        match status {
            StatusCode::OK => ResponseCode::Ok,
            StatusCode::CREATED => ResponseCode::Created,
            StatusCode::ACCEPTED => ResponseCode::Accepted,
            StatusCode::NO_CONTENT => ResponseCode::NoContent,
            StatusCode::BAD_REQUEST => ResponseCode::BadRequest,
            StatusCode::UNAUTHORIZED => ResponseCode::Unauthorized,
            StatusCode::PAYMENT_REQUIRED => ResponseCode::PaymentRequired,
            StatusCode::FORBIDDEN => ResponseCode::Forbidden,
            StatusCode::NOT_FOUND => ResponseCode::NotFound,
            StatusCode::CONFLICT => ResponseCode::Conflict,
            StatusCode::INTERNAL_SERVER_ERROR => ResponseCode::InternalServerError,
            StatusCode::SERVICE_UNAVAILABLE => ResponseCode::ServiceUnavailable,
            StatusCode::NOT_IMPLEMENTED => ResponseCode::NotImplemented,
            _ => ResponseCode::Unknown(status),
        }
    }
}
