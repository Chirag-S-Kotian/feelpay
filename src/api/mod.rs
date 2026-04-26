pub mod payments;
pub mod merchants;
pub mod health;
pub mod webhooks;
pub mod auth;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .configure(health::configure)
            .configure(merchants::configure)
            .configure(payments::configure)
            .configure(webhooks::configure)
    );
}
