# Admin Integration Center

The Integration Center is an internal company-admin surface for provider setup. It is not merchant-facing and it is not a white-label or reseller feature.

## Adapter Design

Provider kinds are explicit and provider-agnostic:

- `EmailProvider`
- `SmsProvider`
- `OtpProvider`
- `BankPayoutProvider`
- `KycProvider`
- `NotificationProvider`
- `ReconciliationProvider`

`integration_providers` stores kind, environment, enabled state, default flag, priority, timeout/retry policy, health, timestamps, and non-secret JSON config. `integration_provider_secrets` stores encrypted credentials only. `integration_logs` stores redacted action results for health, tests, and failures.

## Secret Handling

- Secrets are encrypted server-side with the existing `VAULT_ENCRYPTION_KEY` AES-GCM pattern.
- Read APIs return masked values only.
- Logs and audit records keep metadata, never plaintext secrets.
- Rotate credentials through the Integration Center instead of editing database rows.
- API keys, passwords, tokens, client secrets, certificates, and bank credentials must stay in secret rows, not provider config JSON.

## Add A Provider

1. Create a provider with kind, `test` or `live` environment, priority, timeout, retry policy, and non-secret config.
2. Add encrypted secrets.
3. Run connection tests.
4. Enable the provider after tests pass.
5. Set one default per provider kind and environment.

## Setup Notes

### Email

Store SMTP host, port, user, from name/email, reply-to, SMTP security (`starttls`, `tls`, or `none`), and template mapping in config. Store SMTP password as a secret. Use test email before enabling.

Template keys should cover onboarding, OTP, payout, refund, dispute, API key, and webhook failure events.

### SMS / OTP

Store sender ID, route type, callback URL, and template mapping in config. Store API keys or tokens as secrets.

Generic HTTP SMS providers use:

- `sms_base_url`
- `sms_http_method` (`POST` or `PUT`)
- `sms_headers_template`
- `sms_body_template`
- `sender_id`
- `route_type`
- `template_mapping`

Supported placeholders are only `{{phone}}`, `{{message}}`, `{{sender_id}}`, `{{route_type}}`, `{{template_id}}`, and `{{token}}`. The admin test path masks phone numbers in logs and never logs tokens or OTP values.

OTP delivery now prefers the default enabled Integration Center SMS provider when present. The existing OTP store already keeps only hashes, expiries, and attempt counts; send endpoints now also enforce a 60 second resend cooldown and no longer log or return OTP values.

#### Twilio

Twilio is the first provider-specific SMS mapping. Use `provider_code=twilio` with:

- config: `account_sid`, `from_number`, `sender_id`, `provider_code`
- encrypted secret: `auth_token`
- optional template mapping: `test`, `otp`, and any business-event keys you need

The adapter posts to Twilio's Messages API, parses the returned message SID and status, and stores only redacted delivery metadata in Integration Center logs. Twilio documents `sid` as the created message identifier and message statuses such as `queued`, `sent`, `failed`, and `delivered`. Do not enable the provider until `account_sid`, `from_number`, and encrypted `auth_token` are configured. Test SMS and Test OTP use the existing admin actions; missing config fails closed with a clear validation error.

Errors store only safe status/code summaries. They do not persist Twilio credentials, OTP values, message text, or full phone numbers.

### Bank / Payout

Use only official bank, acquirer, or payout APIs. Do not scrape bank websites and do not fake live payouts.

The adapter boundary is provider-neutral and requires these operations from any future official implementation: `test_connection`, `create_payout`, `get_payout_status`, `handle_callback`, optional `fetch_reconciliation_report`, `parse_provider_reference`, and `map_provider_error`.

Store client IDs/keys, callback URLs, certificate/key metadata, settlement-account mapping, timeout/retry config, and reconciliation metadata in config. Store client secrets, certificates, and signing keys as encrypted secrets. Bank payout tests are accepted only in `test` mode. Live bank providers cannot be enabled unless an internal admin explicitly approves that action, and live usage stays disabled until a real official provider mapping is implemented.

Callbacks are idempotent by `(provider_id, provider_event_id)`. Payout creation is idempotent by provider plus idempotency key. Provider responses persist only redacted summaries. A payout is not marked `paid` unless an explicit manual/test reference or mapped provider callback/status confirms it.

Provider onboarding checklist:

- official API docs and sandbox credentials
- callback signature scheme and event identifiers
- payout create/status payload mapping
- provider-reference and UTR/reference field mapping
- reconciliation report/API shape if supported
- timeout/retry limits and error catalog
- live approval by internal admin after safe test verification

Current limitation: no official live bank/acquirer payout adapter is configured in this repo, so the only executable adapter is test-only readiness behavior.

### KYC / KYB

Use `provider_kind=kyc` for provider-neutral PAN, GSTIN, and business-name verification adapters.

