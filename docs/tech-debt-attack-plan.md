# Tech Debt Attack Plan

**Created**: 2026-03-05
**Sources**: known-debt.md, MEMORY.md, mvp-roadmap.md, mvp-progress.md, session-handoff.md, all retrospectives (ST-003/005/006/007/session-2026-03-03), all spikes (SP-001 through SP-007), ST-010 steel thread

---

## Category 1: Quick Wins (< 1 hour each)

### QW-01: Fix stale API info endpoint description
- **Description**: `main.rs` `api_info()` still says "occasions, African and Middle Eastern traditions" instead of DJ-first platform description.
- **Source**: known-debt.md (Audit)
- **Priority**: P3
- **Effort**: S (5 min)
- **Dependencies**: None
- **File**: `backend/src/main.rs`

### QW-02: Remove dead `apply_*` functions in quick_commands.rs
- **Description**: `apply_shuffle`, `apply_sort_by_bpm`, `apply_reverse` operate on `SetlistTrackRow` but service uses `VersionTrackRow`. Only called from their own tests. Remove or unify with generic trait.
- **Source**: ST-007 critic L1, known-debt.md
- **Priority**: P3
- **Effort**: S (30 min)
- **Dependencies**: None
- **File**: `backend/src/services/quick_commands.rs`

### QW-03: Fix TODO stub in import status endpoint
- **Description**: `get_import_status()` at `routes/import.rs:146` always returns NotFound with a TODO comment. Either implement DB lookup or remove the dead route.
- **Source**: Code grep (TODO)
- **Priority**: P2
- **Effort**: S (30 min)
- **Dependencies**: None
- **File**: `backend/src/routes/import.rs`

### QW-04: Unify Flutter catch patterns
- **Description**: 4 catch blocks use `catch (e)` while refactored code uses `on Exception catch (e)`. Files: `setlist_input_form.dart:354`, `audio_provider.dart:99,265,295`.
- **Source**: Frontend critic L1, known-debt.md
- **Priority**: P3
- **Effort**: S (15 min)
- **Dependencies**: None
- **Files**: `frontend/lib/widgets/setlist_input_form.dart`, `frontend/lib/providers/audio_provider.dart`

### QW-05: Deduplicate `MockInterceptor` in Flutter provider tests
- **Description**: `setlist_provider_test.dart` and `track_catalog_provider_test.dart` define local `MockInterceptor` while `mock_api_client.dart` provides a shared one. Unify to shared helper.
- **Source**: Frontend critic L2, known-debt.md
- **Priority**: P3
- **Effort**: S (30 min)
- **Dependencies**: None
- **Files**: `frontend/test/providers/setlist_provider_test.dart`, `frontend/test/providers/track_catalog_provider_test.dart`

### QW-06: Deduplicate `compute_bpm_warnings` functions
- **Description**: `compute_bpm_warnings` (SetlistTrackRow) and `compute_bpm_warnings_from_responses` (SetlistTrackResponse) duplicate logic. Share a generic helper.
- **Source**: ST-006 critic MEDIUM-5, known-debt.md
- **Priority**: P3
- **Effort**: S (30 min)
- **Dependencies**: None
- **File**: `backend/src/services/setlist.rs` (or wherever these live)

### QW-07: Add "via [Source]" attribution labels
- **Description**: SoundCloud requires uploader credit + source label + backlink. Apple ToS requires store badge proximity. Currently only icons shown. Add text labels for all three sources.
- **Source**: known-debt.md (Compliance)
- **Priority**: P2
- **Effort**: S (45 min)
- **Dependencies**: None
- **File**: `frontend/lib/widgets/setlist_track_tile.dart`

---

## Category 2: Missing Retrospectives & Process Gaps

### PG-01: Write ST-004 retrospective
- **Description**: ST-003 and ST-005 have retrospectives but ST-004 does not. Need to document what was learned.
- **Source**: known-debt.md (Audit)
- **Priority**: P3
- **Effort**: S (30 min)
- **Dependencies**: None
- **Output**: `docs/retrospectives/st-004-*.md`

### PG-02: Write ST-008 retrospective
- **Description**: ST-008 (iTunes Preview Fallback) completed and merged via PR #7 but no retrospective exists in `docs/retrospectives/`.
- **Source**: Gap analysis — no retro file found
- **Priority**: P3
- **Effort**: S (30 min)
- **Dependencies**: None
- **Output**: `docs/retrospectives/st-008-itunes-fallback.md`

