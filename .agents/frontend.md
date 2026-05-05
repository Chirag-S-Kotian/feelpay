# Frontend Context (React + TypeScript + Vite)

Load when touching: `dashboard/`, `admin/`.

## Layout

```text
dashboard/src/           Merchant dashboard (React 19 + Vite)
  pages/                 26 page components
  components/            shared UI components
  hooks/                 custom hooks
  App.tsx                router + layout + auth guard
  index.css              global styles (54KB — Spotify-inspired design system)

admin/src/               Company admin panel (React + Vite + Tailwind)
  design.md              admin design tokens
```

## Rules

- Functional components only. No class components.
- Strict TypeScript: no `any`, no `@ts-ignore`, no `as unknown as`.
- Use `pnpm` in both `dashboard/` and `admin/` — never `npm` or `yarn`.
- Follow existing design system in `design.md` and `index.css`.
- Border-radius: pill for buttons, subtle rounding for inputs/cards.
- Do NOT add mocks to `Analytics.tsx`.

## Key Pages (Merchant Dashboard)

Payments, Settlements, Payouts, Customers, Disputes, Exports, PaymentLinks, Webhooks, Settings, Onboarding, Developers, CheckoutPreview.

## Testing

```bash
cd dashboard && pnpm build && pnpm exec tsc --noEmit
cd admin && pnpm build
```
