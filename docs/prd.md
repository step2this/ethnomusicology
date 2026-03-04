# Ethnomusicology — Product Requirements Document

> **Version**: 1.0 — Generated 2026-02-28
> **Source**: 13 Cockburn use cases (UC-001, UC-013–UC-021, UC-023–UC-025)
> **Status**: DJ-first pivot. UC-001 complete. Remaining UCs ready for implementation.

---

## 1. Product Overview

Ethnomusicology is an **LLM-powered DJ assistant** that generates setlists from natural language prompts, sources music from Spotify, Beatport, and SoundCloud, arranges tracks by harmonic compatibility (Camelot wheel), and provides preview playback with purchase links.

The user describes a vibe — *"deep dubby NYC house from the early 90s, Sound Factory vibes, building from 118 to 126 BPM"* — and the system produces a playable, harmonically coherent setlist drawn from their imported catalog, supplemented by LLM suggestions for tracks to acquire.

### Core Value Proposition
- **LLM as crate-digger**: Replace manual genre browsing with natural language music discovery. Works with zero imported tracks — LLM generates pure suggestions when no catalog exists; catalog enhances matching but does not gate generation.
- **Harmonic intelligence**: Automatic Camelot wheel arrangement for smooth key transitions
- **Multi-source catalog**: Unified track library across Spotify, Beatport, and SoundCloud
- **Audio validation**: Hear 30-second previews before committing to a setlist (sequential playback; crossfade removed as too complex for 30s clips)

### Target User
DJs who build sets from digital catalogs and value harmonic mixing, energy flow, and efficient track discovery. Secondary: event curators building occasion-based playlists (Nikah, Eid, Mawlid).

---

## 2. Feature Map

> For current implementation status, see `docs/mvp-progress.md`.

### Epic 1: Multi-Source Import Pipeline
| UC | Feature | Priority | Complexity |
|----|---------|----------|------------|
| UC-001 | Import from Spotify | P0 | Medium |
| UC-013 | Import from Beatport | P0 | Medium |
| UC-014 | Import from SoundCloud | P0 | Medium |

### Epic 2: Audio Analysis & Enrichment
| UC | Feature | Priority | Complexity |
|----|---------|----------|------------|
| UC-015 | Detect BPM and Musical Key | P0 | High |
| UC-018 | Enrich with DJ Metadata (energy, mood, genre) | P1 | Medium |

### Epic 3: LLM Setlist Generation
| UC | Feature | Priority | Complexity |
|----|---------|----------|------------|
| UC-016 | Generate Setlist from Natural Language | P0 | High |
| UC-017 | Arrange by Harmonic Compatibility | P0 | Medium |
| UC-023 | Refine Setlist with Conversation | P2 | High |

### Epic 4: Playback & Output
| UC | Feature | Priority | Complexity |
|----|---------|----------|------------|
| UC-019 | Preview Playback (sequential; crossfade removed) | P1 | Medium |
| UC-025 | Full Browser DJ Mix Playback | P3 | Very High |
| UC-024 | Export Setlist with Transition Notes | P2 | Low |

### Epic 5: Discovery & Acquisition
| UC | Feature | Priority | Complexity |
|----|---------|----------|------------|
| UC-020 | Generate Purchase Links | P1 | Low |
| UC-021 | Browse by DJ Scene and Era | P1 | Medium |

---

## 3. Dependency Graph

> **Pre-Sprint infrastructure**: MusicSourceClient trait, Camelot module, and source-agnostic ImportRepository refactor must be completed before UC-013/UC-014 can begin. These are shared foundation tasks, not owned by any single UC.

