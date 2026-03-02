# Steel Thread: ST-004 Run Playwright E2E Test Against Live Backend and Flutter Web Build

## Classification
- **Goal Level**: Thread
- **Scope**: System (black box)
- **Priority**: P1 High
- **Complexity**: Medium

## Cross-Cutting References

This thread proves that the full stack can be built, served, and tested as an integrated unit. It cuts across every UC that has a frontend screen:

- **UC-016**: Steps 1-3 — proves frontend can submit prompt, reach backend, display result (via seeded catalog, not real Claude for this thread)
- **UC-017**: Steps 1-2 — proves arrange button triggers backend round-trip and re-renders
- **UC-001**: Step 14 — proves the track catalog screen renders data from the API (same screen/component as UC-001, but with seeded data instead of imported Spotify data)
- **All UCs**: Proves the build pipeline (Flutter web build), static serving from backend, and process orchestration that every future E2E test depends on

This is **infrastructure**, not a feature thread. It exists to enable all future E2E verification.

## Actors
- **Primary Actor**: CI/Developer (runs `npx playwright test`)
- **Supporting Actors**:
  - Axum backend (serves API + static Flutter files)
  - SQLite database (seeded with test data)
  - Chromium browser (headless, driven by Playwright)
- **Stakeholders & Interests**:
  - Developer: Wants confidence that frontend and backend actually connect in a real browser, not just unit test mocks
  - Business: E2E tests prevent integration regressions that unit tests miss (wrong API shapes, broken routing, CORS issues)

## Conditions

### Preconditions
1. Backend compiles and passes `cargo test`
2. Frontend passes `flutter analyze` and `flutter test`
3. `flutter build web` produces valid static assets in `frontend/build/web/` (confirmed: works on headless EC2 even without Chrome device target)
4. Node.js and npm are available on the host
5. Chromium is available (Playwright-managed or system)
6. `tower-http` crate has `"fs"` feature enabled in `Cargo.toml` (required for `ServeDir`)
7. `DEV_MODE` config field added to `AppConfig` in `config.rs` (loaded from `DEV_MODE` env var, default `false`)

### Success Postconditions
1. Backend serves Flutter web build at `/` and API at `/api/*` from a single origin on port 3001
2. A dev-only seed endpoint `POST /api/dev/seed` populates the database with sample tracks (gated by `DEV_MODE=true` env var)
3. Playwright navigates to `http://localhost:3001/`, sees the home screen
4. Playwright clicks "Track Catalog", sees seeded track data rendered in the browser
5. Playwright clicks "Generate Setlist", enters a prompt, and the UI transitions to the generation flow (loading state at minimum; full round-trip if Claude API key is available)
6. All Playwright tests exit with code 0
7. The test phase (backend start + seed + Playwright tests) completes in under 30 seconds. The Flutter web build is a separate one-time build step (~60-240s depending on cache) and is not included in the per-run timing target.

### Failure Postconditions
1. If `flutter build web` fails, the pipeline exits with a clear error before starting the backend
2. If the backend fails to start (port in use, migration error), Playwright tests are not attempted
3. If seed endpoint is called without `DEV_MODE=true`, it returns 404 (not silently available in production)

### Invariants
1. No production data is modified — tests use an ephemeral SQLite database
2. The seed endpoint is never available unless `DEV_MODE=true` is explicitly set
3. The backend bind address is configurable (defaults to `0.0.0.0` for container/E2E compatibility, `127.0.0.1` only when explicitly set)
4. Playwright tests are idempotent — can run repeatedly without manual cleanup

## API Contract

| Method | Path | Description | Schema Ref | Status |
|--------|------|-------------|------------|--------|
| GET | / | Serve Flutter web index.html (static) | — | Draft |
| GET | /api/health | Health check (existing) | — | Implemented |
| POST | /api/dev/seed | Seed test data (dev-only) | — | Draft |
| GET | /api/tracks | Paginated track catalog | — | Implemented (ST-001) |
| POST | /api/setlists/generate | Generate setlist from prompt | — | Implemented (ST-003) |
| POST | /api/setlists/{id}/arrange | Arrange setlist harmonically | — | Implemented (ST-003) |

### POST /api/dev/seed (NEW)

**Request**: No body required. Idempotent — safe to call multiple times.

**Response 200** (when `DEV_MODE=true`):
```json
{
  "seeded": true,
  "tracks": 8,
  "message": "Test data seeded successfully"
}
```

