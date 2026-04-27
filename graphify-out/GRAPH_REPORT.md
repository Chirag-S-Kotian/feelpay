# Graph Report - /home/chiragskotian/Desktop/payment  (2026-04-27)

## Corpus Check
- 98 files · ~93,837 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 684 nodes · 1098 edges · 62 communities detected
- Extraction: 68% EXTRACTED · 32% INFERRED · 0% AMBIGUOUS · INFERRED: 351 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 0|Community 0]]
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Community 3|Community 3]]
- [[_COMMUNITY_Community 4|Community 4]]
- [[_COMMUNITY_Community 5|Community 5]]
- [[_COMMUNITY_Community 6|Community 6]]
- [[_COMMUNITY_Community 7|Community 7]]
- [[_COMMUNITY_Community 8|Community 8]]
- [[_COMMUNITY_Community 9|Community 9]]
- [[_COMMUNITY_Community 10|Community 10]]
- [[_COMMUNITY_Community 11|Community 11]]
- [[_COMMUNITY_Community 12|Community 12]]
- [[_COMMUNITY_Community 13|Community 13]]
- [[_COMMUNITY_Community 14|Community 14]]
- [[_COMMUNITY_Community 15|Community 15]]
- [[_COMMUNITY_Community 16|Community 16]]
- [[_COMMUNITY_Community 17|Community 17]]
- [[_COMMUNITY_Community 18|Community 18]]
- [[_COMMUNITY_Community 19|Community 19]]
- [[_COMMUNITY_Community 20|Community 20]]
- [[_COMMUNITY_Community 21|Community 21]]
- [[_COMMUNITY_Community 22|Community 22]]
- [[_COMMUNITY_Community 23|Community 23]]
- [[_COMMUNITY_Community 24|Community 24]]
- [[_COMMUNITY_Community 25|Community 25]]
- [[_COMMUNITY_Community 26|Community 26]]
- [[_COMMUNITY_Community 27|Community 27]]
- [[_COMMUNITY_Community 28|Community 28]]
- [[_COMMUNITY_Community 29|Community 29]]
- [[_COMMUNITY_Community 30|Community 30]]
- [[_COMMUNITY_Community 31|Community 31]]
- [[_COMMUNITY_Community 32|Community 32]]
- [[_COMMUNITY_Community 33|Community 33]]
- [[_COMMUNITY_Community 34|Community 34]]
- [[_COMMUNITY_Community 35|Community 35]]
- [[_COMMUNITY_Community 36|Community 36]]
- [[_COMMUNITY_Community 37|Community 37]]
- [[_COMMUNITY_Community 38|Community 38]]
- [[_COMMUNITY_Community 39|Community 39]]
- [[_COMMUNITY_Community 40|Community 40]]
- [[_COMMUNITY_Community 41|Community 41]]
- [[_COMMUNITY_Community 42|Community 42]]
- [[_COMMUNITY_Community 43|Community 43]]
- [[_COMMUNITY_Community 44|Community 44]]
- [[_COMMUNITY_Community 45|Community 45]]
- [[_COMMUNITY_Community 46|Community 46]]
- [[_COMMUNITY_Community 47|Community 47]]
- [[_COMMUNITY_Community 48|Community 48]]
- [[_COMMUNITY_Community 49|Community 49]]
- [[_COMMUNITY_Community 50|Community 50]]
- [[_COMMUNITY_Community 51|Community 51]]
- [[_COMMUNITY_Community 52|Community 52]]
- [[_COMMUNITY_Community 53|Community 53]]
- [[_COMMUNITY_Community 54|Community 54]]
- [[_COMMUNITY_Community 55|Community 55]]
- [[_COMMUNITY_Community 56|Community 56]]
- [[_COMMUNITY_Community 57|Community 57]]
- [[_COMMUNITY_Community 58|Community 58]]
- [[_COMMUNITY_Community 59|Community 59]]
- [[_COMMUNITY_Community 60|Community 60]]
- [[_COMMUNITY_Community 61|Community 61]]

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

