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

The current admin model does not yet define a safe generic HTTP SMS request schema for URL, headers, and body substitution, so real generic SMS/OTP dispatch is intentionally not enabled in this pass. Delivery callbacks can be recorded through the notification provider when available once a concrete adapter exists.

### Bank / Payout

Use only official bank, acquirer, or payout APIs. Do not scrape bank websites and do not fake live payouts.

Store client IDs, callback URLs, settlement-account mapping, and reconciliation metadata in config. Store client secrets, certificates, and signing keys as encrypted secrets. Bank payout tests are accepted only in `test` mode. Live bank providers cannot be enabled unless an internal admin explicitly approves that action.

UTR/reference tracking and reconciliation file/API settings belong in config plus integration logs.

### KYC / KYB

Use generic provider config for PAN, GSTIN, and business verification adapters where supported. Run only with safe test data unless a separately approved live integration exists.

### Notifications / Webhooks

Use notification providers for outbound delivery and callback/webhook metadata. Store signing tokens as secrets, keep URLs and event mappings in config, and use integration logs for failure inspection.

## Limitations

- SMTP test email uses the configured encrypted credential and performs a real provider delivery attempt.
- `Test default email` resolves the configured default email provider for the selected environment before sending.
- Generic SMS/OTP dispatch is still disabled because the current model does not yet define a safe request schema.
- Live bank payout mode remains intentionally gated.
- Operational platform notifications are not yet rewired to select the default Integration Center provider automatically; that remains a separate integration step beyond the admin test path.

## Validation Note

Real migrated local Postgres validation was not available in this workspace because `pg_isready` reported `/run/postgresql:5432 - no response`. The required fallback is `SQLX_OFFLINE=true cargo test` until a reachable migrated local database is provided.
