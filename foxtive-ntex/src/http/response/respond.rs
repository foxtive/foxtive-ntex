use crate::contracts::ResponseCodeContract;
use crate::enums::ResponseCode;
use crate::error::HttpError;
use crate::http::responder::Responder;
use crate::http::response::ext::{ResponderExt, ResultResponseExt};
use crate::http::{HttpResult, IntoAppResult};
use foxtive::prelude::{AppMessage, AppResult};
use foxtive::{Error, internal_server_error};
use ntex::http::error::BlockingError;
use ntex::web::HttpResponse;
use serde::Serialize;

impl<T> ResponderExt for AppResult<T>
where
    T: Sized + Serialize,
{
    fn respond_code<C: ResponseCodeContract>(self, msg: &str, code: C) -> HttpResult {
        self.send_result_msg(code, msg)
    }

    fn respond_msg(self, msg: &str) -> HttpResult {
        self.send_result_msg(ResponseCode::Ok, msg)
    }

    fn respond(self) -> HttpResult {
        self.send_result(ResponseCode::Ok)
    }

    fn respond_undecorated(self) -> HttpResult {
        self.respond_undecorated_code(ResponseCode::Ok)
    }

    fn respond_undecorated_code(self, code: impl ResponseCodeContract) -> HttpResult {
        match self {
            Ok(data) => Ok(HttpResponse::build(code.status()).json(&data)),
            Err(err) => Err(HttpError::AppError(err)),
        }
    }
}

impl<T> ResponderExt for Result<T, BlockingError<AppMessage>>
where
    T: Serialize + Sized,
{
    fn respond_code<C: ResponseCodeContract>(self, msg: &str, code: C) -> HttpResult {
        <Result<T, foxtive::Error> as ResultResponseExt>::send_result_msg(
            self.into_app_result(),
            code,
            msg,
        )
    }

    fn respond_msg(self, msg: &str) -> HttpResult {
        <Result<T, foxtive::Error> as ResultResponseExt>::send_result_msg(
            self.into_app_result(),
            ResponseCode::Ok,
            msg,
        )
    }

    fn respond(self) -> HttpResult {
        <Result<T, foxtive::Error> as ResultResponseExt>::send_result(
            self.into_app_result(),
            ResponseCode::Ok,
        )
    }

    fn respond_undecorated(self) -> HttpResult {
        self.respond_undecorated_code(ResponseCode::Ok)
    }

    fn respond_undecorated_code(self, code: impl ResponseCodeContract) -> HttpResult {
        match self {
            Ok(data) => Ok(HttpResponse::build(code.status()).json(&data)),
            Err(err) => Err(HttpError::AppError(match err {
                BlockingError::Error(msg) => msg.into_anyhow(),
                BlockingError::Canceled => internal_server_error!("Internal Server Error"),
            })),
        }
    }
}

