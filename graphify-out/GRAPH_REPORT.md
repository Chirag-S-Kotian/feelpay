# Graph Report - /home/chiragskotian/Desktop/payment  (2026-04-28)

## Corpus Check
- 136 files · ~158,017 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 1089 nodes · 1806 edges · 83 communities detected
- Extraction: 70% EXTRACTED · 30% INFERRED · 0% AMBIGUOUS · INFERRED: 550 edges (avg confidence: 0.8)
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
- [[_COMMUNITY_Community 71|Community 71]]
- [[_COMMUNITY_Community 72|Community 72]]
- [[_COMMUNITY_Community 73|Community 73]]
- [[_COMMUNITY_Community 74|Community 74]]
- [[_COMMUNITY_Community 75|Community 75]]
- [[_COMMUNITY_Community 76|Community 76]]
- [[_COMMUNITY_Community 77|Community 77]]
- [[_COMMUNITY_Community 78|Community 78]]
- [[_COMMUNITY_Community 79|Community 79]]
- [[_COMMUNITY_Community 80|Community 80]]
- [[_COMMUNITY_Community 81|Community 81]]
- [[_COMMUNITY_Community 82|Community 82]]

## God Nodes (most connected - your core abstractions)
1. `main()` - 23 edges
2. `apiRequest()` - 19 edges
3. `NotificationService` - 13 edges
4. `merchant_id_from_req()` - 12 edges
5. `create_payment()` - 12 edges
6. `PaymentEngine` - 11 edges
7. `CardProcessor` - 11 edges
8. `send_email()` - 11 edges
9. `ReportService` - 11 edges
10. `BankAccountService` - 11 edges

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
Cohesion: 0.03
Nodes (75): mask_bank_account(), add_bank_account(), AddBankAccountApiRequest, AddBankAccountApiResponse, BankAccountResponse, BankAccountsListResponse, list_bank_accounts(), PaginationQuery (+67 more)

### Community 1 - "Community 1"
Cohesion: 0.04
Nodes (70): get_customer(), accept_dispute(), get_dispute(), list_disputes(), merchant_id_from_req(), submit_evidence(), ExportService, integration_test_notification_delivery_with_preferences() (+62 more)

### Community 2 - "Community 2"
Cohesion: 0.05
Nodes (56): ApiKeyMiddlewareService, ApiKeyMiddlewareService<S>, AuthResponse, captcha(), CaptchaResponse, CaptchaStore, change_password(), ChangePasswordRequest (+48 more)

### Community 3 - "Community 3"
Cohesion: 0.04
Nodes (27): SimulatorProcessor, handleSubmit(), validate(), LedgerService, NetBankingProcessor, PaymentEngine, BatchRefundItem, BatchRefundRequest (+19 more)

### Community 4 - "Community 4"
Cohesion: 0.05
Nodes (16): AccountType, AddBankAccountRequest, BankAccount, BankAccountResponse, BankAccountService, VerificationStatus, BinInfo, BinLookup (+8 more)

### Community 5 - "Community 5"
Cohesion: 0.05
Nodes (28): fetchData(), fetchBankAccounts(), handleDelete(), handleFormClose(), handleSetPrimaryClose(), handleVerifyClose(), createCustomer(), deleteCustomer() (+20 more)

### Community 6 - "Community 6"
Cohesion: 0.06
Nodes (40): ApiKeyMiddleware, BankAccountSubmissionRequest, BrandingRequest, BusinessInfoRequest, get_limits(), get_registration_progress(), initiate_aadhaar_ekyc(), InitiateAadhaarRequest (+32 more)

### Community 7 - "Community 7"
Cohesion: 0.05
Nodes (20): CheckoutSessionResponse, configure(), create_checkout_session(), CreateCheckoutSessionRequest, get_checkout_session(), get_payment_status(), Job, JobQueue (+12 more)

### Community 8 - "Community 8"
Cohesion: 0.07
Nodes (38): BillingCycle, calculate_period(), cancel_subscription(), create_plan(), create_subscription(), CreatePlanRequest, CreateSubscriptionRequest, generate_invoice() (+30 more)

