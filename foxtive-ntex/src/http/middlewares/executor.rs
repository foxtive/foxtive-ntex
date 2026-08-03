use crate::http::middlewares::{Middleware, MiddlewareFlow};
use crate::http::response::error::ResponseError;
use foxtive::metrics::InfraEvent;
use ntex::service::{Middleware as ServiceMiddleware, Service, ServiceCtx};
use ntex::web;
use ntex::web::{Error, WebRequest, WebResponse};
use std::borrow::Cow;
use std::rc::Rc;
use std::time::Instant;
use tracing::{debug, error, info};

#[derive(Clone)]
pub struct MiddlewareChain {
    middlewares: Rc<Vec<Middleware>>,
}

impl MiddlewareChain {
    pub fn new(middlewares: Vec<Middleware>) -> Self {
        MiddlewareChain {
            middlewares: Rc::new(middlewares),
        }
    }

    /// Convenience method for single middleware
    pub fn single(middleware: Middleware) -> Self {
        Self::new(vec![middleware])
    }
}

impl<S, Cfg> ServiceMiddleware<S, Cfg> for MiddlewareChain {
    type Service = MiddlewareChainInternal<S>;

    fn create(&self, service: S, _cfg: Cfg) -> Self::Service {
        MiddlewareChainInternal {
            service,
            middlewares: self.middlewares.clone(),
        }
    }
}

pub struct MiddlewareChainInternal<S> {
    service: S,
    middlewares: Rc<Vec<Middleware>>,
}

impl<S, Err> Service<web::WebRequest<Err>> for MiddlewareChainInternal<S>
where
    S: Service<web::WebRequest<Err>, Response = web::WebResponse, Error = web::Error>,
    Err: web::ErrorRenderer,
{
    type Response = web::WebResponse;
    type Error = web::Error;

    ntex::forward_ready!(service);

    async fn call(
        &self,
        request: web::WebRequest<Err>,
        ctx: ServiceCtx<'_, Self>,
    ) -> Result<Self::Response, Self::Error> {
        let start = Instant::now();
        let (mut req, payload) = request.into_parts();
        info!("{} {}", req.method(), req.path());

        // Execute all "Before" middlewares in order
        for middleware in self.middlewares.iter() {
            if let Middleware::Before(mid) = middleware {
                let flow = mid
                    .handle(req)
                    .await
                    .map_err(|err| Error::from(ResponseError::new(err)))?;

                match flow {
                    MiddlewareFlow::Continue(modified_req) => {
                        req = modified_req;
                    }
                    MiddlewareFlow::Respond(request, response) => {
                        return Ok(WebResponse::new(response, request));
                    }
                }
            }
        }

        // Call the actual handler
        let request = WebRequest::from_parts(req, payload).unwrap();
        debug!("calling http controller -> method...");
        let mut response = ctx.call(&self.service, request).await?;

        // Execute all "After" middlewares in order
        for middleware in self.middlewares.iter() {
            if let Middleware::After(mid) = middleware {
                match mid.handle(response).await {
                    Ok(modified_resp) => {
                        response = modified_resp;
                    }
                    Err(err) => {
                        error!("[middleware-chain-error][after] {err:?}");
                        return Err(Error::from(ResponseError::new(err)));
                    }
                }
            }
        }

        // Emit HTTP request metrics
        let duration = start.elapsed();
        let success = response.status().is_success() || response.status().is_redirection();
        if let Some(app) = response.request().app_state::<std::sync::Arc<foxtive::App>>() {
            if let Some(metrics) = app.metrics() {
                metrics.record(&InfraEvent::OperationCompleted {
                    operation: Cow::Borrowed("http_request"),
                    duration,
                    success,
                });
            }
        }

        Ok(response)
    }
}
