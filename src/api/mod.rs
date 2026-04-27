pub mod analytics;
pub mod auth;
pub mod checkout;
pub mod customers;
pub mod disputes;
pub mod documents;
pub mod fee_plans;
pub mod health;
pub mod merchants;
pub mod notifications;
pub mod onboarding;
pub mod payment_links;
pub mod payments;
pub mod payouts;
pub mod providers;
pub mod settlements;
pub mod tenants;
pub mod webhooks;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .configure(health::configure)
            .configure(merchants::configure)
            .configure(payment_links::configure)
            .configure(payments::configure)
            .configure(disputes::configure)
            .configure(onboarding::configure)
            .configure(settlements::configure)
            .configure(tenants::configure)
            .configure(payouts::configure)
            .configure(webhooks::configure)
            .configure(analytics::configure)
            .configure(documents::configure)
            .configure(fee_plans::configure)
            .configure(checkout::configure),
    );
}