### PG-03: Write ST-009 retrospective
- **Description**: ST-009 (SoundCloud Preview) completed and merged via PR #9 but no retrospective exists.
- **Source**: Gap analysis — no retro file found
- **Priority**: P3
- **Effort**: S (30 min)
- **Dependencies**: None
- **Output**: `docs/retrospectives/st-009-soundcloud-preview.md`

### PG-04: mvp-progress.md has stale data
- **Description**: Several items marked as backlog that are actually complete. UC-019 SoundCloud preview listed as backlog but ST-009 is done. Deezer field-specific search and ISRC lookup listed as backlog but Phase 4 was completed. Test count shows 502 but session-handoff says 510.
- **Source**: Comparison of mvp-progress.md vs session-handoff.md vs MEMORY.md
- **Priority**: P2
- **Effort**: S (20 min)
- **Dependencies**: None
- **File**: `docs/mvp-progress.md`

### PG-05: Commit-per-task discipline not enforced
- **Description**: Monolithic commits (ST-006 landed 5,600 lines in one commit). ST-007 retro action item #6 flagged this as "Critical" after OOM crash nearly lost work. No enforcement mechanism exists.
- **Source**: ST-006 retro action #6, ST-007 retro action #6
- **Priority**: P1
- **Effort**: M (process change, not code)
- **Dependencies**: None
- **Action**: Add explicit rule to CLAUDE.md and consider a pre-push hook that warns on commits >500 lines

### PG-06: Activate GitHub Actions CI/CD
- **Description**: GitHub Actions secrets (`EC2_SSH_KEY`, `EC2_HOST`) were never configured. Deployments are still manual SSH + redeploy scripts.
- **Source**: Session-2026-03-03 retro action #4
- **Priority**: P2
- **Effort**: M (1-2 hours)
- **Dependencies**: QW-01 level cleanup should happen first

---

## Category 3: Deferred Features (planned but not built)

### DF-01: ST-010 — Wire Verification Loop and Confidence UI
- **Description**: Wire `verify_setlist()` into the generation pipeline (opt-in `verify: true`). Add DB migration for confidence/verification_flag/verification_note columns. Frontend confidence dot badges. SP-007 proved the concept; ST-010 productionizes it.
- **Source**: ST-010 steel thread, SP-007 findings
- **Priority**: P1
- **Effort**: L (full steel thread, ~1-2 days)
- **Dependencies**: None (SP-007 complete, `verify_setlist()` exists)
- **Deferred items within ST-010**:
  - V2 search-result feedback loop (Deezer results back to Claude)
  - Verification for refinement/conversation flow
  - Confidence calibration at scale
  - Prompt caching for verification call
  - Performance under concurrent verification requests

### DF-02: Purchase Link Panel (UC-020 / Phase 6)
- **Description**: Multi-store link panel per track: Beatport deep links, Apple Music affiliate links, Bandcamp/Traxsource/Juno search links.
- **Source**: mvp-roadmap.md Phase 6, session-handoff.md next steps
- **Priority**: P2
- **Effort**: L (full UC, ~2-3 days)
- **Dependencies**: Apple affiliate registration, Beatport API access (DF-06)

### DF-03: Daily generation limits enforcement
- **Description**: `user_usage.generation_count` column exists but is never checked during generation. Limits are tracked but not enforced.
- **Source**: known-debt.md, ST-006 steel thread
- **Priority**: P2
- **Effort**: S (1 hour — logic exists, just need the guard check)
- **Dependencies**: None

### DF-04: Retry path for errored tracks
- **Description**: Errored tracks are permanently stuck (`needs_enrichment=0`, `enrichment_error` set). Need a "retry errored" endpoint that clears the error and re-queues for enrichment.
- **Source**: ST-005 retro action #7, known-debt.md
- **Priority**: P2
- **Effort**: S (1 hour)
- **Dependencies**: None

### DF-05: Auto-enrich trigger after import
- **Description**: Plan called for `tokio::spawn` to auto-fire enrichment after `import_playlist()`. Only manual `POST /api/tracks/enrich` exists. ST-006 resolved the endpoint (T5) but auto-trigger was never wired.
- **Source**: ST-005 retro action #6, known-debt.md (resolved note says ST-006 T5 but that was manual endpoint)
- **Priority**: P2
- **Effort**: S (1 hour)
- **Dependencies**: None