**Response 404** (when `DEV_MODE` not set):
Standard 404 — endpoint does not exist.

**Seed data** should include ~8 tracks with realistic DJ metadata:
- Mix of genres (house, techno, afrobeats)
- BPM range: 118-140
- Camelot keys spanning compatible groups (7A, 8A, 9A, 8B for compatible cluster; 1A, 3B for distant)
- Energy levels: 3-8
- At least 2 artists with multiple tracks (tests artist JOIN)

## Main Success Scenario

1. **[Build]** Developer runs `flutter build web --base-href /` — produces `frontend/build/web/`
2. **[Build]** Developer runs `cargo build` — backend compiles with static file serving
3. **[Backend]** Backend starts with `DEV_MODE=true`, binds to `0.0.0.0:3001`
4. **[Backend]** Backend applies migrations, serves Flutter static files at `/` with SPA fallback (`index.html` for all non-API routes)
5. **[Playwright → Backend]** Test setup calls `POST /api/dev/seed` — backend inserts 8 sample tracks into SQLite
6. **[Playwright → Frontend]** Browser navigates to `http://localhost:3001/` — Flutter app loads (WASM/JS bootstrap may take 5-10s on first load). Playwright waits for a known element (`'Salamic Vibes'` text) to be visible with a 15s timeout, not relying on DOM ready events.
7. **[Frontend]** Home screen shows "Salamic Vibes" title with navigation buttons
8. **[Playwright → Frontend]** Test clicks "Track Catalog" button
9. **[Frontend → API]** Flutter app calls `GET /api/tracks?page=1&per_page=25`
10. **[API → DB]** Backend queries seeded tracks with artist JOIN
11. **[API → Frontend]** Backend returns JSON; Flutter renders track list
12. **[Playwright]** Test asserts: seeded track title visible in the browser DOM
13. **[Playwright → Frontend]** Test navigates back to home, clicks "Generate Setlist"
14. **[Frontend]** Setlist generation screen renders with prompt input and "Describe your ideal set" guidance text
15. **[Playwright]** Test asserts: prompt input field visible, Generate button visible
16. **[Playwright]** All assertions pass, test exits with code 0

## Extensions

- **1a. `flutter build web` fails**:
  1. Pipeline script exits with non-zero code and Flutter error output
  2. Backend is never started, Playwright is never run
  3. Developer fixes Flutter build and reruns

- **3a. Backend starts but database migration fails**:
  1. Backend logs migration error (e.g., locked database, corrupt file) and exits
  2. Playwright `webServer` config detects process exit and reports startup failure
  3. Developer checks database state and reruns

- **3b. Backend port 3001 already in use**:
  1. Backend logs "Address already in use" and exits
  2. Playwright `webServer` config detects startup failure and aborts tests
  3. Developer kills the stale process and reruns

- **4a. SPA fallback not configured — direct URL navigation returns 404**:
  1. Playwright navigates to `/tracks` directly and gets a blank page or 404
  2. This means the `ServeDir` fallback to `index.html` is not working
  3. Fix: ensure `tower-http` `ServeDir` has `.fallback()` to serve `index.html`

- **5a. Seed endpoint called without DEV_MODE=true**:
  1. Backend returns 404 (route not registered)
  2. Playwright test setup fails with descriptive error
  3. Developer sets `DEV_MODE=true` and reruns

- **5b. Seed data already exists (idempotent call)**:
  1. Seed endpoint uses `INSERT OR IGNORE` / upsert semantics
  2. Existing data is preserved, no duplicates created
  3. Response still returns 200 with same shape

- **9a. API returns unexpected JSON shape**:
  1. Flutter `fromJson` throws, screen shows error state
  2. Playwright test fails with visible error text assertion
  3. This catches API contract drift between frontend and backend

- **11a. Track catalog is empty (seed failed silently)**:
  1. Frontend shows empty state message
  2. Playwright assertion for track title fails
  3. Developer checks seed endpoint response and DB state

- **6a. Flutter WASM/JS fails to load in Playwright Chromium**:
  1. Browser shows blank page with console errors
  2. Playwright timeout waiting for `'Salamic Vibes'` text fails after 15s
  3. Developer checks Chromium version compatibility and Flutter build output