- Simulator config may use `provider_code=simulator` for safe test-only readiness checks.
- Generic live/test adapters use `provider_code=generic_http`, `base_url`, and encrypted `auth_token`.
- Admin **Test KYC** actions run only against test-mode providers and only with built-in safe sample data.
- Runtime verification results are stored as `verified`, `failed`, `pending`, `provider_error`, or `manual_review_required`.
- Provider attempts persist only masked identifiers, safe response summaries, latency, reference presence, and error metadata. Full PAN, GSTIN, bank account, document numbers, raw provider bodies, tokens, and secrets are never logged.
- Missing or disabled providers fail closed into `manual_review_required`; this adapter does not auto-approve merchants or claim government/RBI approval.
- Current readiness covers PAN, GSTIN, and business/legal-name verification. Bank account and document-metadata verification are intentionally readiness-only until their existing product flows expose a concrete provider contract.

### Merchant Review

- Company admins review merchants before live activation using the lifecycle `draft`, `submitted`, `under_review`, `approved`, `restricted`, `rejected`, or `suspended`.
- Review detail includes masked identifiers, document metadata, bank-account summaries, KYC provider attempts, internal risk notes, and decision history. Internal notes stay admin-only.
- Admin decisions may mark under review, request more information, approve, reject, restrict, suspend, assign a reviewer, or add a risk note. Rejection, restriction, and suspension require reason codes.
- Live API keys and live authenticated flows remain blocked unless both onboarding and review status are `approved`.
- Supported KYC decision notifications remain the existing `kyc_approved` and `kyc_rejected` events. There is no dedicated more-information template/event yet, so the workflow records that action without fabricating an unsupported notification.

### Reconciliation

- New reconciliation work should use the canonical batch/item model documented in `docs/reconciliation.md`.
- Supported source types are internal ledger, settlement report, payout report, bank statement, provider report, and manual CSV.
- Imports store file metadata plus redacted row summaries only; raw bank files and sensitive payloads stay out of the database unless a future secure file-storage design is added.
- This layer identifies mismatches and records review decisions. It does not add live bank or payout APIs.

### Notifications / Webhooks

Use notification providers for outbound delivery and callback/webhook metadata. Store signing tokens as secrets, keep URLs and event mappings in config, and use integration logs for failure inspection.

## Notification Template Manager

Company admins manage transactional templates from **Notification Templates**. Supported event types are onboarding, OTP, KYC approved/rejected, payout processed/failed, refund created/succeeded/failed, dispute opened/deadline, API key created/revoked, webhook failure, and settlement processed/failed.

- Templates are split by `test` or `live` environment and may target `email`, `sms`, or `both`.
- Email templates require a subject and body. SMS templates require an SMS body.
- Allowed variables are extracted only from `{{safe_name}}` placeholders found in the saved template. Preview and test send reject missing or unknown variables instead of executing code.
- Test send uses the selected mapped email/SMS providers. Disabled templates, disabled providers, or missing provider mappings fail closed.
- Live template tests require explicit internal approval; test-mode templates are the normal validation path.
- Template create, update, enable, disable, and test actions are audited. Secrets, OTP values, and full phone numbers are never logged.
- Runtime dispatch currently wires only real existing flows: `otp`, `onboarding`, `kyc_approved`, `kyc_rejected`, `api_key_created`, `api_key_revoked`, `refund_created`, `payout_processed`, `payout_failed`, and terminal `webhook_failure`.
- Company-admin delivery logs expose masked recipients only. Retry payloads remain private to the dispatcher so failed non-OTP deliveries can be replayed without exposing raw addresses in the UI.
- Failed non-OTP deliveries are retried by a backend worker on a one-minute pass up to the configured retry cap. OTP deliveries are never retried by the dispatcher.
- Events not listed above are intentionally not synthesized yet; each needs a real source flow and recipient path before wiring.

Use sample JSON in Preview to verify variables before enabling a template. Test send requires a test recipient email and/or phone number matching the chosen channel.

## Limitations

- SMTP test email uses the configured encrypted credential and performs a real provider delivery attempt.
- `Test default email` resolves the configured default email provider for the selected environment before sending.
- No Twilio credentials are stored in this repo; live Twilio delivery stays disabled until an internal admin saves the required config and encrypted secret.
- Live bank payout mode remains intentionally gated.
- Operational platform notifications are now wired only for the supported runtime event set listed above; other template types remain admin-configurable but inactive until their real flows are connected.

## Postgres Validation

Neon test/dev branches are the preferred online validation target. Local or Docker Postgres remains optional.

1. Create a Neon test/dev branch that is separate from production.
2. Set `TEST_DATABASE_URL` or `NEON_TEST_DATABASE_URL` to that branch URL; include `sslmode=require` when Neon requires it.
3. Keep credentials out of git and never point test variables at the Neon production database.
4. Run `./scripts/validate_test_db.sh`.
5. Refresh SQLx metadata when query macros change: `cargo sqlx prepare --check`.

The validator prefers `TEST_DATABASE_URL`, then `NEON_TEST_DATABASE_URL`, refuses URLs that match `DATABASE_URL` or look production-like, runs migrations against the selected test database, then runs `cargo test`.

If no reachable test database exists, use `SQLX_OFFLINE=true cargo test` as a fallback only. In this workspace, local validation previously failed because `pg_isready` returned `/run/postgresql:5432 - no response` and Docker socket access was unavailable.
