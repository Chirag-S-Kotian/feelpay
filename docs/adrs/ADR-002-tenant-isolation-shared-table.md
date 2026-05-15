# ADR-002: Shared-table Tenant Isolation

Status: Accepted  
Date: 2026-04-28

## Context

Platform uses shared-table internal isolation. Need:
1. Fast onboarding for new merchants and internal platform scopes.
2. Lower infra cost early stage.
3. Internal scope-aware access in API and platform admin.

## Decision

Use shared-table model with `tenant_id` as an internal platform-scoping column across applicable entities.

Controls:
1. Auth middleware injects internal scope context.
2. Queries apply internal scope where required.
3. Platform-admin endpoints may use internal scope management.

## Consequences

Positive:
1. Simpler operations vs DB-per-tenant.
2. Lower cost for initial scale.
3. Easier cross-scope analytics (platform-admin only).

Tradeoffs:
1. Isolation bug risk if an internal scope filter is missed.
2. Strong testing and code review needed.

## Future trigger to revisit

Migrate to stronger isolation model when:
1. Contractual isolation requirements exceed the shared-table model.
2. Regulatory or data residency demands hard separation.
