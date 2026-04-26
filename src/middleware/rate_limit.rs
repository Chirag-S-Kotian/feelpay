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
        // Skip rate-limit on health endpoints
        let path = req.path().to_string();
        if path == "/health" || path == "/v1/health" {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) });
        }

        let ip = req.peer_addr().map(|a| a.ip().to_string());

        // If caller identity cannot be resolved, skip limiter to avoid shared "unknown" bucket.
        let ip = match ip {
            Some(value) if !value.is_empty() => value,
            _ => {
                let fut = self.service.call(req);
                return Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) });
            }
        };

        let max = self.max_per_minute;
        let counters = self.counters.clone();

        // Check rate limit
        let now = Instant::now();
        let window = std::time::Duration::from_secs(60);

        let allowed = {
            let mut map = match counters.lock() {
                Ok(guard) => guard,
                Err(e) => {
                    tracing::error!("Rate limiter lock poisoned: {}", e);
                    let response = HttpResponse::ServiceUnavailable().json(serde_json::json!({
                        "error": {
                            "code": "service_unavailable",
                            "message": "Rate limiter temporarily unavailable",
                        }
                    }));
                    return Box::pin(async move { Ok(req.into_response(response).map_into_right_body()) });
                }
            };

            // Global cleanup: drop old timestamps and remove stale keys.
            map.retain(|_, timestamps| {
                timestamps.retain(|t| now.duration_since(*t) < window);
                !timestamps.is_empty()
            });

            let entry = map.entry(ip.clone()).or_insert_with(Vec::new);
            if entry.len() >= max as usize {
                false
            } else {
                entry.push(now);
                true
            }
        };

        if !allowed {
            let response = HttpResponse::TooManyRequests()
                .json(serde_json::json!({
                    "error": {
                        "code": "rate_limit_exceeded",
                        "message": format!("Rate limit of {} requests per minute exceeded. Please retry after a moment.", max),
                    }
                }));
            return Box::pin(async move {
                Ok(req.into_response(response).map_into_right_body())
            });
        }

        let fut = self.service.call(req);
        Box::pin(async move { fut.await.map(|res| res.map_into_left_body()) })
    }
}
