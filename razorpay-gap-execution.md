# FeelPay Razorpay Gap Execution

## Goal
Close highest-impact dashboard parity gaps vs Razorpay merchant workflow using legal/public references, then ship production-ready UX + API coverage.

## Tasks
- [x] Baseline capability map from public Razorpay docs (dashboard, settings, team, webhooks, disputes, settlements) -> Verify: `findings.md` + feature list exists in this plan.
- [x] Add developer webhook observability panel (recent deliveries, attempts, status) -> Verify: `/developers` shows "Recent Webhook Deliveries" table populated from `/v1/developers/webhooks`.
- [x] Add merchant team management in Settings (invite, role update, remove) -> Verify: `/settings` can call `/v1/merchants/team*` endpoints and update table.
- [x] Add actionable risk/ops alerts in Dashboard -> Verify: `/dashboard` shows "Action Required" cards from `/v1/analytics/alerts`.
- [x] Add API request logs page (filters: status, endpoint, date; detail drawer) -> Verify: `/developers` includes API logs + copyable request-id.
- [x] Add settlement deep-dive parity (gross/fees/tax/net reconciliation breakdown) -> Verify: `/settlements` shows computed breakdown + exportable CSV.
- [x] Add disputes operations parity (SLA timer, evidence checklist templates, retry upload) -> Verify: `/disputes` supports evidence workflow with clear status transitions.
- [x] Add onboarding/live-governance parity (live switch preflight, blockers, compliance checklist) -> Verify: live mode switch blocked until checks pass.
- [x] Performance pass for all new flows (cache, parallel fetch, skeleton) -> Verify: route transition improved; duplicate GET storms reduced via cache+dedeupe+retry.
- [x] Final verification + regression -> Verify: `npm --prefix dashboard run build` and `SQLX_OFFLINE=true cargo check` pass.

## Done When
- [x] Core merchant workflows now cover monitor, configure, collaborate, reconcile, dispute response, go-live gating.
- [x] No blocker UX gaps on Developers, Settings, Dashboard, Settlements, Disputes, Onboarding for implemented parity scope.
- [x] Performance + reliability checks pass for build + backend compile gates.

## Notes
- Direct scrape of `https://dashboard.razorpay.com/app/dashboard` blocked by login redirect; used public docs/features as reference baseline.
- Next implementation slice priority: API Logs + Settlement reconciliation breakdown.
