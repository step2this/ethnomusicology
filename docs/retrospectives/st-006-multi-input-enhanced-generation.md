# Retrospective: ST-006 Multi-Input Seeding + Enhanced Generation

**Date**: 2026-03-03
**Milestone**: Steel Thread 006 — Multi-input seeding, enhanced generation with energy profiles, quality validation, prompt caching
**Scope**: 13 tasks across 5 phases, 6 builders + lead, 27 files changed, ~5,600 lines, 315 total tests (139 new: 92 backend + 47 frontend)

---

## What Worked

1. **Devil's advocate review on task plan caught showstoppers before any code was written.** Found 3 CRITICAL issues: `ContentBlock` name collision with existing response parser enum, `create_test_pool()` in integration tests missing migration 006, and scope creep (daily generation limits were in "Does NOT Prove" but had leaked into T6). All 3 would have blocked multiple builders simultaneously. Cost to fix at planning time: ~10 minutes. Cost to fix mid-implementation: hours of rework + builder restarts.

2. **6 parallel builders with zero merge conflicts.** Strict non-overlapping file ownership worked again — db-builder, energy-builder, claude-builder, generation-builder, import-builder, frontend-builder each had exclusive files. Max concurrency was 4 in Phase 1. The dependency graph (5 phases) was respected without exception.

3. **Combining related tasks into single builders improved efficiency.** T6+T8+T9 (generate service, routes, validation) all touched `services/setlist.rs` and `routes/setlist.rs`. Rather than 3 builders fighting over the same files, one generation-builder handled all 3 sequentially. Same for T10+T11 (provider + screen) going to one frontend-builder.

4. **Critic review caught 2 HIGH issues invisible to the builders.** `compute_seed_match_count` was defined, tested, but never called from production code (dead code that meant postcondition 13 was silently unmet). Spotify tab passed raw URL as `sourcePlaylistId` instead of importing first — would have produced PLAYLIST_NOT_FOUND errors for every user. Both required wiring fixes the lead applied.

5. **Plan-vs-code compliance check worked.** ST-005 retro action item #5 ("plan gaps show up during critic review") was addressed by adding an explicit plan compliance step to the critic prompt. The critic checked all 24 postconditions against the implementation and caught the dead-code gap.

## What Didn't Work

1. **Integration test pool divergence.** `tests/setlist_api_test.rs` has its own `create_test_pool()` that duplicates `db/mod.rs::create_test_pool()`. When migration 006 was added to `db/mod.rs`, the integration test file was missed. 6 integration tests failed with 500 errors until the lead manually patched it. This is the second time this pattern has bitten us (also caught by devil's advocate, but the fix was only applied to `db/mod.rs`, not `tests/*.rs`).

2. **Single monolithic commit.** All 13 tasks, 27 files, and ~5,600 lines landed in one commit (`d108773`). This makes it impossible to bisect if something breaks, hard to review on GitHub, and loses the per-task attribution that would help future debugging. Should have committed per-phase or per-builder.

3. **`build_enhanced_system_prompt` takes `Option<&str>` instead of `Option<&EnergyProfile>`.** The claude-builder created the function before the energy-builder had defined `EnergyProfile`. By the time the enum existed, the string-based interface was already wired in. Parallel builders on dependent types create interface mismatches that the lead must catch during wiring.

4. **No HTTP integration test for playlist-seeded generation.** Service-level tests cover `source_playlist_id` filtering, but there's no end-to-end HTTP test that imports a playlist and then generates from it. The test gap means the two-step Spotify flow is only verified by the frontend (which itself can't be tested locally due to Playwright limitations).

## Patterns Identified

