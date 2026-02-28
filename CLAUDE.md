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

## Architecture

| Layer | Path | Tech |
|-------|------|------|
| Backend API | `backend/` | Rust, Axum 0.8, SQLx |
| Frontend | `frontend/` | Flutter/Dart, Riverpod, GoRouter |
| Landing | `landing/` | Static HTML + Tailwind |
| Database | SQLite (dev) | PostgreSQL (prod) via SQLx |
| LLM | Claude Sonnet API | Setlist generation, music knowledge |
| Audio Analysis | essentia (Python sidecar) | BPM, key, energy detection |

### Key Patterns
- Backend serves JSON; Flutter consumes it
- Backend owns ALL external API keys (Spotify, Beatport, SoundCloud, Anthropic)
- Auth: `X-User-Id` header (temporary, replaced by JWT in UC-008)
- API clients: custom `reqwest` wrappers with shared retry middleware
- Source-agnostic import via `MusicSourceClient` trait
- Repository pattern via `ImportRepository` trait for testability

## External Integrations

| Service | Purpose | API Status |
|---------|---------|-----------|
| Spotify | Music import (UC-001, done) | Integrated, OAuth |
| Beatport | DJ track source (BPM/key native) | v4 API, needs integration |
| SoundCloud | Discovery + streaming | Public API, OAuth 2.1 |
| Anthropic/Claude | Setlist generation | Claude Sonnet default |
| essentia | Audio analysis (BPM, key) | Python sidecar, server-side |

## UX/Design Tooling

- **`design-crit` plugin** (`metedata/design-crit`) — Primary design tool. Run before building each screen. 4-stage process: Brief → Facet Plan → Crit Loops → Design Direction.
- `/wireframe` — Quick ASCII wireframes during use case creation
- `/ux-review` — Post-implementation UX review
- `frontend-design` plugin — Production code generation

## Project Context

- Full plan: `docs/project-plan.md`
- DJ platform research: `docs/research/dj-platform-research.md`
- Use cases: `docs/use-cases/uc-*.md`
- Task decompositions: `docs/tasks/uc-*-tasks.md`
- Evolution plan: `.claude/plans/golden-questing-lollipop.md`

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

- **Branch**: `main` (UC-001 merged, pushed to GitHub)
- **UC-001**: Complete — Spotify OAuth, playlist import, retry/resilience, DB upserts, Flutter import screen
- **UC-013 through UC-025**: Created, reviewed (devil's advocate), fixes applied. Ready for `/task-decompose`.
- **PRD**: `docs/prd.md` — synthesized from all 13 UCs, reviewed and fixed
- **Next**: `/task-decompose UC-013` → implement Sprint 0 infra + Sprint 1 imports
- **Test count**: 49 backend tests, 1 frontend test — all passing
- **GitHub**: `git@github.com:step2this/ethnomusicology.git`