```
UC-001 (Spotify Import) ✅ DONE
  │
  ├──→ UC-013 (Beatport Import)
  │      │  Uses: MusicSourceClient trait (from Pre-Sprint), migration 003
  │      │
  │      ├──→ UC-014 (SoundCloud Import)
  │      │      │  Implements: SoundCloudClient (2nd MusicSourceClient)
  │      │      │  Sets: needs_analysis=true on all imported tracks
  │      │      │
  │      │      └──→ UC-015 (BPM/Key Detection)
  │      │             │  Consumes: needs_analysis queue
  │      │             │  Uses: Camelot module from UC-013
  │      │             │  Requires: essentia Python sidecar
  │      │             │
  │      │             ├──→ UC-018 (DJ Metadata Enrichment)
  │      │             │      │  Uses: Claude API for energy/mood/genre
  │      │             │      │
  │      │             │      └──→ UC-021 (Browse by Scene/Era)
  │      │             │             Uses: enriched genre + Claude for scene tags
  │      │             │
  │      │             └──→ UC-016 (Setlist Generation) ← CORE FEATURE
  │      │                    │  Uses: Claude API + user catalog + BPM/key data
  │      │                    │
  │      │                    ├──→ UC-017 (Harmonic Arrangement)
  │      │                    │      │  Pure Rust: Camelot + BPM + energy scoring
  │      │                    │      │
  │      │                    │      ├──→ UC-019 (Preview Playback)
  │      │                    │      │      │  Web Audio API, multi-source (Deezer→iTunes→SoundCloud)
  │      │                    │      │      │
  │      │                    │      │      └──→ UC-025 (Full DJ Mix) [P3]
  │      │                    │      │
  │      │                    │      └──→ UC-024 (Export Setlist) [P2]
  │      │                    │
  │      │                    └──→ UC-023 (Conversational Refinement) [P2]
  │      │
  │      └──→ UC-020 (Purchase Links)
  │             Leaf feature, no downstream deps
  │
  └──→ (UC-013, UC-014, UC-020 all depend on UC-001's infra)
```

### Critical Path (P0)
```
UC-001 → UC-013 → UC-014 → UC-015 → UC-016 → UC-017
```
All 6 use cases on the critical path must be complete for the MVP DJ experience.

### Parallelizable Work
- UC-013 and UC-014 can run in parallel after UC-013 defines the `MusicSourceClient` trait
- UC-018 and UC-016 can develop concurrently (UC-016 works without enrichment, just lower quality)
- UC-020 is a leaf — can be built any time after import UCs
- UC-024 is a leaf — can be built any time after UC-016/017

---

## 4. System-Wide Requirements

### 4.1 Security Invariants (all UCs)
1. **API credentials never exposed to frontend** — Spotify, Beatport, SoundCloud, and Anthropic keys are backend-only
2. **All external API calls happen server-side** — frontend never calls third-party APIs directly
3. **OAuth tokens encrypted at rest** — Spotify user tokens stored encrypted (UC-001 pattern)
4. **Auth header**: `X-User-Id` (temporary), replaced by JWT in future UC-008
5. **Authentication required before LLM features**: UC-016 and all downstream UCs (017-025) require user authentication. The current `X-User-Id` header is acceptable for Sprint 1-2 (import-only), but JWT auth (or equivalent) must be implemented before Sprint 3 (LLM features) to prevent abuse of per-user rate limits.

### 4.2 Resilience Patterns (UC-001, 013, 014, 015, 016, 018)
1. **Retry with exponential backoff**: 3 retries, 1s/2s/4s delays for 429 and 5xx responses
2. **Partial import commits**: Per-track transactions — failures don't roll back successfully imported tracks
3. **Rate limit awareness**: Read `Retry-After` headers, communicate wait times to user
4. **Graceful degradation**: Always prefer showing partial results over complete failure

### 4.3 LLM Cost Controls (UC-016, 018, 021, 023)
1. **Prompt caching**: System prompt + catalog context cached via `cache_control: { type: "ephemeral" }` (~90% cost reduction)
2. **Per-user daily limits**: Default 20 setlist generations/day (configurable)
3. **Batch processing**: Enrichment (UC-018) and classification (UC-021) batch 20 tracks per LLM call
4. **Model selection**: Claude Sonnet default, Opus only for complex refinements (UC-023)

#### Estimated Monthly Costs (100 active users)
| Component | Cost/Month |
|-----------|-----------|
| Claude API (setlist generation, 20/user/day avg 5) | $30-90 |
| Claude API (enrichment, one-time per track) | $5-15 |
| Claude API (refinement, avg 3 turns/setlist) | $10-30 |
| essentia sidecar hosting (t3.small) | $15 |
| **Total** | **$60-150** |

