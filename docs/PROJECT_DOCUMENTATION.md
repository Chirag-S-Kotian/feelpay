# FEELPAY: High-Performance Multi-Tenant Payment Gateway & Aggregator Platform

## Technical Specification & System Documentation

---

## 1. Executive Summary

**FeelPay** is a production-grade, high-concurrency, multi-tenant payment gateway and payment aggregator platform built specifically for the Indian fintech ecosystem. The platform gives merchants sovereign ownership over transaction processing, smart payment routing, automated KYB/KYC onboarding, Indian tax compliance (TDS & GST), double-entry ledger bookkeeping, and instant settlements.

Unlike standard thin API wrappers around third-party processors, FeelPay implements a full-featured transactional engine from scratch, designed to handle sub-50ms API latencies, zero-data-loss ledger consistency, and strict PCI-DSS & DPDP Act compliance.

- **Primary Backend**: Rust (Actix-Web, SQLx, Tokio)
- **Primary Frontend**: React 19, TypeScript, Vite, Tailwind CSS / Vanilla CSS
- **Admin Operations Panel**: React 19, Vite, Express/Rust Admin API
- **Persistence**: Serverless PostgreSQL (Neon, Mumbai `ap-south-1` region)
- **In-Memory Store & Velocity Limiting**: Redis
- **Cloud Infrastructure**: Render (Rust Gateway) + Cloudflare Pages (Frontend Dashboards)

---

## 2. Motivation & Key Differentiators

The Indian digital payments market is heavily centralized around proprietary aggregators (e.g., Razorpay, Cashfree, PayU). This introduces major bottlenecks for modern businesses:
1. **High Transaction Fees**: 2%–3% flat margins plus 18% GST.
2. **Arbitrary Account Freezes**: Regulatory audits frequently lock merchant funds for weeks without developer recourse.
3. **Manual Tax Reconciliations**: Section 194O of the Indian Income Tax Act requires 1% Tax Deducted at Source (TDS) on gross e-commerce payments. Conventional gateways do not automate these calculations atomically in the ledger.
4. **Vulnerability to Card Testing & Replay Attacks**: Traditional systems lack integrated multi-layer fraud and velocity defenses at the gateway edge.

### The FeelPay Solution:
- **Rust-Powered Gateway Core**: Zero-cost abstractions, memory safety without garbage collection pauses, and compile-time thread safety.
- **Automated Indian Tax & Fee Engine**: Atomic computation of platform fees, 18% GST on fees, and 1% TDS under Section 194O at transaction execution.
- **Smart Routing & Failover**: Dynamic connector routing across Razorpay, Stripe, and Mock PSPs based on real-time success rates, latency, and cost weights.
- **Strict Tenant Isolation**: Row-Level Security (RLS) and scoped middleware preventing cross-tenant data leaks.
- **Defense-in-Depth Fraud Prevention**: Velocity rules, BIN/IP/domain blacklists, cryptographic nonces, SHA-256 device fingerprinting, and 3DS2 risk simulation.

---

## 3. High-Level Architecture & Data Flow

