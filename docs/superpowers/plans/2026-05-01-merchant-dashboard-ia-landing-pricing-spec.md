# Merchant Dashboard + Landing Upgrade Spec

Date: 2026-05-01  
Mode: Superpowers implementation spec

## Goals

1. Fix persistent UX regressions: sidebar stability + page overflow.
2. Improve merchant IA to match Razorpay-style grouped navigation.
3. Improve trust/clarity on auth + landing flows.
4. Expose pricing consistently inside merchant dashboard.
5. Keep profile editing connected to backend DB endpoint.

## Requirements

### Sidebar + Layout

- Sidebar must occupy full viewport height on desktop and mobile drawer.
- Sidebar internal list must scroll independently while logo + environment controls stay visible.
- Navigation should be grouped into clear sections:
  - Core
  - Payment Products
  - Customer Products
  - Account & Settings
- No page-level horizontal overflow on common merchant routes.

### Auth Pages (Login / Signup)

- Brand logo must be larger and visually dominant.
- Hero and form-side branding should remain legible on mobile.
- Existing auth APIs remain unchanged.

### Landing Page

- Add proper navbar information architecture:
  - Deep links: features, pricing, FAQ.
  - Auth CTAs: login, signup.
- Add richer dynamic blocks (config-driven where possible):
  - Feature rails
  - Pricing cards
  - Testimonials
  - FAQ
  - Footer with grouped links
- Preserve fallback content if public config missing.

### Dashboard Pricing

- Add dedicated dashboard pricing route/page.
- Read pricing from `/v1/public/landing-page` for consistency.
- Show fallback pricing if config unavailable.

### Profile Edit + Backend Link

- Profile edit remains on `/settings`.
- Continue using DB-backed endpoint:
  - `PUT /v1/merchants/profile`
- Save behavior must re-fetch profile after update.

## Implementation Notes

- Frontend stack: React + existing CSS system (no forced Tailwind rewrite).
- Backend update limited to default public landing config payload shape.
- Avoid schema migrations for this scope.

## Verification Checklist

- Desktop (1366+) and mobile (375/390/430) layout checks.
- Sidebar:
  - full-height
  - grouped labels visible
  - nav scroll works
- No horizontal body overflow on:
  - settings
  - developers
  - webhooks
  - payments
  - customers
- Landing:
  - navbar anchors work
  - footer visible
  - dynamic sections render
- Pricing dashboard route renders and handles API fallback.
- Build passes (`dashboard`).