### 4.4 Background Workers (UC-015, 018, 021)
1. **Polling interval**: 30 seconds default
2. **Batch size**: 10 tracks (analysis), 20 tracks (enrichment/classification)
3. **Isolation**: Individual track failures never crash the worker
4. **Idempotency**: Re-processing a track produces the same result without side effects
5. **Priority hierarchy**: Beatport native data > essentia analysis > LLM enrichment

### 4.5 Data Model — Source-of-Truth Hierarchy
```
BPM/Key:    Beatport native (gold standard) > essentia analysis > null
Genre:      LLM-refined (UC-018) > Beatport native > SoundCloud user-tagged > null
Energy:     LLM-derived (UC-018) > estimated from BPM > null
Scene/Era:  LLM-derived (UC-021) > null
```

### 4.6 Shared Infrastructure

| Component | Defined In | Used By |
|-----------|-----------|---------|
| `MusicSourceClient` trait | Pre-Sprint 1 (infrastructure) | UC-013, UC-014, future sources |
| `ImportRepository` trait | UC-001 (refactored to source-agnostic in Pre-Sprint 1) | UC-013, UC-014 |
| Camelot module (24-key lookup) | Pre-Sprint 1 (infrastructure) | UC-015, UC-016, UC-017 |
| essentia sidecar (Python) | UC-015 | UC-015 only |
| Claude API client | UC-016 | UC-018, UC-021, UC-023 |
| Web Audio preview engine (sequential) | UC-019 | UC-025 |
| Multi-source preview chain (Deezer→iTunes→SoundCloud) | UC-019 | UC-020 |
| `url_launcher` integration | UC-001 | UC-020 |

> **Sprint 0 (Pre-Sprint)**: Define `MusicSourceClient` trait, `SourceTrack` struct, refactor `ImportRepository` to be source-agnostic, implement Camelot module. These are prerequisites for Sprint 1, not part of any individual UC.

---

## 5. Database Migration Plan

| Migration | Use Case | Changes |
|-----------|----------|---------|
| 001 (done) | UC-001 | `tracks`, `artists`, `track_artists`, `spotify_tokens`, `imports` |
| 002 (done) | UC-001 | Index/constraint refinements |
| 003 | UC-013 | Add to `tracks`: `bpm`, `musical_key`, `camelot_key`, `genre`, `sub_genre`, `label`, `remixer`, `isrc`, `beatport_id`, `soundcloud_urn`, `source`, `needs_analysis`, `analyzed_at`, `analysis_error`, `permalink_url`, `artwork_url`, `stream_access_level` |
| 004 | UC-016 | `setlists` (id, user_id, prompt, model, arrangement_score, created_at), `setlist_tracks` (id, setlist_id, track_id, position, original_position, title, artist, bpm, key, camelot, energy, transition_note, transition_score, source, acquisition_info), `user_usage` (user_id, date, generation_count) |
| 005 | UC-018/021 | Add to `tracks`: `energy_level`, `mood_tags`, `enriched_at`, `enrichment_error`, `scene_tags`, `era_tag` |
| 006 | UC-023 | `setlist_versions` (id, setlist_id, version_number, created_at), `setlist_conversations` (id, setlist_id, role, content, version_id, created_at) |

---

## 6. External Integration Summary

| Service | Auth Model | UCs | Status |
|---------|-----------|-----|--------|
| Spotify Web API v1 | User OAuth 2.0 (Auth Code) | UC-001 | **Integrated** |
| Beatport API v4 | App-level OAuth (Client Credentials) | UC-013, UC-020 | Planned (apply for access) |
| SoundCloud API v2 | App-level OAuth 2.1 | UC-014, UC-019 | Planned |
| Anthropic Claude API | API key (backend-only) | UC-016, 018, 021, 023 | **Integrated** |
| essentia (self-hosted) | Local HTTP sidecar | UC-015 | Post-MVP |
| Deezer API | None (public) | UC-019 | **Integrated** (field-specific search + ISRC lookup) |
| iTunes Search API | None (public) | UC-019, UC-020 | Planned (Deezer fallback + Apple affiliate links) |

### Audio Preview Source Chain

