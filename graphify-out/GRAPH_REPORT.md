# Graph Report - .  (2026-04-27)

## Corpus Check
- 106 files · ~90,442 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 684 nodes · 1098 edges · 62 communities detected
- Extraction: 68% EXTRACTED · 32% INFERRED · 0% AMBIGUOUS · INFERRED: 351 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Core API & Financial Services|Core API & Financial Services]]
- [[_COMMUNITY_Authentication & Middleware Infrastructure|Authentication & Middleware Infrastructure]]
- [[_COMMUNITY_CRM & Customer Engagement|CRM & Customer Engagement]]
- [[_COMMUNITY_Payment Orchestration & UPI Processing|Payment Orchestration & UPI Processing]]
- [[_COMMUNITY_Model Definitions (Customer, Notification, Links)|Model Definitions (Customer, Notification, Links)]]
- [[_COMMUNITY_Merchant Onboarding & Platform Lifecycle|Merchant Onboarding & Platform Lifecycle]]
- [[_COMMUNITY_Secure Vault & Card Tokenization|Secure Vault & Card Tokenization]]
- [[_COMMUNITY_Frontend UI Components & Pages|Frontend UI Components & Pages]]
- [[_COMMUNITY_System Configuration & Platform Operations|System Configuration & Platform Operations]]
- [[_COMMUNITY_Core System Hubs & State Management|Core System Hubs & State Management]]
- [[_COMMUNITY_Document Management & Compliance|Document Management & Compliance]]
- [[_COMMUNITY_Analytics & Data Intelligence|Analytics & Data Intelligence]]
- [[_COMMUNITY_Document & OTP Request Models|Document & OTP Request Models]]
- [[_COMMUNITY_Payment Order & Status Models|Payment Order & Status Models]]
- [[_COMMUNITY_Merchant & Key Management Models|Merchant & Key Management Models]]
- [[_COMMUNITY_Simulation & Testing Processors|Simulation & Testing Processors]]
- [[_COMMUNITY_Provider & Routing Models|Provider & Routing Models]]
- [[_COMMUNITY_User & Permissions Models|User & Permissions Models]]
- [[_COMMUNITY_Payment Processor Traits & Types|Payment Processor Traits & Types]]
- [[_COMMUNITY_Security & IP Whitelisting|Security & IP Whitelisting]]
- [[_COMMUNITY_Regulatory & Compliance Entities|Regulatory & Compliance Entities]]
- [[_COMMUNITY_Payout Request Models|Payout Request Models]]
- [[_COMMUNITY_Dashboard App Structure|Dashboard App Structure]]
- [[_COMMUNITY_Admin Middleware & Claims|Admin Middleware & Claims]]
- [[_COMMUNITY_UPI Ecosystem Entities|UPI Ecosystem Entities]]
- [[_COMMUNITY_Dispute Models|Dispute Models]]
- [[_COMMUNITY_Tenant Models|Tenant Models]]
- [[_COMMUNITY_Webhook Models|Webhook Models]]
- [[_COMMUNITY_Project Milestones (Phases)|Project Milestones (Phases)]]
- [[_COMMUNITY_API Key Models|API Key Models]]
- [[_COMMUNITY_KYC Models|KYC Models]]
- [[_COMMUNITY_Ledger Models|Ledger Models]]
- [[_COMMUNITY_Refund Models|Refund Models]]
- [[_COMMUNITY_Settlement Models|Settlement Models]]
- [[_COMMUNITY_Subscription Models|Subscription Models]]
- [[_COMMUNITY_Health Check API|Health Check API]]
- [[_COMMUNITY_Developer Portal (Frontend)|Developer Portal (Frontend)]]
- [[_COMMUNITY_Audit Log Models|Audit Log Models]]
- [[_COMMUNITY_Fee Rule Models|Fee Rule Models]]
- [[_COMMUNITY_Login Page (Frontend)|Login Page (Frontend)]]
- [[_COMMUNITY_Settlements Page (Frontend)|Settlements Page (Frontend)]]
- [[_COMMUNITY_Analytics Page (Frontend)|Analytics Page (Frontend)]]
- [[_COMMUNITY_Settings Page (Frontend)|Settings Page (Frontend)]]
- [[_COMMUNITY_Dashboard Page (Frontend)|Dashboard Page (Frontend)]]
- [[_COMMUNITY_Signup Page (Frontend)|Signup Page (Frontend)]]
- [[_COMMUNITY_Seeding Scripts|Seeding Scripts]]
- [[_COMMUNITY_Admin App Entry Points|Admin App Entry Points]]
- [[_COMMUNITY_Models Module Root|Models Module Root]]
- [[_COMMUNITY_DB Module Root|DB Module Root]]
- [[_COMMUNITY_Engine Module Root|Engine Module Root]]
- [[_COMMUNITY_Processors Module Root|Processors Module Root]]
- [[_COMMUNITY_Services Module Root|Services Module Root]]
- [[_COMMUNITY_Vault Module Root|Vault Module Root]]
- [[_COMMUNITY_Middleware Module Root|Middleware Module Root]]
- [[_COMMUNITY_Dashboard Vite Config|Dashboard Vite Config]]
- [[_COMMUNITY_Dashboard Main Entry|Dashboard Main Entry]]
- [[_COMMUNITY_Dashboard Payments Page|Dashboard Payments Page]]
- [[_COMMUNITY_Dashboard Disputes Page|Dashboard Disputes Page]]
- [[_COMMUNITY_Admin Vite Config|Admin Vite Config]]
- [[_COMMUNITY_Admin Index HTML|Admin Index HTML]]
- [[_COMMUNITY_Dashboard Index HTML|Dashboard Index HTML]]
- [[_COMMUNITY_README & Documentation|README & Documentation]]

