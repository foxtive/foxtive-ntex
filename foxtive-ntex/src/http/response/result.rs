use crate::contracts::ResponseCodeContract;
use crate::error::HttpError;
use crate::http::HttpResult;
use crate::http::responder::Responder;
use crate::http::response::ext::{OptionResultResponseExt, ResultResponseExt};
use foxtive::prelude::{AppMessage, AppResult};
use serde::Serialize;

impl<T: Serialize> ResultResponseExt for AppResult<T> {
    fn send_result<C: ResponseCodeContract>(self, code: C) -> HttpResult {
        match self {
            Ok(data) => Ok(Responder::send(data, code)),
            Err(err) => Err(HttpError::AppMessage(err)),
        }
    }

    fn send_result_msg<C: ResponseCodeContract>(self, code: C, msg: &str) -> HttpResult {
        match self {
            Ok(data) => Ok(Responder::send_msg(data, code, msg)),
            Err(err) => Err(HttpError::AppMessage(err)),
        }
    }
}

impl<T: Serialize> OptionResultResponseExt<T> for AppResult<T> {
    fn is_empty(&self) -> bool {
        match self {
            Ok(_) => false,
            Err(e) => matches!(e, AppMessage::NotFound(..)),
        }
    }

    fn is_error(&self) -> bool {
        self.as_ref().is_err()
    }

    fn is_error_or_empty(&self) -> bool {
        self.is_error() || self.is_empty()
    }

    fn send_response<C: ResponseCodeContract>(self, code: C, msg: &str) -> HttpResult {
        Ok(Responder::send_msg(
            self.map_err(HttpError::AppMessage)?,
            code,
            msg,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::ResponseCode;
    use foxtive::prelude::AppResult;
    use ntex::http::StatusCode;
    use ntex::web::WebResponseError;
    use serde_json::json;

    #[test]
    fn test_result_response_send_result_err() {
        let err = AppMessage::not_found("app".to_string());
        let result: AppResult<serde_json::Value> = Err(err);

        let response = result.send_result_msg(ResponseCode::NotFound, "fail");
        match response {
            Ok(_) => panic!("Expected Err, but got Ok"),
            Err(e) => {
                // Verify that the error was correctly propagated
                assert_eq!(e.status_code(), StatusCode::NOT_FOUND);
            }
        }
    }

    #[test]
    fn test_option_result_response_is_empty() {
        let result: AppResult<()> = AppMessage::not_found("".to_string()).into_result();

        assert!(result.is_empty());
    }

    #[test]
    fn test_option_result_response_is_error_or_empty() {
        let result_empty: AppResult<()> = AppMessage::not_found("".to_string()).into_result();
        let result_error: AppResult<()> = AppMessage::not_found("".to_string()).into_result();
        let result_ok: AppResult<()> = Ok(());

        assert!(result_empty.is_error_or_empty());
        assert!(result_error.is_error_or_empty());
        assert!(!result_ok.is_error_or_empty());
    }

    #[test]
    fn test_option_result_response_send_response_error_or_empty() {
        let result: AppResult<()> =
            AppMessage::internal_server_error("Internal Server Error").into_result();

        let response = result.send_response(ResponseCode::Ok, "fail");
        match response {
            Err(e) => {
                assert_eq!(
                    e.status_code(),
                    AppMessage::internal_server_error("Internal Server Error").status_code()
                );
            }
            Ok(_) => panic!("Expected Err, but got Ok"),
        }
    }

    #[test]
    fn test_option_result_response_send_response_ok() {
        let data = json!({"key": "value"});
        let result: AppResult<serde_json::Value> = Ok(data.clone());

        let response = result.send_response(ResponseCode::Ok, "suc");
        match response {
            Ok(responder) => {
                assert_eq!(responder.status(), StatusCode::OK);
            }
            Err(e) => panic!("Expected Ok, but got Err: {e:?}"),
        }
    }
}
