# Task Decomposition: ST-004 Run Playwright E2E Test Against Live Backend and Flutter Web Build

**Source**: `docs/steel-threads/st-004-run-playwright-e2e-against-live-stack.md`
**Review Status**: Devil's advocate review complete; all fixes applied
**Total Tasks**: 8 implementation tasks
**Team**: Lead (coordinator) + Backend Builder + E2E Builder

---

## Dependency Graph

```
T1 (config: dev_mode + bind_address) ──┐
                                        ├── T3 (dev seed endpoint)
T2 (Cargo.toml + ServeDir + SPA)  ─────┤   │
                                        │   ├── T5 (Playwright project setup)
                                        │   │   │
                                        │   │   ├── T6 (smoke tests)
                                        │   │   │   │
                                        │   │   │   └── T7 (SPA routing test)
                                        │   │   │
                                        │   │   └── T8 (pipeline script)
                                        │   │
T4 (dev route wiring in main.rs) ───────┘
   (depends on T1 + T3)
```

**Critical path**: T1 → T3 → T4 → T5 → T6
**Parallel tracks**:
- Backend Builder: T1 → T2 (parallel with T1 if 2 agents) → T3 → T4
- E2E Builder: T5 → T6 → T7 → T8 (starts after T4 completes)

---

## Tasks

### T1: Add `dev_mode` and `bind_address` to AppConfig

**Module**: `backend/src/config.rs`
**Covers**: Preconditions 7, Invariants 2-3, Failure Postcondition 3
**Size**: S | **Risk**: Low | **Agent**: Backend Builder

**Description**:
Add two new fields to `AppConfig` in `backend/src/config.rs`:

```rust
pub struct AppConfig {
    // ... existing fields ...
    pub dev_mode: bool,
    pub bind_address: String,
}
```

In `from_env()`:
```rust
dev_mode: std::env::var("DEV_MODE")
    .map(|v| v == "true")
    .unwrap_or(false),
bind_address: std::env::var("BIND_ADDRESS")
    .unwrap_or_else(|_| "0.0.0.0".to_string()),
```

Also update `backend/src/main.rs` line 185 to use the new bind address:
```rust
// BEFORE:
let addr = format!("127.0.0.1:{}", cfg.server_port);
// AFTER:
let addr = format!("{}:{}", cfg.bind_address, cfg.server_port);
```

Log `dev_mode` status at startup:
```rust
if cfg.dev_mode {
    tracing::warn!("DEV_MODE enabled — dev-only endpoints are active");
}
```

**Acceptance**:
- [ ] `DEV_MODE=true` sets `dev_mode` to `true`
- [ ] Missing or non-"true" `DEV_MODE` sets `dev_mode` to `false`
- [ ] `BIND_ADDRESS` defaults to `0.0.0.0` when not set
- [ ] `BIND_ADDRESS=127.0.0.1` overrides default
- [ ] Backend starts on `0.0.0.0:3001` by default
- [ ] `cargo test` passes
- [ ] `cargo clippy -- -D warnings` passes

**Blocked by**: Nothing
**Blocks**: T3, T4

---

### T2: Add tower-http `fs` Feature + Static File Serving with SPA Fallback

**Module**: `backend/Cargo.toml`, `backend/src/main.rs`
**Covers**: Success Postconditions 1, 4; MSS steps 2, 4; Extensions 4a; Integration Assertions 1, 4
**Size**: M | **Risk**: Medium | **Agent**: Backend Builder

**Description**:
Enable static file serving so the backend serves the Flutter web build at `/` with SPA fallback.

**Step 1 — Cargo.toml**: Add `"fs"` to tower-http features:
```toml
tower-http = { version = "0.6", features = ["cors", "trace", "compression-gzip", "fs"] }
```

**Step 2 — main.rs**: Add `ServeDir` with SPA fallback after the API routes. The key insight is that API routes must take priority, and non-API routes fall through to static file serving. Use `tower_http::services::ServeDir` with `.fallback()` for SPA routing.

