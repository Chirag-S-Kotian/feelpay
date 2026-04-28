# Graph Report - /home/chiragskotian/Desktop/payment  (2026-04-27)

## Corpus Check
- 111 files · ~110,356 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 838 nodes · 1359 edges · 71 communities detected
- Extraction: 70% EXTRACTED · 30% INFERRED · 0% AMBIGUOUS · INFERRED: 410 edges (avg confidence: 0.8)
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
- [[_COMMUNITY_Community 62|Community 62]]
- [[_COMMUNITY_Community 63|Community 63]]
- [[_COMMUNITY_Community 64|Community 64]]
- [[_COMMUNITY_Community 65|Community 65]]
- [[_COMMUNITY_Community 66|Community 66]]
- [[_COMMUNITY_Community 67|Community 67]]
- [[_COMMUNITY_Community 68|Community 68]]
- [[_COMMUNITY_Community 69|Community 69]]
- [[_COMMUNITY_Community 70|Community 70]]

## God Nodes (most connected - your core abstractions)
1. `main()` - 23 edges
2. `apiRequest()` - 13 edges
3. `merchant_id_from_req()` - 12 edges
4. `PaymentEngine` - 11 edges
5. `CardProcessor` - 11 edges
6. `login()` - 11 edges
7. `create_payment()` - 11 edges
8. `SimulatorProcessor` - 10 edges
9. `UpiProcessor` - 10 edges
10. `send_email()` - 10 edges

## Surprising Connections (you probably didn't know these)
- `main()` --calls--> `configure()`  [INFERRED]
  /home/chiragskotian/Desktop/payment/admin/server/src/main.rs → /home/chiragskotian/Desktop/payment/src/api/subscriptions.rs
- `main()` --calls--> `configure()`  [INFERRED]
  /home/chiragskotian/Desktop/payment/admin/server/src/main.rs → /home/chiragskotian/Desktop/payment/src/api/checkout.rs
- `main()` --calls--> `configure()`  [INFERRED]
  /home/chiragskotian/Desktop/payment/admin/server/src/main.rs → /home/chiragskotian/Desktop/payment/src/api/documents.rs
- `main()` --calls--> `create_pool()`  [INFERRED]
  /home/chiragskotian/Desktop/payment/admin/server/src/main.rs → /home/chiragskotian/Desktop/payment/src/db/pool.rs
- `main()` --calls--> `configure()`  [INFERRED]
  /home/chiragskotian/Desktop/payment/admin/server/src/main.rs → /home/chiragskotian/Desktop/payment/src/api/emi_bnpl.rs

## Communities

### Community 0 - "Community 0"
Cohesion: 0.04
Nodes (66): mfa_setup(), MfaService, get_customer(), accept_dispute(), get_dispute(), list_disputes(), merchant_id_from_req(), submit_evidence() (+58 more)

### Community 1 - "Community 1"
Cohesion: 0.04
Nodes (26): SimulatorProcessor, LedgerService, NetBankingProcessor, PaymentEngine, batch_refund(), BatchRefundItem, BatchRefundRequest, BatchRefundResponse (+18 more)

### Community 2 - "Community 2"
Cohesion: 0.06
Nodes (51): ApiKeyMiddlewareService, ApiKeyMiddlewareService<S>, AuthResponse, captcha(), CaptchaResponse, CaptchaStore, change_password(), ChangePasswordRequest (+43 more)

### Community 3 - "Community 3"
Cohesion: 0.06
Nodes (43): ApiKeyMiddleware, UnifiedAuth, BankAccountSubmissionRequest, BrandingRequest, BusinessInfoRequest, get_limits(), get_registration_progress(), initiate_aadhaar_ekyc() (+35 more)

### Community 4 - "Community 4"
Cohesion: 0.06
Nodes (24): CreateCustomerRequest, CreateSavedPaymentMethodRequest, Customer, CustomerReferenceType, CustomerResponse, PaymentMethodType, SavedPaymentMethod, SavedPaymentMethodResponse (+16 more)

### Community 5 - "Community 5"
Cohesion: 0.07
Nodes (38): BillingCycle, calculate_period(), cancel_subscription(), create_plan(), create_subscription(), CreatePlanRequest, CreateSubscriptionRequest, generate_invoice() (+30 more)

### Community 6 - "Community 6"
Cohesion: 0.07
Nodes (10): BinInfo, BinLookup, CardProcessor, VaultCrypto, CardVaultService, VaultedCard, VaultLookupRow, CardVaultService (+2 more)

### Community 7 - "Community 7"
Cohesion: 0.08
Nodes (16): fetchData(), createCustomer(), deleteCustomer(), fetchCustomers(), approveKYC(), fetchQueue(), rejectKYC(), fetchMerchants() (+8 more)

### Community 8 - "Community 8"
Cohesion: 0.09
Nodes (30): configure(), create_approval(), delete_document(), generate_upload_url(), get_approval(), get_document(), list_approvals(), list_documents() (+22 more)

