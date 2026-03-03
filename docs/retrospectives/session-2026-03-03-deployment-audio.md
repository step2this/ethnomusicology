# Retrospective: Session 2026-03-03 — AWS Deployment, Audio Playback, and UC-019 Implementation

**Date**: 2026-03-03
**Session Scope**: ST-006 wrap-up, AWS production deployment, audio spike SP-005, UC-019 Phase 1+2 (Deezer preview playback)
**Participants**: Lead + multiple builders (audio, backend, frontend)
**Duration**: Full session
**Scope**: 5 major initiatives, ~30+ files changed, 323 total tests (+55 new), ~8 deploy cycles to tarab.studio

---

## What Worked

1. **Audio spike → PoC → production in one continuous flow.** Deezer search API discovery → Web Audio API proof-of-concept in `audio-poc.html` → full Flutter integration with backend proxy → 98/100 tracks enriched. No false starts, no pivots. The spike confirmed CORS behavior, Deezer free API availability, and crossfade feasibility before writing production code. This is the ideal research-to-implementation pattern.

2. **`/fix-ci` skill worked first try with background worktree.** PR #3 CI failures (E2E test layout mismatch on tabbed UI) were fixed by a spawned agent in a worktree while the team moved on to other work. No blocking wait, no manual debugging. The skill executed the fix, verified test passage, and reported back. This is multi-agent execution at its best.

3. **Deezer as audio source discovery solution.** Free API, no authentication, CORS-enabled CDN for MP3s, 30-second preview quality sufficient for DJ use case. Requires no paid tier or rate-limit negotiation. The choice to use Deezer (instead of Spotify previews or Beatport) was validated by the spike and works reliably.

4. **AWS deployment plan comprehensiveness prevented production blind spots.** Systemd service auto-restart, SQLite backup+integrity verification → S3, graceful SIGTERM shutdown, health check endpoint with DB connectivity, symlink-based rollback strategy, scoped IAM policy, and explicit `DEV_MODE=false` in production. These aren't afterthoughts—they're proven patterns in the written plan. Deployment is repeatable.

5. **Backend MP3 proxy solved CORS for non-authenticated URLs.** Deezer CDN returns `Access-Control-Allow-Origin: *` only from the Deezer domain itself. Proxying via `/api/audio/deezer-search?q=...` works in all browsers without CORS errors. This pattern is reusable for other audio sources.

6. **Route53 + Caddy auto-HTTPS worked without friction.** Domain `tarab.studio` → EIP, Let's Encrypt auto-renewal, basic auth layer. No certificates to manage manually. Domain is professional and memorable (vs. DuckDNS placeholder).

7. **Opus critic review of UC-019 caught real bugs before Phase 1 was even merged.** Three critical issues found: `dart:web_audio` doesn't exist (use `package:web`), CORS requires backend proxy, `dart:html` breaks non-web platforms. These would have surfaced as runtime crashes during Phase 1 build; the critic caught them during the plan review. This prevented rework.

## What Didn't Work

1. **Skipped Forge process for UC-019 Phase 2.** Phase 2 (Deezer enrichment pipeline) was not formally planned via `/uc-create` / `/task-decompose`. Instead, the builder started implementing against the Phase 1 task list. This violated the CLAUDE.md directive to plan before building. The user had to say "stop and make a plan" mid-implementation.

2. **Phase 2 builder thrashed for 14 cycles on column mismatch cascade.** Once Phase 2 began adding `deezer_id` and `deezer_preview_url` columns to `TrackRow`, every `SELECT *` query in the codebase broke until the migration ran. The test pool `sqlx::migrate!()` auto-applies migration 007, but integration tests were out of sync. This created a cascade of failures (column not found → FromRow derive error → test failure) that blocked the builder repeatedly. Better upfront analysis of the migration impact would have identified this.

3. **Flutter service worker caching caused confusion.** Old UI JavaScript was cached locally even after redeploying new frontend assets. The team thought the site was broken (old buttons, old layout) when it was actually the service worker serving stale content. Browser DevTools isn't obvious for debugging this. The lesson: communicate to users that they may need to clear cache or do a hard refresh (`Ctrl+Shift+R`).

