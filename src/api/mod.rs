pub mod admin;
pub mod analytics;
pub mod auth;
pub mod bank_accounts;
pub mod checkout;
pub mod customers;
pub mod disputes;
pub mod documents;
pub mod emi_bnpl;
pub mod exports;
pub mod fee_plans;
pub mod health;
pub mod mandates;
pub mod merchants;
pub mod notification_prefs;
pub mod notifications;
pub mod onboarding;
pub mod pagination;
pub mod payment_links;
pub mod payments;
pub mod payouts;
pub mod providers;
pub mod public_config;
pub mod reports;
pub mod settlements;
pub mod subscriptions;
pub mod virtual_accounts;
pub mod webhooks;
pub mod webhook_test;
pub mod provider_webhooks;

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
        emi_bnpl::get_emi_plans,
        emi_bnpl::get_bnpl_providers,
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
        analytics::get_actionable_alerts,
        analytics::get_volume_chart,
        analytics::get_top_customers,
        mandates::initiate_mandate,
        mandates::list_mandates,
        mandates::get_mandate,
        mandates::cancel_mandate,
        mandates::execute_mandate,
        virtual_accounts::create_virtual_account,
        virtual_accounts::list_virtual_accounts,
        virtual_accounts::get_virtual_account,
        onboarding::initiate_aadhaar_ekyc,
        onboarding::verify_aadhaar_otp,
        onboarding::update_registration_step,
        onboarding::get_registration_progress,
        onboarding::submit_business_info,
        onboarding::get_limits,
        onboarding::update_limits,
        merchants::get_keys,
        merchants::roll_keys,
        merchants::verify_mfa_merchant,
        merchants::reset_password,
        merchants::forgot_password,
        merchants::list_team_members,
        merchants::invite_team_member,
        merchants::remove_team_member,
        merchants::update_team_member_role,
        merchants::update_kyc_status,
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
            crate::models::mandate::Mandate,
            crate::models::mandate::CreateMandateRequest,
            crate::api::settlements::SettlementQuery,
            crate::api::settlements::SettlementResponse,
            crate::models::virtual_account::VirtualAccount,
            crate::models::virtual_account::CreateVirtualAccountRequest,
            crate::models::merchant::MerchantPaymentMethod,
            crate::api::emi_bnpl::EmiPlan,
            crate::api::emi_bnpl::EmiPlansQuery,
            crate::api::onboarding::InitiateAadhaarRequest,
            crate::api::onboarding::VerifyAadhaarOtpRequest,
            crate::api::onboarding::RegistrationStepRequest,
            crate::api::onboarding::RegistrationProgressResponse,
            crate::api::onboarding::BusinessInfoRequest,
            crate::api::onboarding::LimitsRequest,
            crate::api::onboarding::LimitsResponse,
            crate::api::merchants::InviteTeamMemberRequest,
            crate::api::merchants::UpdateRoleRequest,
            crate::api::merchants::UpdateKycStatusRequest,
            crate::api::pagination::Cursor,
            crate::api::pagination::PaginationParams,
        )
    ),
    tags(
        (name = "FeelPay", description = "Payment orchestration API")
    )
)]
pub struct ApiDoc {}