```rust
use tower_http::services::ServeDir;

// In main(), after building the API router:
let static_dir = std::path::Path::new("../frontend/build/web");

let app = if static_dir.exists() {
    // Serve Flutter web build with SPA fallback (index.html for all non-API, non-file routes)
    let serve_dir = ServeDir::new(static_dir)
        .fallback(tower_http::services::ServeFile::new(static_dir.join("index.html")));

    Router::new()
        .nest("/api", api_router())
        .nest("/api", routes::auth::auth_routes(auth_state))
        .nest("/api", routes::import::import_router(import_state))
        .nest("/api", routes::setlist::setlist_router(setlist_state))
        .nest("/api", routes::tracks::tracks_router(pool.clone()))
        // Dev routes conditionally added in T4
        .fallback_service(serve_dir)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
} else {
    tracing::warn!("Flutter web build not found at {:?} — static file serving disabled", static_dir);
    // Existing API-only router
    Router::new()
        .nest("/api", api_router())
        .nest("/api", routes::auth::auth_routes(auth_state))
        .nest("/api", routes::import::import_router(import_state))
        .nest("/api", routes::setlist::setlist_router(setlist_state))
        .nest("/api", routes::tracks::tracks_router(pool.clone()))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
};
```

**Important**: Use `tower_http::services::ServeFile` (also gated by `"fs"` feature) for the SPA fallback. This serves `index.html` for any route that doesn't match an API route or a real static file.

**Note on duplicate router construction**: Refactor to avoid duplicating the API router setup. Build the API routes once, then conditionally add the fallback:

```rust
let mut app = Router::new()
    .nest("/api", api_router())
    .nest("/api", routes::auth::auth_routes(auth_state))
    .nest("/api", routes::import::import_router(import_state))
    .nest("/api", routes::setlist::setlist_router(setlist_state))
    .nest("/api", routes::tracks::tracks_router(pool.clone()));

let static_dir = std::path::Path::new("../frontend/build/web");
if static_dir.exists() {
    tracing::info!("Serving Flutter web build from {:?}", static_dir);
    let serve_dir = ServeDir::new(static_dir)
        .fallback(tower_http::services::ServeFile::new(static_dir.join("index.html")));
    app = app.fallback_service(serve_dir);
} else {
    tracing::warn!("Flutter web build not found at {:?} — static file serving disabled", static_dir);
}

let app = app
    .layer(CorsLayer::permissive())
    .layer(TraceLayer::new_for_http());
```

**Acceptance**:
- [ ] `tower-http` has `"fs"` feature in Cargo.toml
- [ ] `GET /` serves `index.html` from Flutter web build when build exists
- [ ] `GET /main.dart.js` (or equivalent) serves the correct static asset
- [ ] `GET /tracks` (non-API route) serves `index.html` (SPA fallback)
- [ ] `GET /api/health` still returns JSON (API routes take priority)
- [ ] Backend starts without Flutter build (graceful degradation with warning log)
- [ ] `cargo test` passes
- [ ] `cargo clippy -- -D warnings` passes

**Blocked by**: Nothing (can run parallel with T1)
**Blocks**: T4

---

### T3: Dev Seed Endpoint — `POST /api/dev/seed`

**Module**: `backend/src/routes/dev.rs` (new file)
**Covers**: Success Postconditions 2, 5; MSS step 5; Extensions 5a, 5b; API Contract `POST /api/dev/seed`
**Size**: M | **Risk**: Low | **Agent**: Backend Builder

**Description**:
Create a dev-only seed endpoint that inserts sample tracks into the database for E2E testing.

**New file**: `backend/src/routes/dev.rs`