4. **Migration idempotence initially required manual DB bootstrap.** Migration 007 schema changes broke existing test database state until migrations were manually applied. The `sqlx::migrate!()` macro is smart but requires the database file to exist; if the file is in an old state (from a previous run without migration 007), the new code panics. Had to manually patch the DB or rebuild from scratch.

5. **Multiple instances of hacking instead of planning during Phase 2.** Builder tried config tweaks, workarounds, and debug prints before sitting down to analyze the root cause (column mismatch). The user intervened multiple times: "this is thrashing — stop and think." Classic signs: many small failed attempts, trying the same thing twice, no clear hypothesis before each attempt.

6. **AWS deployment plan is comprehensive but Phase 2 (CI/CD automation) was not executed.** The plan is written and clear, but GitHub Actions secrets (`EC2_SSH_KEY`, `EC2_HOST`) were not configured. Deployments are still manual SSH + redeploy scripts on the EC2 host. This is operationally acceptable for MVP but leaves automation on the table.

7. **No pre-migration communication with other systems.** When migration 007 was added, the team didn't immediately check all places that reference `TrackRow` (`SELECT *` queries, integration test pools, API responses). The column mismatch discovered downstream instead of at the migration definition point.

## Patterns Identified

| Pattern | Frequency | Impact | Examples |
|---------|-----------|--------|----------|
| Skipping plan step (Forge) when "just implementing" feels faster | 1 (UC-019 Phase 2) | High — 14 thrash cycles | Builder went straight to code vs. `/task-decompose` |
| Hacking/iteration instead of root-cause analysis | 3+ attempts per cycle | High — time waste | Migration column mismatch, Flutter caching, service pool divergence |
| Schema migration impacts not analyzed upfront | 1 (migration 007) | Medium — discovery during implementation | `TrackRow` changes broke integration tests, required rework |
| Service worker caching not communicated to users | 1 | Low — causes confusion | Old UI cached; no clear messaging about cache invalidation |
| Critical fixes in background agents unrelated to main team focus | 1 (/fix-ci) | Low — but good pattern | E2E test failures fixed while audio team continued |
| Combining planning + implementation in one session after a major feature | Happens frequently | Medium — reduces iteration speed | UC-019 Phase 2 was planned and built in same session; tight feedback loop but error-prone |

## Comparison with Previous Sessions

