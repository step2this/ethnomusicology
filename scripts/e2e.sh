#!/bin/bash
set -e
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "=== ST-004 E2E Pipeline ==="

# --- Build Phase (one-time, cached) ---
echo "--- Building Flutter web ---"
cd "$PROJECT_ROOT/frontend" && flutter build web --base-href /

echo "--- Building backend ---"
cd "$PROJECT_ROOT/backend" && cargo build

# --- Test Phase (fast, repeatable) ---
echo "--- Starting backend (DEV_MODE=true) ---"
DATABASE_URL="sqlite:$PROJECT_ROOT/backend/e2e-test.db?mode=rwc" \
DEV_MODE=true \
BIND_ADDRESS=0.0.0.0 \
  "$PROJECT_ROOT/backend/target/debug/ethnomusicology-backend" &
BACKEND_PID=$!
trap "kill $BACKEND_PID 2>/dev/null; rm -f $PROJECT_ROOT/backend/e2e-test.db" EXIT

# Wait for backend to be ready
echo "--- Waiting for backend health check ---"
for i in $(seq 1 30); do
  if curl -sf http://localhost:3001/api/health > /dev/null 2>&1; then
    echo "Backend ready after ${i}s"
    break
  fi
  if [ "$i" -eq 30 ]; then
    echo "ERROR: Backend failed to start within 30s"
    exit 1
  fi
  sleep 1
done

# Run Playwright tests
echo "--- Running Playwright tests ---"
cd "$PROJECT_ROOT/e2e" && npx playwright test

echo "=== E2E Pipeline Complete ==="
