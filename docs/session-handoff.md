# Session Handoff

## Current State (2026-03-16)

- **Branch**: `docs-test-overhaul` — ready to merge to `main`
- **Status**: Documentation cleanup + test suite overhaul COMPLETE (all 4 phases + two-pass critic review)

## What Was Done This Session

1. **Phase 0 (Docs)**: Updated 13 markdown files to reflect Next.js + Postgres architecture. Archived Flutter references.
2. **Phase 1 (Test Pruning)**: Removed 26 tautological backend DB tests + 14 frontend setter tests. Tokio-inspired testing philosophy applied.
3. **Phase 2 (Playwright)**: Added `--no-sandbox --disable-gpu` to playwright.config.ts. Fixed 6 e2e test failures (mock data keys, strict mode selectors).
4. **Phase 3 (Postgres Migration)**: Fixed SQLite→Postgres test failures: DELETE FROM instead of TRUNCATE, boolean 0/1→TRUE/FALSE, pool management, type fixes.
5. **Critic 7a**: Removed credentials from settings.json, added RUST_TEST_THREADS=1, renamed SqliteImportRepository→PgImportRepository.
6. **Critic 7b**: Documented cargo test DATABASE_URL requirement in CLAUDE.md, added deezer_id and pool.close() findings to known-debt.md.

## Test Counts (post-cleanup)

- Backend: 381 (pruned from 407; 210 pass without DB, 381 pass with Neon DATABASE_URL)
- Frontend vitest: 102
- Playwright e2e: 38
- Total: 521

## Architecture

- **Frontend**: Next.js 16 (`frontend-next/`) — primary. Flutter (`frontend/`) archived.
- **Backend**: Rust/Axum on EC2 (port 3001)
- **Database**: Neon Postgres (migrated from SQLite). `PgImportRepository`.
- **Deployment**: tarab.studio via Caddy + systemd + Route53
- **Pre-commit hook**: cargo fmt + clippy + vitest (cargo test requires DATABASE_URL)

## Commits on `docs-test-overhaul`

1. `021ba64` Update docs and rules for Next.js + Postgres architecture
2. `f572d56` Prune 38 tautological tests (tokio-inspired cleanup)
3. `8fe6993` Fix Playwright headless config and 6 e2e test failures
4. `f392218` Complete SQLite → Postgres migration: all 381 tests pass
5. `7168c70` Remove accidentally committed SQLite backup file
6. `31ccca0` Fix critic 7a findings: rename SqliteImportRepository, remove cargo test from hook
7. `e9fa94b` Address critic 7b findings: document DB test requirements, track new debt

## Next Steps

1. **Merge `docs-test-overhaul` to `main`**
2. **Update /etc/ethnomusicology/env**: Change DATABASE_URL from SQLite to Neon Postgres connection string
3. **Serverless migration**: Lambda + Vercel + Neon + Clerk — plan drafted, not started
4. **Known debt**: deezer_id i32 narrowing, missing pool.close() in route tests (see known-debt.md)