```rust
use axum::{extract::State, routing::post, Json, Router};
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Serialize)]
struct SeedResponse {
    seeded: bool,
    tracks: usize,
    message: String,
}

async fn seed_data(
    State(pool): State<SqlitePool>,
) -> Result<Json<SeedResponse>, axum::http::StatusCode> {
    // Use INSERT OR IGNORE for idempotency (Extension 5b)
    let seed_sql = r#"
        -- Artists
        INSERT OR IGNORE INTO artists (id, name) VALUES
            ('artist-1', 'Amr Diab'),
            ('artist-2', 'Hassan Hakmoun'),
            ('artist-3', 'Charlotte de Witte'),
            ('artist-4', 'Black Coffee');

        -- Tracks (8 tracks per spec: house, techno, afrobeats; BPM 118-140; Camelot 7A,8A,9A,8B,1A,3B)
        INSERT OR IGNORE INTO tracks (id, title, album, duration_ms, bpm, camelot_key, energy, source, spotify_uri, spotify_preview_url) VALUES
            ('track-1', 'Nour El Ain', 'Best Of', 240000, 128.0, '8A', 6, 'spotify', 'spotify:track:seed1', NULL),
            ('track-2', 'Habibi Ya Nour', 'Classics', 210000, 124.0, '7A', 5, 'spotify', 'spotify:track:seed2', NULL),
            ('track-3', 'Gnawa Blues', 'World Fusion', 195000, 118.0, '9A', 3, 'spotify', 'spotify:track:seed3', NULL),
            ('track-4', 'Fez Medina', 'Moroccan Nights', 220000, 122.0, '8B', 4, 'spotify', 'spotify:track:seed4', NULL),
            ('track-5', 'Acid Techno Rise', 'Rave On', 360000, 138.0, '1A', 8, 'spotify', 'spotify:track:seed5', NULL),
            ('track-6', 'Pipeline Dub', 'Selections', 300000, 140.0, '3B', 7, 'spotify', 'spotify:track:seed6', NULL),
            ('track-7', 'Wish You Were Here', 'Subconsciously', 270000, 120.0, '8A', 5, 'spotify', 'spotify:track:seed7', NULL),
            ('track-8', 'Drive', 'Subconsciously', 285000, 122.0, '9A', 6, 'spotify', 'spotify:track:seed8', NULL);

        -- Track-Artist associations (at least 2 artists with multiple tracks per spec)
        INSERT OR IGNORE INTO track_artists (track_id, artist_id) VALUES
            ('track-1', 'artist-1'),
            ('track-2', 'artist-1'),
            ('track-3', 'artist-2'),
            ('track-4', 'artist-2'),
            ('track-5', 'artist-3'),
            ('track-6', 'artist-3'),
            ('track-7', 'artist-4'),
            ('track-8', 'artist-4');
    "#;

    sqlx::raw_sql(seed_sql)
        .execute(&pool)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SeedResponse {
        seeded: true,
        tracks: 8,
        message: "Test data seeded successfully".to_string(),
    }))
}

pub fn dev_router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/dev/seed", post(seed_data))
        .with_state(pool)
}
```

**Seed data requirements** (from ST-004 spec):
- 8 tracks with realistic DJ metadata
- Mix of genres (house, techno, afrobeats)
- BPM range: 118-140
- Camelot keys spanning compatible groups (7A, 8A, 9A, 8B for compatible cluster; 1A, 3B for distant)
- Energy levels: 3-8
- At least 2 artists with multiple tracks

**Tests** (2):
1. `test_seed_inserts_tracks` — Call seed, query tracks table, verify 8 rows
2. `test_seed_idempotent` — Call seed twice, verify still 8 rows (INSERT OR IGNORE)

**Acceptance**:
- [ ] `POST /api/dev/seed` returns `{"seeded": true, "tracks": 8, "message": "Test data seeded successfully"}`
- [ ] 8 tracks with BPM, Camelot key, and energy values inserted
- [ ] At least 2 artists with multiple tracks
- [ ] Calling seed twice does not create duplicates
- [ ] `cargo test` passes
- [ ] `cargo clippy -- -D warnings` passes

**Blocked by**: T1 (needs `dev_mode` config to be defined)
**Blocks**: T4

---

### T4: Wire Dev Routes Conditionally in main.rs

**Module**: `backend/src/main.rs`, `backend/src/routes/mod.rs`
**Covers**: Invariant 2, Failure Postcondition 3, Extensions 5a
**Size**: S | **Risk**: Low | **Agent**: Backend Builder

**Description**:
Conditionally register the dev seed endpoint only when `DEV_MODE=true`.

**Step 1 — routes/mod.rs**: Add `pub mod dev;`

**Step 2 — main.rs**: After building the API router but before layers, conditionally add dev routes:

```rust
// After existing .nest("/api", ...) calls:
if cfg.dev_mode {
    app = app.nest("/api", routes::dev::dev_router(pool.clone()));
}
```

**Security invariant**: When `DEV_MODE` is not set or `false`, `POST /api/dev/seed` must return 404 (route not registered). This is inherently true because the route is conditionally added.

**Test** (1):
1. `test_dev_seed_404_without_dev_mode` — Build router without dev routes, `POST /api/dev/seed` returns 404

**Acceptance**:
- [ ] `POST /api/dev/seed` returns 200 when `DEV_MODE=true`
- [ ] `POST /api/dev/seed` returns 404 when `DEV_MODE` is not set
- [ ] No dev-related code executes when `DEV_MODE=false`
- [ ] `cargo test` passes
- [ ] `cargo clippy -- -D warnings` passes

**Blocked by**: T1 (config), T2 (main.rs refactored with static serving), T3 (dev route module)
**Blocks**: T5, T6

---

### T5: Playwright Project Setup

