# Payments Context (Engine + Vault + Services)

Load when touching: `src/engine/`, `src/vault/`.

## Layout

```text
src/engine/
  pipeline.rs            PaymentEngine orchestration
  payment.rs             payment creation/capture logic
  payout.rs              payout processing
  fees.rs                fee calculation (Decimal-sensitive)
  routing.rs             provider selection
  processor.rs           ProcessorTrait definition
  vault.rs               engine-level vault integration
  processors/
    card.rs              card processor (simulated)
    upi.rs               UPI stub — NO NPCI production connector
    netbanking.rs        netbanking stub
    wallet.rs            wallet stub
    simulator.rs         test payment simulator

src/engine/services/
  ledger.rs              double-entry ledger
  settlement.rs          settlement logic (NO scheduler — T+2 batch manual)
  settlement_advanced.rs advanced settlement features
  subscription.rs        recurring billing
  webhook.rs             webhook dispatch
  notification.rs        notification engine (email + SMS)
  three_ds.rs            3DS authentication flow
  export.rs              data export service (81KB — large)
  report.rs              report generation

src/vault/
  crypto.rs              AES-GCM encryption
  service.rs             tokenization service
  bin_lookup.rs          BIN range lookup
```

## Rules

- All Decimal ops must use checked arithmetic; never `unwrap()`.
- `src/engine/processors/upi.rs` is a stub only — do not add NPCI production connectors.
- T+2 settlement: no scheduler exists. Do not assume or create one.
- `src/api/analytics.rs` is Decimal-sensitive — extra caution.
- Processor implementations follow `ProcessorTrait` in `processor.rs`.
- Fee calculations must match `fee_plans` table schema.

## High-Risk Hubs

PaymentEngine (pipeline.rs), processors, ledger, settlement, fees.
Always trace callers with `rg` before editing.
