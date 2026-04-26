# Graph Report - /home/chiragskotian/Desktop/payment  (2026-04-26)

## Corpus Check
- 46 files · ~13,357 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 215 nodes · 232 edges · 41 communities detected
- Extraction: 88% EXTRACTED · 12% INFERRED · 0% AMBIGUOUS · INFERRED: 29 edges (avg confidence: 0.8)
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

## God Nodes (most connected - your core abstractions)
1. `PaymentEngine` - 10 edges
2. `CardProcessor` - 10 edges
3. `UpiProcessor` - 10 edges
4. `SimulatorProcessor` - 9 edges
5. `main()` - 8 edges
6. `create_payment()` - 8 edges
7. `refund_payment()` - 7 edges
8. `LedgerService` - 6 edges
9. `SettlementService` - 6 edges
10. `merchant_id_from_req()` - 6 edges

## Surprising Connections (you probably didn't know these)
- `main()` --calls--> `create_pool()`  [INFERRED]
  /home/chiragskotian/Desktop/payment/src/main.rs → /home/chiragskotian/Desktop/payment/src/db/pool.rs
- `configure_routes()` --calls--> `configure()`  [INFERRED]
  /home/chiragskotian/Desktop/payment/src/api/mod.rs → /home/chiragskotian/Desktop/payment/src/api/disputes.rs

## Communities

### Community 0 - "Community 0"
Cohesion: 0.13
Nodes (4): AppConfig, main(), PaymentEngine, create_pool()

### Community 1 - "Community 1"
Cohesion: 0.11
Nodes (11): CapturePaymentRequest, ConnectorResponse, ConnectorType, CreatePaymentRequest, PaymentListQuery, PaymentListResponse, PaymentMethod, PaymentOrder (+3 more)

### Community 2 - "Community 2"
Cohesion: 0.14
Nodes (4): CardProcessor, CardVaultService, VaultedCard, VaultLookupRow

### Community 3 - "Community 3"
Cohesion: 0.18
Nodes (14): capture_payment(), CapturePaymentRequest, check_idempotency(), create_payment(), CreatePaymentRequest, get_payment(), IdempotencyRecord, list_payments() (+6 more)

### Community 4 - "Community 4"
Cohesion: 0.31
Nodes (1): SimulatorProcessor

### Community 5 - "Community 5"
Cohesion: 0.22
Nodes (7): ApiKeyResponse, CreateMerchantRequest, LoginRequest, LoginResponse, Merchant, MerchantPublic, UpdateMerchantRequest

### Community 6 - "Community 6"
Cohesion: 0.33
Nodes (7): accept_dispute(), configure(), get_dispute(), list_disputes(), merchant_id_from_req(), submit_evidence(), configure_routes()

### Community 7 - "Community 7"
Cohesion: 0.25
Nodes (2): generate_jwt(), login_merchant()

### Community 8 - "Community 8"
Cohesion: 0.31
Nodes (1): UpiProcessor

### Community 9 - "Community 9"
Cohesion: 0.33
Nodes (1): LedgerService

### Community 10 - "Community 10"
Cohesion: 0.29
Nodes (4): ApiKeyMiddleware, ApiKeyMiddlewareService, JwtMiddleware, JwtMiddlewareService

### Community 11 - "Community 11"
Cohesion: 0.33
Nodes (1): SettlementService

### Community 12 - "Community 12"
Cohesion: 0.33
Nodes (5): MethodData, PaymentMethod, PaymentMethodProcessor, ProcessorResponse, TransactionContext

### Community 13 - "Community 13"
Cohesion: 0.4
Nodes (2): ApiError, FeelPayError

### Community 14 - "Community 14"
Cohesion: 0.5
Nodes (3): WebhookConfigRequest, WebhookEvent, WebhookPayload

### Community 15 - "Community 15"
Cohesion: 0.5
Nodes (3): Dispute, ListDisputesQuery, SubmitEvidenceRequest

### Community 16 - "Community 16"
Cohesion: 0.5
Nodes (2): ApiKeyMiddlewareService<S>, JwtMiddlewareService<S>

### Community 17 - "Community 17"
Cohesion: 0.5
Nodes (0): 

### Community 18 - "Community 18"
Cohesion: 0.67
Nodes (2): CreateRefundRequest, Refund

### Community 19 - "Community 19"
Cohesion: 0.67
Nodes (2): Settlement, SettlementItem

### Community 20 - "Community 20"
Cohesion: 0.67
Nodes (2): LedgerAccount, LedgerEntry

### Community 21 - "Community 21"
Cohesion: 0.67
Nodes (2): KycVerification, MerchantTier

### Community 22 - "Community 22"
Cohesion: 0.67
Nodes (0): 

### Community 23 - "Community 23"
Cohesion: 0.67
Nodes (0): 

### Community 24 - "Community 24"
Cohesion: 1.0
Nodes (1): AppState

### Community 25 - "Community 25"
Cohesion: 1.0
Nodes (1): AuditLog

### Community 26 - "Community 26"
Cohesion: 1.0
Nodes (0): 

### Community 27 - "Community 27"
Cohesion: 1.0
Nodes (0): 

### Community 28 - "Community 28"
Cohesion: 1.0
Nodes (0): 

### Community 29 - "Community 29"
Cohesion: 1.0
Nodes (0): 

### Community 30 - "Community 30"
Cohesion: 1.0
Nodes (0): 

### Community 31 - "Community 31"
Cohesion: 1.0
Nodes (0): 

### Community 32 - "Community 32"
Cohesion: 1.0
Nodes (0): 

### Community 33 - "Community 33"
Cohesion: 1.0
Nodes (0): 

### Community 34 - "Community 34"
Cohesion: 1.0
Nodes (0): 

### Community 35 - "Community 35"
Cohesion: 1.0
Nodes (0): 

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

## Knowledge Gaps
- **45 isolated node(s):** `ApiError`, `AppState`, `Merchant`, `CreateMerchantRequest`, `LoginRequest` (+40 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 24`** (2 nodes): `state.rs`, `AppState`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 25`** (2 nodes): `AuditLog`, `audit.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 26`** (2 nodes): `Login.tsx`, `Login()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 27`** (2 nodes): `Settlements.tsx`, `Settlements()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 28`** (2 nodes): `formatCurrency()`, `Analytics.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 29`** (2 nodes): `Onboarding.tsx`, `handleSubmit()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 30`** (2 nodes): `Settings.tsx`, `copy()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 31`** (2 nodes): `MiniChart()`, `Dashboard.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 32`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 33`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 34`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 35`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 36`** (1 nodes): `mod.rs`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 37`** (1 nodes): `vite.config.js`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 38`** (1 nodes): `main.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 39`** (1 nodes): `Payments.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 40`** (1 nodes): `Disputes.tsx`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `create_payment()` connect `Community 3` to `Community 8`, `Community 1`, `Community 2`?**
  _High betweenness centrality (0.098) - this node is a cross-community bridge._
- **Are the 7 inferred relationships involving `main()` (e.g. with `.new()` and `.from_env()`) actually correct?**
  _`main()` has 7 INFERRED edges - model-reasoned connections that need verification._
- **What connects `ApiError`, `AppState`, `Merchant` to the rest of the system?**
  _45 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.13 - nodes in this community are weakly interconnected._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.11 - nodes in this community are weakly interconnected._
- **Should `Community 2` be split into smaller, more focused modules?**
  _Cohesion score 0.14 - nodes in this community are weakly interconnected._