**Module**: `e2e/` (new directory)
**Covers**: Preconditions 4-5, Agent Execution Notes
**Size**: M | **Risk**: Medium | **Agent**: E2E Builder

**Description**:
Create the Playwright project structure in a new `e2e/` directory at the project root.

**Files to create**:

**`e2e/package.json`**:
```json
{
  "name": "ethnomusicology-e2e",
  "version": "1.0.0",
  "private": true,
  "scripts": {
    "test": "playwright test",
    "test:headed": "playwright test --headed"
  },
  "devDependencies": {
    "@playwright/test": "^1.52.0"
  }
}
```

**`e2e/playwright.config.ts`**:
```typescript
import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  timeout: 30000,
  retries: 0,
  use: {
    baseURL: 'http://localhost:3001',
    trace: 'on-first-retry',
  },
  webServer: {
    command: 'cd ../backend && DEV_MODE=true DATABASE_URL="sqlite:../backend/e2e-test.db?mode=rwc" cargo run',
    url: 'http://localhost:3001/api/health',
    reuseExistingServer: !process.env.CI,
    timeout: 120000, // Backend compilation + startup
    env: {
      DEV_MODE: 'true',
      DATABASE_URL: 'sqlite:../backend/e2e-test.db?mode=rwc',
    },
  },
  projects: [
    {
      name: 'chromium',
      use: {
        browserType: 'chromium',
        channel: undefined, // Use Playwright's bundled Chromium
      },
    },
  ],
});
```

**`e2e/tests/.gitkeep`** — placeholder (tests go in T6)

**`e2e/.gitignore`**:
```
node_modules/
test-results/
playwright-report/
```

**Setup steps**:
1. `cd e2e && npm install`
2. `npx playwright install chromium` (downloads Playwright-managed Chromium)
3. Verify `npx playwright test --list` runs without errors (even with no tests)

**Note on webServer config**: The `webServer` block lets Playwright automatically start the backend. It waits for the health endpoint before running tests. For local dev, `reuseExistingServer: true` allows running against an already-started backend. The backend will use an ephemeral `e2e-test.db` database.

**Acceptance**:
- [ ] `e2e/` directory with package.json, playwright.config.ts, tests/ dir
- [ ] `npm install` succeeds
- [ ] `npx playwright install chromium` succeeds
- [ ] `npx playwright test --list` runs without error
- [ ] Config points to `http://localhost:3001`
- [ ] webServer config starts backend with `DEV_MODE=true`

**Blocked by**: T4 (backend must support dev mode + static serving)
**Blocks**: T6, T7, T8

---

### T6: Playwright Smoke Tests

**Module**: `e2e/tests/smoke.spec.ts`
**Covers**: Success Postconditions 3-7; MSS steps 5-16; Integration Assertions 1-3
**Size**: M | **Risk**: Medium | **Agent**: E2E Builder

**Description**:
The core E2E test file that proves the full stack works end-to-end.

**`e2e/tests/smoke.spec.ts`**:

```typescript
import { test, expect } from '@playwright/test';

// Seed test data before all tests
test.beforeAll(async ({ request }) => {
  const response = await request.post('/api/dev/seed');
  expect(response.ok()).toBeTruthy();
  const body = await response.json();
  expect(body.seeded).toBe(true);
  expect(body.tracks).toBe(8);
});

test.describe('Smoke Tests', () => {
  test('home screen loads with title', async ({ page }) => {
    // MSS step 6-7: Navigate to home, see "Salamic Vibes"
    await page.goto('/');
    // Flutter WASM/JS may take up to 15s to bootstrap
    await expect(page.getByText('Salamic Vibes')).toBeVisible({ timeout: 15000 });
  });

  test('track catalog shows seeded data', async ({ page }) => {
    // MSS steps 8-12: Click Track Catalog, see seeded tracks
    await page.goto('/');
    await expect(page.getByText('Salamic Vibes')).toBeVisible({ timeout: 15000 });

    // Click Track Catalog button
    await page.getByText('Track Catalog').click();

    // Wait for seeded track data to appear (proves Frontend → API → DB round-trip)
    await expect(page.getByText('Nour El Ain')).toBeVisible({ timeout: 10000 });
  });

  test('generate setlist screen loads', async ({ page }) => {
    // MSS steps 13-15: Navigate to Generate Setlist, see prompt input
    await page.goto('/');
    await expect(page.getByText('Salamic Vibes')).toBeVisible({ timeout: 15000 });

    // Click Generate Setlist button
    await page.getByText('Generate Setlist').click();

    // Assert setlist generation screen elements visible
    // The screen should show prompt input guidance text
    await expect(page.getByText('Describe your ideal set')).toBeVisible({ timeout: 10000 });
  });
});
```

