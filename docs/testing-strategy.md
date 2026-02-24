# Testing Strategy: Ethnomusicology App

## Testing Pyramid — 70/20/10 Ratio

| Layer | Ratio | Backend (Rust/Axum) | Frontend (Flutter) |
|-------|-------|--------------------|--------------------|
| **Unit** | 70% | Handler logic, models, serialization, validators | Riverpod providers, models, services, pure Dart |
| **Integration** | 20% | HTTP round-trips (axum-test), DB (#[sqlx::test]), mocked APIs (wiremock) | Widget tests, golden tests (alchemist) |
| **E2E** | 10% | Full stack: Flutter web → Axum → SQLite via Playwright MCP | Critical user flows only |

## Tool Stack

### Backend (Rust)
| Tool | Purpose | Priority |
|------|---------|----------|
| cargo-nextest | Parallel test runner (3x faster) | P0 |
| axum-test | Ergonomic handler testing | P0 |
| wiremock | Mock Spotify/YouTube/Last.fm/MusicBrainz APIs | P0 |
| #[sqlx::test] | Database test isolation with fixtures | P0 |
| proptest | Property-based testing (serialization round-trips) | P1 |
| cargo-llvm-cov | Code coverage reports | P1 |

### Frontend (Flutter)
| Tool | Purpose | Priority |
|------|---------|----------|
| mocktail | Dart mocking for Riverpod tests | P0 |
| riverpod_test | Provider testing helpers | P0 |
| alchemist | Golden/visual regression tests (replaced golden_toolkit) | P0 |
| Built-in AccessibilityGuideline | WCAG compliance checks | P0 |

### E2E / MCP Tools
| Tool | Purpose | Priority |
|------|---------|----------|
| Playwright MCP | Browser automation for Flutter web E2E | P1 |
| Marionette MCP | Flutter app control (inspect widgets, tap, screenshot) | P2 - Evaluate |

## CI/CD Matrix

| Trigger | Backend | Frontend | E2E |
|---------|---------|----------|-----|
| Every commit | fmt + clippy + nextest | analyze + test | — |
| Every PR | + coverage (llvm-cov) | + golden tests + a11y | Critical paths |
| Nightly | + full proptest | + full E2E | Full suite + cargo audit |

## Critical E2E Flows (only these get E2E tests)
1. Browse playlists → view tracks
2. Search for tracks
3. Create/edit playlist (drag-and-drop reorder)
4. Play audio preview (waterfall: Spotify → YouTube → link)
5. User login/auth (Sprint 4)
