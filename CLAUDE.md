# Ethnomusicology Project - Claude Code Directives

## Vision: DJ-First Music Platform

LLM-powered substitute DJ that generates setlists from natural language prompts, sources music from Spotify + Beatport + SoundCloud, supports harmonic mixing (Camelot wheel), and provides sequential preview playback with purchase links. Occasion-based features (Nikah, Eid, Mawlid) remain but are secondary to the DJ experience.

## Workflow: The Forge

Use the Forge (`.claude/` directory) for all development:
1. `/uc-create` — Create Cockburn-style use case
2. `/uc-review` — Devil's Advocate review (apply all fixes before coding)
3. `/task-decompose` — Break into implementable tasks with dependency graph
4. `design-crit` — Run for any UC with a frontend screen (Brief → Facet Plan → Crit Loops → Design Direction)
5. **Devil's Advocate Review** — After every plan is drafted, run a devil's advocate review before implementation. Use a Plan agent to find weaknesses, gaps, missing pieces, parallelism risks, integration seams, scope creep, and testing gaps. Address all critical and high-severity findings before coding begins.
6. Implement with agent teams — **lead MUST delegate to subagents, never implement solo** (see Agent Teams section). Concretely:
   - Create team via `TeamCreate` or spawn `Agent` subagents with `model: "sonnet"`
   - Assign non-overlapping files per task decomposition's file ownership matrix
   - Backend + frontend builders run in parallel (no shared files)
   - Lead coordinates, reviews, wires integration files (main.rs, mod.rs, routes)
   - **NEVER use external shell scripts** (e.g., `ralph-loop.sh`) for iteration — use Agent Teams or in-session Agent subagents for fresh context per task
7. **Two-Pass Critic Review** — Two sequential critic passes before verification. Both are MANDATORY — no exceptions, including tech debt PRs.
   - **7a. Security & Architecture Critic** — Spawn a fresh-context critic agent. Reads the diff cold. Checks: auth bypass, injection risks, data leaks, missing ownership checks, plan-vs-code compliance, missed edge cases, dead code, unused imports, test gaps. Critic sends feedback with file:line references. Lead assigns fixes before proceeding.
   - **7b. Code Quality Review** — Spawn a second fresh-context critic agent focused on language-specific quality:
     - **Rust**: ownership patterns, unwrap() in handlers, error propagation (? vs expect), clippy idioms, missing `?` on DB calls, transaction correctness, unused derives
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
- Use **sonnet for all coding agents**, **opus for critics and complex planning**. **Never use haiku.**
- **Pure-function modules** (scoring, algorithms, parsers) are ideal subagent targets — isolated, testable, no coordination overhead
- **Integration/wiring modules** (main.rs, routes, mod.rs) stay with the lead since they depend on all other modules

### Code Review: Two-Pass Critic Protocol (REQUIRED)

> **MANDATORY: Every implementation MUST have BOTH critic passes before `/verify-uc`.**
> **No exceptions — including tech debt PRs, "small fixes", and hotfixes.**
> Each critic agent runs in a FRESH context — it has NOT watched the code being written.
> This breaks the "I wrote it so I think it's fine" blind spot.
> A single critic cannot be expert in security AND Rust idioms AND Next.js patterns. Split the work.

**Pass 7a: Security & Architecture Critic**

After builders complete their work and quality gates pass:
1. Spawn a **Critic Agent** (fresh context, no worktree, use opus model)
2. Critic reads the diff (`git diff main...HEAD`), the plan, and the test output
3. Critic checks: missed edge cases, dead code, unused imports, naming inconsistencies, security issues (auth bypass, injection, data leaks, missing ownership checks), plan-vs-code compliance, test gaps
4. Critic sends feedback to lead — specific, actionable, with file:line references
5. Lead assigns fixes to builders or applies wiring fixes directly

**Pass 7b: Code Quality Review**

After 7a findings are resolved:
1. Spawn a **second fresh-context Critic Agent** focused on language quality (use opus model)
2. Critic reads the same diff
3. **Rust checklist**: unwrap() in request handlers (must use ?), error propagation patterns, transaction correctness, clippy idiom violations, dead derives, missing error variants
4. **Next.js/React checklist**:
   - ROUTING: Every new page MUST have a `page.tsx` in the App Router — unreachable pages are CRITICAL bugs
   - Error handling on all mutations (onError callback or try/catch with user feedback)
   - Loading state indicators on async actions (Loader2 spinner or disabled button)
   - Theme tokens (`text-primary`, `bg-card`, etc.) not hardcoded hex values
   - Keyboard accessibility: clickable divs need `role="button"`, `tabIndex`, `onKeyDown`
   - TanStack Query for server state, Zustand for client state — never mix
   - No hardcoded `#D4AF37` or `#1B2A4A` — use CSS custom property tokens
