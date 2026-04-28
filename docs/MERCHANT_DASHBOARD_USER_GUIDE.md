# Merchant Dashboard User Guide

## Access

1. Open dashboard URL.
2. Login with merchant email/password.
3. Complete MFA if enabled.

## Main modules

1. Dashboard  
   - Revenue summary, success rate, alerts.
2. Payments  
   - List payments, filter by status/date/method.
3. Payment Links  
   - Create hosted links, track status, cancel links.
4. Customers  
   - Add/list/delete customers, view saved methods.
5. Settlements  
   - View settlement cycles, net amount, UTR status.
6. Analytics  
   - Volume and performance charts.
7. Developers  
   - API keys, webhook config, integration helpers.
8. Notification Preferences  
   - Channel toggles per event type.
9. Scheduled Reports / Exports  
   - Generate CSV/Excel/PDF, schedule recurring reports.
10. Bank Accounts  
   - Add account, penny-drop verify, set primary account.

## First-day setup flow

1. Update business profile.
2. Submit KYC docs.
3. Add and verify bank account.
4. Configure webhook endpoint + secret.
5. Generate API keys.
6. Run test payment.

## Payment operations

### Create payment link

1. Open `Payment Links`.
2. Click create.
3. Enter amount, expiry, metadata.
4. Share generated URL.

### Refund payment

1. Open `Payments`.
2. Select payment.
3. Click refund.
4. Enter amount + reason.

## Settlement operations

1. Open `Settlements`.
2. Filter by date/status.
3. Match `gross`, `fees`, `net`.
4. Track UTR for payout completion.

## Reports and exports

1. Open `Exports` for ad-hoc download.
2. Open `Scheduled Reports` for recurring email delivery.
3. Use idempotency key for safe retries in API integrations.

## Team management

1. Open `Settings/Team`.
2. Invite member by email.
3. Assign role.
4. Remove or change role when needed.

## Security best practices

1. Rotate API keys periodically.
2. Enable and enforce MFA.
3. Restrict webhook endpoint by signature checks.
4. Review suspicious alerts daily.
5. Keep least-privilege team roles.

## Troubleshooting quick table

1. Payment failed spikes: check provider status, failure reason analytics.
2. Webhook missing: verify endpoint uptime + signature handling.
3. Settlement delay: check hold/dispute flags and settlement status.
4. Login issue: use forgot-password flow, verify MFA clock sync.
