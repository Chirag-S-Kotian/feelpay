# Graph Report - /home/chiragskotian/Desktop/payment  (2026-04-26)

## Corpus Check
- 78 files · ~48,015 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 367 nodes · 428 edges · 57 communities detected
- Extraction: 82% EXTRACTED · 18% INFERRED · 0% AMBIGUOUS · INFERRED: 78 edges (avg confidence: 0.8)
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

## God Nodes (most connected - your core abstractions)
1. `main()` - 11 edges
2. `PaymentEngine` - 10 edges
3. `CardProcessor` - 10 edges
4. `UpiProcessor` - 10 edges
5. `SimulatorProcessor` - 9 edges
6. `create_payment()` - 9 edges
7. `refund_payment()` - 8 edges
8. `LedgerService` - 6 edges
9. `SettlementService` - 6 edges
10. `merchant_id_from_req()` - 6 edges

## Surprising Connections (you probably didn't know these)
- `handleSubmit()` --calls--> `apiRequest()`  [INFERRED]
  /home/chiragskotian/Desktop/payment/dashboard/src/pages/Onboarding.tsx → /home/chiragskotian/Desktop/payment/admin/src/hooks/useApi.ts
- `apiRequest()` --calls--> `fetchData()`  [INFERRED]
  /home/chiragskotian/Desktop/payment/admin/src/hooks/useApi.ts → /home/chiragskotian/Desktop/payment/admin/src/pages/AdminDashboard.tsx
- `main()` --calls--> `create_pool()`  [INFERRED]
  /home/chiragskotian/Desktop/payment/admin/server/src/main.rs → /home/chiragskotian/Desktop/payment/src/db/pool.rs
- `request_payout()` --calls--> `process_payout()`  [INFERRED]
  /home/chiragskotian/Desktop/payment/src/api/payouts.rs → /home/chiragskotian/Desktop/payment/src/engine/payout.rs
- `verify_account()` --calls--> `verify_bank_account()`  [INFERRED]
  /home/chiragskotian/Desktop/payment/src/api/payouts.rs → /home/chiragskotian/Desktop/payment/src/engine/payout.rs

## Communities

### Community 0 - "Community 0"
Cohesion: 0.08
Nodes (15): AppConfig, main(), BankAccountRequest, KycSubmissionRequest, merchant_id_from_req(), onboarding_status(), submit_bank_account(), submit_kyc() (+7 more)

### Community 1 - "Community 1"
Cohesion: 0.09
Nodes (9): BinInfo, BinLookup, VaultCrypto, CardVaultService, VaultedCard, VaultLookupRow, CardVaultService, VaultedCard (+1 more)

### Community 2 - "Community 2"
Cohesion: 0.11
Nodes (17): LedgerService, capture_payment(), CapturePaymentRequest, check_idempotency(), create_payment(), CreatePaymentRequest, get_payment(), IdempotencyRecord (+9 more)

### Community 3 - "Community 3"
Cohesion: 0.1
Nodes (17): ApiKeyMiddlewareService<S>, AuthResponse, Claims, generate_jwt(), JwtMiddlewareService<S>, login(), LoginRequest, register() (+9 more)

### Community 4 - "Community 4"
Cohesion: 0.13
Nodes (3): PaymentEngine, TransactionPipeline, UpiProcessor

### Community 5 - "Community 5"
Cohesion: 0.13
Nodes (13): configure(), accept_dispute(), configure(), get_dispute(), list_disputes(), merchant_id_from_req(), submit_evidence(), configure_routes() (+5 more)

### Community 6 - "Community 6"
Cohesion: 0.13
Nodes (9): CapturePaymentRequest, ConnectorResponse, ConnectorType, CreatePaymentRequest, PaymentListQuery, PaymentListResponse, PaymentMethod, PaymentOrder (+1 more)

### Community 7 - "Community 7"
Cohesion: 0.13
Nodes (11): calculate_fees(), FeeCalculation, ApiKeyResponse, CreateMerchantRequest, ForgotPasswordRequest, LoginRequest, LoginResponse, Merchant (+3 more)

### Community 8 - "Community 8"
Cohesion: 0.18
Nodes (14): AnalyticsSummary, AnalyticsSummaryRow, DayVolume, DayVolumeRow, extract_merchant_id(), get_summary(), get_top_customers(), get_volume_chart() (+6 more)

### Community 9 - "Community 9"
Cohesion: 0.31
Nodes (1): SimulatorProcessor

### Community 10 - "Community 10"
Cohesion: 0.29
Nodes (5): fetchData(), handleSubmit(), apiRequest(), getApiKey(), getToken()

### Community 11 - "Community 11"
Cohesion: 0.25
Nodes (1): CardProcessor

### Community 12 - "Community 12"
Cohesion: 0.43
Nodes (5): get_balance(), get_settlement(), list_settlements(), merchant_id_from_req(), SettlementQuery

### Community 13 - "Community 13"
Cohesion: 0.33
Nodes (5): MethodData, PaymentMethod, PaymentMethodProcessor, ProcessorResponse, TransactionContext