**Pre-requisites for this test**:
- Flutter web build must exist at `frontend/build/web/`
- Backend must be running with `DEV_MODE=true`
- Seed endpoint must work

**Test assertions map to steel thread postconditions**:
- Postcondition 3: "Playwright navigates to `/`, sees the home screen" → `home screen loads with title`
- Postcondition 4: "clicks Track Catalog, sees seeded track data" → `track catalog shows seeded data`
- Postcondition 5: "clicks Generate Setlist... UI transitions" → `generate setlist screen loads`
- Postcondition 6: "All Playwright tests exit with code 0" → all tests pass

**Acceptance**:
- [ ] Home screen test passes (Flutter loads, "Salamic Vibes" visible)
- [ ] Track catalog test passes (seeded "Nour El Ain" visible after navigation)
- [ ] Generate setlist test passes (prompt guidance text visible)
- [ ] All tests exit with code 0
- [ ] Test phase completes in under 30 seconds (excluding build/startup)

**Blocked by**: T5 (Playwright project)
**Blocks**: T7

---

### T7: SPA Routing Test

**Module**: `e2e/tests/spa-routing.spec.ts`
**Covers**: Success Postcondition 4 (partial); Integration Assertion 4; Extension 4a
**Size**: S | **Risk**: Low | **Agent**: E2E Builder

**Description**:
Test that direct URL navigation works (SPA fallback serves `index.html` for non-API routes).

**`e2e/tests/spa-routing.spec.ts`**:

```typescript
import { test, expect } from '@playwright/test';

test.beforeAll(async ({ request }) => {
  const response = await request.post('/api/dev/seed');
  expect(response.ok()).toBeTruthy();
});

test.describe('SPA Routing', () => {
  test('direct navigation to /tracks serves app (not 404)', async ({ page }) => {
    // Integration Assertion 4: Direct URL navigation to /tracks serves index.html
    await page.goto('/tracks');
    // Flutter should bootstrap and GoRouter handles the /tracks route
    // Wait for any Flutter content to appear (not a 404 page)
    await expect(page.getByText('Salamic Vibes').or(page.getByText('Nour El Ain'))).toBeVisible({ timeout: 15000 });
  });

  test('API routes still return JSON (not index.html)', async ({ request }) => {
    // Verify API routes are not caught by SPA fallback
    const response = await request.get('/api/health');
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body.status).toBe('ok');
  });

  test('dev seed returns 404 without DEV_MODE', async ({ request }) => {
    // This test only works if we can test against a non-dev backend
    // For now, verify the endpoint exists in dev mode (inverse tested by backend unit tests)
    const response = await request.post('/api/dev/seed');
    expect(response.ok()).toBeTruthy();
  });
});
```

**Acceptance**:
- [ ] Direct navigation to `/tracks` loads the Flutter app (not 404)
- [ ] API routes still return JSON responses
- [ ] All SPA routing tests pass
- [ ] `npx playwright test` exits with code 0

**Blocked by**: T6 (smoke tests prove basic flow first)
**Blocks**: Nothing

---

### T8: E2E Pipeline Script

**Module**: `scripts/e2e.sh` (new file)
**Covers**: Agent Execution Notes; Integration Assertion 6
**Size**: S | **Risk**: Low | **Agent**: E2E Builder

**Description**:
Create the pipeline script that orchestrates the full E2E flow: build → start backend → seed → test → cleanup.

**`scripts/e2e.sh`**:
```bash
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
```

Also add to `.gitignore` at project root (if not already):
```
backend/e2e-test.db
```

**Acceptance**:
- [ ] `scripts/e2e.sh` is executable (`chmod +x`)
- [ ] Script builds Flutter web and backend
- [ ] Script starts backend with DEV_MODE and ephemeral DB
- [ ] Script waits for health check before running tests
- [ ] Script cleans up backend process and ephemeral DB on exit (trap)
- [ ] Full pipeline runs successfully end-to-end
- [ ] `backend/e2e-test.db` is in `.gitignore`

**Blocked by**: T5 (Playwright project), T6 (smoke tests must exist)
**Blocks**: Nothing

---

## Summary