### Community 0 - "Community 0"
Cohesion: 0.04
Nodes (52): mfa_setup(), MfaService, accept_dispute(), get_dispute(), list_disputes(), merchant_id_from_req(), submit_evidence(), ApiError (+44 more)

### Community 1 - "Community 1"
Cohesion: 0.06
Nodes (51): ApiKeyMiddlewareService, ApiKeyMiddlewareService<S>, AuthResponse, captcha(), CaptchaResponse, CaptchaStore, change_password(), ChangePasswordRequest (+43 more)

### Community 2 - "Community 2"
Cohesion: 0.06
Nodes (44): create_customer(), CustomerListResponse, delete_customer(), get_customer(), get_customer_payment_methods(), list_customers(), merchant_id_from_req(), PaginationMeta (+36 more)

### Community 3 - "Community 3"
Cohesion: 0.06
Nodes (23): LedgerService, PaymentEngine, batch_refund(), BatchRefundItem, BatchRefundRequest, BatchRefundResponse, BatchRefundResult, capture_payment() (+15 more)

### Community 4 - "Community 4"
Cohesion: 0.06
Nodes (24): CreateCustomerRequest, CreateSavedPaymentMethodRequest, Customer, CustomerReferenceType, CustomerResponse, PaymentMethodType, SavedPaymentMethod, SavedPaymentMethodResponse (+16 more)

### Community 5 - "Community 5"
Cohesion: 0.08
Nodes (28): ApiKeyMiddleware, UnifiedAuth, send_otp(), verify_otp(), BankAccountRequest, BrandingRequest, BusinessInfoRequest, get_limits() (+20 more)

### Community 6 - "Community 6"
Cohesion: 0.07
Nodes (10): BinInfo, BinLookup, CardProcessor, VaultCrypto, CardVaultService, VaultedCard, VaultLookupRow, CardVaultService (+2 more)

### Community 7 - "Community 7"
Cohesion: 0.08
Nodes (16): fetchData(), createCustomer(), deleteCustomer(), fetchCustomers(), approveKYC(), fetchQueue(), rejectKYC(), fetchMerchants() (+8 more)

### Community 8 - "Community 8"
Cohesion: 0.08
Nodes (16): configure(), CheckoutSessionResponse, configure(), create_checkout_session(), CreateCheckoutSessionRequest, get_checkout_session(), get_payment_status(), AppConfig (+8 more)

### Community 9 - "Community 9"
Cohesion: 0.13
Nodes (14): CardProcessor, LedgerService, Merchant Dashboard, PaymentEngine, SettlementService, SimulatorProcessor, UpiProcessor, Vault (+6 more)

### Community 10 - "Community 10"
Cohesion: 0.21
Nodes (14): create_approval(), delete_document(), generate_upload_url(), get_approval(), get_document(), list_approvals(), list_documents(), ListApprovalsQuery (+6 more)

### Community 11 - "Community 11"
Cohesion: 0.18
Nodes (14): AnalyticsSummary, AnalyticsSummaryRow, DayVolume, DayVolumeRow, extract_merchant_id(), get_summary(), get_top_customers(), get_volume_chart() (+6 more)

### Community 12 - "Community 12"
Cohesion: 0.14
Nodes (13): ApprovalRequest, CreateApprovalRequest, Document, DocumentResponse, GenerateUploadUrlRequest, OtpResponse, OtpVerification, ProcessApprovalRequest (+5 more)

### Community 13 - "Community 13"
Cohesion: 0.15
Nodes (8): CapturePaymentRequest, ConnectorResponse, ConnectorType, CreatePaymentRequest, PaymentListQuery, PaymentListResponse, PaymentOrder, PaymentStatus

### Community 14 - "Community 14"
Cohesion: 0.18
Nodes (9): ApiKeyResponse, CreateMerchantRequest, ForgotPasswordRequest, LoginRequest, LoginResponse, Merchant, MerchantPublic, ResetPasswordRequest (+1 more)

