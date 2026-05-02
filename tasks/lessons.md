# Lessons

- Auth/login handlers must not `SELECT *` into broad domain models. Select stable auth columns only, and probe optional schema columns before referencing them. This avoids local DB migration drift turning login into 500.
- After adding API routes, verify live process through backend/proxy. `404` on a route that exists in code often means stale server binary or dev proxy still pointed at old process.
- If create endpoint tolerates Redis down, follow-up endpoints must use same fallback store. Never return successful session creation if pay/status cannot read that session.
- When user asks for “Razorpay-level” UI, research provider capabilities first, then implement full controls: styling, features, payment configuration, method order, simulation states, and preview parity. Avoid thin cosmetic preview.
- For sandbox checkout, do not force simulator connector with real method payload unless pipeline/processor contract supports it. Send simulator payload and pass selected display method separately.