- **14a. Claude API key not set — generation shows error**:
  1. Backend returns 503 `LLM_ERROR` when generate is called
  2. Frontend shows error state with "Try Again" button
  3. For ST-004, the test only asserts the UI loads correctly — it does NOT require a successful generation (that's a future enhancement with mock Claude)

## Integration Assertions

1. **[Build → Serve]** `flutter build web` output is served correctly by the Axum backend — all JS/CSS/assets load without 404
2. **[Frontend → API]** Flutter app's relative `/api` base URL resolves to the same-origin backend — no CORS errors, no misdirected requests
3. **[API → DB → API → Frontend]** Track catalog round-trip: seeded tracks in SQLite → API JSON response → rendered in browser DOM. Proves the full data pipeline works outside unit tests.
4. **[SPA Routing]** Direct URL navigation to `/tracks` serves `index.html` (not 404), and GoRouter handles client-side routing
5. **[Dev Tooling]** Seed endpoint only exists when `DEV_MODE=true` — verifying it returns 404 without the flag is an explicit test
6. **[Process Orchestration]** Backend startup, Flutter build, and Playwright execution can be orchestrated by a single script — proves the test pipeline is automatable

## Does NOT Prove

- **Does NOT prove setlist generation E2E** — that requires Claude API (real or mocked). ST-004 only asserts the generation screen loads. Future enhancement adds a mock Claude for full round-trip.
- **Does NOT prove Spotify OAuth flow** — OAuth E2E requires real Spotify credentials and callback URL. Not suitable for automated tests.
- **Does NOT prove PostgreSQL compatibility** — stays on SQLite. Postgres migration is a separate effort.
- **Does NOT prove Docker or container deployment** — runs processes directly on EC2.
- **Does NOT prove CI/CD pipeline** — tests are run manually. GitHub Actions integration is a follow-up.
- **Does NOT prove performance under load** — single browser, single user.
- **Does NOT prove mobile or responsive behavior** — Playwright runs at default desktop viewport.
- **Does NOT prove WASM build mode** — uses default JS compilation. WASM (`--wasm`) is a future optimization.

## Agent Execution Notes

- **Verification Command**:
  ```bash
  cd e2e && npx playwright test
  ```

- **Full pipeline script** (to be created at `scripts/e2e.sh`):
  ```bash
  #!/bin/bash
  set -e
  PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

  # Build phase (one-time, cached)
  cd "$PROJECT_ROOT/frontend" && flutter build web --base-href /
  cd "$PROJECT_ROOT/backend" && cargo build

  # Test phase (fast, repeatable)
  DATABASE_URL="sqlite:$PROJECT_ROOT/backend/e2e-test.db?mode=rwc" \
  DEV_MODE=true \
  cargo run &
  BACKEND_PID=$!
  trap "kill $BACKEND_PID 2>/dev/null; rm -f $PROJECT_ROOT/backend/e2e-test.db" EXIT
  sleep 3  # wait for backend to start

  cd "$PROJECT_ROOT/e2e" && npx playwright test
  ```

- **Test File**: `e2e/tests/smoke.spec.ts`
- **Depends On**: ST-001 (track catalog API), ST-003 (setlist generation API)
- **Blocks**: All future E2E test threads, CI/CD pipeline setup
- **Estimated Complexity**: M (medium) — 5-7 tasks across backend + e2e
- **Agent Assignment**: Lead coordinates, 2 builders (backend static serving + seed endpoint, e2e Playwright setup)

## Acceptance Criteria

- [ ] `flutter build web --base-href /` produces valid static assets
- [ ] Backend serves Flutter web build at `/` with SPA fallback (all non-API routes serve `index.html`)
- [ ] Backend binds to `0.0.0.0` by default (configurable via `BIND_ADDRESS` env var)
- [ ] `POST /api/dev/seed` inserts sample tracks when `DEV_MODE=true`
- [ ] `POST /api/dev/seed` returns 404 when `DEV_MODE` is not set
- [ ] Playwright test navigates to home screen and asserts title visible
- [ ] Playwright test clicks "Track Catalog" and asserts seeded track data visible
- [ ] Playwright test clicks "Generate Setlist" and asserts prompt input visible
- [ ] SPA routing works: direct navigation to `/tracks` renders the track catalog (not 404)
- [ ] All Playwright tests exit with code 0
- [ ] Test phase (backend start + seed + Playwright) completes in under 30 seconds
- [ ] Code passes quality gates (`cargo fmt`, `clippy`, `cargo test`, `flutter analyze`, `flutter test`)
- [ ] Critic agent approves
