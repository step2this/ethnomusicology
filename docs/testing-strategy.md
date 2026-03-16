# Testing Strategy

## Philosophy

Inspired by tokio runtime test suite patterns:
- Test **behavioral contracts**, not implementation details
- Use **real implementations** (real DB, real router) — minimize mocks
- Mock only **external boundaries** you cannot control (Claude API, Spotify API)
- Keep DB integration tests to a **minimum** — only non-trivial behavior (upserts, pagination, CASCADE, ON CONFLICT)
- Prefer **Playwright e2e** for user-flow coverage over unit-testing route handlers

## Testing Pyramid

### Unit Tests (fastest, most)
- **Backend (Rust)**: Pure-function algorithm tests — Camelot scoring, arrangement, match scoring. Run via `cargo test`.
- **Frontend (Vitest)**: Component rendering, API client contracts, store behavior. Run via `bunx vitest run`. Uses happy-dom environment.

### Integration Tests (medium)
- **Backend**: Route handler tests via `tower::oneshot` with real DB and mocked Claude client. Located in `backend/tests/`.
- **Frontend**: MSW (Mock Service Worker) for API mocking in component tests.

### E2E Tests (slowest, fewest)
- **Playwright** with headless Chromium on EC2. Tests full user flows: generate setlist, browse library, import from Spotify.
- Located in `frontend-next/e2e/`.
- Run via `bunx playwright test`.

## Tools

| Layer | Tool | Config |
|-------|------|--------|
| Backend unit/integration | cargo test | Neon Postgres via DATABASE_URL |
| Frontend unit/component | Vitest 4.x + happy-dom + MSW | `vitest.config.ts` |
| E2E | Playwright 1.58.2 + Chromium | `playwright.config.ts` |
| Linting | cargo clippy, cargo fmt | Pre-commit hook |

## CI Commands

```bash
# Backend
cd backend
cargo fmt --check
cargo clippy -- -D warnings
DATABASE_URL=... RUST_TEST_THREADS=4 cargo test

# Frontend
cd frontend-next
bunx vitest run
bunx playwright test
```

## What NOT to Test

- Trivial CRUD roundtrips (insert → get → assert equal)
- Tautological setter tests (setState(x) → expect state === x)
- CSS class assertions (break on refactor, test implementation not behavior)
- Hardcoded href assertions (mirror implementation, not contract)
