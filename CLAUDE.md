# Ethnomusicology Project - Claude Code Directives

## Vision: DJ-First Music Platform

LLM-powered substitute DJ that generates setlists from natural language prompts, sources music from Spotify + Beatport + SoundCloud, supports harmonic mixing (Camelot wheel), and provides crossfade preview playback with purchase links. Occasion-based features (Nikah, Eid, Mawlid) remain but are secondary to the DJ experience.

## Workflow: The Forge

Use the Forge (`.claude/` directory) for all development:
1. `/uc-create` — Create Cockburn-style use case
2. `/uc-review` — Devil's Advocate review (apply all fixes before coding)
3. `/task-decompose` — Break into implementable tasks with dependency graph
4. `design-crit` — Run for any UC with a frontend screen (Brief → Facet Plan → Crit Loops → Design Direction)
5. **Devil's Advocate Review** — After every plan is drafted, run a devil's advocate review before implementation. Use a Plan agent to find weaknesses, gaps, missing pieces, parallelism risks, integration seams, scope creep, and testing gaps. Address all critical and high-severity findings before coding begins. Proven valuable in ST-003 planning — caught 14 issues including missing retry logic, unspecified migration DDL, and no test for hallucinated track IDs.
6. Implement with agent teams — **lead MUST delegate to subagents, never implement solo** (see Agent Teams section)
7. **Critic Review** — Spawn a fresh-context critic agent to review the diff before verification. This is NOT optional. See "Code Review: Critic Agent" in Agent Teams section.
8. `/verify-uc` — Validate implementation against postconditions
9. `/grade-work` — Score completed work
10. `/session-handoff` — Save state before ending sessions

## Walking Skeleton

Steel threads and spikes extend the Forge to de-risk frontend-backend integration before full UC implementation.

### PDCA Mapping

| PLAN | DO | CHECK | ACT/ADJUST |
|------|----|-------|------------|
| `/spike-create` (unknowns) | Implement | `/verify-uc ST-NNN` | Update parent UCs |
| `/st-create` | (agent teams, | `/grade-work ST-NNN` | Update API contract |
| `/uc-review ST-NNN` | parallel on contract) | | Update CLAUDE.md |
| `/task-decompose ST-NNN` | | | |
| `/api-contract ST-NNN` | | | |
| `design-crit` (if frontend) | | | |

### New Commands

| Command | Purpose |
|---------|---------|
| `/st-create` | Create a Cockburn-format steel thread with cross-cutting fields |
| `/spike-create` | Create a time-boxed spike with hypothesis/findings/decision |
| `/api-contract` | Write/update OpenAPI section for a steel thread's endpoints |

### Artifact Locations

| Artifact | Path |
|----------|------|
| Steel threads | `docs/steel-threads/st-NNN-slug.md` |
| Spikes | `docs/spikes/sp-NNN-slug.md` |
| API contract | `docs/api/openapi.yaml` |

### Workflow

1. **Spike** unknowns first (`/spike-create`) — time-boxed, lightweight
2. **Create steel thread** (`/st-create`) — thin vertical slice across UCs
3. **Define API contract** (`/api-contract ST-NNN`) — OpenAPI single source of truth
4. **Review** (`/uc-review ST-NNN`) — devil's advocate, validates contract against assertions
5. **API Contract Review Gate** — both frontend and backend agents confirm before implementation
6. **Implement** — backend TO the contract, frontend FROM the contract. **Lead delegates, never implements solo.**
7. **Critic Review** — fresh-context agent reviews diff cold (REQUIRED before verify)
8. **Verify** (`/verify-uc ST-NNN`) — integration assertions + contract conformance
9. **Grade** (`/grade-work ST-NNN`) — Integration Proof category (20% weight)

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
- Use traits/interfaces at boundaries so parallel agents don't block each other (e.g., `ImportRepository` trait, `MusicSourceClient` trait)
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

This replaces the in-session Reviewer role from `implementation-team.md`, which reviewed code it watched being written and thus couldn't catch context-rot artifacts.

### Lessons Learned (UC-001)
- Two backend agents can work in parallel if they own separate directories (`src/api/` vs `src/db/`)
- Agents handle `mod.rs` creation and module wiring well when given clear boundaries
- Define shared traits up front in prompts so agents produce compatible interfaces
- Run `cargo check` before spawning agents to warm the dependency cache — saves 30s+ per agent
- Frontend can be done sequentially after backend since it depends on API shapes
- `package:web` fails in Flutter test VM — use `url_launcher` for cross-platform URL handling

