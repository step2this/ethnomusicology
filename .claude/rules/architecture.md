---
paths:
  - "backend/**"
  - "frontend/**"
  - "e2e/**"
  - "landing/**"
---

# Architecture

| Layer | Path | Tech |
|-------|------|------|
| Backend API | `backend/` | Rust, Axum 0.8, SQLx |
| Frontend | `frontend/` | Flutter/Dart, Riverpod, GoRouter |
| Landing | `landing/` | Static HTML + Tailwind |
| Database | SQLite (dev) | PostgreSQL (prod) via SQLx |
| LLM | Claude Sonnet API | Setlist generation, music knowledge, track enrichment (BPM/key/energy estimation) |
| Audio Analysis | essentia (async sidecar) | Post-MVP: audio-accurate BPM/key. 1-2 GB container, Starlette/FastAPI, async queue |
| E2E Tests | `e2e/` | Playwright, GitHub Actions CI (ST-004). DEV_MODE=true enables `/api/dev/seed` for test data. |

## Key Patterns
- Backend serves JSON; Flutter consumes it
- Backend owns ALL external API keys (Spotify, Beatport, SoundCloud, Anthropic)
- Auth: `X-User-Id` header (temporary, replaced by JWT in UC-008)
- API clients: custom `reqwest` wrappers with shared retry middleware
- Source-agnostic import via `MusicSourceClient` trait (planned for UC-013/014 — not yet implemented)
- Repository pattern via `ImportRepository` trait for testability
- API contract: `docs/api/openapi.yaml` is single source of truth — backend implements TO it, frontend implements FROM it
- Error responses: nested `{"error": {"code": "...", "message": "..."}}` per OpenAPI spec (implemented in ST-001)
- Camelot conversion: `from_spotify_key()` + `from_notation()` in `services/camelot.rs` (implemented in ST-005). Supports Spotify pitch_class/mode, essentia note/scale, and direct Camelot notation.
- Shared `Arc<dyn ClaudeClientTrait>` across routes — one client instance for setlist generation and enrichment
- Artist data: relational (`artists` + `track_artists` JOIN), API flattens to comma-separated string

## External Integrations

| Service | Purpose | API Status |
|---------|---------|-----------|
| Spotify | Music import (UC-001, done) | Integrated, OAuth. Preview URLs may return null; CORS risk on web (SP-002) |
| Beatport | DJ track source (BPM/key native) | v4 API, OAuth2 w/ public client_id workaround. No official dev access. Rate limits unknown (SP-001) |
| SoundCloud | Discovery + streaming | Public API, OAuth 2.1. CORS issues with streaming URL 302 redirects (SP-002) |
| Anthropic/Claude | Setlist generation + track enrichment | Claude Sonnet default. Enrichment: batch estimation of BPM/key/energy from title+artist (ST-005). Daily cap: 250 tracks/user. |
| essentia | Audio analysis (BPM, key) | Post-MVP. Python sidecar (1-2 GB), async queue. Key → Camelot via `from_notation()` (ready in ST-005) |