| Pattern | Frequency | Impact |
|---------|-----------|--------|
| Devil's advocate on plans catches CRITICAL issues | 3 issues (ST-006) | High — prevented multi-builder rework |
| Duplicate test helpers diverge from main code | 2nd occurrence (ST-005, ST-006) | High — silent test failures |
| Critic catches dead code / unwired functions | 1 instance (seed match count) | High — postcondition silently unmet |
| Combining same-file tasks into one builder | 2 instances (T6+T8+T9, T10+T11) | Medium — eliminated file conflicts |
| Parallel builders create interface mismatches | 1 instance (string vs enum) | Medium — tech debt |
| Monolithic commits lose traceability | Every steel thread | Low — operational friction |

## Comparison with Previous Steel Threads

| Metric | ST-003 | ST-005 | ST-006 | Trend |
|--------|--------|--------|--------|-------|
| Builder agents | 0 (solo) | 5 | 6 | Scaling up |
| Max parallelism | 1 | 3 | 4 | Improving |
| Critic HIGH findings | 0 | 4 | 2 | Holding (builders improving) |
| Devil's advocate CRITICAL | N/A | N/A | 3 | New process, high value |
| Files changed | ~14 | ~13 | 27 | Largest ST yet |
| Lines changed | ~2,800 | ~1,255 | ~5,600 | 2x ST-003 but 6 builders |
| Lines per builder avg | 2,800 | 250 | 930 | Healthy (< context limit) |
| Tests before → after | 0→135 | 153→176 | 176→315 | +139 new tests |
| Merge conflicts | Multiple | 0 | 0 | Non-overlapping works |
| Context rot errors | 5+ | 0 | 0 | Eliminated by multi-agent |
| Plan compliance check | None | None | Yes (critic) | New — caught 1 gap |

## Action Items

### Immediate

| # | Action | Status |
|---|--------|--------|
| 1 | Fix CI failures on PR #3 | In progress (fix-ci agent) |
| 2 | Merge PR #3 to main | Blocked on #1 |
| 3 | Update CLAUDE.md current state | Pending |
| 4 | Update mvp-progress.md | Pending |

### Future

| # | Action | Priority | Notes |
|---|--------|----------|-------|
| 5 | Deduplicate `create_test_pool()` — single source in `db/mod.rs`, `pub` export for integration tests | High | 2nd time this caused failures. Refactor to one canonical pool builder. |
| 6 | Commit per-phase or per-builder, not monolithic | Medium | Enables bisect, better PR review, per-task attribution |
| 7 | Wire `EnergyProfile` enum into `build_enhanced_system_prompt` | Medium | Replace string matching with compiler-enforced enum |
| 8 | Add HTTP integration test for import→generate flow | Medium | Only service-level test exists; no full round-trip |
| 9 | Persist `score_breakdown` to DB or recompute on `get_setlist` | Medium | Lost on page refresh |
| 10 | Deduplicate `compute_bpm_warnings` / `compute_bpm_warnings_from_responses` | Low | Share generic helper |
| 11 | Add visual mini-curve to energy profile selector | Low | UX polish, currently text-only chips |

## Key Learnings

1. **Devil's advocate review on the task plan is the highest-ROI quality step.** Three CRITICAL issues caught before any builder started. The cost to fix at plan time was minutes; mid-implementation it would have been hours of rework across multiple builders. This should be mandatory for every ST going forward.

2. **Duplicate test helpers are a recurring trap.** When `create_test_pool()` exists in both `db/mod.rs` and `tests/setlist_api_test.rs`, adding a migration to one but not the other creates silent failures that only surface when running the full test suite. Need a single canonical implementation.

3. **Combine tasks that touch the same files into one builder.** Splitting T6, T8, T9 across 3 builders would have created constant merge conflicts on `services/setlist.rs`. One builder handling all 3 sequentially was cleaner and likely faster than the coordination overhead of 3 builders.

4. **Plan-vs-code compliance is a real gap that the critic step fills.** `compute_seed_match_count` was defined and tested but never called — postcondition 13 would have shipped unmet. Adding explicit postcondition checking to the critic prompt caught this.

5. **Shut down idle builders proactively.** Builders that finish their phase and go idle still consume notification bandwidth and can cause confusion. Shutting them down as soon as their tasks complete keeps the team clean and reduces noise for the lead.