### DF-06: Beatport API Integration
- **Description**: Apply for Beatport API v4 access. Rich DJ metadata (BPM, key, genre, label, remixer) + preview audio. Requires application and may take weeks for approval.
- **Source**: mvp-roadmap.md, SP-001 findings
- **Priority**: P2
- **Effort**: S to apply, L to integrate
- **Dependencies**: Apply early — approval timeline unknown
- **Action**: Submit application immediately. Integration work blocked until approved.

### DF-07: Global transport control (sticky player bar)
- **Description**: Sticky play/pause bar like Beatport — user requested. Currently playback controls are inline.
- **Source**: session-handoff.md next steps
- **Priority**: P2
- **Effort**: M (1-2 days)
- **Dependencies**: None

### DF-08: Granular generation progress
- **Description**: After submitting the LLM prompt, show stages: "Searching catalog...", "Generating setlist...", "Enriching tracks..." instead of a single spinner.
- **Source**: MEMORY.md backlog, session-handoff.md next steps
- **Priority**: P3
- **Effort**: M (backend SSE/streaming + frontend progress states)
- **Dependencies**: None

### DF-09: Energy profile visual mini-curve
- **Description**: Plan specified "visual mini-curve" for energy profile selector but implementation uses text-only ChoiceChips.
- **Source**: ST-006 critic LOW-1, known-debt.md
- **Priority**: P3
- **Effort**: S (custom paint or SVG in Flutter)
- **Dependencies**: None

### DF-10: `SortByBpm` descending option
- **Description**: Plan specified `SortByBpm { ascending: bool }` but implemented as always ascending.
- **Source**: ST-007 critic L2, known-debt.md
- **Priority**: P3
- **Effort**: S (30 min)
- **Dependencies**: None

### DF-11: Set `parent_version_id` on LLM-refined versions
- **Description**: Only set on reverts. Normal refinements create versions with `parent_version_id: None`, so version lineage chain is incomplete.
- **Source**: ST-007 critic L4, known-debt.md
- **Priority**: P3
- **Effort**: S (30 min)
- **Dependencies**: None

### DF-12: Waveform visualization
- **Description**: Preview waveform rendering on track tiles. Deferred as post-MVP polish.
- **Source**: mvp-progress.md
- **Priority**: P3
- **Effort**: L (essentia or Canvas-based rendering)
- **Dependencies**: DF-13 (essentia sidecar) or client-side Web Audio API analysis

### DF-13: Held-Karp optimal arrangement for n<=20
- **Description**: Current arrangement uses greedy nearest-neighbor. Held-Karp gives optimal TSP solution for small setlists but was deferred.
- **Source**: mvp-progress.md (UC-017)
- **Priority**: P3
- **Effort**: M (algorithm is well-known, needs Rust implementation + tests)
- **Dependencies**: None

### DF-14: mood_tags field
- **Description**: Listed in UC-018 postconditions but not implemented. `enriched_at` exists, `mood_tags` does not.
- **Source**: mvp-progress.md
- **Priority**: P3
- **Effort**: S (add column, populate during enrichment)
- **Dependencies**: None

---

## Category 4: Architecture & Infrastructure Debt

### AI-01: Scope down `sst-deployer` IAM from AdministratorAccess
- **Description**: IAM user has AdministratorAccess. Must scope to S3-only before storing credentials in GitHub Actions secrets.
- **Source**: known-debt.md (HIGH), session-2026-03-03 retro action #5
- **Priority**: P0
- **Effort**: S (30 min — write scoped IAM policy)
- **Dependencies**: Must be done BEFORE PG-06 (CI/CD activation)

### AI-02: Replace `CorsLayer::permissive()` with domain-scoped CORS
- **Description**: Unconditional permissive CORS in `main.rs`. Should allow only `tarab.studio` (and localhost in dev mode).
- **Source**: known-debt.md, AWS deploy plan T1
- **Priority**: P1
- **Effort**: S (30 min)
- **Dependencies**: None

### AI-03: Add graceful shutdown on SIGTERM
- **Description**: `axum::serve` runs bare. `systemctl restart` drops in-flight requests. Wire `with_graceful_shutdown(tokio::signal)`.
- **Source**: known-debt.md, AWS deploy plan review
- **Priority**: P1
- **Effort**: S (30 min)
- **Dependencies**: None

### AI-04: Deduplicate `create_test_pool()` across test files
- **Description**: `tests/setlist_api_test.rs` has its own `create_test_pool()` that diverges from `db/mod.rs`. Has caused failures in ST-005 and ST-006 — second-most recurring footgun in the project. Refactor to single canonical pool builder, `pub` exported for integration tests.
- **Source**: ST-006 retro action #5, known-debt.md (HIGH), bitten twice
- **Priority**: P0
- **Effort**: S (1 hour)
- **Dependencies**: None
- **Files**: `backend/src/db/mod.rs`, `backend/tests/setlist_api_test.rs`, any other test files with their own pool