### Lessons Learned (Walking Skeleton)
- Spikes before steel threads: SP-001/002/003 revealed constraints (container sizing, CORS, key format) that would have caused rework mid-implementation
- Error response shape must be agreed early: flat `{"error": "msg"}` vs nested `{"error": {"code": "...", "message": "..."}}` spans every endpoint — fix once before building more
- Migration 003 is a gating prerequisite: tracks table lacks bpm, camelot_key, source columns that every DJ feature depends on
- Artist data is relational: track API must JOIN track_artists + artists tables and concatenate names
- API contract review gate works: both frontend and backend agents must confirm OpenAPI spec before writing implementation code
- Research-only spikes are valid: SP-002 produced actionable decisions without a prototype

### Lessons Learned (ST-003)
- **Lead-as-solo-builder is an anti-pattern**: Lead implemented all 9 tasks (~2,800 lines, 14 new files) solo despite plan specifying 4 parallel agents. By task T8-T9 (frontend), context was exhausted — chasing formatting errors, wrong package names, assertion mismatches that a fresh agent would catch instantly.
- **Pre-commit hook was incomplete**: Only ran backend checks (cargo fmt/clippy/test). Flutter analyze/test was NOT enforced. Fixed: hook now runs both backend and frontend gates.
- **In-session reviewer is theater**: The Reviewer role in implementation-team.md watches code being written in the same session. It cannot catch what it already "knows" — this is the self-review blind spot. Replaced with Critic Agent pattern: fresh context, reads diff cold, finds what the builder missed.
- **Pure-function modules are perfect subagent targets**: camelot.rs (17 tests), arrangement.rs (10 tests) — zero IO, deterministic, isolated. These should always be delegated to separate agents.
- **Ralph Wiggum loops fit pure modules**: `while true; do claude; done` with test backpressure works for isolated, well-tested modules. Quality comes from iteration + automated feedback, not self-assessment. Consider for future camelot/arrangement-type work.
- **Context rot is real and measurable**: Early tasks (T1-T3) produced clean code on first try. Later tasks (T8-T9) required 5+ fix iterations for issues a fresh agent wouldn't create (wrong package name, constructor mismatch, base URL invalid in test VM).
- **Generator-Critic separation is essential**: The agent that writes code should not be the only one reviewing it. A separate agent with fresh context and adversarial intent catches a different class of bugs.

### Spike Findings Summary

| Spike | Hypothesis | Result | Key Decision |
|-------|-----------|--------|-------------|
| SP-001 Beatport | v4 API provides BPM/key with usable access | Partially confirmed | OAuth2 w/ public client_id scraping. BPM=integer, key=shortName (needs Camelot map). Rate limits unknown — throttle conservatively |
| SP-002 Flutter Audio | just_audio plays Spotify previews in Chrome | Partially confirmed | CORS high risk (may need backend proxy). Crossfade is manual 2-player impl. `audioplayers` preferred over `just_audio` for stability |
| SP-003 essentia | Sidecar extracts BPM/key with <5s latency | Partially confirmed | Needs 1-2 GB container (not 512 MB). Async queue required. Key = separate note + scale strings. Use TempoCNN for 30s previews |

## Architecture

| Layer | Path | Tech |
|-------|------|------|
| Backend API | `backend/` | Rust, Axum 0.8, SQLx |
| Frontend | `frontend/` | Flutter/Dart, Riverpod, GoRouter |
| Landing | `landing/` | Static HTML + Tailwind |
| Database | SQLite (dev) | PostgreSQL (prod) via SQLx |
| LLM | Claude Sonnet API | Setlist generation, music knowledge |
| Audio Analysis | essentia (async sidecar) | BPM, key, energy detection. 1-2 GB container, Starlette/FastAPI, async queue |

### Key Patterns
- Backend serves JSON; Flutter consumes it
- Backend owns ALL external API keys (Spotify, Beatport, SoundCloud, Anthropic)
- Auth: `X-User-Id` header (temporary, replaced by JWT in UC-008)
- API clients: custom `reqwest` wrappers with shared retry middleware
- Source-agnostic import via `MusicSourceClient` trait
- Repository pattern via `ImportRepository` trait for testability
- API contract: `docs/api/openapi.yaml` is single source of truth — backend implements TO it, frontend implements FROM it
- Error responses: nested `{"error": {"code": "...", "message": "..."}}` per OpenAPI spec (migration from flat format required, see ST-001)
- Camelot conversion: both Beatport and essentia return raw key notation — shared lookup utility needed (24 entries: 12 major + 12 minor)
- Artist data: relational (`artists` + `track_artists` JOIN), API flattens to comma-separated string

## External Integrations

