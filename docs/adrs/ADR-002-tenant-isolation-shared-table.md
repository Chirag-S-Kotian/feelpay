# ADR-002: Shared-table Tenant Isolation

Status: Accepted  
Date: 2026-04-28

## Context

Platform supports multi-tenant white-label model. Need:
1. Fast onboarding for new tenants.
2. Lower infra cost early stage.
3. Tenant-scoped access in API and admin.

## Decision

Use shared-table model with `tenant_id` column across tenant-owned entities.

Controls:
1. Auth middleware injects tenant context.
2. Queries apply tenant scope where required.
3. Admin endpoints support tenant-scoped management.

## Consequences

Positive:
1. Simpler operations vs DB-per-tenant.
2. Lower cost for initial scale.
3. Easier cross-tenant analytics (admin-only).

Tradeoffs:
1. Isolation bug risk if tenant filter missed.
2. Strong testing and code review needed.

## Future trigger to revisit

Migrate to stronger isolation model when:
1. Large enterprise tenant contractual isolation needed.
2. Regulatory or data residency demands hard separation.
