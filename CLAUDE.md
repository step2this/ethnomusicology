# Ethnomusicology Project - Claude Code Directives

## Workflow: The Forge

Use the Forge (`.claude/` directory) for all development:
1. `/uc-create` — Create Cockburn-style use case
2. `/uc-review` — Devil's Advocate review (apply all fixes before coding)
3. `/task-decompose` — Break into implementable tasks with dependency graph
4. Implement with agent teams (see below)
5. `/verify-uc` — Validate implementation against postconditions
6. `/grade-work` — Score completed work
7. `/session-handoff` — Save state before ending sessions

## Agent Teams

Use multi-agent teams for non-trivial work (anything spanning >3 files):
- Create a team via `TeamCreate`, spawn builders in parallel
- **Backend agents work in the same repo** — assign non-overlapping module directories
- Use traits/interfaces at boundaries so parallel agents don't block each other (e.g., `ImportRepository` trait lets the service layer be tested without the real DB)
- The lead coordinates, reviews, and wires modules together — does not implement directly

### Lessons Learned (UC-001)
- Two backend agents can work in parallel if they own separate directories (`src/api/` vs `src/db/`)
- Agents handle `mod.rs` creation and module wiring well when given clear boundaries
- Define shared traits up front in prompts so agents produce compatible interfaces
- Run `cargo check` before spawning agents to warm the dependency cache — saves 30s+ per agent
- Frontend can be done sequentially after backend since it depends on API shapes
- Haiku agents work well for research tasks (EC2 setup, tool evaluation)

## Architecture

| Layer | Path | Tech |
|-------|------|------|
| Backend API | `backend/` | Rust, Axum 0.8, SQLx |
| Frontend | `frontend/` | Flutter/Dart, Riverpod, GoRouter |
| Landing | `landing/` | Static HTML + Tailwind |
| Database | SQLite (dev) | PostgreSQL (prod) via SQLx |

- Backend serves JSON; Flutter consumes it
- Backend owns ALL external API keys (Spotify, YouTube, Last.fm, MusicBrainz)
- Auth: `X-User-Id` header (temporary, replaced by JWT in UC-008)
- Spotify client: custom `reqwest` wrapper (not `rspotify` crate) for full error control

## Project Context

- Music playlist app for Muslim families planning occasions (Nikah, Eid, Mawlid, etc.)
- Seed data: "Salamic Vibes" Spotify playlist (54 tracks)
- Target: African/Middle Eastern Muslim musical traditions
- Full plan: `docs/project-plan.md`
- Use cases: `docs/use-cases/uc-*.md`
- Task decompositions: `docs/tasks/uc-*-tasks.md`

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
cd backend && cargo test                   # Run 49 tests
cd backend && cargo clippy -- -D warnings  # Lint

# Frontend
cd frontend && flutter run -d chrome       # Start web app
cd frontend && flutter analyze             # Lint
cd frontend && flutter test                # Run tests
```

## Current State

- **Branch**: `feature/uc-001-spotify-import` (ready to merge to main)
- **UC-001**: Implemented — Spotify OAuth, playlist import, retry/resilience, DB upserts, Flutter import screen
- **Next**: UC-002 (Enrich Track Metadata), UC-006 (Preview/Play Track), DNS setup for EC2 access
- **Test count**: 49 backend tests passing
