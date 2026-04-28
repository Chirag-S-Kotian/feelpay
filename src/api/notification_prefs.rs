use crate::engine::services::notification::{
    NotificationService, SetNotificationPreferenceRequest,
};
use crate::error::FeelPayError;
use crate::middleware::auth::JwtClaims;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Notification preference response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferenceResponse {
    pub event_type: String,
    pub email_enabled: bool,
    pub sms_enabled: bool,
    pub in_app_enabled: bool,
}

/// Notification preferences list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferencesListResponse {
    pub preferences: Vec<NotificationPreferenceResponse>,
}

/// Get notification preferences endpoint
#[utoipa::path(
    get,
    path = "/api/v1/merchants/{merchant_id}/notification-preferences",
    params(
        ("merchant_id" = Uuid, Path, description = "Merchant ID")
    ),
    responses(
        (status = 200, description = "Notification preferences", body = NotificationPreferencesListResponse),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn get_notification_preferences(
    merchant_id: web::Path<Uuid>,
    claims: JwtClaims,
    db: web::Data<PgPool>,
) -> Result<HttpResponse, FeelPayError> {
    let merchant_id = merchant_id.into_inner();

    // Verify merchant ownership
    if claims.merchant_id != Some(merchant_id) {
        return Err(FeelPayError::Forbidden(
            "You do not have permission to access this merchant".into(),
        ));
    }

    let notification_service = NotificationService::new(db.get_ref().clone());
    let prefs = notification_service
        .list_notification_preferences(merchant_id, None)
        .await?;

    let preferences = prefs
        .into_iter()
        .map(|pref| NotificationPreferenceResponse {
            event_type: pref.event_type,
            email_enabled: pref.email_enabled,
            sms_enabled: pref.sms_enabled,
            in_app_enabled: pref.in_app_enabled,
        })
        .collect();

    Ok(HttpResponse::Ok().json(NotificationPreferencesListResponse { preferences }))
}

/// Update notification preference endpoint
#[utoipa::path(
    put,
    path = "/api/v1/merchants/{merchant_id}/notification-preferences/{event_type}",
    params(
        ("merchant_id" = Uuid, Path, description = "Merchant ID"),
        ("event_type" = String, Path, description = "Event type")
    ),
    request_body = SetNotificationPreferenceRequest,
    responses(
        (status = 200, description = "Preference updated", body = NotificationPreferenceResponse),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn update_notification_preference(
    path: web::Path<(Uuid, String)>,
    req: web::Json<SetNotificationPreferenceRequest>,
    claims: JwtClaims,
    db: web::Data<PgPool>,
) -> Result<HttpResponse, FeelPayError> {
    let (merchant_id, event_type) = path.into_inner();

    // Verify merchant ownership
    if claims.merchant_id != Some(merchant_id) {
        return Err(FeelPayError::Forbidden(
            "You do not have permission to access this merchant".into(),
        ));
    }

    let notification_service = NotificationService::new(db.get_ref().clone());
    let pref = notification_service
        .set_notification_preferences(merchant_id, None, &event_type, req.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(NotificationPreferenceResponse {
        event_type: pref.event_type,
        email_enabled: pref.email_enabled,
        sms_enabled: pref.sms_enabled,
        in_app_enabled: pref.in_app_enabled,
    }))
}

/// Get user notification preferences endpoint
#[utoipa::path(
    get,
    path = "/api/v1/merchants/{merchant_id}/notification-preferences/user/{user_id}",
    params(
        ("merchant_id" = Uuid, Path, description = "Merchant ID"),
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User notification preferences", body = NotificationPreferencesListResponse),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn get_user_notification_preferences(
    path: web::Path<(Uuid, Uuid)>,
    claims: JwtClaims,
    db: web::Data<PgPool>,
) -> Result<HttpResponse, FeelPayError> {
    let (merchant_id, user_id) = path.into_inner();

    // Verify merchant ownership
    if claims.merchant_id != Some(merchant_id) {
        return Err(FeelPayError::Forbidden(
            "You do not have permission to access this merchant".into(),
        ));
    }

    let notification_service = NotificationService::new(db.get_ref().clone());
    let prefs = notification_service
        .list_notification_preferences(merchant_id, Some(user_id))
        .await?;

    let preferences = prefs
        .into_iter()
        .map(|pref| NotificationPreferenceResponse {
            event_type: pref.event_type,
            email_enabled: pref.email_enabled,
            sms_enabled: pref.sms_enabled,
            in_app_enabled: pref.in_app_enabled,
        })
        .collect();

    Ok(HttpResponse::Ok().json(NotificationPreferencesListResponse { preferences }))
}

/// Update user notification preference endpoint
#[utoipa::path(
    put,
    path = "/api/v1/merchants/{merchant_id}/notification-preferences/user/{user_id}/{event_type}",
    params(
        ("merchant_id" = Uuid, Path, description = "Merchant ID"),
        ("user_id" = Uuid, Path, description = "User ID"),
        ("event_type" = String, Path, description = "Event type")
    ),
    request_body = SetNotificationPreferenceRequest,
    responses(
        (status = 200, description = "Preference updated", body = NotificationPreferenceResponse),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn update_user_notification_preference(
    path: web::Path<(Uuid, Uuid, String)>,
    req: web::Json<SetNotificationPreferenceRequest>,
    claims: JwtClaims,
    db: web::Data<PgPool>,
) -> Result<HttpResponse, FeelPayError> {
    let (merchant_id, user_id, event_type) = path.into_inner();

    // Verify merchant ownership
    if claims.merchant_id != Some(merchant_id) {
        return Err(FeelPayError::Forbidden(
            "You do not have permission to access this merchant".into(),
        ));
    }

    let notification_service = NotificationService::new(db.get_ref().clone());
    let pref = notification_service
        .set_notification_preferences(merchant_id, Some(user_id), &event_type, req.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(NotificationPreferenceResponse {
        event_type: pref.event_type,
        email_enabled: pref.email_enabled,
        sms_enabled: pref.sms_enabled,
        in_app_enabled: pref.in_app_enabled,
    }))
}
