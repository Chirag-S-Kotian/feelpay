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
pub mod payments;
pub mod payment_links;
pub mod providers;
pub mod settlements;
pub mod subscriptions;

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "FeelPay API",
        version = "1.0.0",
        description = "Payment Gateway API for Indian market - Cards, UPI, Net Banking, Wallets",
        contact(
            name = "FeelPay Support",
            email = "support@feelpay.in"
        )
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development"),
        (url = "https://api.feelpay.in", description = "Production")
    ),
    paths(
        onboarding::submit_kyc,
        onboarding::onboarding_status,
        onboarding::submit_bank_account,
        onboarding::update_branding,
        payments::create_payment,
        payments::list_payments,
        payments::get_payment,
        payments::capture_payment,
        payments::refund_payment,
        merchants::create_merchant,
        merchants::login_merchant,
        merchants::get_profile,
        merchants::update_profile,
        customers::create_customer,
        customers::get_customer,
        customers::list_customers,
        customers::get_customer_payment_methods,
        customers::delete_customer,
        payment_links::create_payment_link,
        payment_links::list_payment_links,
        payment_links::get_payment_link,
        payment_links::cancel_payment_link,
        settlements::list_settlements,
        settlements::get_settlement,
        settlements::get_settlement_stats,
        analytics::get_dashboard_stats,
    ),
    components(
        schemas(
            crate::models::merchant::CreateMerchantRequest,
            crate::models::merchant::BusinessType,
            crate::models::merchant::MerchantStatus,
            crate::models::customer::CreateCustomerRequest,
            crate::models::customer::CustomerResponse,
            crate::models::customer::SavedPaymentMethodResponse,
            crate::api::customers::CustomerListResponse,
            crate::api::customers::PaginationMeta,
            crate::models::payment_order::PaymentOrder,
            crate::models::payment_order::PaymentStatus,
            crate::models::dispute::Dispute,
            crate::api::onboarding::KycSubmissionRequest,
            crate::api::onboarding::BankAccountSubmissionRequest,
            crate::api::onboarding::OnboardingStatusResponse,
            crate::api::onboarding::BrandingRequest,
            crate::api::merchants::LoginRequest,
            crate::api::merchants::LoginResponse,
            crate::api::merchants::MerchantResponse,
            crate::api::payments::CreatePaymentRequest,
            crate::api::payments::RefundPaymentRequest,
            crate::models::payment_link::CreatePaymentLinkRequest,
            crate::models::payment_link::PaymentLinkResponse,
            crate::api::settlements::SettlementQuery,
            crate::api::settlements::SettlementResponse,
        )
    ),
    tags(
        (name = "FeelPay", description = "Payment orchestration API")
    )
)]
pub struct ApiDoc {}