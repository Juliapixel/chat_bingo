use std::{future::Future, pin::Pin, rc::Rc};

use actix_web::{dev::{Service, ServiceRequest, ServiceResponse, Transform}, HttpMessage};
use log::debug;

use super::jwt::validate_jwt;

/// checks for the `jwt` cookie, validates it and then injects [Claims](super::jwt::Claims)
/// data into the request, so it is available to services
#[derive(Debug, Clone, Default)]
pub struct TwitchAuthMiddleware();

impl<S, B> Transform<S, ServiceRequest> for TwitchAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(AuthMiddleware{service: Rc::new(service)}))
    }
}

pub struct AuthMiddleware<S: Service<ServiceRequest>> {
    service: Rc<S>
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, ctx: &mut core::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        Box::pin(async move {
            if let Some(auth_cookie) = req.cookie("jwt") {
                if let Ok(claims) = validate_jwt(auth_cookie.value()) {
                    debug!("found jwt token with claims: {claims:?}");
                    req.extensions_mut().insert(claims.claims);
                }
            }

            return svc.call(req).await;
        })
    }
}
