use std::{future::Future, pin::Pin};

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};

#[derive(Debug, Clone, Default)]
pub struct TwitchAuth {

}

impl<S, B> Transform<S, ServiceRequest> for TwitchAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(AuthMiddleware{service}))
    }
}

pub struct AuthMiddleware<S: Service<ServiceRequest>> {
    service: S
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, _ctx: &mut core::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    // TODO: check if the given token is valid
    fn call(&self, req: ServiceRequest) -> Self::Future {
        Box::pin(self.service.call(req))
    }
}
