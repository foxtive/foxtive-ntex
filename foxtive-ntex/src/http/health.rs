//! Built-in health check HTTP endpoint.
//!
//! When enabled via [`ServerBuilder::health_check_path`](crate::http::server::ServerBuilder::health_check_path),
//! this handler is registered at the configured path and returns the aggregated
//! health report from the foxtive [`App`](foxtive::App).

use foxtive::health::{HealthReport, HealthStatus};
use foxtive::App;
use ntex::web::{HttpRequest, HttpResponse};
use std::sync::Arc;

/// ntex handler that returns the application health report as JSON.
///
/// Returns HTTP 200 when all checks are healthy, or HTTP 503 when any
/// check is degraded or unhealthy.
pub async fn health_handler(req: HttpRequest) -> HttpResponse {
    let app = match req.app_state::<Arc<App>>() {
        Some(app) => app,
        None => {
            let body = serde_json::json!({
                "status": "error",
                "message": "Application state not available"
            });
            return HttpResponse::InternalServerError().json(&body);
        }
    };

    let report: HealthReport = app.check_health().await;
    build_health_response(&report)
}

/// Build an HTTP JSON response from a [`HealthReport`].
fn build_health_response(report: &HealthReport) -> HttpResponse {
    let status_str = match &report.status {
        HealthStatus::Healthy => "healthy",
        HealthStatus::Degraded { .. } => "degraded",
        HealthStatus::Unhealthy { .. } => "unhealthy",
    };

    let checks: Vec<serde_json::Value> = report
        .checks
        .iter()
        .map(|(name, status)| {
            let (s, detail) = match status {
                HealthStatus::Healthy => ("healthy", None),
                HealthStatus::Degraded { detail } => ("degraded", Some(detail.as_str())),
                HealthStatus::Unhealthy { detail } => ("unhealthy", Some(detail.as_str())),
            };
            let mut obj = serde_json::json!({
                "name": name,
                "status": s,
            });
            if let Some(d) = detail {
                obj["detail"] = serde_json::Value::String(d.to_string());
            }
            obj
        })
        .collect();

    let http_status = http_status_for(&report.status);
    let body = serde_json::json!({
        "status": status_str,
        "duration_ms": report.duration.as_millis(),
        "checks": checks,
    });
    HttpResponse::build(http_status).json(&body)
}

/// Map a [`HealthStatus`] to an HTTP status code.
fn http_status_for(status: &HealthStatus) -> ntex::http::StatusCode {
    match status {
        HealthStatus::Healthy => ntex::http::StatusCode::OK,
        _ => ntex::http::StatusCode::SERVICE_UNAVAILABLE,
    }
}
