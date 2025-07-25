use crate::contracts::ResponseCodeContract;
use crate::enums::ResponseCode;
use crate::helpers::responder::Responder;
use crate::http::response::ext::AppMessageExt;
use crate::http::{HttpError, HttpResult, IntoHttpResult};
use foxtive::prelude::AppMessage;
use foxtive::results::AppResult;
use ntex::http::error::BlockingError;

impl AppMessageExt for AppMessage {
    fn respond(self) -> HttpResult {
        let status = self.status_code();
        match status.is_success() {
            true => Ok(Responder::message(
                &self.message(),
                ResponseCode::from_status(self.status_code()),
            )),
            false => Err(HttpError::AppMessage(self)),
        }
    }
}

impl AppMessageExt for AppResult<AppMessage> {
    fn respond(self) -> HttpResult {
        match self {
            Ok(msg) => msg.respond(),
            Err(err) => Err(HttpError::AppError(err)),
        }
    }
}

impl AppMessageExt for Result<AppMessage, AppMessage> {
    fn respond(self) -> HttpResult {
        match self {
            Ok(msg) => msg.respond(),
            Err(err) => err.respond(),
        }
    }
}

impl AppMessageExt for Result<AppMessage, BlockingError<AppMessage>> {
    fn respond(self) -> HttpResult {
        match self {
            Ok(msg) => msg.respond(),
            Err(err) => match err {
                BlockingError::Error(msg) => msg.respond(),
                BlockingError::Canceled => AppMessage::InternalServerError.into_http_result(),
            },
        }
    }
}

impl AppMessageExt for Result<AppMessage, BlockingError<foxtive::Error>> {
    fn respond(self) -> HttpResult {
        match self {
            Ok(msg) => msg.respond(),
            Err(err) => match err {
                BlockingError::Error(err) => Err(HttpError::AppError(err)),
                BlockingError::Canceled => AppMessage::InternalServerError.into_http_result(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::http::response::ext::AppMessageExt;
    use foxtive::Error;
    use foxtive::prelude::AppMessage;
    use ntex::http::StatusCode;
    use ntex::http::error::BlockingError;
    use ntex::web::WebResponseError;

    #[test]
    fn test_app_message_respond_success() {
        let msg = AppMessage::SuccessMessage("Yes");
        let result = msg.respond();
        assert!(result.is_ok());
    }

    #[test]
    fn test_app_message_respond_error() {
        let msg = AppMessage::InternalServerError;
        let result = msg.respond();
        assert!(result.is_err());
    }

    #[test]
    fn test_app_message_result_respond() {
        let msg: Result<AppMessage, Error> = Ok(AppMessage::InternalServerError);
        let result = msg.respond();
        assert!(result.is_err());
    }

    #[test]
    fn test_app_message_result_error_respond() {
        let msg = Err(AppMessage::InternalServerError);
        let result = msg.respond();
        assert!(result.is_err());
    }

    #[test]
    fn test_app_message_result_blocking_error_respond() {
        let msg = Err(BlockingError::Error(foxtive::Error::from(
            AppMessage::InternalServerError,
        )));
        let result = msg.respond();
        assert!(result.is_err());
    }

    #[test]
    fn test_app_message_result_blocking_error_canceled_respond() {
        let msg: Result<AppMessage, BlockingError<AppMessage>> = Err(BlockingError::Canceled);
        let result = msg.respond();
        assert!(result.is_err());

        let msg: Result<AppMessage, BlockingError<AppMessage>> =
            Ok(AppMessage::SuccessMessage("Yep"));
        let status = msg.respond().unwrap().status();
        assert_eq!(status, StatusCode::OK);

        let msg: Result<AppMessage, BlockingError<AppMessage>> =
            Err(BlockingError::Error(AppMessage::WarningMessage("Hmm")));
        let status = msg.respond().unwrap_err().status_code();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_app_message_result_blocking_error_canceled_respond_with_error() {
        let msg: Result<AppMessage, BlockingError<Error>> = Err(BlockingError::Canceled);
        let result = msg.respond();
        assert!(result.is_err());
    }
}