Preview playback uses a cascading fallback chain per track:
1. **Deezer ISRC lookup** — `GET /track/isrc:{ISRC}` — instant exact match when Spotify ISRC is available
2. **Deezer field-specific search** — `artist:"X" track:"Y" strict=on` — high accuracy for mainstream catalog
3. **iTunes Search API** — free, no auth, 100M+ catalog, 30s AAC previews — best Deezer fallback
4. **SoundCloud** — OAuth required, good for underground/independent electronic music
5. **No preview** — show search links (Beatport, Google) for manual lookup

> Spotify preview URLs are deprecated (Nov 2024) and are not in this chain.
> Preview URL expiry is not a concern — URLs are fetched fresh each session.

---

## 7. Track Discovery & Acquisition

DJs need to investigate and purchase tracks they discover via setlist generation. The platform surfaces links directly in the setlist UI rather than requiring manual search.

### Implemented
- **Google search links** on track title + artist — opens search in new tab (implemented)
- **Spotify links** for catalog tracks — direct link to track in Spotify app

### Planned (UC-020)
Multi-store purchase link panel, shown per track in the setlist:

| Store | Link Type | Revenue |
|-------|-----------|---------|
| Beatport | Deep link to track page | None (no affiliate program) |
| Apple Music | Affiliate link via iTunes Search API | ~7% commission |
| Bandcamp | Search link (no direct track URL API) | None |
| Traxsource | Search link | None |
| Juno Download | Search link | None |

**Apple Music affiliate program** (via `apple.com/itunes/affiliates`) is the simplest near-term revenue opportunity — requires only adding `at=` parameter to iTunes links.

### Competitive Context
- No competitor offers LLM-powered natural language setlist generation
- KADO (200K DJ sets training data) operates at track-recommendation level, not setlist generation
- DJ streaming integrations (Beatport LINK, Tidal in Rekordbox) are closed hardware/platform partnerships — not available to indie developers

---

## 8. Implementation Order

### Sprint 1: Import Foundation (UC-013 + UC-014)
**Goal**: Multi-source catalog with MusicSourceClient trait
- Define `MusicSourceClient` trait and Camelot module
- Implement `BeatportClient` with native BPM/key import
- Implement `SoundCloudClient` with needs_analysis flagging
- Run migration 003
- Update import screen with source selector
- **Exit criteria**: Import from all 3 sources working, 49+ backend tests passing

### Sprint 2: Audio Intelligence (UC-015 + UC-018)
**Goal**: Every track has BPM, key, energy, mood, and genre
- Deploy essentia Python sidecar (Dockerfile)
- Build background analysis worker
- Build background enrichment worker (Claude API batches)
- **Exit criteria**: Imported tracks get BPM/key within minutes, energy/mood/genre within hours

### Sprint 3: Setlist Engine (UC-016 + UC-017)
**Goal**: Natural language → harmonically arranged setlist
- Build Claude API client with prompt caching
- Build setlist generation endpoint
- Build harmonic arrangement algorithm (greedy + 2-opt)
- Build setlist UI with BPM/energy/key visualizations
- Run migration 004
- **Exit criteria**: "Deep NYC house 90s" prompt produces a 15-track setlist with BPM flow and Camelot arrangement

### Sprint 4: Enhanced Experience (UC-019 + UC-020 + UC-021)
**Goal**: Hear it, buy it, browse it
- Build sequential preview playback via multi-source chain (Deezer field-specific search + ISRC lookup → iTunes Search API fallback → SoundCloud fallback)
- Build purchase link panel: Beatport, Apple Music (affiliate), Bandcamp, Traxsource, Juno
- Build scene/era classification and browse UI
- Run migration 005
- **Exit criteria**: User can preview tracks, click through to buy from multiple stores, and browse by scene

### Sprint 5: Polish (UC-023 + UC-024)
**Goal**: Refine and export
- Build conversational refinement with setlist versioning
- Build multi-format export (CSV, JSON, Plain Text, Copy-to-Clipboard; PDF deferred to v2)
- Run migration 006
- **Exit criteria**: User can iteratively refine a setlist via chat and export the result

### Sprint 6: Aspirational (UC-025)
**Goal**: Full browser DJ mix
- Extend preview engine to beat-matched mixing with crossfade
- Build full setlist playback with phrase detection
- **Exit criteria**: 5-track setlist plays as continuous mix with beat-matched transitions