## God Nodes (most connected - your core abstractions)
1. `main()` - 18 edges
2. `apiRequest()` - 13 edges
3. `PaymentEngine` - 11 edges
4. `CardProcessor` - 11 edges
5. `login()` - 11 edges
6. `SimulatorProcessor` - 10 edges
7. `UpiProcessor` - 10 edges
8. `merchant_id_from_req()` - 10 edges
9. `create_payment()` - 10 edges
10. `merchant_id_from_req()` - 9 edges

## Surprising Connections (you probably didn't know these)
- `configure_routes()` --calls--> `configure()`  [INFERRED]
  /home/chiragskotian/Desktop/payment/src/api/mod.rs → /home/chiragskotian/Desktop/payment/src/api/payouts.rs
- `main()` --calls--> `create_pool()`  [INFERRED]
  /home/chiragskotian/Desktop/payment/admin/server/src/main.rs → /home/chiragskotian/Desktop/payment/src/db/pool.rs
- `configure()` --calls--> `configure_routes()`  [INFERRED]
  /home/chiragskotian/Desktop/payment/src/api/analytics.rs → /home/chiragskotian/Desktop/payment/src/api/mod.rs
- `configure()` --calls--> `configure_routes()`  [INFERRED]
  /home/chiragskotian/Desktop/payment/src/api/disputes.rs → /home/chiragskotian/Desktop/payment/src/api/mod.rs
- `handleSubmit()` --calls--> `apiRequest()`  [INFERRED]
  /home/chiragskotian/Desktop/payment/dashboard/src/pages/Onboarding.tsx → /home/chiragskotian/Desktop/payment/admin/src/hooks/useApi.ts

## Communities

### Community 0 - "Core API & Financial Services"
Cohesion: 0.04
Nodes (52): mfa_setup(), MfaService, accept_dispute(), get_dispute(), list_disputes(), merchant_id_from_req(), submit_evidence(), ApiError (+44 more)

### Community 1 - "Authentication & Middleware Infrastructure"
Cohesion: 0.06
Nodes (51): ApiKeyMiddlewareService, ApiKeyMiddlewareService<S>, AuthResponse, captcha(), CaptchaResponse, CaptchaStore, change_password(), ChangePasswordRequest (+43 more)

### Community 2 - "CRM & Customer Engagement"
Cohesion: 0.06
Nodes (44): create_customer(), CustomerListResponse, delete_customer(), get_customer(), get_customer_payment_methods(), list_customers(), merchant_id_from_req(), PaginationMeta (+36 more)

