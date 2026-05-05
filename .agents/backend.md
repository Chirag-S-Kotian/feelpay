# Backend Context (Rust + Actix + sqlx)

Load when touching: `src/api/`, `src/middleware/`, `src/models/`, `src/config.rs`, `src/main.rs`, `src/error.rs`, `src/state.rs`, `src/lib.rs`.

## Layout

```text
src/api/             HTTP routes (30 modules) + admin/ sub-routes
src/api/mod.rs       router config, CORS, middleware wiring
src/middleware/      auth, rbac, rate_limit, ip_whitelist, replay_prevention, velocity_check, maintenance_mode
src/models/          sqlx FromRow structs (26 modules)
src/config.rs        AppConfig from env
src/main.rs          server bootstrap, middleware stack, route mounting
src/error.rs         AppError enum, ResponseError impl
src/state.rs         AppState (PgPool, config, services)
```

## Rules

- Use `?` propagation; never `unwrap()` on DB results or Decimal ops.
- All handlers return `Result<HttpResponse, AppError>`.
- Query params use `#[derive(Deserialize)]` structs; optional fields are `Option<T>`.
- Match existing error variants in `src/error.rs`; add new variants only if no fit.
- Auth middleware extracts `Claims` via `HttpRequest::extensions()`.
- RBAC: roles checked in middleware, not in handler bodies.

## Key Dependencies

- `sqlx` (Postgres, offline mode via `.sqlx/`), `actix-web 4`, `serde`, `chrono`, `rust_decimal`.
- After query changes, run `cargo sqlx prepare` to update `.sqlx/` cache.

## Testing

```bash
cargo check          # type check
cargo test           # unit + integration
```
