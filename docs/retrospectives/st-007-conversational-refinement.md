# Retrospective: ST-007 Conversational Setlist Refinement

**Date**: 2026-03-04
**Milestone**: Steel Thread 007 — Conversational refinement backend + Flutter architecture refactor
**Scope**: 5 phases, 5 builders + lead + 2 critics, 10 commits on branch, 432 total tests (56 new backend + 53 new frontend)

---

## What Worked

1. **Session handoff doc was the single most important recovery artifact.** Two Claude sessions hit an EC2 OOM mid-work. All changes were uncommitted. When resuming, `docs/session-handoff.md` was the only record of what each session was doing, which tasks were complete, and which files were owned by which stream. Without it, recovery would have meant re-reading every modified file to reconstruct intent. Recovery took ~10 minutes instead of hours.

2. **Sequential builder pipeline (T2→T4→T5) worked cleanly with zero regressions.** Each builder took handoff from the previous, ran `cargo test` to confirm all prior tests still passed, added their module, and verified again. No merge conflicts, no broken state passed forward. The dependency graph (T2 must complete before T4, T4 before T5) was respected exactly.

3. **Backend critic caught a real panic risk.** H1: `truncate_to_length()` sliced bytes on `char_indices().last()` which can cut a multi-byte UTF-8 codepoint in half, causing a panic on Arabic music titles. This is precisely the kind of edge case that never appears in ASCII-only test data. A fresh-eyes critic with no attachment to the code found it immediately. The fix was applied before any commit.

4. **Frontend critic approved quickly with minimal findings.** All 3 findings were cosmetic (LOW). The Riverpod 2.x migration and god-widget decomposition were clean enough that the critic had no architectural concerns. The structured migration approach (provider-by-provider, then widget split) paid off.

5. **Non-overlapping file ownership meant zero merge conflicts across both work streams.** Backend (T1–T5) and Flutter arch refactor touched completely separate files. When both sets of uncommitted changes were found on the same branch post-crash, committing them was a clean `git add` by directory with no conflict resolution needed.

6. **Single branch for both work streams simplified recovery.** The Flutter arch refactor had been started on a separate branch that never received any commits — all work was in a dirty working tree. Because both streams ended up on `feature/st-007-conversational-refinement`, there was one place to look and one place to commit. Multiple active branches with uncommitted state would have been harder to recover.

---

## What Didn't Work

1. **OOM crash with zero committed work.** Both parallel sessions ran their builders, accumulated ~10 tasks' worth of changes, and crashed before a single commit. This violated the "commit per phase" guidance that existed in spirit but not in explicit, enforced form. The crash recovery worked, but it was pure luck that the working tree was intact. A more severe crash (disk corruption, force-kill) would have lost everything.

2. **No formal parallel session protocol — file ownership was implicit.** Both sessions were started by a human knowing roughly what each was doing. There was no written record of which session owned which files. The `.claude/rules/parallel-sessions.md` file was created *after* the crash as a direct result of this gap. The protocol should have existed before the sessions started.

3. **Empty branch for the Flutter arch refactor.** The second session started on a separate branch, did all its work, then never committed. The branch name existed but had no commits. When the session crashed, the work was found as uncommitted changes on the ST-007 branch (the working directory, not the Flutter branch). This created confusion about where the work actually was.

4. **Phase 1 builders ran without worktree isolation.** The plan specified worktrees for Phase 1 parallel builders. In practice, the phase 1 work (T1+T3) was done in a single session without worktrees. If the T1 and T3 builders had crashed mid-work with conflicting partial changes in the same working tree, recovery would have been much harder.

---

## Patterns Identified

| Pattern | Frequency | Impact |
|---------|-----------|--------|
| Session handoff doc is the primary recovery mechanism | 1st explicit instance | Critical — 10-min recovery vs potential hours |
| OOM crash with uncommitted work | 1st instance | High — would be data loss on disk corruption |
| Single shared branch simpler than multiple active branches | 1st documented | Medium — reduces recovery complexity |
| Critic catches UTF-8/encoding bugs in LLM text pipelines | Recurring risk | High — Arabic titles are in the happy path |
| Parallel sessions without explicit ownership doc | 1st instance | High — created `parallel-sessions.md` |

---

## Comparison with Previous Steel Threads