### Community 9 - "Community 9"
Cohesion: 0.07
Nodes (24): CardProcessor, LedgerService, Merchant Dashboard, PaymentEngine, SettlementService, UpiProcessor, Vault, AppConfig (+16 more)

### Community 10 - "Community 10"
Cohesion: 0.12
Nodes (7): CreateScheduledReportRequest, ReportFormat, ReportFrequency, ReportService, ReportType, ScheduledReport, UpdateScheduledReportRequest

### Community 11 - "Community 11"
Cohesion: 0.12
Nodes (12): get_default_preferences(), NotificationDeliveryLog, NotificationPreference, NotificationService, get_notification_preferences(), get_user_notification_preferences(), NotificationPreferenceResponse, NotificationPreferencesListResponse (+4 more)

### Community 12 - "Community 12"
Cohesion: 0.12
Nodes (21): CreateMerchantRequestInput, extract_merchant_id_internal(), get_keys(), get_merchant(), get_profile(), invite_team_member(), InviteTeamMemberRequest, list_payment_methods() (+13 more)

### Community 13 - "Community 13"
Cohesion: 0.09
Nodes (12): AmountRange, CreateExportJobRequest, DateRange, ExportJob, ExportJobType, integration_test_bank_account_verification_flow(), integration_test_end_to_end_transaction_export(), integration_test_webhook_test_delivery() (+4 more)

### Community 14 - "Community 14"
Cohesion: 0.15
Nodes (16): AnalyticsSummary, AnalyticsSummaryRow, DayVolume, DayVolumeRow, extract_merchant_id(), get_actionable_alerts(), get_dashboard_stats(), get_top_customers() (+8 more)

### Community 15 - "Community 15"
Cohesion: 0.17
Nodes (16): configure(), create_approval(), delete_document(), generate_upload_url(), get_approval(), get_document(), list_approvals(), list_documents() (+8 more)

### Community 16 - "Community 16"
Cohesion: 0.17
Nodes (13): handle_retry(), process_webhook_queue(), queue_webhook(), sign_webhook(), trigger_dispute_webhook(), trigger_payment_webhook(), trigger_refund_webhook(), trigger_settlement_webhook() (+5 more)

### Community 17 - "Community 17"
Cohesion: 0.14
Nodes (12): ApiKeyResponse, BusinessType, CreateMerchantRequest, ForgotPasswordRequest, LoginRequest, LoginResponse, Merchant, MerchantPaymentMethod (+4 more)

### Community 18 - "Community 18"
Cohesion: 0.14
Nodes (13): ApprovalRequest, CreateApprovalRequest, Document, DocumentResponse, GenerateUploadUrlRequest, OtpResponse, OtpVerification, ProcessApprovalRequest (+5 more)

### Community 19 - "Community 19"
Cohesion: 0.27
Nodes (4): test_sign_webhook(), test_verify_webhook_signature(), WebhookTestResult, WebhookTestService

### Community 20 - "Community 20"
Cohesion: 0.29
Nodes (1): SimulatorProcessor

### Community 21 - "Community 21"
Cohesion: 0.2
Nodes (9): AssignFeePlanRequest, CreateFeePlanRequest, CreateSettlementBatchRequest, FeePlan, FeePlanResponse, MerchantFeeAssignment, SettlementBatch, UpdateFeePlanRequest (+1 more)

### Community 22 - "Community 22"
Cohesion: 0.29
Nodes (8): create_customer(), CustomerListResponse, delete_customer(), get_customer_payment_methods(), list_customers(), merchant_id_from_req(), PaginationMeta, PaginationParams

### Community 23 - "Community 23"
Cohesion: 0.22
Nodes (8): Claims, CreateUserRequest, Permission, Role, RolePermission, User, UserResponse, UserRole

### Community 24 - "Community 24"
Cohesion: 0.25
Nodes (2): ApiError, FeelPayError

### Community 25 - "Community 25"
Cohesion: 0.25
Nodes (5): CircuitBreakerState, CreateProviderRequest, CreateRoutingRuleRequest, ProviderHealthResponse, ProviderRoutingRule

