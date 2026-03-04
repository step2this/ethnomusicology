# Task Decomposition: ST-008 iTunes Preview Fallback

## Devil's Advocate Resolutions

- **C2 (rename blast radius)**: Frontend rename `Deezer*` → `Preview*` done as T0 prep PR before builders
- **H1 (Apple badge ToS)**: Track tile shows Apple Music icon+link when source=itunes
- **H2 (proxy Content-Type)**: Forward upstream Content-Type instead of hardcoding audio/mpeg
- **H3 (backward compat)**: Keep `/api/audio/deezer-search` as alias for transition
- **H4 (rate limiting)**: Backend semaphore: max 20 concurrent iTunes requests
- **M1 (match scoring)**: Shared fuzzy title/artist matching (Levenshtein or contains) — discard <50% match
- **M2 (timeout budget)**: 2s per source, abort chain when global 5s budget exceeded
- **M3 (proxy source param)**: Source param is informational only; host validation from URL parsing
- **L1 (country)**: Hardcode `country=US` for iTunes
- **L2 (OpenAPI)**: Update as part of backend task

## Task Dependency Graph

```
T0 (frontend rename, lead direct) ─────────────────────────────────┐
T1 (backend: unified search + iTunes + proxy fix) ──────────────────┼──→ T3 (frontend: update provider + tile) ──→ T4 (tests) ──→ C1 (critic)
T2 (backend: match scoring utility) ───────────────────────────────┘
```

**Phase 0**: T0 (rename Deezer* → Preview*, standalone commit)
**Phase 1**: T1 + T2 in parallel (non-overlapping backend files)
**Phase 2**: T3 (frontend, depends on T0 + T1)
**Phase 3**: T4 (tests)
**Phase 4**: C1 (critic)

## Tasks

### T0: Frontend rename Deezer* → Preview* (lead direct, ~0 new lines)
**Files (ALL renames, no logic changes):**
- `frontend/lib/providers/deezer_provider.dart` → keep filename, rename types:
  - `DeezerSearchStatus` → `PreviewSearchStatus`
  - `DeezerTrackInfo` → `PreviewTrackInfo`
  - `DeezerPreviewState` → `PreviewState`
  - `DeezerPreviewNotifier` → `PreviewNotifier`
  - `deezerPreviewProvider` → `previewProvider`
- Update ALL 8 consuming files (audio_provider, refinement_provider, setlist_result_view, setlist_track_tile, setlist_generation_screen, + 3 test files)
- Rename `_deezerStatusDot()` → `_previewStatusDot()`
- `flutter analyze && flutter test` must pass
- Commit as standalone PR/commit before ST-008 builders start

### T1: Backend unified search endpoint + iTunes + proxy fix (~200 lines) — `backend-builder`
**Files:**
- `backend/src/routes/audio.rs` (MODIFY — major changes):
  - New handler: `audio_search()` for `GET /api/audio/search?title=X&artist=Y`
  - Fallback chain: Deezer (strict → fuzzy) → iTunes → return null
  - Per-source timeout: 2s each, global 5s budget
  - iTunes search: `GET https://itunes.apple.com/search?term={artist}+{title}&media=music&entity=song&limit=5&country=US`
  - Parse iTunes response: extract `previewUrl`, `trackViewUrl`, `trackId`
  - Rate limiting: `tokio::sync::Semaphore` with 20 permits for iTunes
  - Fix proxy: forward upstream `Content-Type` header instead of hardcoding `audio/mpeg`
  - Whitelist Apple CDN: `audio-ssl.itunes.apple.com`, `*.mzstatic.com`
  - Keep `/api/audio/deezer-search` as backward-compat alias
- `backend/src/main.rs` (MODIFY — register new route)
- `backend/src/routes/mod.rs` (if needed)
- `docs/api/openapi.yaml` (MODIFY — add /api/audio/search)

**Response shape:**
```json
{
  "source": "deezer" | "itunes" | null,
  "preview_url": "/api/audio/proxy?url=..." | null,
  "external_url": "https://music.apple.com/..." | null,
  "search_queries": ["artist:\"X\" track:\"Y\" strict=on", "iTunes: artist title"],
  "deezer_id": 12345 | null,
  "itunes_id": 67890 | null
}
```

