# Ethnomusicology Web App - Project Plan

## Context

Build a web app that helps people create music playlists for different occasions using music from African and Middle Eastern Muslim regions (e.g., an uplifting playlist for a Nikah). Starting from a 54-track Spotify playlist called "Salamic Vibes" featuring artists like Tinariwen, Ali Farka Toure, Ballake Sissoko, Fairuz, and Amr Diab.

---

## Tech Stack Decision

### Recommendation: Axum (Rust) Backend + Flutter Frontend (Hybrid)

| Layer | Choice | Why |
|-------|--------|-----|
| **Backend** | **Axum 0.8 (Rust)** | Learn Rust where it shines. Clean JSON API serves Flutter web, future mobile, and any other client. Tower middleware, async, type safety. |
| **Frontend** | **Flutter (Dart)** | Cross-platform UI. `just_audio` for playlist playback. `ReorderableListView` for drag-and-drop. Built-in RTL/Arabic support. Same codebase becomes mobile app. |
| **Landing pages** | **Static HTML + Tailwind** | SEO-friendly entry point. Minimal, fast-loading. Links into the Flutter app. |
| **Database** | **SQLite via SQLx** (dev), **PostgreSQL** (prod) | SQLite for local dev simplicity. SQLx supports both with same query syntax. Migrate to PostgreSQL when deploying. |
| **Spotify** | **rspotify 0.15** (backend) | Keep API keys server-side. Handle OAuth token refresh on backend. |
| **YouTube** | **reqwest + YouTube Data API v3** (backend) | Cache results in DB. Rate limit protection server-side. |
| **Music metadata** | **Last.fm + MusicBrainz** (backend) | Aggregate and cache. Frontend never calls these directly. |
| **Deployment** | **Fly.io** | Supports both Rust backend and Flutter web static hosting. Persistent volumes for DB. |

### Architecture

```
Flutter Web/Mobile (Dart)          Axum Backend (Rust)
┌─────────────────────┐           ┌─────────────────────────┐
│ UI Widgets          │           │ Routes (JSON API)        │
│ Audio Player        │──JSON────│ Services                 │
│ State (Riverpod)    │  HTTP    │  ├─ spotify.rs           │
│ Offline Cache       │           │  ├─ youtube.rs           │
│ RTL/Localization    │           │  ├─ lastfm.rs            │
└─────────────────────┘           │  ├─ musicbrainz.rs       │
   Same codebase for:             │  └─ recommendation.rs    │
   - Web (now)                    │ DB (PostgreSQL via SQLx) │
   - iOS (later)                  └─────────────────────────┘
   - Android (later)
```

---

## Use Cases (MVP - Phase 1)

### UC-01: Import Seed Catalog from Spotify Playlist (Critical)
- System ingests the 54-track "Salamic Vibes" playlist into local DB
- Stores: title, artist(s), album, duration, Spotify URI, preview URL
- Must handle Feb 2026 API restrictions (playlist must be owned/collaborated)
- *Depends on: nothing*

### UC-02: Enrich Track Metadata with Cultural/Regional Taxonomy (Critical)
- Each track annotated with: region, country, musical tradition, language, mood tags, occasion-suitability scores
- MusicBrainz/Last.fm pre-populate where possible; curator fills the rest
- Taxonomy covers: Gnawa, Tuareg blues, Chaabi, Taarab, Qawwali, Rai, Mbalax, etc.
- *Depends on: UC-01*

### UC-03: Browse and Discover Music Catalog (Critical)
- Browse by Region (Maghreb, Sahel, Levant, Horn of Africa), by Tradition, by Occasion, by Artist
- Artist pages with bio, tradition context, track list
- *Depends on: UC-01, UC-02*

### UC-04: Create Occasion-Specific Playlist (Critical - flagship feature)
- User selects occasion (Nikah, Eid al-Fitr, Eid al-Adha, Mawlid, Sufi gathering, etc.)
- System recommends tracks organized by playlist phase (processional, ceremony, celebration)
- User adds/removes/reorders, names, and saves
- *Depends on: UC-03, UC-05*