### Community 9 - "Community 9"
Cohesion: 0.12
Nodes (21): CreateMerchantRequestInput, extract_merchant_id_internal(), get_keys(), get_merchant(), get_profile(), invite_team_member(), InviteTeamMemberRequest, list_payment_methods() (+13 more)

### Community 10 - "Community 10"
Cohesion: 0.11
Nodes (12): AppConfig, BnplProvider, configure(), EmiPlan, EmiPlansQuery, impersonate_merchant(), io_other(), main() (+4 more)

### Community 11 - "Community 11"
Cohesion: 0.09
Nodes (15): CheckoutSessionResponse, configure(), create_checkout_session(), CreateCheckoutSessionRequest, get_checkout_session(), get_payment_status(), CapturePaymentRequest, ConnectorResponse (+7 more)

### Community 12 - "Community 12"
Cohesion: 0.1
Nodes (15): CardProcessor, LedgerService, Merchant Dashboard, PaymentEngine, SettlementService, UpiProcessor, Vault, SettlementService (+7 more)

### Community 13 - "Community 13"
Cohesion: 0.15
Nodes (16): AnalyticsSummary, AnalyticsSummaryRow, DayVolume, DayVolumeRow, extract_merchant_id(), get_actionable_alerts(), get_dashboard_stats(), get_top_customers() (+8 more)

### Community 14 - "Community 14"
Cohesion: 0.17
Nodes (13): handle_retry(), process_webhook_queue(), queue_webhook(), sign_webhook(), trigger_dispute_webhook(), trigger_payment_webhook(), trigger_refund_webhook(), trigger_settlement_webhook() (+5 more)

### Community 15 - "Community 15"
Cohesion: 0.14
Nodes (12): ApiKeyResponse, BusinessType, CreateMerchantRequest, ForgotPasswordRequest, LoginRequest, LoginResponse, Merchant, MerchantPaymentMethod (+4 more)

### Community 16 - "Community 16"
Cohesion: 0.14
Nodes (13): ApprovalRequest, CreateApprovalRequest, Document, DocumentResponse, GenerateUploadUrlRequest, OtpResponse, OtpVerification, ProcessApprovalRequest (+5 more)

### Community 17 - "Community 17"
Cohesion: 0.29
Nodes (1): SimulatorProcessor

### Community 18 - "Community 18"
Cohesion: 0.2
Nodes (6): CircuitBreakerState, CreateProviderRequest, CreateRoutingRuleRequest, PaymentProvider, ProviderHealthResponse, ProviderRoutingRule

### Community 19 - "Community 19"
Cohesion: 0.2
Nodes (9): AssignFeePlanRequest, CreateFeePlanRequest, CreateSettlementBatchRequest, FeePlan, FeePlanResponse, MerchantFeeAssignment, SettlementBatch, UpdateFeePlanRequest (+1 more)

### Community 20 - "Community 20"
Cohesion: 0.29
Nodes (8): create_customer(), CustomerListResponse, delete_customer(), get_customer_payment_methods(), list_customers(), merchant_id_from_req(), PaginationMeta, PaginationParams

### Community 21 - "Community 21"
Cohesion: 0.22
Nodes (8): Claims, CreateUserRequest, Permission, Role, RolePermission, User, UserResponse, UserRole

### Community 22 - "Community 22"
Cohesion: 0.25
Nodes (2): ApiError, FeelPayError

### Community 23 - "Community 23"
Cohesion: 0.43
Nodes (6): cancel_mandate(), execute_mandate(), get_mandate(), initiate_mandate(), list_mandates(), merchant_id_from_req()

### Community 24 - "Community 24"
Cohesion: 0.33
Nodes (5): MethodData, PaymentMethod, PaymentMethodProcessor, ProcessorResponse, TransactionContext

### Community 25 - "Community 25"
Cohesion: 0.53
Nodes (4): create_virtual_account(), get_virtual_account(), list_virtual_accounts(), merchant_id_from_req()

### Community 26 - "Community 26"
Cohesion: 0.33
Nodes (3): ApiKeyIpWhitelist, ApiKeyIpWhitelistMiddleware, ApiKeyWhitelistData

### Community 27 - "Community 27"
Cohesion: 0.33
Nodes (6): Admin Panel, Escrow Account, FeelPay, PA/PG License, PCI-DSS, UPI AutoPay

### Community 28 - "Community 28"
Cohesion: 0.4
Nodes (4): BankAccount, CreateBankAccountRequest, CreatePayoutRequest, Payout

### Community 29 - "Community 29"
Cohesion: 0.4
Nodes (3): AdminAuthMiddleware, AdminAuthMiddlewareService, Claims

### Community 30 - "Community 30"
Cohesion: 0.4
Nodes (5): NPCI, PSP, RBI, UPI, VPA

### Community 31 - "Community 31"
Cohesion: 0.5
Nodes (3): Dispute, ListDisputesQuery, SubmitEvidenceRequest

### Community 32 - "Community 32"
Cohesion: 0.5
Nodes (3): CreateTenantRequest, Tenant, UpdateTenantRequest

