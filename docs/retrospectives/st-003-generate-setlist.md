# Retrospective: ST-003 Generate Setlist from Prompt

**Date**: 2026-03-02
**Milestone**: Steel Thread 003 — LLM-powered setlist generation + harmonic arrangement
**Scope**: 9 tasks (T1-T9), 14 new files, ~2,800 lines of code, 135 backend tests + 12 frontend tests

---

## What Worked

1. **Detailed upfront plan**: The devil's advocate review caught 14 issues before implementation began. Shared type definitions (exact SQL DDL, Rust structs, API response shapes) eliminated interface mismatches between modules.

2. **Pre-commit hook enforcement**: Backend quality gates (fmt, clippy, test) caught formatting and lint issues automatically. Every commit was clean on the backend side.

3. **Test-driven implementation**: Writing tests alongside code (not after) caught real bugs: hallucinated track_id reclassification, markdown fence stripping, edge cases in Camelot wheel wrapping.

## What Didn't Work

1. **Lead agent implemented everything solo**: The plan explicitly specified 4 parallel agents (Lead, Backend-1, Backend-2, Frontend). Instead, the lead agent wrote all 9 tasks sequentially. This violated the "lead coordinates, does not implement" directive in CLAUDE.md.

2. **Context rot in later tasks**: Early tasks (T1-T3) compiled on first try. By T8-T9 (frontend), the agent was making errors it wouldn't make with fresh context: wrong package name (`ethnomusicology` vs `ethnomusicology_frontend`), invalid base URL for test VM, constructor mismatches. Each required multiple fix iterations.

3. **No independent code review**: The agent reviewed its own code. There was no separate critic agent or fresh-context reviewer. Self-review misses the same class of issues the writer created — it's the "I wrote it so it's fine" blind spot.

4. **Pre-commit hook was half-built**: Only backend checks were enforced. Flutter analyze + test were NOT in the hook. Frontend code quality was not gated.

5. **No mechanism to enforce team delegation**: CLAUDE.md said "always prefer multi-agent teams" but nothing prevented the lead from ignoring it. Directives without enforcement are suggestions.

## Patterns Identified

| Pattern | Frequency | Impact |
|---------|-----------|--------|
| Context rot in long sessions | Every session >6 tasks | High — later code has more errors per line |
| Self-review blind spot | Every solo implementation | High — catches different bugs than fresh reviewer |
| Pure-function modules ideal for delegation | camelot.rs, arrangement.rs | Medium — zero coordination overhead |
| Formatting errors compound | Every new file | Low — fixable but noisy |

## Action Items

### Immediate (Applied)

| # | Action | Status |
|---|--------|--------|
| 1 | Add Flutter checks to pre-commit hook | Done |
| 2 | Add "lead MUST NOT implement" imperative to CLAUDE.md (blockquote emphasis) | Done |
| 3 | Add "multi-agent mandatory for >2 files" imperative to CLAUDE.md | Done |
| 4 | Add Critic Agent requirement to Forge workflow | Done |
| 5 | Add ST-003 lessons learned to CLAUDE.md | Done |
| 6 | Fix workflow numbering (was two step 7s) | Done |

### Future

| # | Action | Priority |
|---|--------|----------|
| 7 | Create `/code-review` slash command that spawns a fresh-context critic | High |
| 8 | Explore Ralph Wiggum loops for pure-function modules (camelot, arrangement) | Medium |
| 9 | Add enforcement hook that detects lead writing >200 lines and warns | Medium |
| 10 | Update implementation-team.md Reviewer role to use fresh-context pattern | High |

## Key Learnings

1. **Directives are not enforcement.** "Always use teams" written in CLAUDE.md doesn't prevent a solo run. You need either hooks (automated enforcement) or workflow gates (human checkpoints) to make it real.

2. **Context rot is measurable.** Track error rate per task. If task N has 0 fix iterations and task N+6 has 5, the context is degraded. Split work across agents to keep each one fresh.

3. **The generator-critic separation is essential.** The agent that writes code should not be the only one reviewing it. A separate agent with fresh context catches a fundamentally different class of bugs.

4. **Pure-function modules are delegation gold.** No IO, no state, deterministic — these should ALWAYS go to a subagent. The lead should never write them directly.

5. **Ralph Wiggum loops fit test-heavy pure modules.** Autonomous `while true; do claude; done` with test backpressure would work well for modules like camelot.rs (17 tests) and arrangement.rs (10 tests). Quality comes from iteration + automated feedback, not self-assessment.
