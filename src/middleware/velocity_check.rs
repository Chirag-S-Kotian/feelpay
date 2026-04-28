use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use chrono::Utc;
use futures::future::LocalBoxFuture;
use redis::Commands;
use std::rc::Rc;

/// Velocity checks for fraud prevention.
/// Tracks: same card in N seconds, same IP in N seconds, same device in N seconds.
pub struct VelocityCheck {
    redis_client: redis::Client,
}

#[derive(Clone, Debug)]
pub struct VelocityLimits {
    pub same_card_per_minute: u32, // Max transactions with same card per minute
    pub same_ip_per_minute: u32,   // Max transactions from same IP per minute
    pub same_device_per_minute: u32, // Max transactions from same device per minute
}

impl Default for VelocityLimits {
    fn default() -> Self {
        VelocityLimits {
            same_card_per_minute: 5,
            same_ip_per_minute: 10,
            same_device_per_minute: 10,
        }
    }
}

impl VelocityCheck {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        Ok(VelocityCheck {
            redis_client: client,
        })
    }

    pub async fn check_velocity(
        &self,
        card_last4: Option<&str>,
        ip_address: &str,
        device_id: Option<&str>,
        limits: &VelocityLimits,
    ) -> Result<VelocityCheckResult, String> {
        let mut conn = self
            .redis_client
            .get_connection()
            .map_err(|e| format!("Redis connection failed: {}", e))?;

        let mut violations = Vec::new();

        // Check card velocity
        if let Some(card) = card_last4 {
            let card_key = format!("velocity:card:{}", card);
            let count: u32 = conn.get(&card_key).unwrap_or(0);

            if count >= limits.same_card_per_minute {
                violations.push(VelocityViolation::SameCardExceeded {
                    count,
                    limit: limits.same_card_per_minute,
                });
            } else {
                // Increment and set 60-second expiry
                let _: () = conn
                    .incr(&card_key, 1)
                    .map_err(|e| format!("Redis incr failed: {}", e))?;
                let _: () = conn
                    .expire(&card_key, 60)
                    .map_err(|e| format!("Redis expire failed: {}", e))?;
            }
        }

        // Check IP velocity
        let ip_key = format!("velocity:ip:{}", ip_address);
        let ip_count: u32 = conn.get(&ip_key).unwrap_or(0);

        if ip_count >= limits.same_ip_per_minute {
            violations.push(VelocityViolation::SameIpExceeded {
                count: ip_count,
                limit: limits.same_ip_per_minute,
            });
        } else {
            let _: () = conn
                .incr(&ip_key, 1)
                .map_err(|e| format!("Redis incr failed: {}", e))?;
            let _: () = conn
                .expire(&ip_key, 60)
                .map_err(|e| format!("Redis expire failed: {}", e))?;
        }

        // Check device velocity
        if let Some(device) = device_id {
            let device_key = format!("velocity:device:{}", device);
            let device_count: u32 = conn.get(&device_key).unwrap_or(0);

            if device_count >= limits.same_device_per_minute {
                violations.push(VelocityViolation::SameDeviceExceeded {
                    count: device_count,
                    limit: limits.same_device_per_minute,
                });
            } else {
                let _: () = conn
                    .incr(&device_key, 1)
                    .map_err(|e| format!("Redis incr failed: {}", e))?;
                let _: () = conn
                    .expire(&device_key, 60)
                    .map_err(|e| format!("Redis expire failed: {}", e))?;
            }
        }

        Ok(VelocityCheckResult {
            passed: violations.is_empty(),
            violations,
            checked_at: Utc::now(),
        })
    }
}

#[derive(Clone, Debug)]
pub enum VelocityViolation {
    SameCardExceeded { count: u32, limit: u32 },
    SameIpExceeded { count: u32, limit: u32 },
    SameDeviceExceeded { count: u32, limit: u32 },
}

#[derive(Clone, Debug)]
pub struct VelocityCheckResult {
    pub passed: bool,
    pub violations: Vec<VelocityViolation>,
    pub checked_at: chrono::DateTime<chrono::Utc>,
}

impl<S, B> Transform<S, ServiceRequest> for VelocityCheck
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = VelocityCheckMiddleware<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(VelocityCheckMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct VelocityCheckMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for VelocityCheckMiddleware<S>
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

        Box::pin(async move {
            // Velocity checks applied at payment endpoint level, not middleware
            // This is a placeholder for integration at API layer
            service.call(req).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_velocity_limits_default() {
        let limits = VelocityLimits::default();
        assert_eq!(limits.same_card_per_minute, 5);
        assert_eq!(limits.same_ip_per_minute, 10);
        assert_eq!(limits.same_device_per_minute, 10);
    }

    #[test]
    fn test_velocity_violation_creation() {
        let violation = VelocityViolation::SameCardExceeded { count: 6, limit: 5 };
        match violation {
            VelocityViolation::SameCardExceeded { count, limit } => {
                assert_eq!(count, 6);
                assert_eq!(limit, 5);
            }
            _ => panic!("Wrong violation type"),
        }
    }
}
