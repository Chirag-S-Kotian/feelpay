# Load Test Runbook (500 TPS)

## Goal

Sustain `500 TPS` for 5 minutes using k6.

## Script

`load/k6-payments.js`

## Command

```bash
BASE_URL=http://127.0.0.1:8080 k6 run --summary-export=load/summary-500tps.json load/k6-payments.js
```

## Pass criteria

1. Scenario rate holds near 500 req/s.
2. `http_req_failed < 1%`
3. `p95(http_req_duration) < 500ms`

## Evidence files

1. `load/summary-500tps.json`
2. Runtime terminal output capture
3. Infra metrics snapshot (CPU, memory, DB connections)

