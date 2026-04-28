# FeelPay Webhook Integration Guide

## Overview

FeelPay sends event webhooks to merchant endpoint for payment, refund, settlement, dispute lifecycle.

Signing:
- Header: `X-FeelPay-Signature`
- Algo: HMAC-SHA256 (hex digest)
- Secret: merchant webhook secret

Verify signature before processing payload.

## Endpoint requirements

Merchant endpoint must:
1. Accept `POST`.
2. Return `2xx` within 10s for success.
3. Be idempotent by event `id`.
4. Use HTTPS in production.

## Sample event payload

```json
{
  "id": "evt_3f840f51b57f4f57b0be7a6d3ef5f02e",
  "event_type": "payment.success",
  "merchant_id": "9cf9c396-5fc5-4f9f-ae22-e4c0fa3ac15a",
  "transaction_id": "22e2354f-0266-4d41-b307-5b7ce183a2b9",
  "order_id": "ORD-10001",
  "amount_paise": 125000,
  "currency": "INR",
  "status": "success",
  "created_at": "2026-04-28T12:10:23Z"
}
```

## Expected headers

```http
Content-Type: application/json
X-FeelPay-Signature: <hex_hmac_sha256>
X-FeelPay-Event-Type: payment.success
X-FeelPay-Event-Id: evt_3f840f51b57f4f57b0be7a6d3ef5f02e
```

## Signature verification

Digest input: raw request body bytes  
Digest output: lowercase hex SHA256 HMAC

### Node.js example

```js
import crypto from "crypto";

function verifySignature(rawBody, signature, secret) {
  const expected = crypto
    .createHmac("sha256", secret)
    .update(rawBody)
    .digest("hex");
  return crypto.timingSafeEqual(
    Buffer.from(expected, "hex"),
    Buffer.from(signature, "hex")
  );
}
```

### Python example

```python
import hmac
import hashlib

def verify_signature(raw_body: bytes, signature: str, secret: str) -> bool:
    expected = hmac.new(secret.encode(), raw_body, hashlib.sha256).hexdigest()
    return hmac.compare_digest(expected, signature)
```

## Retry policy

If merchant endpoint returns non-2xx or times out:
1. Retry with exponential backoff.
2. Keep same payload and event id.
3. Stop after max retries, mark failed in delivery logs.

## Idempotency rule

Store processed `event_id`.  
If duplicate event id received, return `200` without reprocessing.

## Test and debug

Merchant webhook test logs:
- `GET /v1/webhooks/test/logs` (merchant scope)

Admin webhook testing UI:
- Admin panel -> `Webhook Testing`

Signature verification utility:
- `POST /v1/webhooks/verify`
- Request body:
```json
{
  "payload": "<base64-encoded-raw-json>",
  "signature": "t=1714302000,v1=<hexsig>",
  "secret": "whsec_..."
}
```

## Security checklist

1. Always verify signature.
2. Reject old timestamped signatures (>5 min skew) if timestamp format used.
3. Use constant-time compare.
4. Keep secrets in vault or env, not code.
5. Log only masked payload for PII.
