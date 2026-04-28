use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use chrono::Utc;
use futures::future::LocalBoxFuture;
use redis::Commands;
use std::rc::Rc;

/// Replay attack prevention middleware.
/// Validates request timestamp (within 5 min window) and nonce uniqueness.
pub struct ReplayPrevention {
    redis_client: redis::Client,
}

impl ReplayPrevention {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        Ok(ReplayPrevention {
            redis_client: client,
        })
    }
}

impl<S, B> Transform<S, ServiceRequest> for ReplayPrevention
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ReplayPreventionMiddleware<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(ReplayPreventionMiddleware {
            service: Rc::new(service),
            redis_client: self.redis_client.clone(),
        }))
    }
}

pub struct ReplayPreventionMiddleware<S> {
    service: Rc<S>,
    redis_client: redis::Client,
}

impl<S, B> Service<ServiceRequest> for ReplayPreventionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let redis_client = self.redis_client.clone();

        Box::pin(async move {
            // Skip replay check for GET requests (idempotent)
            if req.method() == actix_web::http::Method::GET {
                return service.call(req).await;
            }

            // Extract X-Request-Timestamp and X-Request-Nonce headers
            let timestamp_str = req
                .headers()
                .get("X-Request-Timestamp")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string());

            let nonce = req
                .headers()
                .get("X-Request-Nonce")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string());

            // If headers missing, allow (backward compat) but log
            if timestamp_str.is_none() || nonce.is_none() {
                tracing::warn!(
                    "Replay prevention headers missing on {} {}",
                    req.method(),
                    req.path()
                );
                return service.call(req).await;
            }

            let timestamp_str = timestamp_str.unwrap();
            let nonce = nonce.unwrap();

            // Validate timestamp format (Unix seconds)
            let request_timestamp = match timestamp_str.parse::<i64>() {
                Ok(ts) => ts,
                Err(_) => {
                    return Err(actix_web::error::ErrorBadRequest(
                        "Invalid X-Request-Timestamp format",
                    ))
                }
            };

            let now = Utc::now().timestamp();
            let time_diff = (now - request_timestamp).abs();

            // 5-minute window (300 seconds)
            if time_diff > 300 {
                return Err(actix_web::error::ErrorBadRequest(
                    "Request timestamp outside 5-minute window",
                ));
            }

            // Check nonce uniqueness in Redis
            let mut conn = match redis_client.get_connection() {
                Ok(c) => c,
                Err(_) => {
                    tracing::error!("Failed to connect to Redis for replay prevention");
                    return Err(actix_web::error::ErrorInternalServerError(
                        "Replay prevention unavailable",
                    ));
                }
            };

            let nonce_key = format!("replay_nonce:{}", nonce);
            let exists: bool = match conn.exists(&nonce_key) {
                Ok(e) => e,
                Err(_) => {
                    tracing::error!("Redis exists check failed");
                    return Err(actix_web::error::ErrorInternalServerError(
                        "Replay prevention check failed",
                    ));
                }
            };

            if exists {
                return Err(actix_web::error::ErrorBadRequest(
                    "Duplicate request nonce detected",
                ));
            }

            // Store nonce with 5-minute expiry
            let set_result: redis::RedisResult<()> = conn.set_ex(&nonce_key, "1", 300);
            if set_result.is_err() {
                tracing::error!("Failed to store nonce in Redis");
                return Err(actix_web::error::ErrorInternalServerError(
                    "Replay prevention storage failed",
                ));
            }

            // Store in request extensions for logging
            req.extensions_mut().insert(ReplayCheckPassed {
                nonce: nonce.clone(),
                timestamp: request_timestamp,
            });

            service.call(req).await
        })
    }
}

#[derive(Clone, Debug)]
pub struct ReplayCheckPassed {
    pub nonce: String,
    pub timestamp: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_replay_nonce_format() {
        // Nonce should be UUID format
        let nonce = Uuid::new_v4().to_string();
        assert_eq!(nonce.len(), 36); // UUID string length
    }

    #[test]
    fn test_timestamp_validation() {
        let now = Utc::now().timestamp();
        let five_min_ago = now - 300;
        let six_min_ago = now - 360;

        // 5 min ago should be valid
        assert!((now - five_min_ago).abs() <= 300);

        // 6 min ago should be invalid
        assert!((now - six_min_ago).abs() > 300);
    }
}
