# Reconciliation

## Design

The canonical reconciliation foundation uses:

- `reconciliation_batches`
- `reconciliation_source_imports`
- `reconciliation_items`
- `reconciliation_mismatches`
- `reconciliation_adjustments`

Batches store metadata only. They do not store raw bank files or raw provider payloads.

## Sources

Supported sources are `internal_ledger`, `settlement_report`, `payout_report`, `bank_statement`, `provider_report`, and `manual_csv`.

## Matching

The current matcher supports:

- transaction/order/provider-reference matching for payments
- settlement UTR matching
- payout reference matching
- duplicate-reference detection
- missing internal record detection
- missing external record detection for `internal_ledger` batches
- amount, status, currency, and date-window mismatches

Money is stored as integer paise and currency defaults to `INR`.

## CSV Shape

Safe sample imports use these fields:

```csv
transaction_id,refund_id,settlement_id,payout_id,amount_paise,currency,utr_reference,provider_reference,order_reference,status
```

Use masked or non-sensitive references only. The platform stores a redacted summary, not the raw row.

## Admin Workflow

Internal admins create batches, inspect matched and mismatched items, mark items reviewed with internal notes, close batches, and export result CSV files. Upload/import, review, close, and export actions are audited.

Settlement and payout hardening adds explicit reconciliation links from company settlement batches, payout batches, and payout items back to reconciliation batches/items when an internal workflow has matched them.

Reconciliation mismatch creation also emits internal risk evaluations so admins can review repeated mismatch patterns without exposing those risk internals to merchants.

## Boundaries

- Bank and provider APIs are still outside this foundation. No live bank/payout API was added.
- Internal payout state is separate from bank confirmation. `paid` requires an explicit manual, test, or provider reference; fake UTRs and implied bank success are not generated.
- Existing legacy reconciliation tables remain for older company-admin flows; the batch/item model is the canonical foundation for new work.
- Manual matching and adjustment workflows are foundational only; they do not move money.
