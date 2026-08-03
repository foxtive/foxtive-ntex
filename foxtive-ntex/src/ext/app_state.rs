use crate::setup::state::AppState;
use foxtive::App;
use ntex::web::HttpRequest;
use std::sync::Arc;

/// Get a reference to the foxtive `App` from an ntex request.
///
/// Requires that `Arc<App>` was registered as ntex state via
/// `.state(app.clone())`.
pub fn app_from_req(req: &HttpRequest) -> Option<&Arc<App>> {
    req.app_state::<Arc<App>>()
}

/// Get a reference to the ntex `AppState` from an ntex request.
pub fn app_state_from_req(req: &HttpRequest) -> Option<&AppState> {
    req.app_state::<AppState>()
}

/// Get a reference to a custom service registered in the foxtive `App` DI container.
///
/// Returns `Arc<T>` — a cheap clonable handle to the service.
pub fn fox_service<T: Send + Sync + 'static>(req: &HttpRequest) -> Option<Arc<T>> {
    app_from_req(req).and_then(|app| app.get::<T>())
}
