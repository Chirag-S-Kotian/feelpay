pub mod analytics;
pub mod auth;
pub mod disputes;
pub mod health;
pub mod merchants;
pub mod onboarding;
pub mod payments;
pub mod payouts;
pub mod settlements;
pub mod tenants;
pub mod webhooks;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .configure(health::configure)
            .configure(merchants::configure)
            .configure(payments::configure)
            .configure(disputes::configure)
            .configure(onboarding::configure)
            .configure(settlements::configure)
            .configure(tenants::configure)
            .configure(payouts::configure)
            .configure(webhooks::configure)
            .configure(analytics::configure),
    );
}
