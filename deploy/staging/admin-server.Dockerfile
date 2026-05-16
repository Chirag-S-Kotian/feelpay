FROM rust:1-bookworm AS builder

WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY admin/server/Cargo.toml admin/server/Cargo.lock ./admin/server/
COPY admin/server/src ./admin/server/src
RUN cargo build --manifest-path admin/server/Cargo.toml --release

FROM debian:bookworm-slim

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates curl libssl3 \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app
RUN mkdir -p uploads
COPY --from=builder /src/admin/server/target/release/feelpay-admin-server ./feelpay-admin-server

EXPOSE 8081
CMD ["./feelpay-admin-server"]
