use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::Error;
use futures::future::{ok, LocalBoxFuture, Ready};
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct RequestId;

impl<S, B> Transform<S, ServiceRequest> for RequestId
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestIdMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RequestIdMiddleware { service })
    }
}

pub struct RequestIdMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestIdMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let request_id = request_id_from_headers(&req);
        req.headers_mut().insert(
            HeaderName::from_static("x-request-id"),
            HeaderValue::from_str(&request_id)
                .unwrap_or_else(|_| HeaderValue::from_static("invalid")),
        );

        let fut = self.service.call(req);
        Box::pin(async move {
            let mut response = fut.await?;
            response.headers_mut().insert(
                HeaderName::from_static("x-request-id"),
                HeaderValue::from_str(&request_id)
                    .unwrap_or_else(|_| HeaderValue::from_static("invalid")),
            );
            Ok(response)
        })
    }
}

fn request_id_from_headers(req: &ServiceRequest) -> String {
    req.headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| is_safe_request_id(value))
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

fn is_safe_request_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

#[cfg(test)]
mod tests {
    use super::is_safe_request_id;

    #[test]
    fn request_id_validation_rejects_header_injection() {
        assert!(is_safe_request_id("req_123-abc"));
        assert!(!is_safe_request_id("req\nbad"));
        assert!(!is_safe_request_id(""));
    }
}