### T2: Backend match scoring utility (~40 lines) — `backend-builder`
**Files:**
- `backend/src/services/match_scoring.rs` (NEW)
  - `fn title_similarity(query_title: &str, result_title: &str) -> f64` — normalized score 0.0-1.0
  - `fn artist_similarity(query_artist: &str, result_artist: &str) -> f64`
  - `fn is_acceptable_match(query_title: &str, query_artist: &str, result_title: &str, result_artist: &str) -> bool` — threshold 0.5
  - Simple approach: lowercase, check if query is substring of result or vice versa. No Levenshtein needed for MVP.
- `backend/src/services/mod.rs` (MODIFY — add module)

### T3: Frontend update provider + tile for multi-source (~80 lines) — `frontend-builder`
**Files (after T0 rename):**
- `frontend/lib/services/api_client.dart` (MODIFY):
  - Replace `searchDeezerPreview()` + `_deezerSearch()` with single `searchPreview(title, artist)` calling `GET /api/audio/search?title=X&artist=Y`
  - Parse unified response: extract `source`, `preview_url`, `external_url`, `search_queries`
- `frontend/lib/providers/deezer_provider.dart` (MODIFY):
  - `PreviewTrackInfo` gets new field: `source` (String? — "deezer", "itunes", null)
  - `PreviewTrackInfo` gets new field: `externalUrl` (String?)
  - `searchQuery` becomes `searchQueries` (List<String>) for multi-step chain display
  - Update `prefetchForSetlist()` to call new `searchPreview()` method
- `frontend/lib/widgets/setlist_track_tile.dart` (MODIFY):
  - `_previewStatusDot()`: show Deezer icon for deezer, Apple icon for itunes, red X for null
  - Tooltip shows all search queries tried
  - When source=itunes: show small Apple Music link/badge near play button (ToS compliance)
- `frontend/lib/widgets/setlist_result_view.dart` (MODIFY if needed)

### T4: Tests (~100 lines) — `test-builder`
**Files:**
- `backend/tests/audio_search_test.rs` (NEW): integration test for unified search endpoint
- `backend/src/services/match_scoring.rs` (inline tests): similarity scoring
- `frontend/test/providers/deezer_provider_test.dart` (MODIFY → rename to preview_provider or update): multi-source state transitions
- `frontend/test/services/api_client_test.dart` (MODIFY): new searchPreview method

### C1: Critic review
- Fresh context opus agent reads `git diff main...HEAD` and plan
- Checks: proxy Content-Type fix, Apple CDN whitelist, backward compat alias, match scoring, ToS badge

## File Inventory

| File | Action | Owner | Task |
|------|--------|-------|------|
| `frontend/lib/providers/deezer_provider.dart` | MODIFY (rename types) | lead | T0 |
| `frontend/lib/providers/audio_provider.dart` | MODIFY (rename refs) | lead | T0 |
| `frontend/lib/providers/refinement_provider.dart` | MODIFY (rename refs) | lead | T0 |
| `frontend/lib/widgets/setlist_result_view.dart` | MODIFY (rename refs) | lead | T0 |
| `frontend/lib/widgets/setlist_track_tile.dart` | MODIFY (rename refs) | lead | T0 |
| `frontend/lib/screens/setlist_generation_screen.dart` | MODIFY (rename refs) | lead | T0 |
| `frontend/test/providers/deezer_provider_test.dart` | MODIFY (rename refs) | lead | T0 |
| `frontend/test/providers/audio_provider_test.dart` | MODIFY (rename refs) | lead | T0 |
| `frontend/test/services/api_client_test.dart` | MODIFY (rename refs) | lead | T0 |
| `backend/src/routes/audio.rs` | MODIFY (major) | backend-builder | T1 |
| `backend/src/main.rs` | MODIFY | backend-builder | T1 |
| `docs/api/openapi.yaml` | MODIFY | backend-builder | T1 |
| `backend/src/services/match_scoring.rs` | NEW | backend-builder | T2 |
| `backend/src/services/mod.rs` | MODIFY | backend-builder | T2 |
| `frontend/lib/services/api_client.dart` | MODIFY | frontend-builder | T3 |
| `frontend/lib/providers/deezer_provider.dart` | MODIFY (logic) | frontend-builder | T3 |
| `frontend/lib/widgets/setlist_track_tile.dart` | MODIFY (icons) | frontend-builder | T3 |

**Estimated total: ~420 new/modified lines across 17 files**
