# Session Handoff — 2026-03-03

## What Was Accomplished This Session

### 1. ST-006 Wrap-up
- PR #3 CI fixed (background `/fix-ci` agent — E2E test updated for tabbed UI)
- PR #3 merged to main (squash)
- ST-006 retrospective written: `docs/retrospectives/st-006-multi-input-enhanced-generation.md`
- Known debt and lessons learned updated in `.claude/rules/`

### 2. AWS Deployment (tarab.studio)
- systemd service: auto-restart, env file for secrets, `DEV_MODE=false`
- `sqlx::migrate!()`: proper migration tracking (replaces raw SQL runner)
- Route53 domain: `tarab.studio` with auto-HTTPS via Caddy
- SQLite backups: VACUUM INTO + integrity check → S3 every 6 hours
- Deploy scripts: symlink-based rollback + exponential backoff health check
- GitHub Actions deploy workflow (needs EC2_SSH_KEY + EC2_HOST secrets to activate)
- CORS locked to explicit origins, graceful shutdown on SIGTERM
- `/api/health/ready` endpoint with DB connectivity check
- Fixed API description to reflect DJ-first pivot

### 3. Audio Playback Spike (SP-005)
- Deezer search API: free, no auth, returns 30s MP3 preview URLs
- Deezer CDN: `Access-Control-Allow-Origin: *` on MP3s
- Backend proxy: `GET /api/audio/deezer-search?q=...` (Deezer search API lacks CORS)
- PoC: `tarab.studio/audio-poc.html` — Web Audio API crossfade between two Deezer previews
- **Result: Audio playback in browser is PROVEN. Crossfade works.**

### 4. UC-019 Planning (approved, ready to build)
- Full task decomposition at `docs/tasks/uc-019-tasks.md`
- Critic review found 3 critical issues: dart:web_audio doesn't exist (use package:web), CORS needs backend MP3 proxy, dart:html breaks non-web platforms
- 6 tasks, 4 builders + lead, clean file boundaries

## Current State

### Git
- **Branch**: `main` (clean, pushed)
- **Latest commit**: `3c3e7e6` — audio spike
- **Tests**: 268 backend + 47 frontend = 315 total (all passing)

### Deployment
- **URL**: `https://tarab.studio` (basic auth: reviewer / password)
- **Backend**: systemd service `ethnomusicology.service` (active, running)
- **Frontend**: `/opt/ethnomusicology/frontend-current` symlink
- **Database**: `/opt/ethnomusicology/data/ethnomusicology.db` (all 6 migrations + _sqlx_migrations)
- **Backups**: S3 `ethnomusicology-backups` bucket, cron every 6 hours
- **Domain**: Route53 hosted zone `tarab.studio` → EIP 52.72.57.136

### IAM (incomplete)
- `sst-deployer` still has `AdministratorAccess` — scoped S3 policy created but not yet swapped
- Do this AFTER activating GitHub Actions CI/CD

## What the Next Session Should Do

### Immediate: Build UC-019
1. Read plan: `docs/tasks/uc-019-tasks.md`
2. T0 (Lead): Add `package:web` to pubspec.yaml, define abstract interfaces
3. Spawn team: backend-builder (T1), api-builder (T2), audio-builder (T3), ui-builder (T5)
4. Wire, test, deploy
5. Critic review before merge

### After UC-019
- Ember Crate / TR-808 design implementation (design-crit outputs at `.design-crit/`)
- Activate GitHub Actions CI/CD (add SSH key secrets)
- Scope down IAM
- ST-007 conversational refinement (post-MVP)

## Key Files
| File | Purpose |
|------|---------|
| `docs/tasks/uc-019-tasks.md` | UC-019 task plan (approved) |
| `.claude/plans/declarative-swimming-twilight.md` | Same plan (Claude Code plan file) |
| `docs/spikes/audio-crossfade-poc.html` | Working audio crossfade PoC |
| `backend/src/routes/audio.rs` | Deezer search proxy endpoint |
| `docs/steel-threads/st-aws-deployment-plan.md` | Full deployment plan with devil's advocate findings |
| `.design-crit/decisions.md` | Ember Crate design system (10 decisions locked) |
| `CLAUDE.md` | Standing orders |
| `~/.claude/projects/-home-ubuntu-ethnomusicology/memory/MEMORY.md` | Cross-session memory |
