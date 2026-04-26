# Payment Gateway Research Findings

## 1. Indian Payment Ecosystem

### Payment Methods
| Method | Type | Speed | Use Case |
|--------|------|-------|----------|
| UPI | Instant, mobile-first | Real-time | Consumer payments, e-commerce, P2M |
| Cards (Debit/Credit) | Instant | Real-time | Online shopping, subscriptions |
| Net Banking | Bank redirect | Real-time | High-value, corporate transactions |
| Wallets (Paytm, PhonePe) | Instant | Real-time | Frequent, small-ticket |
| IMPS | Instant, 24/7 | Real-time | Urgent inter-bank transfers |
| NEFT | Batch-processed | Delayed | Standard transfers, routine business |
| RTGS | Real-time, high-value | Real-time | Large corporate settlements (min ₹2L) |
| UPI AutoPay | Recurring mandates | Scheduled | Subscriptions, SIPs, EMIs |

### UPI Architecture (NPCI)
- **Four-party model**: Payer PSP → NPCI Switch → Beneficiary Bank → Payee
- **Stateless routing**: NPCI switch = high-throughput, sub-second latency
- **VPA**: Virtual Payment Address replaces sensitive bank details
- **SAGA pattern**: Distributed transaction coordination
- **Integration flows**: Intent (mobile), Dynamic QR, Web Collect, SDK
- **2FA mandatory**: Device binding + UPI PIN
- **UPI AutoPay limits**: ₹15K standard, ₹1L for insurance/SIP/credit cards

## 2. RBI Regulatory Framework (2025 Master Directions)

### PA License Requirements
- **PA-O (Online)**: What we're building
- **Min net worth**: ₹15 crore at application, ₹25 crore by 3rd year
- **Escrow accounts**: Mandatory for merchant funds
- **PCI-DSS + PA-DSS**: Annual compliance
- **CERT-In audits**: Annual cybersecurity audit
- **KYC/CKYCR**: Mandatory merchant due diligence
- **Fit and proper**: Director/promoter criteria

### Compliance Implications for Our Platform
- We're building the SOFTWARE, not seeking license
- Merchants using our platform would need PA license OR
- We operate as a tech provider to licensed PAs
- Still must be PCI-DSS compliant in architecture

## 3. Security Requirements (PCI-DSS v4.0)

### Architecture
- **CDE isolation**: Cardholder Data Environment must be segmented
- **Network segmentation**: Firewalls, VLANs between CDE and other networks
- **Edge tokenization**: Replace PANs with tokens before entering internal network
- **Encryption**: TLS 1.2+ in transit, AES-256 at rest
- **Key management**: Generation, rotation, split knowledge, dual control
- **Access control**: Least privilege, unique IDs
- **Continuous monitoring**: Audit logs, vulnerability scans, annual pen tests

### Tokenization
- Tokens must have no intrinsic value
- Cannot be reverse-engineered
- Vault must be hardened, access-controlled, isolated
- No PANs outside CDE

## 4. Database Schema Patterns

### Core Tables
- **merchants**: id, name, api_key_hash, webhook_url, status
- **transactions**: id, merchant_id, amount, currency, status, type, idempotency_key
- **refunds**: id, original_txn_id, amount, status, idempotency_key
- **settlements**: id, merchant_id, amount, status, date
- **idempotency_keys**: key, merchant_id, request_hash, response_code, response_body

### Best Practices
- Double-entry bookkeeping (immutable ledger)
- NEVER store raw card data
- BigInt/Decimal for amounts (no floats!)
- Heavy indexing on txn_id, merchant_id, idempotency_key
- Row-level locking for concurrency
- UUID for all PKs

## 5. Transaction State Machine

```
Created → Authorized → Captured → Settled
  ↓           ↓           ↓          ↓
Failed      Voided     Refunded   Refunded
```

### States
- **Created**: Record exists, no financial action yet
- **Authorized**: Funds held at customer bank
- **Captured**: Merchant requested fund transfer
- **Settled**: Funds in merchant account
- **Voided**: Auth cancelled, hold released
- **Refunded**: Full/partial reversal
- **Failed/Declined**: Bank rejected

### Design Rules
- Terminal states: Settled, Refunded, Voided, Failed
- Every transition = audit log entry
- Idempotency keys on all mutations
- Reconciliation for timeout handling
- Support partial capture/refund

## 6. Technology Decision: Rust

### Why Rust over Go for Payment Gateway
| Factor | Rust | Go |
|--------|------|-----|
| Latency | Deterministic, no GC pauses | GC spike risk |
| Memory safety | Compile-time guarantees | Runtime |
| Throughput | Superior raw performance | Good, sufficient for I/O |
| Security | Strongest type system | Adequate |
| Learning curve | Steep | Gentle |
| Ecosystem | Growing (Hyperswitch proves viability) | Mature |

**Decision: Rust** — Juspay's Hyperswitch proves Rust works for Indian payments at scale. Memory safety + zero-cost abstractions = ideal for financial processing. Learning curve accepted for long-term gains.

### Reference: Hyperswitch (Juspay)
- Open-source payment orchestrator in Rust
- Apache 2.0 license
- PostgreSQL + Redis
- Smart routing, card vault, control center
- 100+ processor integrations
- PCI-compliant architecture

## 7. Webhook & API Design Patterns

### Webhook Design
- Exponential backoff retry (1s, 2s, 4s, 8s... max 24h)
- Signature verification (HMAC-SHA256)
- Idempotent processing
- Dead letter queue for failed deliveries
- Max retry count (typically 8-12)

### API Design
- RESTful, versioned (/api/v1/)
- API key + secret authentication
- Rate limiting per merchant
- Request signing for sensitive operations
- Comprehensive error codes
- Pagination for list endpoints

## 8. 2026 Research Addendum (India E-commerce Focus)

### Webhook Security Benchmark (2026 docs)
- Signature verification must use raw body
- Timestamped payload signing prevents replay
- HMAC-SHA256 mandatory for integrity
- Internal platform should define one stable signature contract
- Source: industry webhook security standards

### Platform Expectations
- Idempotency on mutation endpoints (`Idempotency-Key`)
- Refund/dispute lifecycle APIs + webhook first architecture
- Test/live key separation and robust error semantics
- Source: global payment platform architecture patterns

### UPI Recurring (NPCI)
- UPI AutoPay supports recurring mandates with category-specific caps
- Baseline cap publicly documented at ₹15,000 and higher cap for specific MCCs
- Source: NPCI AutoPay product overview
