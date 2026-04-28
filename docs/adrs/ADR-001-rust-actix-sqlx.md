# ADR-001: Rust + Actix + sqlx Core Stack

Status: Accepted  
Date: 2026-04-28

## Context

Payment gateway needs:
1. High throughput.
2. Strong type safety.
3. Low latency under load.
4. Predictable memory behavior.

## Decision

Use Rust with:
1. Actix Web for HTTP server/middleware.
2. sqlx for compile-time checked SQL.
3. PostgreSQL as system-of-record DB.

## Consequences

Positive:
1. Strong safety model for finance-critical paths.
2. Good latency/throughput characteristics.
3. SQL explicit in code, easier query review.

Tradeoffs:
1. Steeper onboarding for non-Rust team members.
2. Longer compile cycles vs dynamic languages.

## Alternatives considered

1. Node.js + TypeScript
2. Go + Gin

Rejected for lower compile-time guarantees on current team constraints.
