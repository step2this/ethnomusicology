# Design Brief: Ethnomusicology — Import & Catalog

> Multi-source import screen and DJ metadata catalog for an LLM-powered DJ setlist platform.

## Target Users and Context of Use
DJs who build sets from digital catalogs. Laptop at a desk (primary), occasionally tablet. They think in BPM, key, and energy — not artist/title. Sessions involve importing tracks from a source, then browsing their enriched catalog to build sets. Desktop-first, used during set prep (not live performance). Usage frequency: a few times per week during active gig prep.

## Core Interaction Loop
Select source → paste URL → import tracks → browse catalog with DJ metadata (BPM, Camelot key, genre, label, source). For this scope: **source tab → URL → import → enriched track list**.

## Differentiator
Multi-source import with DJ-grade metadata visible at a glance. Beatport imports arrive with native BPM/key — no analysis needed. Camelot key color-coding gives instant harmonic compatibility awareness. The catalog view is designed for DJs (Rekordbox-influenced), not casual listeners.

## Platform and Constraints
- **Platform:** Web (Flutter Web, Chrome primary)
- **Tech stack:** Flutter/Dart, Riverpod state management, GoRouter navigation, Material 3
- **Device targets:** Desktop-first, responsive to tablet. No mobile-first requirement.
- **Hard constraints:** Arabic/RTL text support (Noto Sans Arabic in design system). No streaming audio on these screens.

## Scope
### In v1
- Tabbed import screen: Spotify | Beatport | SoundCloud (disabled/coming soon)
- Beatport tab: URL input, validation, import flow (no OAuth — app-level credentials)
- Spotify tab: existing OAuth + URL flow (preserve current behavior)
- Import progress indicator and summary (reuse existing patterns)
- Track catalog with enriched tiles: album art, title, artist, BPM, Camelot key (color-coded chip), genre, source badge
- Camelot key color-coding using wheel-position tinting

### Out of v1
- Setlist generation UI (UC-016, future sprint)
- Crossfade preview player (UC-019)
- Catalog search, sort, and filter controls
- Dense table view toggle (future enhancement)
- Mobile layout optimization
- SoundCloud import (UC-014, implemented later but tab placeholder shown)

## Existing Design Language
Comprehensive design system at `.claude/skills/design-system.md`:
- Material 3 via `ColorScheme.fromSeed`, Navy #1A237E primary, Gold #F9A825 secondary
- Noto Sans (body) + Noto Sans Arabic + Playfair Display (headings)
- 4dp spacing grid with defined tokens (xs=4, sm=8, md=16, lg=24, xl=32)
- Elevation: Material 3 tonal elevation (5 levels)
- Border radius tokens: xs=4, sm=8, md=12, lg=16, xl=24
- Existing components: TrackTile (basic ListTile), ConnectionCard, ImportProgress, ImportSummary
- Motion specs defined (micro=100ms, fast=150ms, standard=300ms, emphasis=500ms)

## Accessibility Requirements
Unknown — will audit in accessibility area. Material 3 defaults provide baseline. Arabic text rendering supported via Noto Sans Arabic font choice. Touch targets follow 48dp Material minimum.

---

## Design Decisions (from brief)

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Source selector | Tabs on single screen | Keeps flow together, easy switching. Beatport tab skips OAuth card. |
| Catalog density | Enriched tiles | Album art + DJ metadata chips. More visual than a table, fits the design system's TrackTile pattern. |
| Camelot key treatment | Colored chips | Chip with tinted background by wheel position. Immediate visual harmonic awareness. |