| Metric | ST-006 | Session 2026-03-03 | Trend |
|--------|--------|-------------------|-------|
| Builders involved | 6 | 4+ (audio, backend, frontend, lead) | Same scale |
| Merge conflicts | 0 | 0 | Maintained |
| Test count delta | +139 | +55 | Smaller increment |
| Major initiatives | 1 (ST-006) | 5 (ST-006 wrap + AWS + SP-005 + UC-019 P1+P2) | Much broader |
| Plan adherence | High (devil's advocate enforced) | Mixed (Phase 2 skipped planning) | Deviation |
| Critic review quality | Excellent (found 2 dead-code + 1 unwired issue) | Good (found 3 bugs in UC-019 plan) | Maintained |
| Hacking cycles (iterations per task) | Low (<3) | High (14 on Phase 2) | Regression |
| Manual operations (deploys, DB fixes) | 0 | 3+ | Increased |
| Session duration | Full day | Full day | Baseline |

## Action Items

### Immediate

| # | Action | Status | Priority |
|---|--------|--------|----------|
| 1 | Update CLAUDE.md current state (ST-006 done, UC-019 Phase 1+2 done, SP-005 done) | Pending | High |
| 2 | Update mvp-progress.md (UC-019 crossfade + enrichment marked ✅) | Pending | High |
| 3 | Update mvp-roadmap.md phase status | Pending | Medium |
| 4 | Activate GitHub Actions CI/CD (add EC2_SSH_KEY and EC2_HOST secrets) | Pending | Medium |
| 5 | Scope down IAM (sst-deployer has AdministratorAccess, needs S3-only policy) | Pending | High |

### Future

| # | Action | Priority | Notes |
|---|--------|----------|-------|
| 6 | **Enforce Forge process for all UCs**: No implementation without `/uc-create` + `/task-decompose` | High | ST-005 retro added this; UC-019 Phase 2 violated it. Make it non-negotiable. |
| 7 | **Pre-migration schema impact analysis**: When adding columns, audit all SELECT *, joins, and test pools before migration lands | High | 2nd time `TrackRow` changes surprised integration tests. Need checklist. |
| 8 | **Service worker cache invalidation messaging**: Document for users (hard refresh, cache clear) and consider cache busting header on deployment | Medium | Users will see stale UI after deploys unless informed. |
| 9 | **Migration idempotence guarantee**: Document `sqlx::migrate!()` behavior; ensure test DB is always migrated before tests run | Medium | Manual DB bootstrap is operational friction. |
| 10 | **Deduplicate integration test pool helpers**: `tests/setlist_api_test.rs` still has its own `create_test_pool()`; merge into single source | Medium | Pattern from ST-006 action item #5 — still not fixed. |
| 11 | **Add pre-implementation planning checklist for Phase 2+ work**: Before builder starts Phase N, document task list, file boundaries, DB changes | Medium | Phase 2 thrashing suggests builder didn't have clear decomposition. |
| 12 | **Activate Phase 2 of AWS deployment plan**: Configure GitHub Actions secrets, test deploy workflow, verify rollback | Medium | Plan is complete; automation is written. Needs activation. |
| 13 | **Add `/api/health/ready` integration test**: Verify health check endpoint with DB connectivity | Low | Currently no integration test for health endpoint. |
| 14 | **Waveform visualization spike (optional)**: UC-019 scoped out waveform viz. If time, spike essentia or Canvas waveform rendering | Low | Post-MVP polish. User interest unclear. |

## Key Learnings

1. **Skipping the plan step (Forge) creates avoidable thrash cycles.** UC-019 Phase 2 had no formal task decomposition. The builder started implementing and hit column-mismatch cascades that a 15-minute pre-implementation analysis would have flagged. The Forge process (especially `/task-decompose`) exists because it catches these dependencies. When a builder says "I can just code it," the response should be "decompose first."

2. **Schema migrations have hidden downstream costs.** Adding columns to `TrackRow` broke `SELECT *` queries in 5+ places and integration test pools in 2+ places. These aren't obvious from the migration definition. Need a pre-implementation checklist: "Find all `TrackRow` references and verify they handle new columns." This applies to all structural changes, not just migrations.

3. **Root-cause analysis beats hacking.** Builder spent 14 cycles trying config tweaks and debug prints. One conversation: "What error are you actually seeing?" → "column doesn't exist" → "Ah, the test pool doesn't have migration 007 applied." Root cause found immediately; hacking was unnecessary. Teach builders to ask "What's the root cause?" before iterating.

4. **Opus critics find real bugs that fresh eyes catch.** The UC-019 plan review found 3 critical issues (dart:web_audio doesn't exist, CORS needs proxy, dart:html breaks platforms). These weren't in the original plan but a fresh Opus review revealed them. This pattern holds: have critics review plans, not just code.

5. **Combining broad initiatives in one session increases context complexity but can be efficient.** ST-006 wrap, AWS hardening, audio spike, and UC-019 Phase 1+2 all shipped in one session. No false starts, no rework on ST-006/SP-005, smooth handoff. However, the UC-019 Phase 2 thrashing shows that breadth without planning creates local failures. Solution: maintain the breadth (good for shipping fast) but enforce planning rigor on each piece.

6. **Service worker caching is a deployment/UX gap, not a code bug.** Users will see old UI after new deploys unless they clear cache. This isn't a defect in the app; it's a gap in the deployment communication. Add a user-visible notice or cache-busting strategy (version hash in asset names, `no-cache` headers, etc.).

7. **Background agents (like `/fix-ci` skill) are force multipliers.** While the team focused on audio and enrichment, a separate agent fixed E2E test failures without context switching. This is parallelism that works. Encourage more of this pattern.

---

## Summary

This session accomplished a lot: AWS deployment hardened to production standards, audio playback proven and integrated, UC-019 shipping with 98% track coverage. The audio pipeline is end-to-end and repeatable.

However, process discipline slipped. UC-019 Phase 2 skipped planning and hit cascading failures. The team had to intervene multiple times. This is a pattern to watch: as the codebase grows, the cost of skipping planning increases exponentially (each change touches more files, more tests, more DB state).

**The fix**: enforce Forge strictly. Reward it publicly when it catches issues (it will). Call out plan skipping when it happens. The goal is to make planning a reflex, not a suggestion.