```mermaid
flowchart TD
    subgraph ClientLayer [Client & Merchant Interfaces]
        C[End Customer Browser / Checkout]
        M[Merchant Dashboard - React 19]
        A[Admin Operations Panel - React 19]
    end

    subgraph EdgeSecurity [Gateway Edge & Security Layer]
        GW[Rust Gateway - Actix Web :8080]
        AUTH[Auth Middleware - JWT & API Key]
        REPLAY[Replay Attack Prevention - Nonce & 5-min Window]
        VELOCITY[Velocity Limiter - Redis Rate Checker]
        BLACK[Blacklist Engine - IP / BIN / Domain]
    end

    subgraph CoreEngines [FeelPay Core Processing Engines]
        ORDER[Order & Payment Orchestrator]
        ROUTER[Smart Routing Engine]
        VAULT[PCI-DSS Card Vault - AES-256-GCM]
        THREEDS[3DS2 Challenge Simulator]
        LEDGER[Double-Entry Ledger Engine]
        SETTLE[Settlement Engine - T+0 & T+2]
    end

    subgraph ExternalConnectors [Acquiring Connectors & Services]
        RP[Razorpay Connector]
        ST[Stripe Connector]
        SIM[Mock / Bank Simulator]
        NOTIF[Notifications - SendGrid & Twilio]
    end

    subgraph PersistenceLayer [Data Storage]
        PG[(PostgreSQL - Neon Mumbai ap-south-1)]
        RD[(Redis In-Memory Cache)]
    end

    C -->|HTTPS POST /v1/payments| GW
    M -->|HTTPS REST API /v1/*| GW
    A -->|Admin Operations /v1/admin/*| GW

    GW --> AUTH --> REPLAY --> VELOCITY --> BLACK --> ORDER
    ORDER --> ROUTER
    ROUTER -->|Select Connector| RP
    ROUTER -->|Select Connector| ST
    ROUTER -->|Test / Fallback| SIM
    ORDER --> VAULT
    ORDER --> THREEDS
    ORDER -->|State Transition & Double Entry| LEDGER
    LEDGER -->|Atomic Mutations| PG
    VELOCITY <--> RD
    BLACK <--> RD
    SETTLE -->|Schedule / Instant Payout| PG
    ORDER -->|Async Webhook Dispatch| NOTIF
```

---

## 4. Core System Modules

### 4.1 Payment Orchestrator & Smart Routing
- **Order Creation**: Initiates a payment order with currency, minor units (`amount_in_paise`), receipt reference, and notes.
- **Payment Attempts**: Supports multiple attempts per order. Prevents duplicate charges via idempotency keys.
- **Smart Router**: Scores available connectors (Razorpay, Stripe, Simulated) using:
  $$\text{Score} = (w_{\text{success}} \times S) - (w_{\text{latency}} \times L) - (w_{\text{fee}} \times F)$$
  If the primary connector fails with a transient error, the engine seamlessly cascades to the secondary provider without breaking the merchant session.

### 4.2 3DS2 Challenge Flow & Security Vault
- **Card Vaulting**: Sensitive Primary Account Numbers (PAN), CVVs, and expiry details are encrypted using **AES-256-GCM** with 96-bit authentication tags and unique initialization vectors (IVs).
- **3DS2 Challenge Simulator**: Implements an Access Control Server (ACS) challenge protocol.
  - Generates signed JWT challenge tokens using `HS256`.
  - Serves an interactive OTP challenge modal.
  - Verifies challenge response and validates step-up authentication.

### 4.3 Double-Entry Ledger & Indian Tax Engine
All monetary values are strictly tracked in minor units (`paise`, where ₹100.00 = 10,000 paise) as 64-bit integers to eliminate floating-point imprecision.
When a transaction of amount $A$ succeeds:
1. **Gross Credit**: Merchant balance credited with $A$.
2. **Platform Fee Debit**: Calculated as:
   $$\text{Fee} = \text{Flat Fee} + (\text{Percentage} \times A)$$
3. **GST Debit**: 18% Goods and Services Tax charged on the platform fee:
   $$\text{GST} = \text{Fee} \times 0.18$$
4. **TDS Withholding (Section 194O)**: 1% Tax Deducted at Source on the gross transaction value:
   $$\text{TDS} = A \times 0.01$$
5. **Net Settled Balance**:
   $$\text{Net Balance} = A - \text{Fee} - \text{GST} - \text{TDS}$$

### 4.4 Automated KYB/KYC Merchant Onboarding
A multi-state onboarding state machine governs compliance:
- **States**: `draft` $\rightarrow$ `submitted` $\rightarrow$ `under_review` $\rightarrow$ `approved` / `rejected` / `restricted`
- **Verification Criteria**:
  - 10-character Alphanumeric Permanent Account Number (PAN) validation.
  - 15-character Goods and Services Tax Identification Number (GSTIN) structure matching.
  - Indian Financial System Code (IFSC) bank branch format validation.
  - Business proof & authorized signatory document uploads.
  - Dedicated Admin Verification Console for approval/rejection audits.