### AI-05: Wire `EnergyProfile` enum into `build_enhanced_system_prompt`
- **Description**: Takes `Option<&str>` and matches string literals instead of `Option<&EnergyProfile>`. Bypasses compiler enforcement.
- **Source**: ST-006 critic MEDIUM-2, known-debt.md
- **Priority**: P2
- **Effort**: S (30 min)
- **Dependencies**: None

### AI-06: Concurrency guard on enrich endpoint
- **Description**: Two simultaneous POST `/api/tracks/enrich` calls double-process same tracks. Add AtomicBool or mutex.
- **Source**: ST-005 critic HIGH-3, known-debt.md
- **Priority**: P1
- **Effort**: S (30 min)
- **Dependencies**: None

### AI-07: Service worker cache invalidation strategy
- **Description**: Users see stale UI after deploys. Need cache-busting headers or version hash in asset names, plus user-facing messaging.
- **Source**: Session-2026-03-03 retro action #8
- **Priority**: P2
- **Effort**: M (1-2 hours)
- **Dependencies**: None

### AI-08: Migration versioning before ALTER TABLE
- **Description**: Current migrations use `CREATE TABLE IF NOT EXISTS` (re-run safe). First `ALTER TABLE` migration will fail on re-run. Need to ensure `sqlx::migrate!()` tracking is reliable before ST-010 adds columns.
- **Source**: known-debt.md
- **Priority**: P1
- **Effort**: S (verify `sqlx::migrate!()` tracking works, document behavior)
- **Dependencies**: Must be confirmed BEFORE DF-01 (ST-010)

### AI-09: Distinguish `Timeout`/`ServiceBusy` in RefinementError
- **Description**: `Timeout` and `ServiceBusy` error variants missing from `RefinementError`. Clients can't distinguish timeout from other LLM errors.
- **Source**: ST-007 critic L3, known-debt.md
- **Priority**: P3
- **Effort**: S (30 min)
- **Dependencies**: None

---

## Category 5: Quality & Testing Gaps

### QT-01: Claude API error path untested
- **Description**: `MockClaude` always returns `Ok`. No test exercises the error variant. Need tests for timeout, rate limit, malformed response.
- **Source**: ST-005 grade, known-debt.md
- **Priority**: P1
- **Effort**: S (1 hour)
- **Dependencies**: None

### QT-02: No HTTP integration test for source_playlist_id filtering
- **Description**: Service-level tests exist but no full HTTP round-trip test for import then generate with source_playlist_id.
- **Source**: ST-006 critic MEDIUM-3, known-debt.md
- **Priority**: P2
- **Effort**: M (1-2 hours)
- **Dependencies**: None

### QT-03: `score_breakdown` not returned from `get_setlist`
- **Description**: After arrangement, refreshing the page loses `score_breakdown` (not persisted to DB). Need either DB columns or recomputation on GET.
- **Source**: ST-006 critic MEDIUM-4, known-debt.md
- **Priority**: P2
- **Effort**: M (1-2 hours — decide persist vs recompute, implement)
- **Dependencies**: None

### QT-04: No test for undo-with-only-v0 edge case
- **Description**: `handle_quick_command` checks `versions.len() < 2` for undo but no explicit test covers bootstrap followed by immediate undo.
- **Source**: ST-007 critic L5, known-debt.md
- **Priority**: P3
- **Effort**: S (15 min)
- **Dependencies**: None

### QT-05: Cost cap allows overshoot on enrichment
- **Description**: Cap checked once before processing; doesn't subtract already-used cost from fetch limit. Batch could exceed cap.
- **Source**: ST-005 critic HIGH-2, known-debt.md
- **Priority**: P2
- **Effort**: S (30 min)
- **Dependencies**: None

### QT-06: No `/api/health/ready` integration test
- **Description**: Health check endpoint with DB connectivity has no integration test.
- **Source**: Session-2026-03-03 retro action #13
- **Priority**: P3
- **Effort**: S (30 min)
- **Dependencies**: None

### QT-07: `_InitialStateNotifier` test workaround
- **Description**: `setlist_generation_test.dart` subclasses `SetlistNotifier` to inject initial state, bypassing `build()`. Not ideal but functional.
- **Source**: Frontend critic L3, known-debt.md
- **Priority**: P3
- **Effort**: S (30 min — use proper provider override)
- **Dependencies**: None