### Community 3 - "Payment Orchestration & UPI Processing"
Cohesion: 0.06
Nodes (23): LedgerService, PaymentEngine, batch_refund(), BatchRefundItem, BatchRefundRequest, BatchRefundResponse, BatchRefundResult, capture_payment() (+15 more)

### Community 4 - "Model Definitions (Customer, Notification, Links)"
Cohesion: 0.06
Nodes (24): CreateCustomerRequest, CreateSavedPaymentMethodRequest, Customer, CustomerReferenceType, CustomerResponse, PaymentMethodType, SavedPaymentMethod, SavedPaymentMethodResponse (+16 more)

### Community 5 - "Merchant Onboarding & Platform Lifecycle"
Cohesion: 0.08
Nodes (28): ApiKeyMiddleware, UnifiedAuth, send_otp(), verify_otp(), BankAccountRequest, BrandingRequest, BusinessInfoRequest, get_limits() (+20 more)

### Community 6 - "Secure Vault & Card Tokenization"
Cohesion: 0.07
Nodes (10): BinInfo, BinLookup, CardProcessor, VaultCrypto, CardVaultService, VaultedCard, VaultLookupRow, CardVaultService (+2 more)

### Community 7 - "Frontend UI Components & Pages"
Cohesion: 0.08
Nodes (16): fetchData(), createCustomer(), deleteCustomer(), fetchCustomers(), approveKYC(), fetchQueue(), rejectKYC(), fetchMerchants() (+8 more)

### Community 8 - "System Configuration & Platform Operations"
Cohesion: 0.08
Nodes (16): configure(), CheckoutSessionResponse, configure(), create_checkout_session(), CreateCheckoutSessionRequest, get_checkout_session(), get_payment_status(), AppConfig (+8 more)

### Community 9 - "Core System Hubs & State Management"
Cohesion: 0.13
Nodes (14): CardProcessor, LedgerService, Merchant Dashboard, PaymentEngine, SettlementService, SimulatorProcessor, UpiProcessor, Vault (+6 more)

### Community 10 - "Document Management & Compliance"
Cohesion: 0.21
Nodes (14): create_approval(), delete_document(), generate_upload_url(), get_approval(), get_document(), list_approvals(), list_documents(), ListApprovalsQuery (+6 more)

### Community 11 - "Analytics & Data Intelligence"
Cohesion: 0.18
Nodes (14): AnalyticsSummary, AnalyticsSummaryRow, DayVolume, DayVolumeRow, extract_merchant_id(), get_summary(), get_top_customers(), get_volume_chart() (+6 more)

### Community 12 - "Document & OTP Request Models"
Cohesion: 0.14
Nodes (13): ApprovalRequest, CreateApprovalRequest, Document, DocumentResponse, GenerateUploadUrlRequest, OtpResponse, OtpVerification, ProcessApprovalRequest (+5 more)

### Community 13 - "Payment Order & Status Models"
Cohesion: 0.15
Nodes (8): CapturePaymentRequest, ConnectorResponse, ConnectorType, CreatePaymentRequest, PaymentListQuery, PaymentListResponse, PaymentOrder, PaymentStatus

### Community 14 - "Merchant & Key Management Models"
Cohesion: 0.18
Nodes (9): ApiKeyResponse, CreateMerchantRequest, ForgotPasswordRequest, LoginRequest, LoginResponse, Merchant, MerchantPublic, ResetPasswordRequest (+1 more)

### Community 15 - "Simulation & Testing Processors"
Cohesion: 0.29
Nodes (1): SimulatorProcessor

### Community 16 - "Provider & Routing Models"
Cohesion: 0.2
Nodes (6): CircuitBreakerState, CreateProviderRequest, CreateRoutingRuleRequest, PaymentProvider, ProviderHealthResponse, ProviderRoutingRule

### Community 17 - "User & Permissions Models"
Cohesion: 0.22
Nodes (8): Claims, CreateUserRequest, Permission, Role, RolePermission, User, UserResponse, UserRole

### Community 18 - "Payment Processor Traits & Types"
Cohesion: 0.33
Nodes (5): MethodData, PaymentMethod, PaymentMethodProcessor, ProcessorResponse, TransactionContext

