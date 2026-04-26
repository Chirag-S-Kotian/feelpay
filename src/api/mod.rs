pub mod auth;
pub mod disputes;
pub mod health;
pub mod merchants;
pub mod onboarding;
pub mod payments;
pub mod settlements;
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
            .configure(webhooks::configure),
    );
}