---

## Category 6: Future Spikes Needed

### FS-01: iOS/mobile deployment spike
- **Description**: Test Flutter cross-platform deployment to iOS. This is the primary reason Flutter was chosen. Need to verify: build pipeline, App Store requirements, platform-specific audio behavior, responsive layout.
- **Source**: MEMORY.md backlog, session-handoff.md next steps
- **Priority**: P2
- **Effort**: M (spike: 1-2 days)
- **Dependencies**: None

### FS-02: Essentia sidecar implementation spike (UC-015)
- **Description**: SP-003 confirmed essentia can extract BPM/key from 30s previews. Need production implementation: FastAPI sidecar, async queue, containerization, 1-2GB memory allocation.
- **Source**: SP-003 findings, mvp-progress.md (UC-015 essentia: backlog)
- **Priority**: P3
- **Effort**: L (spike + implementation: 3-5 days)
- **Dependencies**: None, but LLM estimation is "good enough" for MVP

### FS-03: V2 verification — Deezer search-result feedback loop
- **Description**: Feed Deezer search results back to Claude for grounded verification. If search finds "DJ Hell - Mind Games" when Claude suggested "UR - Mind Games", send that correction back. Grounds verification in real data instead of LLM self-assessment.
- **Source**: SP-007 action item #4, ST-010 "Does NOT Prove" section
- **Priority**: P3
- **Effort**: M (spike: 1 day, implementation: 2 days)
- **Dependencies**: DF-01 (ST-010 must be done first)

### FS-04: Confidence calibration data collection
- **Description**: Log confidence vs. Deezer match rate to measure prediction accuracy at scale. Need data pipeline before calibrating.
- **Source**: SP-007 action item #5, ST-010 "Does NOT Prove"
- **Priority**: P3
- **Effort**: M (1-2 days)
- **Dependencies**: DF-01 (ST-010)

### FS-05: SoundCloud AI input restriction implications
- **Description**: SC terms prohibit using content as "input to AI." Current decision: SC is playback-only source. Review if ever wanting to use SC for track discovery/recommendation.
- **Source**: known-debt.md (Compliance)
- **Priority**: P3
- **Effort**: S (legal/product review, not code)
- **Dependencies**: None

---

## Execution Order

### Wave 1: Safety & Correctness (do first, can parallelize)

These items prevent bugs and security issues. Do them before adding features.

| Item | Description | Effort | Can Parallelize With |
|------|-------------|--------|---------------------|
| **AI-01** | Scope down IAM from AdministratorAccess | S | Everything |
| **AI-04** | Deduplicate `create_test_pool()` | S | AI-01, AI-02, AI-03 |
| **AI-02** | Scoped CORS (replace permissive) | S | AI-01, AI-04 |
| **AI-03** | Graceful shutdown | S | AI-01, AI-04 |
| **AI-06** | Concurrency guard on enrich | S | All above |
| **QT-01** | Test Claude API error paths | S | All above |
| **QT-05** | Fix enrichment cost cap overshoot | S | All above |

**Bundle as**: Single PR "Security & reliability hardening" — all backend, non-overlapping files.

### Wave 2: Quick Cleanup (can parallelize, 1-2 hours total)

| Item | Description | Effort | Can Parallelize With |
|------|-------------|--------|---------------------|
| **QW-01** | Fix API info description | S | All |
| **QW-02** | Remove dead `apply_*` functions | S | All |
| **QW-03** | Fix import status TODO stub | S | All |
| **QW-04** | Unify Flutter catch patterns | S | QW-05 (same area) |
| **QW-05** | Deduplicate MockInterceptor | S | QW-04 |
| **QW-06** | Deduplicate BPM warning functions | S | All |
| **QW-07** | Add "via [Source]" attribution labels | S | All |
| **AI-05** | EnergyProfile enum in system prompt | S | All |
| **DF-10** | SortByBpm descending option | S | All |
| **DF-11** | Set parent_version_id on refinements | S | All |

**Bundle as**: Two PRs:
1. "Backend cleanup" — QW-01, QW-02, QW-03, QW-06, AI-05, DF-10, DF-11
2. "Frontend cleanup" — QW-04, QW-05, QW-07

### Wave 3: Process & Docs (can parallelize with Wave 2)