5. Critic sends feedback with file:line references
6. Lead assigns fixes before proceeding to `/verify-uc`

## Quality Gates

Pre-commit hook (`.claude/settings.json`) enforces:
```
Backend:  cargo fmt --check && cargo clippy -- -D warnings
Frontend: cd frontend-next && bunx vitest run
```

**Note:** `cargo test` requires `DATABASE_URL` pointing to Neon Postgres. Run manually when DB changes are made:
```
DATABASE_URL=<neon-url> RUST_TEST_THREADS=1 cargo test
```

## Commit Discipline

**Commit after every completed task**, not at the end of a sprint. This is crash recovery insurance — an OOM kill or session timeout should never lose more than one task's worth of work. ST-007 learned this the hard way when an OOM crash nearly lost an entire session. Each commit should be small, focused, and independently valid.

## Research Before Hacking

When you encounter a technology or integration you haven't used before, **STOP and research best practices before writing code**. Red flags that you're hacking: writing debug scripts, adding elaborate workarounds, trying multiple config tweaks in sequence. Instead: search the web for established patterns, check official docs, and use proven solutions.

## Shell Command Style

**Use separate Bash tool calls** for each command so auto-approve works without repeated permission prompts. Do NOT chain with `&&`.

## Key Commands

```bash
cd backend
cargo run                    # Start API server (port 3001)
cargo test                   # Run tests
cargo clippy -- -D warnings  # Lint

cd frontend-next
bun --bun next dev          # Start dev server (port 3000)
bunx vitest run             # Run tests
bun --bun next build        # Production build
```

## Project Context

- Use cases: `docs/use-cases/uc-*.md` | Tasks: `docs/tasks/{uc,st}-*-tasks.md`
- Steel threads: `docs/steel-threads/st-*.md` | Spikes: `docs/spikes/sp-*.md`
- Retrospectives: `docs/retrospectives/st-*-slug.md`
- MVP roadmap: `docs/mvp-roadmap.md` | Progress: `docs/mvp-progress.md`
- API contract: `docs/api/openapi.yaml` | PRD: `docs/prd.md`
- Design-crit outputs: `.design-crit/` | Research: `docs/research/`

## Current State

> **Live status lives in `MEMORY.md` and `docs/mvp-progress.md`. Do NOT duplicate here.**
> **Session-specific state lives in `docs/session-handoff.md`.**

- **GitHub**: `git@github.com:step2this/ethnomusicology.git`
- **Deployed**: `tarab.studio` (Caddy + systemd + Neon Postgres + Route53)

## Parallel Session Protocol

When multiple Claude Code sessions work simultaneously:
1. Read `docs/session-handoff.md` FIRST — it lists active work and file ownership
2. Claim your files in the handoff doc before starting
3. NEVER modify a file owned by another session
4. Commit after every completed task (crash recovery)
5. Shared files (main.rs, mod.rs, package.json) → only one session touches them
6. See `.claude/rules/parallel-sessions.md` for full protocol

## Post-Milestone Checklist (MANDATORY)

After every ST/UC completion, before creating a PR:
1. `/retrospective` — capture lessons
2. Update `docs/mvp-progress.md` — mark postconditions
3. Update `MEMORY.md` — test counts, current state
4. Update `.claude/rules/known-debt.md` — add accepted critic findings
5. Update `docs/api/openapi.yaml` — if new endpoints
6. `/session-handoff` — write handoff

## Context-Specific Rules

Detailed reference material loads automatically via `.claude/rules/` when working in relevant directories:
- `architecture.md` — tech stack, key patterns, external integrations (loads for `backend/`, `frontend/`, `e2e/`)
- `walking-skeleton.md` — PDCA, commands, workflow (loads for `docs/steel-threads/`, `docs/spikes/`)
- `lessons-learned.md` — UC-001, ST-003, ST-005 lessons + spike findings (loads for `docs/`)
- `known-debt.md` — tracked technical debt items (loads for `docs/`, `backend/`)
- `design-ux.md` — design-crit plugin, wireframe, UX review tools (loads for `frontend/`, `.design-crit/`)
