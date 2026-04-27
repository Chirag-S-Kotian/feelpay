use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpMessage, HttpRequest, HttpResponse};
use futures::future::{ok, LocalBoxFuture, Ready};
use std::net::IpAddr;

pub struct ApiKeyIpWhitelist;

impl<S, B> Transform<S, ServiceRequest> for ApiKeyIpWhitelist
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = ApiKeyIpWhitelistMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ApiKeyIpWhitelistMiddleware { service })
    }
}

pub struct ApiKeyIpWhitelistMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ApiKeyIpWhitelistMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Extract all data from extensions first - this ensures borrows are released
        // before we move req into the service call
        let (whitelist_data, whitelist_enabled) = {
            let ext = req.extensions();
            let data = ext.get::<ApiKeyWhitelistData>().cloned();
            let enabled = ext.get::<&bool>().copied().unwrap_or(&false);
            (data, enabled)
        };
        
        let client_ip = req.peer_addr().map(|a| a.ip().to_string()).unwrap_or_default();
        let whitelist: Vec<String> = whitelist_data.map(|d| d.0).unwrap_or_default();

        if !whitelist_enabled {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) });
        }

        if whitelist.is_empty() {
            let response = HttpResponse::Forbidden().json(serde_json::json!({
                "error": {
                    "code": "ip_whitelist_required",
                    "message": "IP whitelist is not configured for this API key"
                }
            }));
            return Box::pin(async move { Ok(req.into_response(response).map_into_right_body()) });
        }

        if !whitelist.contains(&client_ip) {
            let response = HttpResponse::Forbidden().json(serde_json::json!({
                "error": {
                    "code": "ip_not_whitelisted",
                    "message": "Your IP address is not authorized to use this API key",
                    "your_ip": client_ip
                }
            }));
            return Box::pin(async move { Ok(req.into_response(response).map_into_right_body()) });
        }

        let fut = self.service.call(req);
        Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) })
    }
}

#[derive(Clone)]
pub struct ApiKeyWhitelistData(pub Vec<String>);

pub fn extract_client_ip(req: &HttpRequest) -> Option<IpAddr> {
    if let Some(forwarded) = req.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse() {
                    return Some(ip);
                }
            }
        }
    }
    
    if let Some(real_ip) = req.headers().get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse() {
                return Some(ip);
            }
        }
    }
    
    req.peer_addr().map(|a| a.ip())
}
