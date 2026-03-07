# Retrospective: Phase 7 — Purchase Links (UC-020)

**Date**: 2026-03-07
**Milestone**: UC-020 Purchase Links
**Scope**: SP-009 (store viability spike), UC-020 (purchase link panel), 18 commits

---

## What Worked

1. **Spike-first approach paid off again.** SP-009 validated store URL templates and hit rates before any code was written. Confirmed Bandcamp (70%) and Beatport (60%) as viable, identified Traxsource/Juno as unverifiable (403). No rework during implementation.

2. **Design-crit before building.** DC1 produced 3 options (Chip Strip, Popover Tray, Inline Grid). User locked Option A (Chip Strip) quickly. No UI rework.

3. **Two-pass critic review caught real issues.** 7a found 1 HIGH (partial-input bug where title-only queries broke), 3 MEDIUM. 7b found 2 MEDIUM (icon mapping, code style). All fixed before merge.

4. **Clean separation of concerns.** Purchase links (search URLs to DJ stores) kept visually and architecturally separate from source attribution (Spotify/Deezer/SoundCloud icons). UC-020 devil's advocate review #2 flagged this early.

5. **Simple architecture held.** Pure URL construction with no external API calls. Backend endpoint is ~50 lines. No database migration needed. On-demand computation works for fresh and saved setlists.

6. **Commit-per-task discipline.** 18 commits across the phase. Each task committed independently. No lost work risk.

---

## What Didn't Work

1. **Autonomous ralph loop produced unreliable data.** SP-009 ran via `ralph-loop.sh` shell script. Critic found: overview.html destroyed, state.json overwritten, potentially hallucinated affiliate details (Brandreward network name, specific commission percentages). Required post-hoc critic review to catch.

2. **Stale progress docs accumulated.** Session handoff and mvp-progress were out of date when the ralph loop ran. Critic flagged this as HIGH. Progress docs should be updated at each commit, not batched.

3. **Pre-commit hook is slow.** Full backend + frontend test suite runs on every commit attempt (~60s). Acceptable for correctness but slows iteration. Consider splitting into fast (fmt + clippy + analyze) and full (tests) hooks.

---

## Patterns Identified

| Pattern | Frequency | Impact |
|---------|-----------|--------|
| Spike-first validates architecture before coding | 3rd time (SP-004, SP-007, SP-009) | Positive — zero rework |
| Two-pass critic catches different issue classes | 2nd time (Phase 8, Phase 7) | Positive — keeps shipping |
| Autonomous loops produce unreliable research data | 1st documented | Negative — requires manual verification |
| Simple URL-template architecture avoids complexity | 1st time | Positive — lowest-effort feature in the project |

---

## Action Items

| # | Action | Priority | Status |
|---|--------|----------|--------|
| 1 | Stop using ralph-loop.sh — use Agent Teams or in-session agents | High | Done (documented in CLAUDE.md) |
| 2 | Spot-check SP-009 affiliate details before registration | Medium | Backlog |
| 3 | Consider fast/full pre-commit hook split | Low | Backlog |

---

## Metrics

- **Test count**: 407 backend + 166 frontend = 573 total
- **Commits**: 18 (from first spike task through verification)
- **Critic findings**: 7a: 1 HIGH + 3 MEDIUM, 7b: 2 MEDIUM — all resolved
- **Postconditions**: 7/7 covered, 3/3 invariants verified, 4/4 extensions implemented
- **Architecture complexity**: Zero database migrations, zero external API calls

---

## Key Learnings

1. **URL-template architecture is the right abstraction for "search this track on store X."** No server-side verification needed — store search pages handle "not found" gracefully. This is the same pattern used by 1001Tracklists.

2. **Autonomous loops need guardrails.** The ralph loop was useful for research breadth but destructive for file integrity. Agent Teams provide the same parallelism with better isolation.

3. **Phase 7 was the simplest feature in the project.** Pure computation, no external dependencies, no state management. This validates the architectural decision to keep purchase links stateless and on-demand.
