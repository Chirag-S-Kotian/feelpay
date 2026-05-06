#!/bin/bash

# FeelPay Admin E2E Smoke Runner
# This script ensures the admin dashboard is healthy before deployment.

set -e

echo "🚀 Starting FeelPay Admin E2E Smoke Tests..."

# Check if admin directory exists
if [ ! -d "admin" ]; then
  echo "❌ Error: admin directory not found. Run this from project root."
  exit 1
fi

cd admin

# Ensure dependencies are installed
if [ ! -d "node_modules" ]; then
  echo "📦 Installing dependencies..."
  pnpm install
fi

# Run the smoke tests
echo "🧪 Running smoke tests..."
pnpm test:e2e:smoke

echo "✅ Smoke tests completed successfully!"
