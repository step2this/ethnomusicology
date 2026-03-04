# Workflow v2: Process Upgrade Plan

## Diagnosis

After completing 6 steel threads, 4 spikes, 13 use cases, and deploying to production, we've accumulated significant process debt alongside a working product. The Forge workflow produces high-quality work **when followed**, but it has three systemic failures:

1. **Document rot** — we write great docs initially but never update them after implementation
2. **No enforcement** — the process gets skipped under pressure (UC-019 Phase 2 thrashed 14 cycles)
3. **No multi-session protocol** — two Claudes working simultaneously have no coordination mechanism

## Part 1: Process Fixes

### 1.1 Single Source of Truth (Kill the Rot)

**Problem:** CLAUDE.md, MEMORY.md, PRD, MVP roadmap, MVP progress, session handoff, and architecture rules all track overlapping status information. They drift immediately.

**Fix:** Establish a clear hierarchy:
- **CLAUDE.md** → Stable directives only (workflow, rules, commands). NO current state, NO branch info, NO test counts. These change too fast.
- **MEMORY.md** → Current state + user preferences + lessons. This is the "live" doc that gets updated every session.
- **`docs/session-handoff.md`** → Ephemeral. Overwritten each session. The "what's happening right now" doc.
- **`docs/mvp-progress.md`** → The canonical status matrix. Updated as part of every `/verify-uc` run (automated gate).
- **PRD, roadmap, UC docs** → Treated as specs, not status trackers. Remove status columns from PRD. Roadmap gets updated only at milestones.

**Action items:**
- [ ] Strip current state from CLAUDE.md (point to MEMORY.md and mvp-progress.md instead)
- [ ] Fix all CLAUDE.md contradictions (haiku, &&, branch, test count)
- [ ] Add "update mvp-progress.md" as a mandatory step in `/verify-uc`
- [ ] Rewrite PRD status section to reference mvp-progress.md instead of maintaining its own

### 1.2 Enforce the Forge (Pre-Flight Checks)

**Problem:** Steps get skipped. UC-019 Phase 2 skipped task decomposition and critic review.

**Fix:** Add a **pre-flight gate** that the lead must run before spawning any builder. This is already partially captured in `pre-implementation-checklist.md` but it's not enforced.

**New rule for CLAUDE.md:**
```
## Pre-Flight Gate (MANDATORY)
Before ANY implementation work begins:
1. Task file exists in docs/tasks/
2. Session handoff updated with file ownership table
3. Devil's advocate review completed (for >3 tasks)
4. Lead has NOT written any implementation code

Skipping this gate is the #1 predictor of thrashing.
```

### 1.3 Multi-Session Protocol

**Problem:** No coordination mechanism for 2-3 parallel Claude Code sessions.

**Fix:** Formalize what worked organically in the OOM session:

**New file: `.claude/rules/parallel-sessions.md`**
```
Trigger: Always loaded (or on docs/**, backend/**, frontend/**)

## Parallel Session Protocol

### Starting a Parallel Session
1. Read `docs/session-handoff.md` FIRST
2. Claim your files in the "File Ownership" table (edit the handoff doc)
3. Create your feature branch (or worktree) from main
4. Your branch name format: `feature/{st|uc}-NNN-{slug}-{stream}`
   e.g., `feature/st-007-refinement-backend`, `feature/st-007-refinement-frontend`

### File Ownership Rules
- Each session MUST list files it owns in session-handoff.md
- NEVER modify a file owned by another session
- Shared files (main.rs, mod.rs, pubspec.yaml) → only one session touches them
- If you need a shared file: finish your work, commit, then coordinate

### Communication
- Session handoff is the message board — check it before starting work
- Each session commits frequently (every completed task)
- If OOM/crash: git log + session-handoff.md = full recovery

### Merge Strategy
- Each stream creates its own PR
- If streams touch the same repo area: merge one first, rebase the other
- If truly independent (backend vs frontend): can merge in any order
```

### 1.4 Mandatory Post-Milestone Checklist

**Problem:** Retros, status updates, and debt tracking get forgotten.

**Fix:** Add to the Forge workflow after `/grade-work`:

```
## Post-Milestone (MANDATORY)
After every ST/UC completion:
1. `/retrospective` — capture lessons
2. Update `docs/mvp-progress.md` — mark postconditions done
3. Update `MEMORY.md` — test counts, current state
4. Update `.claude/rules/known-debt.md` — add critic findings
5. Update `docs/api/openapi.yaml` — if new endpoints added
6. Clean up stale branches: `git branch -d feature/old-branch`
```

## Part 2: Tool Cleanup

### 2.1 Deduplicate
| Current | Action |
|---------|--------|
| `ralph-loop.md` + `fix-ci.md` | Merge into single `fix-ci.md` with `model: "sonnet"` |
| Two `grade-work` commands | Delete simple `commands/grade-work.md`, keep skill version |
| `code-quality.md` vs pre-commit hook | Keep both (different use cases: standalone vs commit-time) |

### 2.2 Update Stale Files
| File | What's Wrong | Fix |
|------|-------------|-----|
| `CLAUDE.md` Current State | Branch, tests, completed items all wrong | Remove section, point to MEMORY.md |
| `CLAUDE.md` Agent Teams | "Use Haiku for research" | Change to "Use sonnet or opus only. Never haiku." |
| `CLAUDE.md` Shell Style | "Chain with &&" | Change to "Separate tool calls for auto-approve" |
| `.claude/rules/architecture.md` | Says PostgreSQL in prod | Fix to SQLite. Add Deezer integration. |
| `.claude/agents/ux-team.md` | Occasion-first framing | Update to DJ-first |
| `.claude/agents/testing-team.md` | References uninstalled tools | Mark as aspirational or remove |
| `.claude/skills/cockburn-template.md` | Stale actor list (just_audio, MusicBrainz) | Update to current stack |
| `docs/steel-threads/st-006-*.md` | Status says IN PROGRESS | Mark COMPLETE |
| `docs/prd.md` | Status column frozen | Remove status column, add pointer to mvp-progress.md |