### Community 33 - "Community 33"
Cohesion: 0.5
Nodes (4): Phase 1: Core Engine, Phase 2: Vault, Phase_3_UPI_Engine, Phase_4_Ledger_Settlements

### Community 34 - "Community 34"
Cohesion: 0.67
Nodes (2): ApiKey, CreateApiKeyRequest

### Community 35 - "Community 35"
Cohesion: 0.67
Nodes (2): KycVerification, MerchantTier

### Community 36 - "Community 36"
Cohesion: 0.67
Nodes (2): LedgerAccount, LedgerEntry

### Community 37 - "Community 37"
Cohesion: 0.67
Nodes (2): CreateRefundRequest, Refund

### Community 38 - "Community 38"
Cohesion: 0.67
Nodes (2): Settlement, SettlementItem

### Community 39 - "Community 39"
Cohesion: 0.67
Nodes (2): CreateVirtualAccountRequest, VirtualAccount

### Community 40 - "Community 40"
Cohesion: 0.67
Nodes (2): CreateMandateRequest, Mandate

### Community 41 - "Community 41"
Cohesion: 0.67
Nodes (0): 

### Community 42 - "Community 42"
Cohesion: 0.67
Nodes (0): 

### Community 43 - "Community 43"
Cohesion: 0.67
Nodes (0): 

### Community 44 - "Community 44"
Cohesion: 1.0
Nodes (1): AuditLog

### Community 45 - "Community 45"
Cohesion: 1.0
Nodes (1): FeeRule

### Community 46 - "Community 46"
Cohesion: 1.0
Nodes (1): ApiDoc

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
Nodes (0): 

### Community 61 - "Community 61"
Cohesion: 1.0
Nodes (0): 

### Community 62 - "Community 62"
Cohesion: 1.0
Nodes (0): 

### Community 63 - "Community 63"
Cohesion: 1.0
Nodes (0): 

### Community 64 - "Community 64"
Cohesion: 1.0
Nodes (0): 

### Community 65 - "Community 65"
Cohesion: 1.0
Nodes (0): 

### Community 66 - "Community 66"
Cohesion: 1.0
Nodes (0): 

### Community 67 - "Community 67"
Cohesion: 1.0
Nodes (0): 

### Community 68 - "Community 68"
Cohesion: 1.0
Nodes (0): 

### Community 69 - "Community 69"
Cohesion: 1.0
Nodes (1): Phase_5_Onboarding

### Community 70 - "Community 70"
Cohesion: 1.0
Nodes (1): Phase_6_Dashboard

## Knowledge Gaps
- **201 isolated node(s):** `AppState`, `ApiKey`, `CreateApiKeyRequest`, `AuditLog`, `CreateCustomerRequest` (+196 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 44`** (2 nodes): `AuditLog`, `audit.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 45`** (2 nodes): `FeeRule`, `fee.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 46`** (2 nodes): `mod.rs`, `ApiDoc`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 47`** (2 nodes): `Sidebar()`, `App.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 48`** (2 nodes): `Login.tsx`, `Login()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 49`** (2 nodes): `Settlements.tsx`, `Settlements()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 50`** (2 nodes): `formatCurrency()`, `Analytics.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 51`** (2 nodes): `Settings.tsx`, `copy()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 52`** (2 nodes): `Payments.tsx`, `SkeletonRow()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 53`** (2 nodes): `MiniChart()`, `Dashboard.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 54`** (2 nodes): `Signup.tsx`, `Signup()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 55`** (2 nodes): `seed_merchant.py`, `create_test_merchant()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 56`** (2 nodes): `App.tsx`, `main.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 57`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 58`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 59`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 60`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 61`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 62`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 63`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 64`** (1 nodes): `vite.config.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 65`** (1 nodes): `main.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 66`** (1 nodes): `Disputes.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 67`** (1 nodes): `vite.config.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 68`** (1 nodes): `vite-env.d.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 69`** (1 nodes): `Phase_5_Onboarding`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 70`** (1 nodes): `Phase_6_Dashboard`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `main()` connect `Community 10` to `Community 0`, `Community 1`, `Community 2`, `Community 3`, `Community 5`, `Community 8`, `Community 11`?**
  _High betweenness centrality (0.058) - this node is a cross-community bridge._
- **Why does `calculate_fees()` connect `Community 0` to `Community 11`, `Community 15`?**
  _High betweenness centrality (0.032) - this node is a cross-community bridge._
- **Why does `process_webhook_queue()` connect `Community 14` to `Community 0`?**
  _High betweenness centrality (0.027) - this node is a cross-community bridge._
- **Are the 18 inferred relationships involving `main()` (e.g. with `.new()` and `.from_env()`) actually correct?**
  _`main()` has 18 INFERRED edges - model-reasoned connections that need verification._
- **Are the 9 inferred relationships involving `apiRequest()` (e.g. with `handleSubmit()` and `createLink()`) actually correct?**
  _`apiRequest()` has 9 INFERRED edges - model-reasoned connections that need verification._
- **What connects `AppState`, `ApiKey`, `CreateApiKeyRequest` to the rest of the system?**
  _201 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.04 - nodes in this community are weakly interconnected._