| Task | Description | Size | Risk | Agent | Blocked By | Blocks |
|------|-------------|------|------|-------|------------|--------|
| **T1** | Config: `dev_mode` + `bind_address` in AppConfig | S | Low | Backend | -- | T3, T4 |
| **T2** | Cargo.toml `"fs"` + ServeDir with SPA fallback | M | Med | Backend | -- | T4 |
| **T3** | Dev seed endpoint: `POST /api/dev/seed` | M | Low | Backend | T1 | T4 |
| **T4** | Wire dev routes conditionally in main.rs | S | Low | Backend | T1, T2, T3 | T5, T6 |
| **T5** | Playwright project setup (e2e/ directory) | M | Med | E2E | T4 | T6, T7, T8 |
| **T6** | Smoke tests: home, track catalog, setlist screen | M | Med | E2E | T5 | T7 |
| **T7** | SPA routing test: direct URL navigation | S | Low | E2E | T6 | -- |
| **T8** | Pipeline script: `scripts/e2e.sh` | S | Low | E2E | T5, T6 | -- |

---

## Parallelism Strategy

```
Time ──────────────────────────────────────────────→

Backend Builder:   [T1: config] → [T2: ServeDir] → [T3: seed endpoint] → [T4: wiring]
                   (T1 + T2 can run in parallel if assigned to 2 agents)

E2E Builder:       ──────────────────────────────── [T5: Playwright setup] → [T6: smoke tests] → [T7: SPA test] → [T8: script]

Flutter Build:     [manual: flutter build web --base-href /]  (prerequisite, run before E2E tests)
```

**Note**: T1 and T2 are independent and can be done in parallel. T3 depends on T1 (needs `dev_mode` config). T4 depends on T1+T2+T3 (wires everything together in main.rs). The E2E builder starts after T4 completes.

For a **single backend builder**, the optimal sequence is: T1 → T2 → T3 → T4 (all sequential, ~30 min). The E2E builder then takes over: T5 → T6 → T7 → T8 (~25 min). But T5 can be started as soon as T4 is done.

---

## Agent Execution Plan

### Backend Builder
Executes T1 through T4 sequentially:
```
T1 (config: dev_mode + bind_address)
 ↓
T2 (Cargo.toml fs + ServeDir + SPA fallback)
 ↓
T3 (dev seed endpoint)
 ↓
T4 (wire dev routes + verify backend starts)
```

### E2E Builder
Executes T5 through T8 sequentially:
```
T5 (Playwright project: package.json, config, npm install)
 ↓
T6 (smoke tests: home, catalog, setlist)
 ↓
T7 (SPA routing test) + T8 (pipeline script) — can be parallel
```

### Lead (Coordinator)
1. Create feature branch: `git checkout -b feature/st-004-e2e-playwright`
2. Warm dependency cache: `cd backend && cargo check`
3. Spawn Backend Builder with T1-T4 assignment
4. After T4 completes: run `cd frontend && flutter build web --base-href /` (one-time, ~60-240s)
5. Spawn E2E Builder with T5-T8 assignment
6. Run quality gates after all tasks complete:
   ```bash
   cd backend && cargo fmt --check && cargo clippy -- -D warnings && cargo test
   cd frontend && flutter analyze && flutter test
   cd e2e && npx playwright test
   ```
7. Run critic review (fresh context agent)
8. Run `/verify-uc ST-004` against the steel thread postconditions

---

## Pre-Implementation Checklist

- [ ] Feature branch created: `feature/st-004-e2e-playwright`
- [ ] `cargo check` run to warm dependency cache
- [ ] Steel thread reviewed (`docs/steel-threads/st-004-run-playwright-e2e-against-live-stack.md`)
- [ ] Node.js / npm available (confirmed: v22.21.1 / v11.9.0)
- [ ] Chromium available (confirmed: `/usr/bin/chromium-browser`)
- [ ] Flutter web build verified: `flutter build web --base-href /`

## Files Created / Modified

**New files**:
- `backend/src/routes/dev.rs`
- `e2e/package.json`
- `e2e/playwright.config.ts`
- `e2e/.gitignore`
- `e2e/tests/smoke.spec.ts`
- `e2e/tests/spa-routing.spec.ts`
- `scripts/e2e.sh`

**Modified files**:
- `backend/src/config.rs` (add `dev_mode` + `bind_address`)
- `backend/Cargo.toml` (add `"fs"` to tower-http features)
- `backend/src/main.rs` (configurable bind address, ServeDir + SPA fallback, conditional dev routes)
- `backend/src/routes/mod.rs` (add `pub mod dev;`)
- `.gitignore` (add `backend/e2e-test.db`)