### Community 19 - "Security & IP Whitelisting"
Cohesion: 0.33
Nodes (3): ApiKeyIpWhitelist, ApiKeyIpWhitelistMiddleware, ApiKeyWhitelistData

### Community 20 - "Regulatory & Compliance Entities"
Cohesion: 0.33
Nodes (6): Admin Panel, Escrow Account, FeelPay, PA/PG License, PCI-DSS, UPI AutoPay

### Community 21 - "Payout Request Models"
Cohesion: 0.4
Nodes (4): BankAccount, CreateBankAccountRequest, CreatePayoutRequest, Payout

### Community 22 - "Dashboard App Structure"
Cohesion: 0.4
Nodes (0): 

### Community 23 - "Admin Middleware & Claims"
Cohesion: 0.4
Nodes (3): AdminAuthMiddleware, AdminAuthMiddlewareService, Claims

### Community 24 - "UPI Ecosystem Entities"
Cohesion: 0.4
Nodes (5): NPCI, PSP, RBI, UPI, VPA

### Community 25 - "Dispute Models"
Cohesion: 0.5
Nodes (3): Dispute, ListDisputesQuery, SubmitEvidenceRequest

### Community 26 - "Tenant Models"
Cohesion: 0.5
Nodes (3): CreateTenantRequest, Tenant, UpdateTenantRequest

### Community 27 - "Webhook Models"
Cohesion: 0.5
Nodes (3): WebhookConfigRequest, WebhookEvent, WebhookPayload

### Community 28 - "Project Milestones (Phases)"
Cohesion: 0.5
Nodes (4): Phase 1: Core Engine, Phase 2: Vault, Phase_3_UPI_Engine, Phase_4_Ledger_Settlements

### Community 29 - "API Key Models"
Cohesion: 0.67
Nodes (2): ApiKey, CreateApiKeyRequest

### Community 30 - "KYC Models"
Cohesion: 0.67
Nodes (2): KycVerification, MerchantTier

### Community 31 - "Ledger Models"
Cohesion: 0.67
Nodes (2): LedgerAccount, LedgerEntry

### Community 32 - "Refund Models"
Cohesion: 0.67
Nodes (2): CreateRefundRequest, Refund

### Community 33 - "Settlement Models"
Cohesion: 0.67
Nodes (2): Settlement, SettlementItem

### Community 34 - "Subscription Models"
Cohesion: 0.67
Nodes (2): Subscription, SubscriptionInvoice

### Community 35 - "Health Check API"
Cohesion: 0.67
Nodes (0): 

### Community 36 - "Developer Portal (Frontend)"
Cohesion: 0.67
Nodes (0): 

### Community 37 - "Audit Log Models"
Cohesion: 1.0
Nodes (1): AuditLog

### Community 38 - "Fee Rule Models"
Cohesion: 1.0
Nodes (1): FeeRule

### Community 39 - "Login Page (Frontend)"
Cohesion: 1.0
Nodes (0): 

### Community 40 - "Settlements Page (Frontend)"
Cohesion: 1.0
Nodes (0): 

### Community 41 - "Analytics Page (Frontend)"
Cohesion: 1.0
Nodes (0): 

### Community 42 - "Settings Page (Frontend)"
Cohesion: 1.0
Nodes (0): 

### Community 43 - "Dashboard Page (Frontend)"
Cohesion: 1.0
Nodes (0): 

### Community 44 - "Signup Page (Frontend)"
Cohesion: 1.0
Nodes (0): 

### Community 45 - "Seeding Scripts"
Cohesion: 1.0
Nodes (0): 

### Community 46 - "Admin App Entry Points"
Cohesion: 1.0
Nodes (0): 

### Community 47 - "Models Module Root"
Cohesion: 1.0
Nodes (0): 

### Community 48 - "DB Module Root"
Cohesion: 1.0
Nodes (0): 

### Community 49 - "Engine Module Root"
Cohesion: 1.0
Nodes (0): 

### Community 50 - "Processors Module Root"
Cohesion: 1.0
Nodes (0): 

### Community 51 - "Services Module Root"
Cohesion: 1.0
Nodes (0): 

### Community 52 - "Vault Module Root"
Cohesion: 1.0
Nodes (0): 

