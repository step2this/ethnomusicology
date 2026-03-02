# Ethnomusicology Project - Claude Code Directives

## Vision: DJ-First Music Platform

LLM-powered substitute DJ that generates setlists from natural language prompts, sources music from Spotify + Beatport + SoundCloud, supports harmonic mixing (Camelot wheel), and provides crossfade preview playback with purchase links. Occasion-based features (Nikah, Eid, Mawlid) remain but are secondary to the DJ experience.

## Workflow: The Forge

Use the Forge (`.claude/` directory) for all development:
1. `/uc-create` — Create Cockburn-style use case
2. `/uc-review` — Devil's Advocate review (apply all fixes before coding)
3. `/task-decompose` — Break into implementable tasks with dependency graph
4. `design-crit` — Run for any UC with a frontend screen (Brief → Facet Plan → Crit Loops → Design Direction)
5. Implement with agent teams (see below)
6. `/verify-uc` — Validate implementation against postconditions
7. `/grade-work` — Score completed work
8. `/session-handoff` — Save state before ending sessions

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
6. **Implement** — backend TO the contract, frontend FROM the contract
7. **Verify** (`/verify-uc ST-NNN`) — integration assertions + contract conformance
8. **Grade** (`/grade-work ST-NNN`) — Integration Proof category (20% weight)

## Agent Teams

**Always prefer multi-agent teams** for any non-trivial work:
- Create a team via `TeamCreate`, spawn builders in parallel
- **Backend agents work in the same repo** — assign non-overlapping module directories
- Use traits/interfaces at boundaries so parallel agents don't block each other (e.g., `ImportRepository` trait, `MusicSourceClient` trait)
- The lead coordinates, reviews, and wires modules together — does not implement directly
- Use Haiku agents for research tasks (API evaluation, tool research)

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
  - ST-001 (Track Catalog API): Created, API contract written, reviewed. Ready for `/task-decompose`
  - ST-002, ST-003: Not yet created
- **Next**: Fix ST-001 review findings → `/task-decompose ST-001` → implement ST-001
- **Test count**: 49 backend tests, 1 frontend test — all passing
- **Known debt**: `backend/src/error.rs` returns flat `{"error": "msg"}` — must migrate to nested format per OpenAPI. Migration 003 (DJ metadata columns) not yet created.
- **GitHub**: `git@github.com:step2this/ethnomusicology.git`