| Service | Purpose | API Status |
|---------|---------|-----------|
| Spotify | Music import (UC-001, done) | Integrated, OAuth. Preview URLs may return null; CORS risk on web (SP-002) |
| Beatport | DJ track source (BPM/key native) | v4 API, OAuth2 w/ public client_id workaround. No official dev access. Rate limits unknown (SP-001) |
| SoundCloud | Discovery + streaming | Public API, OAuth 2.1. CORS issues with streaming URL 302 redirects (SP-002) |
| Anthropic/Claude | Setlist generation | Claude Sonnet default |
| essentia | Audio analysis (BPM, key) | Python sidecar (1-2 GB), async queue required. Key output is raw notation, needs Camelot conversion (SP-003) |

## UX/Design Tooling

- **`design-crit` plugin** (`.design-crit/`) — Primary design tool. Run before building each screen. 4-stage process: Brief → Facet Plan → Crit Loops → Design Direction. Outputs: design tokens, decision log, and interactive HTML prototypes.
- `/wireframe` — Quick ASCII wireframes during use case creation
- `/ux-review` — Post-implementation UX review
- `frontend-design` plugin — Production code generation

## Project Context

- Full plan: `docs/project-plan.md`
- DJ platform research: `docs/research/dj-platform-research.md`
- Use cases: `docs/use-cases/uc-*.md`
- Task decompositions: `docs/tasks/uc-*-tasks.md`
- Design-crit outputs: `.design-crit/` (tokens, decisions, prototypes)
- Steel threads: `docs/steel-threads/st-*.md`
- Spikes: `docs/spikes/sp-*.md`
- API contract: `docs/api/openapi.yaml`

## Quality Gates

Both must pass before any commit (enforced by `.claude/settings.json` pre-commit hook):
```
Backend:  cargo fmt --check && cargo clippy -- -D warnings && cargo test
Frontend: flutter analyze && flutter test
```

## Research Before Hacking

When you encounter a technology or integration you haven't used before (e.g., Playwright + Flutter, headless browser rendering, CI pipelines), **STOP and research best practices before writing code**. Red flags that you're hacking: writing debug scripts, adding elaborate workarounds, trying multiple config tweaks in sequence. Instead: search the web for established patterns, check official docs, and use proven solutions. If a problem has been solved, use that solution.

## Shell Command Style

**Chain commands with `&&`** to avoid repeated permission prompts. This enables longer autonomous runs:
```bash
# Good: single permission prompt
cd backend && cargo fmt --check && cargo clippy -- -D warnings && cargo test

# Bad: three separate prompts
cd backend
cargo fmt --check
cargo clippy -- -D warnings
```

## Key Commands

```bash
# Backend
cd backend && cargo run                    # Start API server (port 3001)
cd backend && cargo test                   # Run tests
cd backend && cargo clippy -- -D warnings  # Lint

# Frontend
cd frontend && flutter run -d chrome       # Start web app
cd frontend && flutter analyze             # Lint
cd frontend && flutter test                # Run tests
```

## Current State

- **Branch**: `main` (clean, pushed to GitHub)
- **UC-001**: Complete — Spotify OAuth, playlist import, retry/resilience, DB upserts, Flutter import screen
- **UC-013 through UC-025**: Created, reviewed (devil's advocate), fixes applied
- **PRD**: `docs/prd.md` — synthesized from all 13 UCs, reviewed and fixed
- **Design-crit**: Complete (10/10 facets locked). Prototypes: track-catalog, import-sheet, empty-catalog
- **Walking Skeleton**:
  - SP-001 (Beatport): Complete — OAuth2 w/ public client_id, BPM/key native but raw format, rate limits unknown
  - SP-002 (Flutter Audio): Complete — CORS high risk, crossfade manual (2 players), `audioplayers` preferred
  - SP-003 (essentia): Complete — 1-2 GB container, async queue required, key is raw notation
  - ST-001 (Track Catalog API): **Implemented** — GET /api/tracks with pagination, sorting, null handling, nested error format. Merged to main.
  - ST-003 (Generate Setlist from Prompt): **Implemented** — POST /api/setlists/generate, POST /api/setlists/{id}/arrange, GET /api/setlists/{id}. Claude API client, Camelot module, arrangement algorithm (greedy + 2-opt), hallucinated track_id validation, nested error format. Branch: `feature/st-003-generate-setlist`
  - ST-002: Not yet created
- **Next**: `/verify-uc ST-003` → merge to main → create ST-002
- **Test count**: 135 backend tests (121 lib + 2 main + 12 integration), 12 frontend tests — all passing
- **Known debt**: Migration 003 adds DJ metadata columns but they're all NULL until import or analysis populates them. `lib.rs` created for integration test support.
- **GitHub**: `git@github.com:step2this/ethnomusicology.git`
