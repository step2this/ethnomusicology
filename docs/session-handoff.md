# Session Handoff

## Current State (2026-03-16)

- **Branch**: `docs-test-overhaul` (feature branch off main)
- **Active Work**: Documentation cleanup + test suite overhaul
  - Phase 0: Markdown docs updated to reflect Next.js + Postgres architecture
  - Phase 1: Pruning tautological backend DB tests (26) and frontend setter tests (14)
  - Phase 2: Playwright headless setup on EC2
  - Phase 3: Fix remaining Postgres migration test failures on lean suite

## Completed

- ST-011: Next.js migration COMPLETE (Mar 8, 2026)
- Phase 2 (SQLite → Postgres): Code migration complete, test fixes pending
- Phase 1 (Vercel frontend): Deployed at ethnomusicology.vercel.app

## Test Counts (pre-cleanup)

- Backend: ~407 (cargo test)
- Frontend vitest: ~100
- Playwright e2e: 38
- Total: ~545

## Architecture

- **Frontend**: Next.js 16 (`frontend-next/`) — primary. Flutter (`frontend/`) archived.
- **Backend**: Rust/Axum on EC2 (port 3001)
- **Database**: Migrating from SQLite to Neon Postgres
- **Deployment**: tarab.studio via Caddy + systemd + Route53

## Next Steps

1. Complete test suite overhaul (this session)
2. Fix Postgres test failures on lean suite
3. Serverless migration (Lambda + Vercel + Neon + Clerk) — plan drafted
4. Investigate Vercel 404 on /api/setlists/generate
