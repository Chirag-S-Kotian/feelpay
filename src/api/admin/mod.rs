//! Admin API endpoints
//!
//! Contains admin-only endpoints for platform management including:
//! - Analytics and reporting
//! - Merchant management
//! - Transaction management
//! - Settlement management
//! - Tenant management
//! - System configuration

use actix_web::web;

pub mod admin_users;
pub mod analytics;
pub mod audit_logs;
mod audit_support;
pub mod connectors;
pub mod kyc_viewer;
pub mod merchants;
pub mod notes;
pub mod notification_templates;
pub mod provider_credentials;
pub mod revenue;
pub mod search;
pub mod settlements;
pub mod system_config;
pub mod tenants;
pub mod transactions;

pub use admin_users::*;
pub use analytics::*;
pub use audit_logs::*;
pub use kyc_viewer::*;
pub use merchants::*;
pub use notes::*;
pub use notification_templates::*;
pub use provider_credentials::*;
pub use revenue::*;
pub use search::*;
pub use settlements::*;
pub use system_config::*;
pub use tenants::*;
pub use transactions::*;

/// Configure admin API routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            // Merchant notes routes
            .service(
                web::scope("/merchants/{merchant_id}/notes")
                    .route("", web::post().to(create_note))
                    .route("", web::get().to(list_notes)),
            )
            .service(
                web::scope("/notes/{note_id}")
                    .route("", web::put().to(update_note))
                    .route("", web::delete().to(delete_note)),
            )
            // Analytics routes
            .configure(analytics::configure)
            // KYC document viewer routes
            .configure(kyc_viewer::configure)
            .configure(merchants::configure)
            .configure(search::configure)
            .configure(transactions::configure)
            .configure(settlements::configure)
            .configure(revenue::configure)
            .configure(provider_credentials::configure)
            .configure(tenants::configure)
            .configure(admin_users::configure)
            .configure(audit_logs::configure)
            .configure(system_config::configure)
            .configure(notification_templates::configure)
            .configure(connectors::configure),
    );
}
