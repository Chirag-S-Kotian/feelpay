# ADR-003: Webhook Signature Standard

Status: Accepted  
Date: 2026-04-28

## Context

Webhook channel crosses trust boundary. Need tamper detection and replay resistance options.

## Decision

Standardize on HMAC signatures:
1. Primary: SHA256 hex digest in `X-FeelPay-Signature`.
2. Verification helper supports structured signature variants (`t=...,v1=...`, optional `v2`).
3. Constant-time compare for digest matching.

## Consequences

Positive:
1. Clear integration contract for merchants.
2. Lightweight compute cost.
3. Compatible with most language SDKs.

Tradeoffs:
1. Secret rotation coordination required.
2. Incorrect raw-body handling in clients causes false failures.

## Operational notes

1. Keep secrets in secure env/vault.
2. Enforce timestamp skew checks where timestamped signature used.
3. Recommend idempotent event processing keyed by event id.
