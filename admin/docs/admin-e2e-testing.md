# FeelPay Admin E2E Testing Guide

This directory contains Playwright-based end-to-end tests for the FeelPay Admin Dashboard.

## Prerequisites

- Node.js (v18+)
- `pnpm` installed
- Backend running at `http://127.0.0.1:8081` (or as configured in `.env`)
- Frontend running at `http://localhost:3000`

## Installation

From the `admin/` directory:
```bash
pnpm install
pnpm exec playwright install chromium
```

## Running Tests

### E2E Smoke Tests
```bash
pnpm test:e2e:smoke
```

### Full E2E Suite
```bash
pnpm test:e2e
```

### Interactive UI Mode
```bash
pnpm test:e2e:ui
```

## Test Structure

- `admin-smoke.spec.ts`: Core navigation and load tests.
- `admin-api.spec.ts`: Verification of critical backend routes.
- `admin-quality.spec.ts`: Accessibility (Axe) and visual health.

## CI/CD Integration

The suite is configured to:
- Generate HTML reports in `playwright-report/`
- Capture screenshots on failure.
- Record traces on the first retry.
- Record videos on failure.
