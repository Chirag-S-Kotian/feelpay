use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpResponse};
use futures::future::{ok, LocalBoxFuture, Ready};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// IP-based rate limiter middleware.
///  
/// Uses a simple sliding-window counter keyed on client IP.
/// Production: swap for Redis-backed or leaky-bucket.
#[derive(Clone)]
pub struct RateLimiter {
    max_per_minute: u32,
    counters: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl RateLimiter {
    pub fn new(max_per_minute: u32) -> Self {
        Self {
            max_per_minute,
            counters: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimiterMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimiterMiddleware {
            service,
            max_per_minute: self.max_per_minute,
            counters: Arc::clone(&self.counters),
        })
    }
}

pub struct RateLimiterMiddleware<S> {
    service: S,
    max_per_minute: u32,
    counters: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl<S, B> Service<ServiceRequest> for RateLimiterMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Skip rate-limit on health endpoints and OPTIONS requests
        let path = req.path().to_string();
        if path == "/health"
            || path == "/v1/health"
            || req.method() == actix_web::http::Method::OPTIONS
        {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) });
        }

        // Dev UX: avoid localhost throttle loops from React StrictMode double effects + HMR.
        let is_dev_env = std::env::var("APP_ENV")
            .map(|v| {
                let v = v.to_ascii_lowercase();
                v != "production" && v != "prod"
            })
            .unwrap_or(true);
        if is_dev_env {
            let host = req.connection_info().host().to_ascii_lowercase();
            if host.contains("localhost") || host.contains("127.0.0.1") || host.contains("[::1]") {
                let fut = self.service.call(req);
                return Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) });
            }
        }

        let ip = extract_client_ip(&req);

        // If caller identity cannot be resolved, skip limiter to avoid shared "unknown" bucket.
        let ip = match ip {
            Some(value) if !value.is_empty() => value,
            _ => {
                let fut = self.service.call(req);
                return Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) });
            }
        };

        let (bucket, max) = classify_limit(&path, req.method().as_str(), self.max_per_minute);
        let counters = self.counters.clone();

        // Check rate limit
        let now = Instant::now();
        let window = std::time::Duration::from_secs(60);

        let allowed = {
            let mut map = match counters.lock() {
                Ok(guard) => guard,
                Err(e) => {
                    tracing::error!("Rate limiter lock poisoned: {}", e);
                    let mut builder = HttpResponse::ServiceUnavailable();
                    apply_rate_limit_cors(&mut builder, &req);
                    let response = builder.json(serde_json::json!({
                        "error": {
                            "code": "service_unavailable",
                            "message": "Rate limiter temporarily unavailable",
                        }
                    }));
                    return Box::pin(async move {
                        Ok(req.into_response(response).map_into_right_body())
                    });
                }
            };

            // Global cleanup: drop old timestamps and remove stale keys.
            map.retain(|_, timestamps| {
                timestamps.retain(|t| now.duration_since(*t) < window);
                !timestamps.is_empty()
            });

            let entry = map.entry(format!("{bucket}:{ip}")).or_insert_with(Vec::new);
            if entry.len() >= max as usize {
                false
            } else {
                entry.push(now);
                true
            }
        };

        if !allowed {
            tracing::warn!(
                rate_limit_bucket = bucket,
                path = %path,
                method = %req.method(),
                "Request rate limited"
            );
            let mut builder = HttpResponse::TooManyRequests();
            apply_rate_limit_cors(&mut builder, &req);
            let response = builder.json(serde_json::json!({
                "error": {
                    "code": "rate_limit_exceeded",
                    "message": format!("Rate limit of {} requests per minute exceeded. Please retry after a moment.", max),
                }
            }));
            return Box::pin(async move { Ok(req.into_response(response).map_into_right_body()) });
        }

        let fut = self.service.call(req);
        Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) })
    }
}

fn extract_client_ip(req: &ServiceRequest) -> Option<String> {
    if let Some(cf_ip) = req.headers().get("cf-connecting-ip") {
        if let Ok(ip_str) = cf_ip.to_str() {
            let trimmed = ip_str.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                let trimmed = first_ip.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }
    if let Some(real_ip) = req.headers().get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            let trimmed = ip_str.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    if let Some(addr) = req.connection_info().realip_remote_addr() {
        return Some(addr.to_string());
    }
    req.peer_addr().map(|a| a.ip().to_string())
}

fn apply_rate_limit_cors(builder: &mut actix_web::HttpResponseBuilder, req: &ServiceRequest) {
    if let Some(origin) = req.headers().get(actix_web::http::header::ORIGIN) {
        if let Ok(origin_str) = origin.to_str() {
            if origin_str.ends_with(".pages.dev")
                || origin_str.ends_with(".onrender.com")
                || origin_str.starts_with("http://localhost:")
                || origin_str.starts_with("http://127.0.0.1:")
            {
                if let Ok(val) = actix_web::http::header::HeaderValue::from_str(origin_str) {
                    builder.insert_header((actix_web::http::header::ACCESS_CONTROL_ALLOW_ORIGIN, val));
                    builder.insert_header((
                        actix_web::http::header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                        "true",
                    ));
                    builder.insert_header((
                        actix_web::http::header::VARY,
                        "Origin",
                    ));
                }
            }
        }
    }
}

fn classify_limit(path: &str, method: &str, default_limit: u32) -> (&'static str, u32) {
    if path.ends_with("/login") {
        ("login", 10)
    } else if path.contains("/otp/send") || path.ends_with("/aadhaar/initiate") {
        ("otp_request", 5)
    } else if path.contains("/otp/verify") || path.ends_with("/aadhaar/verify") {
        ("otp_verify", 10)
    } else if path.contains("/webhooks/replay") {
        ("webhook_retry", 10)
    } else if method != "GET" && path.contains("/payout") {
        ("payout_mutation", 10)
    } else if path.contains("/exports") || path.contains("/reports") {
        ("report_export", 20)
    } else if (path.starts_with("/admin") || path.starts_with("/v1/admin")) && method != "GET" {
        ("admin_sensitive", 30)
    } else {
        ("default", default_limit)
    }
}

#[cfg(test)]
mod tests {
    use super::classify_limit;

    #[test]
    fn sensitive_routes_use_stricter_rate_limit_buckets() {
        assert_eq!(classify_limit("/v1/auth/login", "POST", 60), ("login", 10));
        assert_eq!(
            classify_limit("/v1/documents/otp/send", "POST", 60),
            ("otp_request", 5)
        );
        assert_eq!(
            classify_limit("/v1/documents/otp/verify", "POST", 60),
            ("otp_verify", 10)
        );
        assert_eq!(
            classify_limit("/v1/webhooks/replay/1", "POST", 60),
            ("webhook_retry", 10)
        );
        assert_eq!(
            classify_limit("/v1/payouts", "POST", 60),
            ("payout_mutation", 10)
        );
        assert_eq!(
            classify_limit("/v1/exports/transactions", "POST", 60),
            ("report_export", 20)
        );
        assert_eq!(
            classify_limit("/v1/admin/users", "POST", 60),
            ("admin_sensitive", 30)
        );
        assert_eq!(
            classify_limit("/admin/integrations/templates", "GET", 600),
            ("default", 600)
        );
    }
}