### 4.5 Settlements & Reconciliation Engine
- **Standard Settlements**: T+2 scheduled automatic batched disbursements to merchant bank accounts.
- **Instant Settlements (T+0)**: Direct UPI payouts into verified VPAs with real-time balance deduction and ledger locking.
- **Reconciliation Worker**: Automatically ingests bank settlement clearing files (CAMT.053 / CSV), matches bank Reference Numbers (UTR) against internal `payment_attempts`, highlights variance anomalies, and exports downloadable CSV/PDF reports.

### 4.6 Defense-in-Depth Fraud Engine
- **Replay Protection**: Cryptographic nonce validation with strict 5-minute time windows (`X-FeelPay-Timestamp`, `X-FeelPay-Nonce`).
- **Velocity Limiters**: Sliding-window rate limiters in Redis checking transaction frequencies per IP, per customer email, and per card BIN.
- **Blacklist Middleware**: Sub-millisecond filtering against blacklisted IP ranges, high-risk email domains, and fraud-reported card BINs.
- **Device Fingerprinting**: Browser parameter hashing (User-Agent, Canvas, WebGL, screen resolution) via SHA-256 to flag session anomalies.

### 4.7 Webhook Dispatcher
- **Signatures**: Every outbound webhook request is signed with the merchant's webhook secret via HMAC-SHA256 (`X-FeelPay-Signature`).
- **Retry Mechanism**: Async worker retries failed endpoints using exponential backoff (1m, 5m, 15m, 1h) up to 5 attempts before marking as dead.

---

## 5. Database Architecture

The PostgreSQL schema is structured across several key relational tables:

| Table | Purpose | Key Columns |
|---|---|---|
| `merchants` | Master merchant entity & status | `id`, `name`, `email`, `status`, `kyc_status`, `created_at` |
| `merchant_credentials` | Dual-environment API keys | `merchant_id`, `environment`, `api_key_hash`, `salt` |
| `payment_orders` | High-level payment intentions | `id`, `merchant_id`, `amount`, `currency`, `receipt`, `status` |
| `payment_attempts` | Concrete transaction attempts | `id`, `order_id`, `connector`, `method`, `status`, `error_code` |
| `ledger_entries` | Double-entry balance movements | `id`, `merchant_id`, `type`, `amount`, `balance_after`, `reference_id` |
| `settlements` | Payout records to merchant banks | `id`, `merchant_id`, `gross_amount`, `fee`, `tax`, `net_amount`, `status` |
| `vault_tokens` | Encrypted card tokenization entries | `id`, `merchant_id`, `token`, `encrypted_payload`, `iv`, `tag`, `bin` |
| `webhook_endpoints` | Merchant webhook configurations | `id`, `merchant_id`, `url`, `secret`, `is_active` |
| `webhook_deliveries` | Event dispatch audit log | `id`, `endpoint_id`, `event_type`, `payload`, `response_code`, `status` |
| `disputes` | Chargebacks and dispute cases | `id`, `payment_id`, `merchant_id`, `reason`, `amount`, `status` |

---

## 6. API Reference (Core Endpoints)

**Base URL**: `http://localhost:8080` (Local) / `https://feelpay-gateway.onrender.com` (Production)  
**Authentication**:
- Merchant Session: `Authorization: Bearer <JWT_TOKEN>`
- Developer API: `x-api-key: <MERCHANT_API_KEY>`

### 6.1 Authentication & Health
- `GET /v1/health` - Gateway health check and database ping.
- `POST /v1/auth/login` - Authenticate merchant user, returns JWT.
- `POST /v1/merchants/register` - Register new merchant entity.

### 6.2 Orders & Payments
- `POST /v1/orders` - Create a new payment order.
  ```json
  {
    "amount": 50000,
    "currency": "INR",
    "receipt": "order_rcptid_11",
    "notes": { "item": "Standard Plan" }
  }
  ```