### Community 15 - "Community 15"
Cohesion: 0.29
Nodes (1): SimulatorProcessor

### Community 16 - "Community 16"
Cohesion: 0.2
Nodes (6): CircuitBreakerState, CreateProviderRequest, CreateRoutingRuleRequest, PaymentProvider, ProviderHealthResponse, ProviderRoutingRule

### Community 17 - "Community 17"
Cohesion: 0.22
Nodes (8): Claims, CreateUserRequest, Permission, Role, RolePermission, User, UserResponse, UserRole

### Community 18 - "Community 18"
Cohesion: 0.33
Nodes (5): MethodData, PaymentMethod, PaymentMethodProcessor, ProcessorResponse, TransactionContext

### Community 19 - "Community 19"
Cohesion: 0.33
Nodes (3): ApiKeyIpWhitelist, ApiKeyIpWhitelistMiddleware, ApiKeyWhitelistData

### Community 20 - "Community 20"
Cohesion: 0.33
Nodes (6): Admin Panel, Escrow Account, FeelPay, PA/PG License, PCI-DSS, UPI AutoPay

### Community 21 - "Community 21"
Cohesion: 0.4
Nodes (4): BankAccount, CreateBankAccountRequest, CreatePayoutRequest, Payout

### Community 22 - "Community 22"
Cohesion: 0.4
Nodes (0): 

### Community 23 - "Community 23"
Cohesion: 0.4
Nodes (3): AdminAuthMiddleware, AdminAuthMiddlewareService, Claims

### Community 24 - "Community 24"
Cohesion: 0.4
Nodes (5): NPCI, PSP, RBI, UPI, VPA

### Community 25 - "Community 25"
Cohesion: 0.5
Nodes (3): Dispute, ListDisputesQuery, SubmitEvidenceRequest

### Community 26 - "Community 26"
Cohesion: 0.5
Nodes (3): CreateTenantRequest, Tenant, UpdateTenantRequest

### Community 27 - "Community 27"
Cohesion: 0.5
Nodes (3): WebhookConfigRequest, WebhookEvent, WebhookPayload

### Community 28 - "Community 28"
Cohesion: 0.5
Nodes (4): Phase 1: Core Engine, Phase 2: Vault, Phase_3_UPI_Engine, Phase_4_Ledger_Settlements

### Community 29 - "Community 29"
Cohesion: 0.67
Nodes (2): ApiKey, CreateApiKeyRequest

### Community 30 - "Community 30"
Cohesion: 0.67
Nodes (2): KycVerification, MerchantTier

### Community 31 - "Community 31"
Cohesion: 0.67
Nodes (2): LedgerAccount, LedgerEntry

### Community 32 - "Community 32"
Cohesion: 0.67
Nodes (2): CreateRefundRequest, Refund

### Community 33 - "Community 33"
Cohesion: 0.67
Nodes (2): Settlement, SettlementItem

### Community 34 - "Community 34"
Cohesion: 0.67
Nodes (2): Subscription, SubscriptionInvoice

### Community 35 - "Community 35"
Cohesion: 0.67
Nodes (0): 

### Community 36 - "Community 36"
Cohesion: 0.67
Nodes (0): 

### Community 37 - "Community 37"
Cohesion: 1.0
Nodes (1): AuditLog

### Community 38 - "Community 38"
Cohesion: 1.0
Nodes (1): FeeRule

### Community 39 - "Community 39"
Cohesion: 1.0
Nodes (0): 

### Community 40 - "Community 40"
Cohesion: 1.0
Nodes (0): 

### Community 41 - "Community 41"
Cohesion: 1.0
Nodes (0): 

### Community 42 - "Community 42"
Cohesion: 1.0
Nodes (0): 

### Community 43 - "Community 43"
Cohesion: 1.0
Nodes (0): 

### Community 44 - "Community 44"
Cohesion: 1.0
Nodes (0): 

### Community 45 - "Community 45"
Cohesion: 1.0
Nodes (0): 