### UC-05: Get Recommendations by Occasion and Mood (Critical)
- Tag-based scoring: occasion match, mood alignment, tradition diversity
- Not ML - intelligence comes from curator-applied tags
- *Depends on: UC-02*

### UC-06: Preview/Play a Track (High)
- Waterfall: Spotify 30s preview -> YouTube embed -> external link
- We are a discovery/curation tool, not a streaming service
- *Depends on: UC-01*

### UC-07: Filter by Region, Tradition, Instrument, Language (High)
- Faceted filtering on enriched metadata
- *Depends on: UC-02*

### UC-08: User Accounts and Saved Playlists (Medium)
- Registration, login, persistent playlist storage
- MVP can start with session-based, add accounts later
- *Depends on: UC-04*

### Phase 2 (Post-MVP)
- UC-09: Export Playlist to Spotify
- UC-10: Add Music from External Sources (YouTube, manual upload)
- UC-11: Cultural Context / Educational Content pages
- UC-12: Share Playlist via Link / Social Media

---

## Devil's Advocate Concerns

| Concern | Mitigation |
|---------|-----------|
| **Spotify API is shrinking** | Design around source-agnostic internal catalog. Spotify = one ingest source + playback layer, not the foundation. |
| **Occasion classification can't be automated** | Accept it. 54 tracks is small enough to curate by hand. Build a good curation UI. Scale later with community + ML. |
| **Cultural sensitivity** | Taxonomy includes sacred/devotional flag. Don't flatten devotional music into "party playlist." State editorial policy. |
| **"Muslim Africa" is not a monolith** | Granular regional/tradition taxonomy. Browse UX leads with specific regions, not homogenizing umbrella. |
| **Cold start: only 54 tracks** | Frame as "curated collection." Use Last.fm `getSimilar` to expand. Prioritize UC-10 for Phase 2. |
| **Preview/playback licensing** | Waterfall approach (Spotify preview -> YouTube embed -> link). Never host audio directly. |

---

## Project Structure

```
ethnomusicology/
  .claude/                        # Forge (copied from rust-term-chat)
    agents/, skills/, commands/, settings.json
  backend/                        # Rust (Axum) API server
    Cargo.toml
    .env                          # API keys (gitignored)
    migrations/
      001_initial_schema.sql
    src/
      main.rs                     # Axum app: router, middleware, CORS, state
      config.rs                   # Environment config
      error.rs                    # App-wide error types
      db/
        mod.rs, tracks.rs, artists.rs, occasions.rs, playlists.rs
      api/
        mod.rs, spotify.rs, lastfm.rs, musicbrainz.rs, youtube.rs
      services/
        mod.rs, enrichment.rs, recommendation.rs, playlist_builder.rs
      routes/
        mod.rs, occasions.rs, playlists.rs, tracks.rs, search.rs, auth.rs
  frontend/                       # Flutter (Dart) cross-platform UI
    pubspec.yaml
    lib/
      main.dart
      config/
        theme.dart                # Material 3 theming, occasion-based palettes
        routes.dart               # GoRouter navigation
        localization.dart         # Arabic/English RTL support
      models/
        playlist.dart, track.dart, occasion.dart, user.dart
      services/
        api_client.dart           # HTTP client to Rust backend
        audio_service.dart        # just_audio wrapper
      providers/                  # Riverpod state management
        playlist_provider.dart, player_provider.dart, auth_provider.dart
      screens/
        home_screen.dart, occasion_screen.dart, playlist_screen.dart,
        search_screen.dart, browse_screen.dart
      widgets/
        track_tile.dart, playlist_card.dart, audio_player_bar.dart,
        occasion_selector.dart, reorderable_track_list.dart
  landing/                        # Static HTML landing pages (SEO)
    index.html
    css/output.css
  docs/
    use-cases/                    # Cockburn-style use case docs
  .gitignore
```

---

