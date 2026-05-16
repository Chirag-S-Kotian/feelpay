FROM node:22-bookworm-slim AS builder

ARG VITE_ADMIN_API_BASE_URL
ARG VITE_DASHBOARD_BASE_URL
ARG VITE_ADMIN_DEBUG_TOOLS=false

ENV VITE_ADMIN_API_BASE_URL=$VITE_ADMIN_API_BASE_URL
ENV VITE_DASHBOARD_BASE_URL=$VITE_DASHBOARD_BASE_URL
ENV VITE_ADMIN_DEBUG_TOOLS=$VITE_ADMIN_DEBUG_TOOLS

WORKDIR /app
COPY admin/package.json admin/pnpm-lock.yaml ./
RUN corepack enable && pnpm install --frozen-lockfile
COPY admin ./
RUN pnpm build

FROM nginx:1.27-alpine

COPY deploy/staging/spa-nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/dist /usr/share/nginx/html

EXPOSE 8080
