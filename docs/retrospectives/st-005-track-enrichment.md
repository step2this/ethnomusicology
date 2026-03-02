# Retrospective: ST-005 Track Enrichment

**Date**: 2026-03-02
**Milestone**: Steel Thread 005 — LLM-based track enrichment + album art + usage tracking
**Scope**: 5 tasks, 3 new files + 10 modified files, ~1,255 lines of code, 176 backend tests (23 new)

---

## What Worked

1. **Multi-agent teams executed correctly**: Lead coordinated 5 builders (migration, camelot, spotify, enrichment, endpoint) across non-overlapping files. Lead never wrote implementation code. This directly addressed the ST-003 retro finding that solo implementation causes context rot.

2. **Parallel builder execution**: 3 builders ran simultaneously on independent tasks (migration, camelot, spotify). Test count grew 153 → 167 across parallel work. Sequential builder (enrichment) depended on migration + camelot completing first — dependency graph was respected.

3. **Critic review caught real bugs**: Fresh-context critic found 4 HIGH issues including a position=0 off-by-one bug and missing BPM/energy range validation. Both would have caused silent data corruption in production. The generator-critic separation principle from ST-003 retro proved its value.

4. **Spike before implementation**: SP-004 confirmed Spotify Audio Features is deprecated before we built around it. Saved potentially days of debugging 403 errors. Research-first approach validated.

5. **Cross-branch work via worktrees**: Fixed ST-004 E2E tests on a git worktree without disrupting the 3 parallel ST-005 builders working on the main feature branch. Clean separation of concerns.

## What Didn't Work

1. **Team cleanup friction**: Old `st-004-e2e` team had orphaned tmux panes that blocked TeamDelete. Required manual tmux pane killing and directory removal. Team lifecycle management needs improvement.

2. **Rebase conflicts after PR merge**: Merging ST-004 PR to main while ST-005 was in flight caused merge conflicts on `main.rs` and `routes/mod.rs`. The stash-rebase-pop workflow produced conflicts that needed manual resolution. Should have either: (a) merged ST-004 before starting ST-005, or (b) branched ST-005 from a commit that included ST-004.

3. **Context loss across session boundary**: This session continued from a compacted conversation. The summary was thorough but required re-reading several files to reconstruct state. The team task list was also lost across the boundary (TaskUpdate returned "Task not found").

4. **No auto-enrich on import trigger**: The plan called for enrichment to fire automatically after `import_playlist()` via `tokio::spawn`. This was NOT implemented — only the manual `POST /api/tracks/enrich` endpoint exists. This is a gap vs the plan.

## Patterns Identified

| Pattern | Frequency | Impact |
|---------|-----------|--------|
| Parallel builders on non-overlapping files | First time done right | High — 3x throughput on independent tasks |
| Critic catches off-by-one bugs | 1 instance (position=0) | High — would cause silent data corruption |
| Worktrees for cross-branch fixes | 1 instance (ST-004 E2E) | Medium — clean isolation |
| Team cleanup is manual and fragile | Every team teardown | Medium — needs automation |
| Rebase conflicts from mid-flight merges | 1 instance | Low — one-time resolution |

## Comparison with ST-003

| Metric | ST-003 | ST-005 | Improvement |
|--------|--------|--------|-------------|
| Builder agents | 0 (lead solo) | 5 (parallel) | Fixed anti-pattern |
| Critic review | None | Fresh-context, 4 HIGH found | Fixed blind spot |
| Bugs caught by critic | 0 | 2 (position, validation) | New capability |
| Context rot errors | 5+ fix iterations on late tasks | 0 (each builder fresh) | Eliminated |
| Lines per agent | ~2,800 (single agent) | ~250 avg (5 agents) | 10x reduction |
| Total test growth | 0 → 135 | 153 → 176 (+23) | Healthy |

## Action Items

### Immediate

| # | Action | Status |
|---|--------|--------|
| 1 | Update mvp-progress.md with ST-005 completions | Done |
| 2 | Update mvp-roadmap.md Phase 1 status | Done |
| 3 | Update CLAUDE.md current state section | Done |
| 4 | Create st-005 steel thread doc | Done |
| 5 | Update memory file with ST-005 lessons | Done |

### Future

| # | Action | Priority |
|---|--------|----------|
| 6 | Add auto-enrich trigger after import (tokio::spawn) | High — planned but not implemented |
| 7 | Add retry path for errored tracks (clear enrichment_error) | Medium |
| 8 | Add concurrency guard on enrich endpoint (AtomicBool or mutex) | Medium |
| 9 | Automate team cleanup in TeamDelete (kill orphan panes) | Low |
| 10 | Consider branching strategy: merge dependencies before starting new work | Low |

## Key Learnings

1. **Multi-agent teams work when boundaries are clean.** Non-overlapping file ownership + shared traits at boundaries = zero merge conflicts between builders. The coordination overhead is minimal compared to the context freshness benefit.

2. **Critic review is non-negotiable.** Two of the four HIGH findings (position=0, range validation) would have been invisible in self-review. The builder who wrote the code would not have noticed — they thought positions were 1-indexed and didn't consider out-of-range LLM responses.

3. **Spike-first saves real time.** SP-004 took ~30 minutes of web research and confirmed that Spotify Audio Features would 403. Building the enrichment service around Spotify first would have wasted hours.

4. **Git worktrees are powerful for cross-branch work.** Fixing ST-004 E2E tests while builders ran on ST-005 required zero coordination — each context was isolated.

5. **Plan gaps show up during critic review.** The auto-enrich trigger was in the plan but got dropped during task decomposition. The critic didn't catch it either (focused on code, not plan compliance). Need a plan-vs-implementation checklist step.
