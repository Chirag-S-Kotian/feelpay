use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;

/// RBAC Middleware to check for granular permissions
pub struct RequirePermission {
    permission: String,
}

impl RequirePermission {
    pub fn new(permission: &str) -> Self {
        Self {
            permission: permission.to_string(),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequirePermission
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequirePermissionService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequirePermissionService {
            service: Rc::new(service),
            permission: self.permission.clone(),
        }))
    }
}

pub struct RequirePermissionService<S> {
    service: Rc<S>,
    permission: String,
}

impl<S, B> Service<ServiceRequest> for RequirePermissionService<S>
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
        let required_permission = self.permission.clone();

        Box::pin(async move {
            let (user_id, role_name) = {
                let extensions = req.extensions();
                (
                    extensions
                        .get::<crate::middleware::auth::UserId>()
                        .map(|u| u.0),
                    extensions
                        .get::<crate::middleware::auth::UserRole>()
                        .map(|r| r.0.clone()),
                )
            };

            let (Some(user_id), Some(role_name)) = (user_id, role_name) else {
                return Err(actix_web::error::ErrorUnauthorized(
                    "Missing authentication context",
                ));
            };

            // Platform super_admin has all permissions
            if role_name == "super_admin" {
                return service.call(req).await;
            }

            let Some(pool) = req.app_data::<web::Data<sqlx::PgPool>>() else {
                return Err(actix_web::error::ErrorInternalServerError(
                    "Database pool unavailable",
                ));
            };

            // Check if the user's role has the required permission
            let row = sqlx::query(
                r#"
                SELECT EXISTS (
                    SELECT 1 
                    FROM user_roles ur
                    JOIN roles r ON ur.role_id = r.id
                    JOIN role_permissions rp ON r.id = rp.role_id
                    JOIN permissions p ON rp.permission_id = p.id
                    WHERE ur.user_id = $1 AND p.name = $2
                ) as has_permission
                "#,
            )
            .bind(user_id)
            .bind(&required_permission)
            .fetch_one(pool.get_ref())
            .await
            .map_err(|e| {
                tracing::error!("RBAC database error: {}", e);
                actix_web::error::ErrorInternalServerError("Authorization check failed")
            })?;

            let has_permission: bool = sqlx::Row::get(&row, "has_permission");

            if has_permission {
                service.call(req).await
            } else {
                Err(actix_web::error::ErrorForbidden(format!(
                    "Missing required permission: {}",
                    required_permission
                )))
            }
        })
    }
}