### Community 14 - "Community 14"
Cohesion: 0.33
Nodes (1): CreateTenantRequest

### Community 15 - "Community 15"
Cohesion: 0.33
Nodes (4): ApiKeyMiddleware, ApiKeyMiddlewareService, JwtMiddleware, JwtMiddlewareService

### Community 16 - "Community 16"
Cohesion: 0.4
Nodes (3): generate_jwt(), impersonate_merchant(), PlatformStats

### Community 17 - "Community 17"
Cohesion: 0.4
Nodes (2): ApiError, FeelPayError

### Community 18 - "Community 18"
Cohesion: 0.4
Nodes (4): Claims, CreateUserRequest, User, UserResponse

### Community 19 - "Community 19"
Cohesion: 0.4
Nodes (4): BankAccount, CreateBankAccountRequest, CreatePayoutRequest, Payout

### Community 20 - "Community 20"
Cohesion: 0.4
Nodes (0): 

### Community 21 - "Community 21"
Cohesion: 0.4
Nodes (3): AdminAuthMiddleware, AdminAuthMiddlewareService, Claims

### Community 22 - "Community 22"
Cohesion: 0.5
Nodes (3): WebhookConfigRequest, WebhookEvent, WebhookPayload

### Community 23 - "Community 23"
Cohesion: 0.5
Nodes (3): Dispute, ListDisputesQuery, SubmitEvidenceRequest

### Community 24 - "Community 24"
Cohesion: 0.5
Nodes (3): CreateTenantRequest, Tenant, UpdateTenantRequest

### Community 25 - "Community 25"
Cohesion: 0.67
Nodes (2): CreateRefundRequest, Refund

### Community 26 - "Community 26"
Cohesion: 0.67
Nodes (2): Settlement, SettlementItem

### Community 27 - "Community 27"
Cohesion: 0.67
Nodes (2): LedgerAccount, LedgerEntry

### Community 28 - "Community 28"
Cohesion: 0.67
Nodes (2): KycVerification, MerchantTier

### Community 29 - "Community 29"
Cohesion: 0.67
Nodes (2): ApiKey, CreateApiKeyRequest

### Community 30 - "Community 30"
Cohesion: 0.67
Nodes (2): Subscription, SubscriptionInvoice

### Community 31 - "Community 31"
Cohesion: 0.67
Nodes (0): 

### Community 32 - "Community 32"
Cohesion: 0.67
Nodes (0): 

### Community 33 - "Community 33"
Cohesion: 1.0
Nodes (1): AppState

### Community 34 - "Community 34"
Cohesion: 1.0
Nodes (1): AuditLog

### Community 35 - "Community 35"
Cohesion: 1.0
Nodes (1): FeeRule

### Community 36 - "Community 36"
Cohesion: 1.0
Nodes (0): 

### Community 37 - "Community 37"
Cohesion: 1.0
Nodes (0): 

### Community 38 - "Community 38"
Cohesion: 1.0
Nodes (0): 

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

## Knowledge Gaps
- **89 isolated node(s):** `ApiError`, `AppState`, `Merchant`, `CreateMerchantRequest`, `LoginRequest` (+84 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 33`** (2 nodes): `state.rs`, `AppState`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 34`** (2 nodes): `AuditLog`, `audit.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 35`** (2 nodes): `FeeRule`, `fee.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 36`** (2 nodes): `Login.tsx`, `Login()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 37`** (2 nodes): `Settlements.tsx`, `Settlements()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 38`** (2 nodes): `formatCurrency()`, `Analytics.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 39`** (2 nodes): `Settings.tsx`, `copy()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 40`** (2 nodes): `MiniChart()`, `Dashboard.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 41`** (2 nodes): `Signup.tsx`, `Signup()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 42`** (2 nodes): `seed_merchant.py`, `create_test_merchant()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 43`** (2 nodes): `App.tsx`, `main.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 44`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 45`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 46`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 47`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 48`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 49`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 50`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 51`** (1 nodes): `vite.config.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 52`** (1 nodes): `main.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 53`** (1 nodes): `Payments.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 54`** (1 nodes): `Disputes.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 55`** (1 nodes): `vite.config.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 56`** (1 nodes): `vite-env.d.ts`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `main()` connect `Community 0` to `Community 16`, `Community 3`, `Community 4`?**
  _High betweenness centrality (0.083) - this node is a cross-community bridge._
- **Why does `create_payment()` connect `Community 2` to `Community 1`, `Community 4`?**
  _High betweenness centrality (0.075) - this node is a cross-community bridge._
- **Why does `create_pool()` connect `Community 0` to `Community 7`?**
  _High betweenness centrality (0.048) - this node is a cross-community bridge._
- **Are the 9 inferred relationships involving `main()` (e.g. with `.new()` and `.from_env()`) actually correct?**
  _`main()` has 9 INFERRED edges - model-reasoned connections that need verification._
- **What connects `ApiError`, `AppState`, `Merchant` to the rest of the system?**
  _89 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.08 - nodes in this community are weakly interconnected._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.09 - nodes in this community are weakly interconnected._