- `POST /v1/payments` - Submit payment details (Card/UPI/Netbanking).
- `POST /v1/payments/{id}/challenge` - Submit 3DS2 challenge OTP for card authorization.
- `GET /v1/payments/{id}` - Retrieve transaction status and attempt history.

### 6.3 Refunds & Disputes
- `POST /v1/refunds` - Process partial or full refund with ledger reversal.
- `GET /v1/disputes` - List merchant chargeback disputes.
- `POST /v1/disputes/{id}/evidence` - Submit representment documentation.

### 6.4 Settlements & Reports
- `GET /v1/settlements` - List settlement history with tax breakdown.
- `POST /v1/settlements/instant` - Request on-demand instant UPI payout (T+0).
- `GET /v1/reports/reconciliation` - Generate bank reconciliation matching report.

---

## 7. Deployment & Infrastructure

```
                    ┌──────────────────────────────────────┐
                    │            DNS & CDN                │
                    │         Cloudflare Proxy             │
                    └──────────────────┬───────────────────┘
                                       │
            ┌──────────────────────────┴──────────────────────────┐
            ▼                                                     ▼
┌───────────────────────────────┐             ┌───────────────────────────────┐
│       Cloudflare Pages        │             │          Render.com           │
│  feelpay-dashboard.pages.dev  │             │   feelpay-gateway.onrender.com│
│  feelpay-admin.pages.dev      │             │   (Rust Actix Web Service)    │
└───────────────────────────────┘             └───────────────┬───────────────┘
                                                              │
                                            ┌─────────────────┴─────────────────┐
                                            ▼                                   ▼
                            ┌───────────────────────────────┐   ┌───────────────────────────────┐
                            │    Neon Serverless Postgres   │   │         Upstash Redis         │
                            │   Region: ap-south-1 (Mumbai) │   │     Rate Limit & Velocity     │
                            └───────────────────────────────┘   └───────────────────────────────┘
```

- **Gateway Backend**: Render Web Service running compiled Rust binary (`cargo build --release`).
- **Merchant Frontend**: Cloudflare Pages (`feelpay-dashboard.pages.dev`).
- **Admin Frontend**: Cloudflare Pages (`feelpay-admin.pages.dev`).
- **Database**: Serverless PostgreSQL on Neon hosted in Mumbai (`ap-south-1`) for sub-10ms domestic latency.
- **CI/CD Pipeline**: GitHub Actions (`.github/workflows/deploy.yml`) testing and packaging code on push to `main`.

---

## 8. Local Setup & Verification Guide

### 8.1 Prerequisites
- Rust (`rustc` & `cargo` 1.75+)
- Node.js (v18+) & `pnpm`
- PostgreSQL instance (or Neon DB connection string)

### 8.2 Environment Configuration
Create `.env` in the root directory:
```bash
DATABASE_URL=postgresql://USER:PASSWORD@HOST/neondb?sslmode=require
JWT_SECRET=your_secure_256_bit_jwt_secret_key_here
VAULT_ENCRYPTION_KEY=64_hex_chars_representing_32_bytes_aes_key
WEBHOOK_SIGNING_SECRET=whsec_your_webhook_signing_secret
UPI_PSP_ID=FEELPAY
UPI_MERCHANT_VPA=merchant@feelpay
```

### 8.3 Guided Startup
FeelPay includes an interactive runner script:
```bash
./run.sh
```
Or start manually:
```bash
# Terminal 1: Backend
cargo run

# Terminal 2: Merchant Dashboard
cd dashboard && pnpm install && pnpm dev

# Terminal 3: Admin Dashboard
cd admin && pnpm install && pnpm dev
```

### 8.4 Verification Commands
```bash
# Check Rust backend compilation
cargo check

# Verify frontend TypeScript types
cd dashboard && pnpm exec tsc --noEmit

# Build production bundles
cd dashboard && pnpm build
cd admin && pnpm build
```