## Key Dependencies

### Backend (Rust - `backend/Cargo.toml`)
```toml
axum = "0.8"
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace", "compression-gzip"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
rspotify = { version = "0.15", features = ["client-reqwest", "reqwest-rustls-tls"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
dotenvy = "0.15"
uuid = { version = "1", features = ["v4"] }
jsonwebtoken = "9"
```

### Frontend (Flutter - `frontend/pubspec.yaml`)
```yaml
dependencies:
  flutter:
    sdk: flutter
  just_audio: ^0.9.0
  dio: ^5.0.0
  flutter_riverpod: ^2.0.0
  go_router: ^14.0.0
  google_fonts: ^6.0.0
  flutter_localizations:
    sdk: flutter
```

---

## Implementation Sequence

### Sprint 0: Project Setup
- Create CLAUDE.md, project plan, session handoff docs
- Copy Forge from rust-term-chat, adapt for monorepo
- Initialize git, .gitignore
- Scaffold Rust backend (Axum hello-world JSON endpoint)
- Install Flutter SDK, scaffold frontend
- Create static landing page
- Verify end-to-end: backend serves JSON, Flutter renders it

### Sprint 1: Data Foundation (UC-01, UC-06)
1. Implement Spotify OAuth flow in Axum backend
2. Build playlist import endpoint (ingest "Salamic Vibes" 54 tracks)
3. Flutter: basic track list screen consuming API
4. Audio playback: `just_audio` in Flutter + Spotify preview URLs from backend
5. YouTube fallback for tracks without Spotify previews
6. Verify: Tracks in DB, Flutter shows them, audio plays

### Sprint 2: Enrichment & Taxonomy (UC-02, UC-07)
1. Build Last.fm + MusicBrainz API clients in Rust backend
2. Auto-enrich tracks with tags and metadata, store in DB
3. Define occasion/mood/region/tradition taxonomy as seed data
4. Build curation admin API endpoints + basic Flutter admin screen
5. Implement faceted filtering API + Flutter filter UI
6. Verify: All 54 tracks enriched, filters work in Flutter

### Sprint 3: Discovery & Playlists (UC-03, UC-04, UC-05)
1. Build recommendation engine in Rust (tag-based occasion scoring)
2. Flutter: browse screens (by region, tradition, occasion)
3. Flutter: playlist creation flow with `ReorderableListView` for drag-and-drop
4. Persistent audio player bar widget (survives navigation)
5. Verify: Can create occasion-specific playlists with relevant recommendations

### Sprint 4: Accounts & Polish (UC-08)
1. User registration/login (JWT auth in Axum, auth state in Flutter Riverpod)
2. Persistent playlist storage
3. RTL/Arabic localization support
4. Responsive design (web and future mobile breakpoints)
5. Deploy to Fly.io (backend + Flutter web build as static assets)

---

## Verification Strategy

Each sprint includes concrete verification:
- **Backend**: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`
- **Frontend**: `flutter analyze`, `flutter test`
- **Integration**: Flutter app successfully calls all backend endpoints
- **Manual smoke testing** via Chrome
- **Quality gates** from Forge pre-commit hooks
- **Use case verification**: `/verify-uc` command validates implementation against acceptance criteria

---

## Resolved Questions

1. **Spotify playlist ownership**: User owns the "Salamic Vibes" playlist - direct API import is possible.
2. **Target audience**: Muslim families planning occasions (Nikah, Eid, etc.) - focused MVP scope.
3. **Forge**: Copy directly from rust-term-chat, adapt settings.json for monorepo structure.
4. **Frontend**: Flutter (hybrid with Rust backend) - best path to future mobile with no rewrite.
5. **Tech stack**: Axum (Rust) backend + Flutter (Dart) frontend (hybrid).
6. **Flutter SDK**: Installed via snap.
7. **Database**: SQLite for local dev, PostgreSQL for production deployment.
8. **Monorepo**: Single repo with `backend/` and `frontend/` subdirectories.