### Community 46 - "Community 46"
Cohesion: 1.0
Nodes (0): 

### Community 47 - "Community 47"
Cohesion: 1.0
Nodes (0): 

### Community 48 - "Community 48"
Cohesion: 1.0
Nodes (0): 

### Community 49 - "Community 49"
Cohesion: 1.0
Nodes (0): 

### Community 50 - "Community 50"
Cohesion: 1.0
Nodes (0): 

### Community 51 - "Community 51"
Cohesion: 1.0
Nodes (0): 

### Community 52 - "Community 52"
Cohesion: 1.0
Nodes (0): 

### Community 53 - "Community 53"
Cohesion: 1.0
Nodes (0): 

### Community 54 - "Community 54"
Cohesion: 1.0
Nodes (0): 

### Community 55 - "Community 55"
Cohesion: 1.0
Nodes (0): 

### Community 56 - "Community 56"
Cohesion: 1.0
Nodes (0): 

### Community 57 - "Community 57"
Cohesion: 1.0
Nodes (0): 

### Community 58 - "Community 58"
Cohesion: 1.0
Nodes (0): 

### Community 59 - "Community 59"
Cohesion: 1.0
Nodes (0): 

### Community 60 - "Community 60"
Cohesion: 1.0
Nodes (1): Phase_5_Onboarding

### Community 61 - "Community 61"
Cohesion: 1.0
Nodes (1): Phase_6_Dashboard

## Knowledge Gaps
- **160 isolated node(s):** `AppState`, `ApiKey`, `CreateApiKeyRequest`, `AuditLog`, `CreateCustomerRequest` (+155 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 37`** (2 nodes): `AuditLog`, `audit.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 38`** (2 nodes): `FeeRule`, `fee.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 39`** (2 nodes): `Login.tsx`, `Login()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 40`** (2 nodes): `Settlements.tsx`, `Settlements()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 41`** (2 nodes): `formatCurrency()`, `Analytics.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 42`** (2 nodes): `Settings.tsx`, `copy()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 43`** (2 nodes): `MiniChart()`, `Dashboard.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 44`** (2 nodes): `Signup.tsx`, `Signup()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 45`** (2 nodes): `seed_merchant.py`, `create_test_merchant()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 46`** (2 nodes): `App.tsx`, `main.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 47`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 48`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 49`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 50`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 51`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 52`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 53`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 54`** (1 nodes): `vite.config.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 55`** (1 nodes): `main.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 56`** (1 nodes): `Payments.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 57`** (1 nodes): `Disputes.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 58`** (1 nodes): `vite.config.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 59`** (1 nodes): `vite-env.d.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 60`** (1 nodes): `Phase_5_Onboarding`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 61`** (1 nodes): `Phase_6_Dashboard`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `main()` connect `Community 8` to `Community 0`, `Community 1`, `Community 3`, `Community 5`, `Community 9`?**
  _High betweenness centrality (0.050) - this node is a cross-community bridge._
- **Why does `calculate_fees()` connect `Community 2` to `Community 0`, `Community 8`, `Community 14`?**
  _High betweenness centrality (0.031) - this node is a cross-community bridge._
- **Are the 16 inferred relationships involving `main()` (e.g. with `.new()` and `.from_env()`) actually correct?**
  _`main()` has 16 INFERRED edges - model-reasoned connections that need verification._
- **Are the 9 inferred relationships involving `apiRequest()` (e.g. with `handleSubmit()` and `createLink()`) actually correct?**
  _`apiRequest()` has 9 INFERRED edges - model-reasoned connections that need verification._
- **Are the 5 inferred relationships involving `login()` (e.g. with `.new()` and `.to_string()`) actually correct?**
  _`login()` has 5 INFERRED edges - model-reasoned connections that need verification._
- **What connects `AppState`, `ApiKey`, `CreateApiKeyRequest` to the rest of the system?**
  _160 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.04 - nodes in this community are weakly interconnected._