# Session Handoff — 2026-03-05 (Evening)

## Branch
`phase-8-setlists-crates-spotify` — freshly created off `main` (clean, no changes yet)

## Test Counts
- Backend: 370 tests passing
- Frontend: 156 tests passing
- Total: 526

## What Was Done This Session

### SP-007: LLM Self-Verification Spike (COMPLETE → merged to main)
- Created `music_skill.md` (~500 token skill doc injected into every generation)
- Created `verification_prompt.md` (fact-checker persona for second-pass)
- Added `confidence` field (high/medium/low) throughout the stack
- Tested live: genre-term fabrication eliminated, confidence calibration useful

### ST-010: Verification Loop + Confidence UI (COMPLETE → PR #10 merged)
- 8 tasks, 2 builders + lead, zero file conflicts
- `verify: true` opt-in flag, persist-after-verify restructure
- Frontend confidence dot badges + inline verification notes

### Tech Debt Wave 1 (COMPLETE → PR #11 merged)
- 13 debt items resolved across backend and frontend
- Process audit: 8 docs reconciled

### Diversity Tuning + MusicBrainz Grounding (COMPLETE → PR #13 merged)
- MusicBrainz API client: verifies tracks against 35M+ recordings
- Graduated verification spectrum, diversity guidance in prompts
- Unique artists per set improved from 58% to 75%

### Bug Fixes & Infrastructure
- Timeout chain fixed: Claude API 90s, Caddy 120s, Dio 120s
- Service worker cache: permanent 3-part fix (cleanup SW + cache-bust + headers)
- `scripts/post-build-web.sh` — MUST run after flutter build web before deploy
- Diagnostic error messages for timeouts/connection issues

## What's In Progress

### Phase 8: Saved Setlists, Crates, and Spotify Discovery
- **Branch**: `phase-8-setlists-crates-spotify` (clean, ready to start)
- **Plan**: `/home/ubuntu/.claude/plans/toasty-popping-wirth.md` (approved, devil's advocate reviewed)
- **Scope**: 3 workstreams, 8 tasks, 5 builders + lead

#### Key Devil's Advocate Fixes to Apply
1. SQLite FK enforcement OFF → use explicit multi-table DELETE in transaction
2. Spotify CC flow needs building from scratch (follow SoundCloud pattern)
3. models.rs ownership split: Builder A owns models.rs, Builder B uses crate_models.rs
4. Delete handles 4 tables: conversations → version_tracks → versions → tracks → setlist
5. Spotify search runs parallel with audio search (tokio::join!)

## Deployed State
- **tarab.studio**: Running latest main (PR #13)
- MusicBrainz grounding + verify toggle + confidence badges all live
- `scripts/post-build-web.sh` MUST be run after flutter build web before deploying

## Next Session Instructions
1. Read this file + the plan at `/home/ubuntu/.claude/plans/toasty-popping-wirth.md`
2. Create team, start T1 (migrations) + T4 (Spotify CC) in parallel
3. After T1: T2 (setlist CRUD) + T3 (crate CRUD) in parallel
4. After T2+T3+T4: T5 (lead wiring)
5. Then frontend: T6 → T7 → T8
6. Critic review before merge
7. Deploy + test: generate → save → library → crate → Spotify link
