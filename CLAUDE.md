# Ethnomusicology Project - Claude Code Directives

## Mandatory: The Forge
ALWAYS use the Forge (`.claude/` directory) for ALL development work:
- ALWAYS create use cases via `/uc-create` before implementation
- ALWAYS validate use cases via `/uc-review` before coding
- ALWAYS decompose tasks via `/task-decompose` before starting work
- ALWAYS run quality gates before committing (cargo fmt, clippy, test)
- ALWAYS verify implementations via `/verify-uc`
- ALWAYS create session handoffs via `/session-handoff` before ending sessions
- ALWAYS grade work via `/grade-work` after completing features

## Mandatory: Agent Teams
ALWAYS use multi-agent teams for non-trivial work:
- ALWAYS create a team plan via `/agent-team-plan` before starting complex tasks
- ALWAYS use parallel sprints via `/parallel-sprint` for multi-track work
- Use the Implementation Team (Builder, Reviewer, Documentation) for coding tasks
- Use the Requirements Team (Architect, Devil's Advocate, Test Designer, Architecture Scout) for planning
- NEVER work as a single agent on tasks that could benefit from parallelization
- Delegate work to specialized agents; the lead agent coordinates but does not implement directly

## Architecture
- **Backend**: Rust (Axum 0.8) in `backend/`
- **Frontend**: Flutter (Dart) in `frontend/`
- **Landing pages**: Static HTML in `landing/`
- **Database**: SQLite (dev) / PostgreSQL (prod) via SQLx
- **API pattern**: Backend serves JSON; Flutter consumes it
- Backend owns all external API keys (Spotify, YouTube, Last.fm, MusicBrainz)

## Project Context
- Music playlist app for Muslim families planning occasions (Nikah, Eid, Mawlid, etc.)
- Seed data: "Salamic Vibes" Spotify playlist (54 tracks, user-owned)
- Target: African/Middle Eastern Muslim musical traditions
- Full plan: `docs/project-plan.md`

## Quality Gates
- Backend: `cargo fmt --check && cargo clippy -- -D warnings && cargo test`
- Frontend: `flutter analyze && flutter test`
- Both must pass before any commit

## Key Commands
- Backend dev: `cd backend && cargo run`
- Frontend dev: `cd frontend && flutter run -d chrome`
- Backend test: `cd backend && cargo test`
- Frontend test: `cd frontend && flutter test`
