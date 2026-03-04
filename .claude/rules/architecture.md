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
| Database | SQLite (dev + prod) | via SQLx, `sqlx::migrate!()` for versioned migrations. S3 backup cron every 6 hours. |
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
| Spotify | Music import + metadata (UC-001) | Integrated, OAuth. Audio Features API DEPRECATED Nov 2024 — do NOT use. Import and catalog only. |
| Deezer | 30s preview playback (UC-019) | Search API (no auth). Backend proxies MP3 streams to avoid CORS. `deezer_preview_url` persisted on tracks table. |
| Beatport | DJ track source (deferred) | v4 API, OAuth2 w/ public client_id workaround. No official dev access. Rate limits unknown (SP-001) |
| SoundCloud | Discovery + streaming (deferred) | Public API, OAuth 2.1. CORS issues with streaming URL 302 redirects (SP-002) |
| Anthropic/Claude | Setlist generation + enrichment + refinement | Sonnet default, Opus for complex refinement. Enrichment: batch BPM/key/energy estimation (ST-005). Refinement: multi-turn converse() (ST-007). |
| essentia | Audio analysis (deferred post-MVP) | Python sidecar (1-2 GB), async queue. Key → Camelot via `from_notation()` (ready in ST-005) |
