# FeelPay API Reference

Base URL: `http://localhost:8080` (dev), `https://api.feelpay.in` (prod)  
Version header: `x-api-version: 2026-04-28`

## Authentication

Use one mode per endpoint:

1. JWT Bearer
   - Header: `Authorization: Bearer <jwt>`
2. API key
   - Header: `x-api-key: <merchant_api_key>`

## Health

### `GET /v1/health`

Sample request:
```bash
curl http://127.0.0.1:8080/v1/health
```

Sample response:
```json
{
  "status": "ok"
}
```

## Merchant Auth

### `POST /v1/merchants/register`

Sample request:
```bash
curl -X POST http://127.0.0.1:8080/v1/merchants/register \
  -H "Content-Type: application/json" \
  -d '{
    "name":"Demo Merchant",
    "email":"owner@demo.com",
    "password":"StrongPass123!",
    "business_name":"Demo Retail Pvt Ltd",
    "business_type":"ecommerce"
  }'
```

Sample response:
```json
{
  "id": "9cf9c396-5fc5-4f9f-ae22-e4c0fa3ac15a",
  "name": "Demo Merchant",
  "email": "owner@demo.com",
  "api_key": "fp_test_...",
  "status": "pending_kyc"
}
```

### `POST /v1/merchants/login`

Sample request:
```bash
curl -X POST http://127.0.0.1:8080/v1/merchants/login \
  -H "Content-Type: application/json" \
  -d '{"email":"owner@demo.com","password":"StrongPass123!"}'
```

Sample response:
```json
{
  "token": "<jwt>",
  "merchant_id": "9cf9c396-5fc5-4f9f-ae22-e4c0fa3ac15a",
  "expires_in": 3600
}
```

## Payments

### `POST /v1/payments`

Auth: API key

Sample request:
```bash
curl -X POST http://127.0.0.1:8080/v1/payments \
  -H "Content-Type: application/json" \
  -H "x-api-key: fp_test_xxx" \
  -d '{
    "order_id":"ORD-10001",
    "amount":125000,
    "currency":"INR",
    "payment_method":"card",
    "customer_email":"alice@example.com",
    "customer_phone":"9876543210"
  }'
```

Sample response:
```json
{
  "id":"22e2354f-0266-4d41-b307-5b7ce183a2b9",
  "order_id":"ORD-10001",
  "status":"authorized",
  "amount":125000,
  "currency":"INR"
}
```

### `GET /v1/payments/{id}`

Auth: API key

Sample request:
```bash
curl http://127.0.0.1:8080/v1/payments/22e2354f-0266-4d41-b307-5b7ce183a2b9 \
  -H "x-api-key: fp_test_xxx"
```

Sample response:
```json
{
  "id":"22e2354f-0266-4d41-b307-5b7ce183a2b9",
  "order_id":"ORD-10001",
  "status":"captured",
  "amount":125000,
  "currency":"INR"
}
```

### `POST /v1/payments/{id}/refund`

Auth: API key

Sample request:
```bash
curl -X POST http://127.0.0.1:8080/v1/payments/22e2354f-0266-4d41-b307-5b7ce183a2b9/refund \
  -H "Content-Type: application/json" \
  -H "x-api-key: fp_test_xxx" \
  -d '{"amount":25000,"reason":"customer_request"}'
```

Sample response:
```json
{
  "id":"18e8045f-1296-4f46-8f4f-8f4548edbc3d",
  "status":"processed",
  "amount":25000
}
```

## Checkout

### `POST /v1/checkout/session`

Sample request:
```bash
curl -X POST http://127.0.0.1:8080/v1/checkout/session \
  -H "Content-Type: application/json" \
  -d '{
    "merchant_id":"9cf9c396-5fc5-4f9f-ae22-e4c0fa3ac15a",
    "order_id":"ORD-20001",
    "amount":55000,
    "currency":"INR",
    "customer_email":"alice@example.com",
    "return_url":"https://merchant.example/return"
  }'
```

Sample response:
```json
{
  "session_id":"cs_5f6402a5f4844884bb1e47fa8fdb1f61",
  "checkout_url":"https://pay.clientbrand.com/checkout/cs_5f6402a5f4844884bb1e47fa8fdb1f61",
  "embed_url":"https://pay.clientbrand.com/checkout/embed/cs_5f6402a5f4844884bb1e47fa8fdb1f61",
  "expires_at":"2026-04-28T14:55:10.266352Z"
}
```

## Settlements

### `GET /v1/settlements`

Auth: JWT

Sample request:
```bash
curl http://127.0.0.1:8080/v1/settlements \
  -H "Authorization: Bearer <jwt>"
```

Sample response:
```json
[
  {
    "id":"f94f965e-4bc8-4054-9c8f-2d673f6dbecf",
    "status":"processing",
    "gross_amount_paise":1000000,
    "net_amount_paise":978200
  }
]
```

## Exports

### `POST /v1/exports/jobs`

Auth: JWT

Sample request:
```bash
curl -X POST http://127.0.0.1:8080/v1/exports/jobs \
  -H "Authorization: Bearer <jwt>" \
  -H "Content-Type: application/json" \
  -d '{
    "job_type":"transaction",
    "filters":{"date_range":{"start":"2026-04-01","end":"2026-04-30"}},
    "idempotency_key":"export-apr-2026-txn"
  }'
```

Sample response:
```json
{
  "job_id":"3fc4cdf7-dd56-4a20-a95b-bf7c3f6d2f65",
  "status":"queued"
}
```

## Error shape

Sample error:
```json
{
  "error":"bad_request",
  "message":"Invalid payload"
}
```

Use `swagger-ui` for full schema detail:
- `GET /swagger-ui/`
- `GET /api-docs/openapi.json`
