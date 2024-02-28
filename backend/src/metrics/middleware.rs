use std::{future::Future, pin::Pin, rc::Rc};

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use once_cell::sync::Lazy;
use prometheus::{register_int_counter_vec, IntCounterVec};

/// pushes basic metrics to the global prometheus registry
#[derive(Debug, Clone, Copy, Default)]
pub struct Prometheus;

impl<S, B> Transform<S, ServiceRequest> for Prometheus
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = PrometheusMiddleware<S>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(PrometheusMiddleware{service: Rc::new(service)}))
    }
}

impl Prometheus {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug)]
pub struct PrometheusMiddleware<S: Service<ServiceRequest>> {
    service: Rc<S>
}

impl<S, B> Service<ServiceRequest> for PrometheusMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, ctx: &mut core::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        static TOTAL_REQUESTS: Lazy<IntCounterVec> = Lazy::new(|| {
            register_int_counter_vec!(
                "http_requests_total",
                "total http requests received",
                &["method", "status", "path"]
            ).expect("failed at initializing http_requests_total counter")
        });

        let method = req.method().clone();
        let path = req.path().to_owned();
        let service = self.service.clone();
        Box::pin(async move {
            let resp = service.call(req).await;
            match &resp {
                Ok(res) => {
                    TOTAL_REQUESTS.with_label_values(&[method.as_str(), res.status().clone().as_str(), &path]).inc()
                },
                Err(_) => todo!(),
            };
            resp
        })
    }
}