| Metric | ST-005 | ST-006 | ST-007 | Trend |
|--------|--------|--------|--------|-------|
| Builder agents | 5 | 6 | 5 | Stable |
| Max parallelism | 3 | 4 | 3 (2 streams) | Stable |
| Critic HIGH findings | 4 | 2 | 1 | Improving |
| Critic LOW findings | — | — | 5 backend + 3 frontend | Now tracked |
| Files changed | ~13 | 27 | ~15 | Focused scope |
| Tests before → after | 176→315 | 315→376 | 376→432 | +56 backend |
| New frontend tests | 47 | — | 53 | Arch refactor |
| Merge conflicts | 0 | 0 | 0 | Non-overlapping works |
| Uncommitted work lost to crash | 0 | 0 | 0 (luck) | Fragile — needs commit discipline |
| Recovery incidents | 0 | 0 | 1 OOM | First OOM crash |
| Per-task commits | No | No | No | Still not done |

---

## Action Items

### Immediate (applied this session)

| # | Action | Status |
|---|--------|--------|
| 1 | Create `.claude/rules/parallel-sessions.md` with explicit ownership protocol | DONE |
| 2 | Add Post-Milestone Checklist to `CLAUDE.md` | DONE |
| 3 | Add Parallel Session Protocol to `CLAUDE.md` | DONE |
| 4 | Fix H1 (UTF-8 truncate panic) before commit | DONE — `c5d2431` |
| 5 | Accept 5 backend + 3 frontend LOW findings in `known-debt.md` | DONE |

### Future

| # | Action | Priority | Notes |
|---|--------|----------|-------|
| 6 | Commit after every completed task, not just at phase boundaries | Critical | OOM proved this is non-negotiable. Consider: lead commits stub after each builder hands off. |
| 7 | Use worktree isolation for parallel builders | High | Crash containment. If one builder's working tree is corrupted, others are unaffected. |
| 8 | Remove dead `apply_*` functions in `quick_commands.rs` or unify with `VersionTrackRow` | Medium | ST-007 critic L1. Only called from tests, not from production code. |
| 9 | Add `ascending` param to `SortByBpm` quick command | Low | ST-007 critic L2. Currently always ascending. |
| 10 | Set `parent_version_id` on LLM-refined versions | Low | ST-007 critic L4. History works via version_number ordering, but lineage chain is incomplete. |
| 11 | Add test for undo-with-only-v0 edge case | Low | ST-007 critic L5. |
| 12 | Unify Flutter catch patterns (`catch (e)` vs `on Exception catch (e)`) | Low | Frontend critic L1. |
| 13 | Deduplicate `MockInterceptor` in Flutter provider tests | Low | Frontend critic L2. |

---

## Key Learnings

1. **Commit after every task — OOM proved this is non-negotiable.** Both parallel sessions lost their work to an EC2 OOM crash. The recovery succeeded only because the working tree was intact after reboot. A harsher failure (disk error, git index corruption) would have meant re-implementing everything. "Commit per phase" is too coarse — commit after every completed task.

2. **Session handoff is the single most important cross-session artifact.** When the sessions crashed, `docs/session-handoff.md` was the map that said what was built, what each session was working on, and which files each owned. No other artifact provided this. Keep it updated not just at session end, but after each milestone.

3. **Explicit file ownership is required before starting parallel sessions.** Implicit knowledge of "session A does backend, session B does frontend" is not sufficient. If both sessions crash, the person recovering needs a written record to understand the state. The parallel-sessions protocol now makes this mandatory.

4. **Stale `.git/index.lock` is a common crash artifact.** After an OOM or process kill, git may leave a lock file that blocks all git operations. Check for and remove `.git/index.lock` as the first step in any crash recovery, before investigating file state.

5. **A single branch for related parallel work simplifies recovery.** The Flutter arch refactor was on a separate branch with no commits — all its work was in the working tree of the ST-007 branch. This created confusion about where the work was. For tightly related work streams (backend + frontend for same ST), a single branch with clear directory ownership is easier to recover than multiple branches with uncommitted state.

6. **Builder agents should use worktree isolation.** If a builder crashes mid-task with half-written files, it contaminates the working tree for all subsequent builders. Worktrees contain the blast radius to one isolated directory. The plan specified worktrees for Phase 1 — this should be enforced, not optional.