### Community 26 - "Community 26"
Cohesion: 0.43
Nodes (6): cancel_mandate(), execute_mandate(), get_mandate(), initiate_mandate(), list_mandates(), merchant_id_from_req()

### Community 27 - "Community 27"
Cohesion: 0.29
Nodes (5): get_webhook_test_logs(), PaginationQuery, TestWebhookRequest, TestWebhookResponse, WebhookTestLogsResponse

### Community 28 - "Community 28"
Cohesion: 0.33
Nodes (5): MethodData, PaymentMethod, PaymentMethodProcessor, ProcessorResponse, TransactionContext

### Community 29 - "Community 29"
Cohesion: 0.53
Nodes (4): create_virtual_account(), get_virtual_account(), list_virtual_accounts(), merchant_id_from_req()

### Community 30 - "Community 30"
Cohesion: 0.33
Nodes (3): ApiKeyIpWhitelist, ApiKeyIpWhitelistMiddleware, ApiKeyWhitelistData

### Community 31 - "Community 31"
Cohesion: 0.33
Nodes (6): Admin Panel, Escrow Account, FeelPay, PA/PG License, PCI-DSS, UPI AutoPay

### Community 32 - "Community 32"
Cohesion: 0.4
Nodes (4): BankAccount, CreateBankAccountRequest, CreatePayoutRequest, Payout

### Community 33 - "Community 33"
Cohesion: 0.4
Nodes (0): 

### Community 34 - "Community 34"
Cohesion: 0.5
Nodes (2): fetchExportJobs(), handleExport()

### Community 35 - "Community 35"
Cohesion: 0.5
Nodes (2): fetchLogs(), handleFilterChange()

### Community 36 - "Community 36"
Cohesion: 0.4
Nodes (3): AdminAuthMiddleware, AdminAuthMiddlewareService, Claims

### Community 37 - "Community 37"
Cohesion: 0.4
Nodes (5): NPCI, PSP, RBI, UPI, VPA

### Community 38 - "Community 38"
Cohesion: 0.5
Nodes (3): Dispute, ListDisputesQuery, SubmitEvidenceRequest

### Community 39 - "Community 39"
Cohesion: 0.5
Nodes (3): CreateTenantRequest, Tenant, UpdateTenantRequest

### Community 40 - "Community 40"
Cohesion: 0.67
Nodes (2): fetchExportJobs(), handleExport()

### Community 41 - "Community 41"
Cohesion: 0.5
Nodes (0): 

### Community 42 - "Community 42"
Cohesion: 0.5
Nodes (0): 

### Community 43 - "Community 43"
Cohesion: 0.5
Nodes (0): 

### Community 44 - "Community 44"
Cohesion: 0.5
Nodes (0): 

### Community 45 - "Community 45"
Cohesion: 0.5
Nodes (4): Phase 1: Core Engine, Phase 2: Vault, Phase_3_UPI_Engine, Phase_4_Ledger_Settlements

### Community 46 - "Community 46"
Cohesion: 0.67
Nodes (2): ApiKey, CreateApiKeyRequest

### Community 47 - "Community 47"
Cohesion: 0.67
Nodes (2): KycVerification, MerchantTier

### Community 48 - "Community 48"
Cohesion: 0.67
Nodes (2): LedgerAccount, LedgerEntry

### Community 49 - "Community 49"
Cohesion: 0.67
Nodes (2): CreateRefundRequest, Refund

### Community 50 - "Community 50"
Cohesion: 0.67
Nodes (2): Settlement, SettlementItem

### Community 51 - "Community 51"
Cohesion: 0.67
Nodes (2): CreateVirtualAccountRequest, VirtualAccount

### Community 52 - "Community 52"
Cohesion: 0.67
Nodes (2): CreateMandateRequest, Mandate

### Community 53 - "Community 53"
Cohesion: 0.67
Nodes (0): 

### Community 54 - "Community 54"
Cohesion: 0.67
Nodes (0): 

### Community 55 - "Community 55"
Cohesion: 0.67
Nodes (0): 

