FROM node:22-bookworm-slim AS builder

ARG VITE_API_BASE_URL
ARG VITE_SANDBOX_API_BASE_URL
ARG VITE_LIVE_API_BASE_URL

ENV VITE_API_BASE_URL=$VITE_API_BASE_URL
ENV VITE_SANDBOX_API_BASE_URL=$VITE_SANDBOX_API_BASE_URL
ENV VITE_LIVE_API_BASE_URL=$VITE_LIVE_API_BASE_URL

WORKDIR /app
COPY dashboard/package.json dashboard/pnpm-lock.yaml ./
RUN corepack enable && pnpm install --frozen-lockfile
COPY dashboard ./
RUN pnpm build

FROM nginx:1.27-alpine

COPY deploy/staging/spa-nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/dist /usr/share/nginx/html

EXPOSE 8080