| Item | Description | Effort |
|------|-------------|--------|
| **PG-01** | Write ST-004 retrospective | S |
| **PG-02** | Write ST-008 retrospective | S |
| **PG-03** | Write ST-009 retrospective | S |
| **PG-04** | Update mvp-progress.md | S |
| **PG-05** | Enforce commit-per-task | S (process) |

**Bundle as**: Single commit "Process: missing retrospectives and doc updates"

### Wave 4: Small Features & Test Gaps (after Waves 1-2)

| Item | Description | Effort | Can Parallelize With |
|------|-------------|--------|---------------------|
| **DF-03** | Enforce daily generation limits | S | DF-04, DF-05 |
| **DF-04** | Retry errored tracks endpoint | S | DF-03, DF-05 |
| **DF-05** | Auto-enrich trigger after import | S | DF-03, DF-04 |
| **QT-02** | HTTP integration test for playlist gen | M | QT-03, QT-04 |
| **QT-03** | Persist/recompute score_breakdown | M | QT-02 |
| **QT-04** | Test undo-with-only-v0 | S | All |
| **QT-06** | Health endpoint integration test | S | All |
| **AI-08** | Verify migration versioning | S | All |

**Bundle as**: Two PRs:
1. "Enrichment improvements" — DF-03, DF-04, DF-05
2. "Test coverage gaps" — QT-02, QT-03, QT-04, QT-06, AI-08

### Wave 5: ST-010 Verification Loop (after Wave 4)

| Item | Description | Effort |
|------|-------------|--------|
| **DF-01** | ST-010: Wire verification loop + confidence UI | L |

**This is a full steel thread** — use the Forge process (`/uc-review`, `/task-decompose`, multi-agent team). The steel thread doc already exists at `docs/steel-threads/st-010-wire-verification-loop-and-confidence-ui.md`.

Ensure AI-08 (migration versioning) is confirmed before starting.

### Wave 6: Infrastructure & UX (after or parallel with Wave 5)

| Item | Description | Effort | Can Parallelize With |
|------|-------------|--------|---------------------|
| **PG-06** | Activate GitHub Actions CI/CD | M | DF-07, AI-07 |
| **AI-07** | Service worker cache busting | M | PG-06, DF-07 |
| **DF-07** | Global transport control (sticky bar) | M | PG-06, AI-07 |
| **DF-08** | Granular generation progress | M | All |
| **DF-09** | Energy profile mini-curve | S | All |

**Bundle as**:
1. "CI/CD + deploy improvements" — PG-06, AI-07
2. "Playback UX: global transport" — DF-07
3. "Generation UX: progress stages" — DF-08

### Wave 7: Future Features & Spikes (backlog, do when ready)

| Item | Description | Effort | Priority |
|------|-------------|--------|----------|
| **FS-01** | iOS/mobile spike | M | P2 |
| **DF-02** | Purchase link panel (UC-020) | L | P2 |
| **DF-06** | Beatport API access (apply NOW, build later) | S+L | P2 |
| **DF-13** | Held-Karp optimal arrangement | M | P3 |
| **DF-14** | mood_tags field | S | P3 |
| **FS-02** | Essentia sidecar | L | P3 |
| **FS-03** | V2 verification feedback loop | M | P3 |
| **FS-04** | Confidence calibration | M | P3 |
| **DF-12** | Waveform visualization | L | P3 |

**Action NOW**: Submit Beatport API v4 application (DF-06) immediately regardless of wave. Approval takes weeks.

---

## Summary Statistics

| Category | Count | P0 | P1 | P2 | P3 |
|----------|-------|----|----|----|----|
| Quick Wins | 7 | 0 | 0 | 2 | 5 |
| Process Gaps | 6 | 0 | 1 | 2 | 3 |
| Deferred Features | 14 | 0 | 1 | 6 | 7 |
| Architecture/Infra | 9 | 2 | 3 | 2 | 2 |
| Quality/Testing | 7 | 0 | 1 | 3 | 3 |
| Future Spikes | 5 | 0 | 0 | 1 | 4 |
| **Total** | **48** | **2** | **6** | **16** | **24** |

**Estimated total effort**: Waves 1-4 can be completed in 2-3 focused sessions. Wave 5 (ST-010) is 1-2 days. Waves 6-7 are ongoing backlog.

**P0 items (do immediately)**:
1. AI-01: Scope down IAM (security risk)
2. AI-04: Deduplicate `create_test_pool()` (recurring footgun, bitten twice)

**One action to take right now regardless of everything else**:
- Submit Beatport API v4 application (DF-06) — it's zero-effort to apply and the approval timeline is the bottleneck.
