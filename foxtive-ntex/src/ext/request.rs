use foxtive::prelude::{AppMessage, AppResult};
use ntex::http::header;
use ntex::util::Bytes;
use ntex::web::HttpRequest;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value, json};
use std::sync::Arc;
use tracing::debug;

use crate::http::extractors::ClientInfo;
use foxtive::App;
use foxtive::tokio::Tokio;

#[allow(dead_code)]
pub trait RequestExt {
    /// Get an owned `Arc<App>` from the request.
    ///
    /// Requires that `Arc<App>` was registered as ntex state via `.state(app.clone())`.
    fn app(&self) -> AppResult<Arc<App>>;

    /// Get a reference to the [`Tokio`] runtime from the application.
    ///
    /// Shorthand for `req.app()?.tokio()`.
    ///
    /// Requires that `Arc<App>` was registered as ntex state via `.state(app.clone())`.
    ///
    /// [`Tokio`]: foxtive::tokio::Tokio
    fn tokio(&self) -> AppResult<&Tokio>;

    /// Get a service from the DI container.
    ///
    /// Shorthand for `req.app()?.require::<T>()`.
    fn service<T: Send + Sync + 'static>(&self) -> AppResult<Arc<T>>;

    #[cfg(feature = "database")]
    fn db_pool(&self) -> foxtive::prelude::AppResult<&foxtive::database::DBPool>;

    fn client_info(&self) -> ClientInfo;

    fn get_headers(&self) -> Map<String, Value>;

    fn json<T: DeserializeOwned>(bytes: Bytes) -> AppResult<T>;

    fn ip(&self) -> Option<String>;

    fn user_agent(&self) -> Option<String>;
}

impl RequestExt for HttpRequest {
    fn app(&self) -> AppResult<Arc<App>> {
        use crate::ext::app_state::app_from_req;
        app_from_req(self).cloned().ok_or_else(|| {
            AppMessage::internal_server_error("Arc<App> not registered as ntex app data")
        })
    }

    fn tokio(&self) -> AppResult<&Tokio> {
        use crate::ext::app_state::app_from_req;
        app_from_req(self).map(|app| app.tokio()).ok_or_else(|| {
            AppMessage::internal_server_error("Arc<App> not registered as ntex app data")
        })
    }

    fn service<T: Send + Sync + 'static>(&self) -> AppResult<Arc<T>> {
        self.app()?.require::<T>()
    }

    #[cfg(feature = "database")]
    fn db_pool(&self) -> foxtive::prelude::AppResult<&foxtive::database::DBPool> {
        use crate::ext::app_state::app_from_req;
        app_from_req(self)
            .ok_or_else(|| {
                AppMessage::internal_server_error("Arc<App> not registered as ntex app data")
            })?
            .db()
    }

    fn client_info(&self) -> ClientInfo {
        ClientInfo {
            ip: self.ip(),
            ua: self.user_agent(),
        }
    }

    fn get_headers(&self) -> Map<String, Value> {
        let mut headers_json_object = Map::new();

        for (name, value) in self.headers().iter() {
            headers_json_object.insert(name.to_string(), json!(value.to_str().unwrap()));
        }

        headers_json_object
    }

    fn json<T: DeserializeOwned>(bytes: Bytes) -> AppResult<T> {
        let raw = String::from_utf8(bytes.to_vec())?;
        debug!("[json-body]: {raw}");
        Ok(serde_json::from_str::<T>(&raw)?)
    }

    fn ip(&self) -> Option<String> {
        self.connection_info()
            .remote()
            .map(|v| v.to_string())
            .or_else(|| self.peer_addr().map(|s| s.to_string()))
    }

    fn user_agent(&self) -> Option<String> {
        self.headers()
            .get(header::USER_AGENT)
            .map(|ua| ua.to_str().unwrap().to_string())
    }
}