impl<T> ResponderExt for Result<T, BlockingError<Error>>
where
    T: Serialize + Sized,
{
    fn respond_code<C: ResponseCodeContract>(self, msg: &str, code: C) -> HttpResult {
        Ok(Responder::send_msg(self?, code, msg))
    }

    fn respond_msg(self, msg: &str) -> HttpResult {
        Ok(Responder::send_msg(self?, ResponseCode::Ok, msg))
    }

    fn respond(self) -> HttpResult {
        Ok(Responder::send(self?, ResponseCode::Ok))
    }

    fn respond_undecorated(self) -> HttpResult {
        self.respond_undecorated_code(ResponseCode::Ok)
    }

    fn respond_undecorated_code(self, code: impl ResponseCodeContract) -> HttpResult {
        match self {
            Ok(data) => Ok(HttpResponse::build(code.status()).json(&data)),
            Err(err) => Err(HttpError::AppError(match err {
                BlockingError::Error(err) => err,
                BlockingError::Canceled => internal_server_error!("Internal Server Error"),
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foxtive::helpers::json::JsonEmpty;
    use foxtive::invalid;
    use ntex::http::StatusCode;
    use ntex::http::error::BlockingError;
    use ntex::web::WebResponseError;
    use serde_json::json;

    #[test]
    fn test_respond() {
        let data = json!({"key": "value"});
        let app_result: AppResult<_> = Ok(data.clone());

        let result = app_result.respond();
        match result {
            Ok(response) => {
                assert_eq!(response.status(), StatusCode::OK);
            }
            Err(e) => panic!("Expected Ok, but got Err: {e:?}"),
        }
    }

    #[test]
    fn test_respond_msg() {
        let data = json!({"key": "value"});
        let app_result: AppResult<_> = Ok(data.clone());

        let result = app_result.respond_msg("Success");
        match result {
            Ok(response) => {
                assert_eq!(response.status(), StatusCode::OK);
            }
            Err(e) => panic!("Expected Ok, but got Err: {e:?}"),
        }
    }

    #[test]
    fn test_respond_error() {
        let error = BlockingError::Canceled;
        let result: Result<JsonEmpty, BlockingError<AppMessage>> = Err(error);

        let result = result.respond();
        match result {
            Ok(_) => panic!("Expected Err, but got OK"),
            Err(e) => {
                assert_eq!(e.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    #[test]
    fn test_respond_msg_error() {
        let error = BlockingError::Error(AppMessage::invalid("invalid"));
        let result: Result<JsonEmpty, BlockingError<AppMessage>> = Err(error);

        let result = result.respond_msg("Error occurred");
        match result {
            Ok(_) => panic!("Expected Err, but got Ok"),
            Err(e) => {
                assert_eq!(e.status_code(), StatusCode::BAD_REQUEST);
            }
        }
    }

    #[test]
    fn test_app_result_respond_undecorated() {
        let ok: AppResult<_> = Ok(json!({"key": "value"}));
        assert_eq!(ok.respond_undecorated().unwrap().status(), StatusCode::OK);

        let err: AppResult<JsonEmpty> = Err(invalid!("error"));
        assert_eq!(
            err.respond_undecorated().unwrap_err().status_code(),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn test_app_result_respond_undecorated_code() {
        let ok: AppResult<_> = Ok(json!({"key": "value"}));
        assert_eq!(
            ok.respond_undecorated_code(ResponseCode::Created)
                .unwrap()
                .status(),
            StatusCode::CREATED
        );

        let err: AppResult<JsonEmpty> = Err(invalid!("error"));
        assert_eq!(
            err.respond_undecorated_code(ResponseCode::Created)
                .unwrap_err()
                .status_code(),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn test_blocking_app_message_respond_undecorated() {
        let ok: Result<_, BlockingError<AppMessage>> = Ok(json!({"key": "value"}));
        assert_eq!(ok.respond_undecorated().unwrap().status(), StatusCode::OK);

        let err: Result<JsonEmpty, BlockingError<AppMessage>> =
            Err(BlockingError::Error(AppMessage::invalid("error")));
        assert_eq!(
            err.respond_undecorated().unwrap_err().status_code(),
            StatusCode::BAD_REQUEST
        );

        let canceled: Result<JsonEmpty, BlockingError<AppMessage>> = Err(BlockingError::Canceled);
        assert_eq!(
            canceled.respond_undecorated().unwrap_err().status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_blocking_app_message_respond_undecorated_code() {
        let ok: Result<_, BlockingError<AppMessage>> = Ok(json!({"key": "value"}));
        assert_eq!(
            ok.respond_undecorated_code(ResponseCode::Accepted)
                .unwrap()
                .status(),
            StatusCode::ACCEPTED
        );

        let err: Result<JsonEmpty, BlockingError<AppMessage>> = Err(BlockingError::Canceled);
        assert_eq!(
            err.respond_undecorated_code(ResponseCode::Created)
                .unwrap_err()
                .status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_blocking_error_respond_undecorated() {
        let data = json!({"key": "value"});
        let result: Result<_, BlockingError<Error>> = Ok(data.clone());

        let response = result.respond_undecorated();
        match response {
            Ok(http_response) => {
                assert_eq!(http_response.status(), StatusCode::OK);
            }
            Err(e) => panic!("Expected Ok, but got Err: {e:?}"),
        }
    }

    #[test]
    fn test_blocking_error_respond_undecorated_code() {
        let data = json!({"key": "value"});
        let result: Result<_, BlockingError<Error>> = Ok(data.clone());

        let response = result.respond_undecorated_code(ResponseCode::NoContent);
        match response {
            Ok(http_response) => {
                assert_eq!(http_response.status(), StatusCode::NO_CONTENT);
            }
            Err(e) => panic!("Expected Ok, but got Err: {e:?}"),
        }
    }

    #[test]
    fn test_blocking_error_respond_undecorated_error() {
        let error = BlockingError::Error(invalid!("foxtive error"));
        let result: Result<JsonEmpty, BlockingError<Error>> = Err(error);

        let response = result.respond_undecorated();
        match response {
            Ok(_) => panic!("Expected Err, but got Ok"),
            Err(e) => {
                assert_eq!(e.status_code(), StatusCode::BAD_REQUEST);
            }
        }
    }

    #[test]
    fn test_blocking_error_respond_undecorated_code_error() {
        let error = BlockingError::Error(invalid!("foxtive error"));
        let result: Result<JsonEmpty, BlockingError<Error>> = Err(error);

        let response = result.respond_undecorated_code(ResponseCode::Created);
        match response {
            Ok(_) => panic!("Expected Err, but got Ok"),
            Err(e) => {
                assert_eq!(e.status_code(), StatusCode::BAD_REQUEST);
            }
        }
    }

    #[test]
    fn test_blocking_error_respond_undecorated_canceled() {
        let error = BlockingError::Canceled;
        let result: Result<JsonEmpty, BlockingError<Error>> = Err(error);

        let response = result.respond_undecorated();
        match response {
            Ok(_) => panic!("Expected Err, but got Ok"),
            Err(e) => {
                assert_eq!(e.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    #[test]
    fn test_blocking_error_respond_undecorated_code_canceled() {
        let error = BlockingError::Canceled;
        let result: Result<JsonEmpty, BlockingError<Error>> = Err(error);

        let response = result.respond_undecorated_code(ResponseCode::Accepted);
        match response {
            Ok(_) => panic!("Expected Err, but got Ok"),
            Err(e) => {
                assert_eq!(e.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }
}
