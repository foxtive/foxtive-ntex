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
    MethodNotAllowed,
    RequestTimeout,
    Conflict,
    Gone,
    UnsupportedMediaType,
    UnprocessableEntity,
    TooManyRequests,
    PayloadTooLarge,
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    MovedPermanently,
    NotModified,
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
            ResponseCode::MethodNotAllowed => Cow::Borrowed("009"),
            ResponseCode::RequestTimeout => Cow::Borrowed("010"),
            ResponseCode::Conflict => Cow::Borrowed("011"),
            ResponseCode::Gone => Cow::Borrowed("012"),
            ResponseCode::UnsupportedMediaType => Cow::Borrowed("013"),
            ResponseCode::UnprocessableEntity => Cow::Borrowed("014"),
            ResponseCode::TooManyRequests => Cow::Borrowed("015"),
            ResponseCode::PayloadTooLarge => Cow::Borrowed("016"),
            ResponseCode::InternalServerError => Cow::Borrowed("017"),
            ResponseCode::NotImplemented => Cow::Borrowed("018"),
            ResponseCode::BadGateway => Cow::Borrowed("019"),
            ResponseCode::ServiceUnavailable => Cow::Borrowed("020"),
            ResponseCode::GatewayTimeout => Cow::Borrowed("021"),
            ResponseCode::MovedPermanently => Cow::Borrowed("022"),
            ResponseCode::NotModified => Cow::Borrowed("023"),
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
            ResponseCode::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            ResponseCode::RequestTimeout => StatusCode::REQUEST_TIMEOUT,
            ResponseCode::Conflict => StatusCode::CONFLICT,
            ResponseCode::Gone => StatusCode::GONE,
            ResponseCode::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ResponseCode::UnprocessableEntity => StatusCode::UNPROCESSABLE_ENTITY,
            ResponseCode::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            ResponseCode::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            ResponseCode::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ResponseCode::NotImplemented => StatusCode::NOT_IMPLEMENTED,
            ResponseCode::BadGateway => StatusCode::BAD_GATEWAY,
            ResponseCode::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            ResponseCode::GatewayTimeout => StatusCode::GATEWAY_TIMEOUT,
            ResponseCode::MovedPermanently => StatusCode::MOVED_PERMANENTLY,
            ResponseCode::NotModified => StatusCode::NOT_MODIFIED,
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
            "009" => ResponseCode::MethodNotAllowed,
            "010" => ResponseCode::RequestTimeout,
            "011" => ResponseCode::Conflict,
            "012" => ResponseCode::Gone,
            "013" => ResponseCode::UnsupportedMediaType,
            "014" => ResponseCode::UnprocessableEntity,
            "015" => ResponseCode::TooManyRequests,
            "016" => ResponseCode::PayloadTooLarge,
            "017" => ResponseCode::InternalServerError,
            "018" => ResponseCode::NotImplemented,
            "019" => ResponseCode::BadGateway,
            "020" => ResponseCode::ServiceUnavailable,
            "021" => ResponseCode::GatewayTimeout,
            "022" => ResponseCode::MovedPermanently,
            "023" => ResponseCode::NotModified,
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
            StatusCode::METHOD_NOT_ALLOWED => ResponseCode::MethodNotAllowed,
            StatusCode::REQUEST_TIMEOUT => ResponseCode::RequestTimeout,
            StatusCode::CONFLICT => ResponseCode::Conflict,
            StatusCode::GONE => ResponseCode::Gone,
            StatusCode::UNSUPPORTED_MEDIA_TYPE => ResponseCode::UnsupportedMediaType,
            StatusCode::UNPROCESSABLE_ENTITY => ResponseCode::UnprocessableEntity,
            StatusCode::TOO_MANY_REQUESTS => ResponseCode::TooManyRequests,
            StatusCode::PAYLOAD_TOO_LARGE => ResponseCode::PayloadTooLarge,
            StatusCode::INTERNAL_SERVER_ERROR => ResponseCode::InternalServerError,
            StatusCode::NOT_IMPLEMENTED => ResponseCode::NotImplemented,
            StatusCode::BAD_GATEWAY => ResponseCode::BadGateway,
            StatusCode::SERVICE_UNAVAILABLE => ResponseCode::ServiceUnavailable,
            StatusCode::GATEWAY_TIMEOUT => ResponseCode::GatewayTimeout,
            StatusCode::MOVED_PERMANENTLY => ResponseCode::MovedPermanently,
            StatusCode::NOT_MODIFIED => ResponseCode::NotModified,
            _ => ResponseCode::Unknown(status),
        }
    }
}