### Community 53 - "Middleware Module Root"
Cohesion: 1.0
Nodes (0): 

### Community 54 - "Dashboard Vite Config"
Cohesion: 1.0
Nodes (0): 

### Community 55 - "Dashboard Main Entry"
Cohesion: 1.0
Nodes (0): 

### Community 56 - "Dashboard Payments Page"
Cohesion: 1.0
Nodes (0): 

### Community 57 - "Dashboard Disputes Page"
Cohesion: 1.0
Nodes (0): 

### Community 58 - "Admin Vite Config"
Cohesion: 1.0
Nodes (0): 

### Community 59 - "Admin Index HTML"
Cohesion: 1.0
Nodes (0): 

### Community 60 - "Dashboard Index HTML"
Cohesion: 1.0
Nodes (1): Phase_5_Onboarding

### Community 61 - "README & Documentation"
Cohesion: 1.0
Nodes (1): Phase_6_Dashboard

## Knowledge Gaps
- **160 isolated node(s):** `AppState`, `ApiKey`, `CreateApiKeyRequest`, `AuditLog`, `CreateCustomerRequest` (+155 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Audit Log Models`** (2 nodes): `AuditLog`, `audit.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Fee Rule Models`** (2 nodes): `FeeRule`, `fee.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Login Page (Frontend)`** (2 nodes): `Login.tsx`, `Login()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Settlements Page (Frontend)`** (2 nodes): `Settlements.tsx`, `Settlements()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Analytics Page (Frontend)`** (2 nodes): `formatCurrency()`, `Analytics.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Settings Page (Frontend)`** (2 nodes): `Settings.tsx`, `copy()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Dashboard Page (Frontend)`** (2 nodes): `MiniChart()`, `Dashboard.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Signup Page (Frontend)`** (2 nodes): `Signup.tsx`, `Signup()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Seeding Scripts`** (2 nodes): `seed_merchant.py`, `create_test_merchant()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Admin App Entry Points`** (2 nodes): `App.tsx`, `main.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Models Module Root`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `DB Module Root`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Engine Module Root`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Processors Module Root`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Services Module Root`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Vault Module Root`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Middleware Module Root`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Dashboard Vite Config`** (1 nodes): `vite.config.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Dashboard Main Entry`** (1 nodes): `main.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Dashboard Payments Page`** (1 nodes): `Payments.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Dashboard Disputes Page`** (1 nodes): `Disputes.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Admin Vite Config`** (1 nodes): `vite.config.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Admin Index HTML`** (1 nodes): `vite-env.d.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Dashboard Index HTML`** (1 nodes): `Phase_5_Onboarding`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `README & Documentation`** (1 nodes): `Phase_6_Dashboard`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `main()` connect `System Configuration & Platform Operations` to `Core API & Financial Services`, `Authentication & Middleware Infrastructure`, `Payment Orchestration & UPI Processing`, `Merchant Onboarding & Platform Lifecycle`, `Core System Hubs & State Management`?**
  _High betweenness centrality (0.050) - this node is a cross-community bridge._
- **Why does `calculate_fees()` connect `CRM & Customer Engagement` to `System Configuration & Platform Operations`, `Core API & Financial Services`, `Merchant & Key Management Models`?**
  _High betweenness centrality (0.031) - this node is a cross-community bridge._
- **Are the 16 inferred relationships involving `main()` (e.g. with `.new()` and `.from_env()`) actually correct?**
  _`main()` has 16 INFERRED edges - model-reasoned connections that need verification._
- **Are the 9 inferred relationships involving `apiRequest()` (e.g. with `handleSubmit()` and `createLink()`) actually correct?**
  _`apiRequest()` has 9 INFERRED edges - model-reasoned connections that need verification._
- **Are the 5 inferred relationships involving `login()` (e.g. with `.to_string()` and `.default()`) actually correct?**
  _`login()` has 5 INFERRED edges - model-reasoned connections that need verification._
- **What connects `AppState`, `ApiKey`, `CreateApiKeyRequest` to the rest of the system?**
  _160 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Core API & Financial Services` be split into smaller, more focused modules?**
  _Cohesion score 0.04 - nodes in this community are weakly interconnected._