### 2.3 Fill Documentation Gaps
| Missing | Priority | Action |
|---------|----------|--------|
| Deployment skill/rule | High | Create `.claude/rules/deployment.md` covering tarab.studio architecture |
| ST-007 steel thread doc | Medium | Create `docs/steel-threads/st-007-conversational-refinement.md` |
| SP-005 spike doc | Low | Create from session retro notes (historical record) |
| OpenAPI for ST-007 | High | Add /refine, /revert/{v}, /history endpoints |
| Critic review skill | Medium | Extract critic checklist into `.claude/skills/critic-checklist.md` |

## Part 3: Workflow v2

### The New Forge Workflow

```
Phase 0: PLAN
  1. /uc-create or /st-create — write the spec
  2. /uc-review — devil's advocate review (fix ALL issues before proceeding)
  3. /task-decompose — break into tasks with file ownership
  4. design-crit — if frontend screen involved
  5. Devil's advocate on task plan — catch structural issues
  6. /api-contract — if new endpoints

Phase 1: PRE-FLIGHT
  7. Pre-flight checklist (task file, handoff, ownership table)
  8. Update session-handoff.md with file ownership
  9. If parallel sessions: claim files, create branches

Phase 2: BUILD
  10. Spawn agent team — lead coordinates, NEVER codes
  11. Builders implement in parallel (non-overlapping files)
  12. Each builder commits after completing their task
  13. Quality gates pass before each commit (pre-commit hook)

Phase 3: REVIEW
  14. Critic agent (opus, fresh context) reviews full diff
  15. Lead assigns fixes from critic findings
  16. Critic approves

Phase 4: VERIFY
  17. /verify-uc — run postcondition checks
  18. /grade-work — score against rubric

Phase 5: CLOSE
  19. /retrospective — capture lessons
  20. Update mvp-progress.md ← NEW (mandatory)
  21. Update known-debt.md with accepted critic findings ← NEW (mandatory)
  22. Update MEMORY.md with current state ← NEW (mandatory)
  23. Update openapi.yaml if new endpoints ← NEW (mandatory)
  24. /session-handoff — write handoff for next session
  25. Clean stale branches ← NEW (mandatory)
  26. Create PR
```

### Multi-Session Sprint Layout

For 2-3 parallel Claudes:

```
┌─────────────────────────────────────────────────────┐
│ Session 0: COORDINATOR (you, the human)             │
│ - Reads session-handoff.md                          │
│ - Assigns streams to sessions                       │
│ - Resolves conflicts if sessions need shared files  │
│ - Merges PRs in order                               │
└─────────────────────────────────────────────────────┘
        │                    │                    │
        ▼                    ▼                    ▼
┌───────────────┐  ┌───────────────┐  ┌───────────────┐
│ Session 1     │  │ Session 2     │  │ Session 3     │
│ Backend work  │  │ Frontend work │  │ Infra/CI/docs │
│               │  │               │  │               │
│ Own branch    │  │ Own branch    │  │ Own branch    │
│ Own files     │  │ Own files     │  │ Own files     │
│ Own team      │  │ Own team      │  │ Own team      │
│               │  │               │  │               │
│ Updates:      │  │ Updates:      │  │ Updates:      │
│ - handoff.md  │  │ - handoff.md  │  │ - handoff.md  │
│ - commits     │  │ - commits     │  │ - commits     │
│   frequently  │  │   frequently  │  │   frequently  │
└───────────────┘  └───────────────┘  └───────────────┘
```

**Key rules for parallel sessions:**
1. Each session is a self-contained Forge run (plan → build → review → verify)
2. File ownership is exclusive — no sharing
3. Session handoff is the coordination bus
4. Commit early and often (crash recovery)
5. If a session needs work from another session: wait for their PR to merge, then rebase

## Part 4: Immediate Action Items

### Priority 1: Fix the Foundation (do now)
- [ ] Fix CLAUDE.md contradictions and remove stale Current State section
- [ ] Fix architecture.md (SQLite, add Deezer)
- [ ] Merge ralph-loop into fix-ci
- [ ] Update MEMORY.md (clean up stale entries, fix UC-019 section)
- [ ] Create parallel-sessions.md rule

### Priority 2: Fill Gaps (this session or next)
- [ ] Create deployment.md rule
- [ ] Update openapi.yaml with ST-007 endpoints
- [ ] Create critic-checklist.md skill
- [ ] Update ux-team.md and testing-team.md agents
- [ ] Delete simple grade-work.md command

### Priority 3: Status Reconciliation (next milestone)
- [ ] Update PRD (remove status column, point to mvp-progress)
- [ ] Mark ST-006 COMPLETE in steel thread doc
- [ ] Write ST-007 steel thread doc
- [ ] Create SP-005 spike doc from session retro
- [ ] Clean up 6 stale git branches

## Part 5: What's Next for the Product

### Remaining MVP Items (from mvp-progress.md)
- **ST-007 Frontend**: Conversational UI (chat input, version history, undo) — backend is done
- **Granular generation progress**: Show LLM processing stages to user
- **UC-020**: Purchase links (URL construction, no API)
- **UC-024**: Export setlist with transition notes

### Post-MVP
- Beatport integration (UC-013, SP-001 findings)
- SoundCloud integration (UC-014)
- essentia sidecar for audio analysis (SP-003 findings)
- iOS/mobile deployment spike
- Held-Karp TSP for n<=20 (UC-017)
- Full browser DJ mix (UC-025, aspirational)