### Community 56 - "Community 56"
Cohesion: 1.0
Nodes (1): AuditLog

### Community 57 - "Community 57"
Cohesion: 1.0
Nodes (1): FeeRule

### Community 58 - "Community 58"
Cohesion: 1.0
Nodes (1): ApiDoc

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
Nodes (0): 

### Community 70 - "Community 70"
Cohesion: 1.0
Nodes (0): 

### Community 71 - "Community 71"
Cohesion: 1.0
Nodes (0): 

### Community 72 - "Community 72"
Cohesion: 1.0
Nodes (0): 

### Community 73 - "Community 73"
Cohesion: 1.0
Nodes (0): 

### Community 74 - "Community 74"
Cohesion: 1.0
Nodes (0): 

### Community 75 - "Community 75"
Cohesion: 1.0
Nodes (0): 

### Community 76 - "Community 76"
Cohesion: 1.0
Nodes (0): 

### Community 77 - "Community 77"
Cohesion: 1.0
Nodes (0): 

### Community 78 - "Community 78"
Cohesion: 1.0
Nodes (0): 

### Community 79 - "Community 79"
Cohesion: 1.0
Nodes (0): 

### Community 80 - "Community 80"
Cohesion: 1.0
Nodes (0): 

### Community 81 - "Community 81"
Cohesion: 1.0
Nodes (1): Phase_5_Onboarding

### Community 82 - "Community 82"
Cohesion: 1.0
Nodes (1): Phase_6_Dashboard

## Knowledge Gaps
- **244 isolated node(s):** `AppState`, `ApiKey`, `CreateApiKeyRequest`, `AuditLog`, `CreateCustomerRequest` (+239 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 56`** (2 nodes): `AuditLog`, `audit.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 57`** (2 nodes): `FeeRule`, `fee.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 58`** (2 nodes): `mod.rs`, `ApiDoc`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 59`** (2 nodes): `Sidebar()`, `App.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 60`** (2 nodes): `Login.tsx`, `Login()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 61`** (2 nodes): `Settlements.tsx`, `Settlements()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 62`** (2 nodes): `formatCurrency()`, `Analytics.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 63`** (2 nodes): `Settings.tsx`, `copy()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 64`** (2 nodes): `Payments.tsx`, `SkeletonRow()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 65`** (2 nodes): `MiniChart()`, `Dashboard.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 66`** (2 nodes): `Signup.tsx`, `Signup()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 67`** (2 nodes): `seed_merchant.py`, `create_test_merchant()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 68`** (2 nodes): `App.tsx`, `main.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 69`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 70`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 71`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 72`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 73`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 74`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 75`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 76`** (1 nodes): `vite.config.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 77`** (1 nodes): `main.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 78`** (1 nodes): `Disputes.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 79`** (1 nodes): `vite.config.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 80`** (1 nodes): `vite-env.d.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 81`** (1 nodes): `Phase_5_Onboarding`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 82`** (1 nodes): `Phase_6_Dashboard`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `handleSubmit()` connect `Community 3` to `Community 5`?**
  _High betweenness centrality (0.062) - this node is a cross-community bridge._
- **Why does `apiRequest()` connect `Community 5` to `Community 3`?**
  _High betweenness centrality (0.055) - this node is a cross-community bridge._
- **Are the 18 inferred relationships involving `main()` (e.g. with `.new()` and `.from_env()`) actually correct?**
  _`main()` has 18 INFERRED edges - model-reasoned connections that need verification._
- **Are the 15 inferred relationships involving `apiRequest()` (e.g. with `handleSubmit()` and `createLink()`) actually correct?**
  _`apiRequest()` has 15 INFERRED edges - model-reasoned connections that need verification._
- **Are the 7 inferred relationships involving `create_payment()` (e.g. with `.to_string()` and `.vault_card()`) actually correct?**
  _`create_payment()` has 7 INFERRED edges - model-reasoned connections that need verification._
- **What connects `AppState`, `ApiKey`, `CreateApiKeyRequest` to the rest of the system?**
  _244 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.03 - nodes in this community are weakly interconnected._