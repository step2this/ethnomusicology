# Retrospective: Phase 8 Session — 2026-03-05

**Date**: 2026-03-05
**Milestone**: ST-010 + Tech Debt + Diversity Tuning + Phase 8 (Saved Setlists/Crates/Spotify)
**Scope**: SP-007, ST-010, PR #11 (tech debt), PR #13 (diversity tuning), Phase 8 (PR #14)

---

## What Worked

1. **Devil's Advocate review on Phase 8 plan caught 5 issues before any builder started.** SQLite FK enforcement gap, Spotify CC flow needing scratch build, models.rs ownership split, multi-table delete ordering, and parallel audio search optimization. All addressed in the plan before builders touched any code.

2. **SP-007 spike-first approach validated.** LLM self-verification spike confirmed that skill doc injection and confidence calibration work before building ST-010. No rework during implementation.

3. **Multi-builder parallelism continued to work.** ST-010 used 2 builders + lead with zero file conflicts. Phase 8 planned for 5 builders across 3 workstreams with clean ownership boundaries.

4. **Security critic caught real issues on Phase 8.** Authentication gaps were identified and addressed before merge. This validates the critic pattern for security review.

5. **Diversity tuning critic caught URL encoding and rate limiting issues.** The architecture/security dimension of the critic review is producing value consistently.

6. **Test counts continued growing.** 510 total (end of Mar 4) to 526 total (end of session). Quality gates (cargo fmt + clippy + test + flutter analyze + flutter test) caught regressions at commit time.

---

## What Didn't Work

1. **Code quality reviews were NOT automatic — required explicit user request.** The Forge workflow step 7 (Critic Review) was interpreted as architecture/security review only. Language-specific code quality (Rust idioms, Flutter patterns, error handling, widget structure) was not included. The user had to explicitly ask for "Rust code quality review" and "Flutter code quality review" after Phase 8 was already considered done.

2. **Flutter code quality review found a CRITICAL routing bug.** The `SetlistDetailScreen` was unreachable because its route was never registered in the router. This would have shipped to production as a dead feature — users would generate and save setlists but never be able to view them from the library. A standard code quality review would have caught this immediately by checking that all new screens are routed.

3. **Flutter review found 3 HIGH issues on top of the CRITICAL.** Missing error handling on delete operations, no loading states on async actions, and hardcoded strings that should use theme tokens. These are standard Flutter code quality concerns that the security-focused critic has no mandate to check.

4. **PR #11 (tech debt) had NO critic review at all.** It was treated as "just small fixes" and skipped the review step entirely. Tech debt PRs can introduce regressions just as easily as feature PRs — the "it's just cleanup" assumption is dangerous.

5. **The critic review checklist in CLAUDE.md is security/architecture biased.** The explicit checklist is: "missed edge cases, dead code, unused imports, naming inconsistencies, security issues, plan deviations, test gaps." It does NOT mention: routing completeness, error handling patterns, loading states, widget structure, Rust ownership patterns, or any language-specific quality dimensions.

---

## Root Cause Analysis

The Forge workflow has a single "Critic Review" step that is described in terms of diff review and security. There is no mention of language-specific code quality as a separate review dimension. This creates two failure modes:

**Failure Mode 1: Narrow interpretation.** The critic agent focuses on what the checklist says — security, dead code, plan compliance — and does not look for Flutter routing bugs or Rust pattern issues because those aren't in the mandate.

**Failure Mode 2: Scope collapse under time pressure.** When a large diff is ready for review, the tendency is to run ONE critic and move on. A single critic cannot be expert in both security architecture AND Rust idioms AND Flutter patterns. The review becomes shallow across all dimensions rather than deep on any.

---

## Patterns Identified

| Pattern | Frequency | Impact |
|---------|-----------|--------|
| Code quality review missing unless user requests it | 1st documented | CRITICAL — routing bug would have shipped |
| Security critic is working well | Recurring (ST-010, PR #13, Phase 8) | Positive — keep this |
| Tech debt PRs skip review | 1st documented (PR #11) | Medium — false sense of safety |
| Single critic tries to cover too many dimensions | Structural gap | High — depth suffers |

---

## Action Items

### Immediate (process changes required)

| # | Action | File | Priority |
|---|--------|------|----------|
| 1 | Split Forge step 7 into 7a (Security/Architecture Critic) + 7b (Code Quality Review) | `CLAUDE.md` | Critical |
| 2 | Add explicit code quality checklist covering Rust + Flutter dimensions | `CLAUDE.md` | Critical |
| 3 | Add lesson to MEMORY.md Critical Process Rules | `MEMORY.md` | High |
| 4 | Add session lessons to `lessons-learned.md` | `.claude/rules/lessons-learned.md` | High |
| 5 | Add process debt to `known-debt.md` | `.claude/rules/known-debt.md` | High |
| 6 | Make critic review mandatory for ALL PRs including tech debt | `CLAUDE.md` | High |

### Specific Fixes from Phase 8 Flutter Review

| # | Finding | Severity | Status |
|---|---------|----------|--------|
| F1 | `SetlistDetailScreen` route not registered in router | CRITICAL | Needs fix |
| F2 | Delete operations missing error handling | HIGH | Needs fix |
| F3 | Async actions missing loading states | HIGH | Needs fix |
| F4 | Hardcoded strings instead of theme tokens | HIGH | Needs fix |

---

## Comparison with Previous Reviews

| Review | Security/Arch Findings | Code Quality Findings | Code Quality Done? |
|--------|----------------------|----------------------|-------------------|
| ST-005 | 4 HIGH (position off-by-one, validation) | Not separately reviewed | No |
| ST-006 | 2 HIGH (dead code, incomplete flow) | Not separately reviewed | No |
| ST-007 | 1 HIGH (UTF-8 panic), 5+3 LOW | Frontend critic done separately | Yes (first time) |
| ST-010 | Architecture reviewed | Not separately reviewed | No |
| PR #11 | NO REVIEW AT ALL | Not reviewed | No |
| PR #13 | URL encoding, rate limiting | Not separately reviewed | No |
| Phase 8 | Auth issues caught | 1 CRITICAL + 3 HIGH found only after user asked | No (until user forced it) |

ST-007 was the only time a separate frontend critic was run, and it was the cleanest Flutter code in the project. This is not a coincidence.

---

## Key Learnings

1. **A single "critic review" step is insufficient for a multi-language project.** Security review and code quality review are different skills requiring different checklists. Combining them into one step means one dimension gets shortchanged. Split into two explicit steps with separate checklists.

2. **"Just cleanup" PRs need review too.** PR #11 (tech debt) was 13 items across backend and frontend with no review. The assumption that small fixes are safe is the same assumption that lets typos become outages. Every PR gets a review — no exceptions.

3. **The Flutter routing bug proves code quality review catches different things than security review.** A security critic looks for auth bypass, injection, data leaks. A code quality reviewer looks for unreachable screens, missing error states, broken navigation. These are orthogonal concerns.

4. **ST-007 was the template — a separate frontend critic was run and approved quickly.** That session's Flutter code was the cleanest shipped so far. The process worked when it was followed. The problem is it was not codified as mandatory, so subsequent sessions skipped it.

5. **The user should not need to be the quality gate.** The entire point of the Forge workflow is that process steps are automatic. When the user has to say "wait, did you do a code quality review?" the process has failed. The checklist must be explicit enough that the agent follows it without prompting.
