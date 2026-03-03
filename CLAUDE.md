# Ethnomusicology Project - Claude Code Directives

## Vision: DJ-First Music Platform

LLM-powered substitute DJ that generates setlists from natural language prompts, sources music from Spotify + Beatport + SoundCloud, supports harmonic mixing (Camelot wheel), and provides crossfade preview playback with purchase links. Occasion-based features (Nikah, Eid, Mawlid) remain but are secondary to the DJ experience.

## Workflow: The Forge

Use the Forge (`.claude/` directory) for all development:
1. `/uc-create` — Create Cockburn-style use case
2. `/uc-review` — Devil's Advocate review (apply all fixes before coding)
3. `/task-decompose` — Break into implementable tasks with dependency graph
4. `design-crit` — Run for any UC with a frontend screen (Brief → Facet Plan → Crit Loops → Design Direction)
5. **Devil's Advocate Review** — After every plan is drafted, run a devil's advocate review before implementation. Use a Plan agent to find weaknesses, gaps, missing pieces, parallelism risks, integration seams, scope creep, and testing gaps. Address all critical and high-severity findings before coding begins.
6. Implement with agent teams — **lead MUST delegate to subagents, never implement solo** (see Agent Teams section)
7. **Critic Review** — Spawn a fresh-context critic agent to review the diff before verification. This is NOT optional. Critic also checks **plan-vs-code compliance** — are all planned features implemented? (ST-005 retro: auto-enrich trigger was planned but dropped during task decomposition and no one caught it.)
8. `/verify-uc` — Validate implementation against postconditions
9. `/grade-work` — Score completed work
10. **Retrospective** — Write `docs/retrospectives/st-NNN-slug.md`, update action items, feed lessons into CLAUDE.md
11. `/session-handoff` — **ALWAYS update `docs/session-handoff.md` when finishing a milestone or ending a session.** This is the primary cross-session continuity mechanism.

## Agent Teams

> **MANDATORY: The lead agent MUST NOT implement code directly.**
> The lead coordinates, reviews, wires modules together, and runs quality gates.
> All implementation code MUST be written by spawned subagents (Builder teammates).
> This is not optional. Violating this rule caused context rot and quality degradation in ST-003.

> **MANDATORY: Use multi-agent teams for any task touching >2 files or >100 lines.**
> Create a team via `TeamCreate`, spawn builders in parallel on non-overlapping files.
> A single agent writing 2,800+ lines across 14 files is an anti-pattern — it exhausts context
> and eliminates the fresh-eyes review benefit that separate agents provide.

- **Backend agents work in the same repo** — assign non-overlapping module directories
- Use traits/interfaces at boundaries so parallel agents don't block each other
- Use Haiku agents for research tasks (API evaluation, tool research)
- **Pure-function modules** (scoring, algorithms, parsers) are ideal subagent targets — isolated, testable, no coordination overhead
- **Integration/wiring modules** (main.rs, routes, mod.rs) stay with the lead since they depend on all other modules

### Code Review: Critic Agent (REQUIRED)

> **MANDATORY: Every implementation MUST have a separate critic review before `/verify-uc`.**
> The critic agent runs in a FRESH context — it has NOT watched the code being written.
> This breaks the "I wrote it so I think it's fine" blind spot.

After builders complete their work and quality gates pass:
1. Spawn a **Critic Agent** (general-purpose, fresh context, no worktree)
2. Critic reads the diff (`git diff main...HEAD`), the plan, and the test output
3. Critic looks for: missed edge cases, dead code, unused imports, naming inconsistencies, security issues, plan deviations, test gaps
4. Critic sends feedback to lead via messaging — specific, actionable, with file:line references
5. Lead assigns fixes to builders or applies wiring fixes directly
6. Only after critic approves does the lead proceed to `/verify-uc`

## Quality Gates

Both must pass before any commit (enforced by `.claude/settings.json` pre-commit hook):
```
Backend:  cargo fmt --check && cargo clippy -- -D warnings && cargo test
Frontend: flutter analyze && flutter test
```

## Research Before Hacking

When you encounter a technology or integration you haven't used before, **STOP and research best practices before writing code**. Red flags that you're hacking: writing debug scripts, adding elaborate workarounds, trying multiple config tweaks in sequence. Instead: search the web for established patterns, check official docs, and use proven solutions.

## Shell Command Style

**Chain commands with `&&`** to avoid repeated permission prompts:
```bash
cd backend && cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

## Key Commands

```bash
cd backend && cargo run                    # Start API server (port 3001)
cd backend && cargo test                   # Run tests
cd backend && cargo clippy -- -D warnings  # Lint
cd frontend && flutter run -d chrome       # Start web app
cd frontend && flutter analyze && flutter test  # Lint + test
```

## Project Context

- Use cases: `docs/use-cases/uc-*.md` | Tasks: `docs/tasks/{uc,st}-*-tasks.md`
- Steel threads: `docs/steel-threads/st-*.md` | Spikes: `docs/spikes/sp-*.md`
- Retrospectives: `docs/retrospectives/st-*-slug.md`
- MVP roadmap: `docs/mvp-roadmap.md` | Progress: `docs/mvp-progress.md`
- API contract: `docs/api/openapi.yaml` | PRD: `docs/prd.md`
- Design-crit outputs: `.design-crit/` | Research: `docs/research/`

## Current State

- **Branch**: `feature/st-006-enhanced-generation` (PR #3 open, CI fix in progress)
- **Completed**: UC-001, ST-001, ST-003, ST-004, ST-005, ST-006, SP-001–SP-004
- **MVP Roadmap**: Phase 0 (SP-004) + Phase 1 (ST-005) + Phase 2 (ST-006) complete. See `docs/mvp-roadmap.md`
- **Next**: AWS deployment, then ST-007 (Conversational Refinement)
- **Test count**: 268 backend tests, 47 frontend tests (315 total) — all passing
- **GitHub**: `git@github.com:step2this/ethnomusicology.git`

## Context-Specific Rules

Detailed reference material loads automatically via `.claude/rules/` when working in relevant directories:
- `architecture.md` — tech stack, key patterns, external integrations (loads for `backend/`, `frontend/`, `e2e/`)
- `walking-skeleton.md` — PDCA, commands, workflow (loads for `docs/steel-threads/`, `docs/spikes/`)
- `lessons-learned.md` — UC-001, ST-003, ST-005 lessons + spike findings (loads for `docs/`)
- `known-debt.md` — tracked technical debt items (loads for `docs/`, `backend/`)
- `design-ux.md` — design-crit plugin, wireframe, UX review tools (loads for `frontend/`, `.design-crit/`)