---

## 8. Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Beatport API access denied (requires application) | Medium | High | Apply early. Fallback: Beatport search scraping or skip to SoundCloud-first. |
| SoundCloud `urn` migration breaks during integration | Low | Medium | Use `urn` from day 1 per their migration guide. Test with both `id` and `urn`. |
| essentia accuracy too low for DJ-grade BPM/key | Medium | High | Validate against known Beatport data. Fallback: use Beatport as ground truth, essentia for Spotify/SoundCloud only. |
| LLM hallucinations in setlist generation | High | Medium | Validate all track_ids against DB. Reclassify hallucinated tracks as suggestions. |
| Claude API costs exceed budget | Medium | Medium | Prompt caching (90% reduction), per-user daily limits, batch enrichment. Monitor usage. |
| Web Audio API CORS issues with preview sources | High | Medium | Backend CORS proxy for Deezer MP3s (implemented). SoundCloud and iTunes may also need proxying. |
| Browser performance insufficient for beat-matching (UC-025) | Medium | Low | Fallback chain: beat-match → crossfade → hard cut. UC-025 is P3 aspirational. |
| Spotify removing preview URLs (ongoing trend) | **Confirmed** | Resolved | Spotify previews deprecated Nov 2024. Multi-source chain (Deezer→iTunes→SoundCloud) replaces them. |
| Deezer freeform search accuracy for electronic music | High | Medium | Use field-specific search `artist:"X" track:"Y" strict=on` + ISRC lookup `GET /track/isrc:{ISRC}` as fallback chain. ~20-30% error rate with freeform. |
| iTunes Search API terms/rate limits | Low | Medium | Free, no auth, 100M+ catalog, 100 req/min. Monitor for ToS changes; SoundCloud is third fallback. |

---

## 9. Acceptance Criteria Summary

### Per-Use-Case Test Counts (estimated)
| UC | Backend Tests | Frontend Tests | Manual Tests |
|----|:---:|:---:|:---:|
| UC-001 | 49 ✅ | 1 ✅ | 3 |
| UC-013 | ~20 | ~3 | 2 |
| UC-014 | ~18 | ~3 | 2 |
| UC-015 | ~15 | ~2 | 3 |
| UC-016 | ~20 | ~5 | 5 |
| UC-017 | ~15 | ~3 | 2 |
| UC-018 | ~10 | ~2 | 1 |
| UC-019 | ~5 | ~8 | 5 |
| UC-020 | ~5 | ~3 | 1 |
| UC-021 | ~10 | ~4 | 2 |
| UC-023 | ~15 | ~5 | 3 |
| UC-024 | ~8 | ~3 | 3 |
| UC-025 | ~3 | ~10 | 8 |
| **Total** | **~193** | **~52** | **~40** |

### System-Wide Quality Gates
- `cargo fmt --check && cargo clippy -- -D warnings && cargo test` — all pass
- `flutter analyze && flutter test` — all pass
- No API credentials in source code or frontend bundles
- All error paths return user-facing messages (no raw stack traces)
- Per-track transaction isolation for all import operations

---

## 10. Coverage Gaps & Open Questions

### Identified Gaps
1. **UC-022 missing**: No use case for user authentication/registration (currently using `X-User-Id` header). Need UC-008 (JWT auth) before production.
2. **No offline mode**: All features require network. Consider caching catalog locally for offline browsing.
3. **No track deduplication across sources**: Same track imported from Spotify and Beatport creates two rows. Need a cross-source dedup UC using ISRC.
4. **No user preferences/settings UC**: No way for user to configure defaults (preferred BPM range, favorite scenes, daily limit overrides).
5. **No admin/monitoring UC**: No observability into background workers, LLM costs, analysis queue depth.

### Suggested Next Actions
1. Create UC for JWT authentication (prerequisite for production)
2. Create UC for cross-source track deduplication via ISRC
3. Apply for Beatport API v4 access immediately (may take weeks to approve)
4. Register SoundCloud OAuth 2.1 app credentials
5. Register for Apple Music affiliate program (easiest near-term revenue)
6. Implement Deezer field-specific search + ISRC fallback chain (Phase 4)
7. Begin Sprint 1 implementation: `/task-decompose UC-013`
