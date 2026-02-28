# Session Handoff — 2026-02-28

## What Was Accomplished This Session

### 1. UC-001 Git Cleanup (Complete)
- Fixed 3 quality gate issues on the feature branch:
  - `cargo fmt` formatting in `main.rs` and `repo.rs`
  - Clippy: removed unnecessary `i64` cast in `repo.rs`
  - Fixed `test_import_with_mock_spotify`: pre-populated encrypted tokens and `X-User-Id` header
  - Replaced `package:web` with `url_launcher` in `spotify_import_screen.dart` (web package fails in Flutter test VM)
- All 49 backend tests + 1 frontend test pass
- Merged `feature/uc-001-spotify-import` → `main` (fast-forward)
- Pushed to GitHub, deleted feature branch

### 2. DJ-First Pivot — Research & Planning (Complete)
- Researched Beatport API v4, SoundCloud API (OAuth 2.1), Bandcamp (no public API — skipped)
- Researched audio analysis: essentia (recommended), librosa, Camelot wheel system
- Researched LLM music knowledge: Claude Sonnet with prompt engineering (not fine-tuning, not RAG)
- Wrote comprehensive research notes: `docs/research/dj-platform-research.md`
- Created evolution plan: `.claude/plans/golden-questing-lollipop.md`

### 3. Key Decisions Locked
- **App identity**: DJ-first pivot. Occasion features secondary.
- **LLM model**: Claude Sonnet default, Opus for complex refinements
- **Bandcamp**: Skipped entirely. Focus on Spotify + Beatport + SoundCloud.
- **Playback**: Crossfade preview (3-5s). Full beat-matching is P3 stretch.
- **UX/Design**: `design-crit` plugin installed for structured design critiques

### 4. CLAUDE.md Updated
- Reflects DJ-first vision, new integrations, design-crit plugin, current state

### 5. Settings Configured
- `.claude/settings.json` updated with auto-approve permissions for all dev tools
- Pre-commit quality gate hook preserved

## Current State

### Git
- **Branch**: `main` (clean, up to date with origin)
- **Latest commit**: `2e77def` — Configure auto-approve permissions
- **No uncommitted changes** (except this handoff file, to be committed)
- **GitHub**: `git@github.com:step2this/ethnomusicology.git`

### Tests
- Backend: **49 passed**, 0 failed
- Frontend: **1 passed**, 0 failed (widget_test only)

## What the Next Session Should Do

### Immediate: Create DJ Use Cases via the Forge

Read the evolution plan first:
```
Read .claude/plans/golden-questing-lollipop.md
```

Then create each use case in order using `/uc-create`:

**Tier 1 — DJ Core (P0):**
1. UC-013: Import Tracks from Beatport
2. UC-014: Import Tracks from SoundCloud
3. UC-015: Detect BPM and Musical Key for Track
4. UC-016: Generate Setlist from Natural Language Prompt
5. UC-017: Arrange Setlist by Harmonic Compatibility

**Tier 2 — Enhanced (P1):**
6. UC-018: Enrich Track with DJ Metadata (energy, mood, genre)
7. UC-019: Crossfade Preview Between Tracks
8. UC-020: Generate Purchase Links for Tracks
9. UC-021: Browse by DJ Scene and Era

**Tier 3 — Polish (P2-P3):**
10. UC-023: Refine Setlist with Conversational Feedback
11. UC-024: Export Setlist with Transition Notes
12. UC-025: Full Browser-Based DJ Mix Playback (aspirational)

After creating each UC, run `/uc-review` to catch gaps. Then run `/prd-from-usecases` to synthesize the full PRD.

### Architecture Prep (can parallel with UC creation)
- Define `MusicSourceClient` trait (follows `ImportRepository` pattern from UC-001)
- Plan DB migrations: `003_dj_metadata.sql`, `004_multi_source.sql`
- Create Forge skills: `camelot-reference.md`, `llm-prompt-template.md`

## Key Reference Files

| File | Purpose |
|------|---------|
| `CLAUDE.md` | Standing orders, architecture, current state |
| `.claude/plans/golden-questing-lollipop.md` | Full DJ pivot plan with locked decisions |
| `docs/research/dj-platform-research.md` | Beatport/SoundCloud/essentia/Camelot research |
| `docs/project-plan.md` | Original project plan (UC-001 through UC-012) |
| `docs/use-cases/uc-001-import-seed-catalog-from-spotify.md` | Reference pattern for new UCs |
| `docs/tasks/uc-001-tasks.md` | Task decomposition pattern reference |
| `.claude/settings.json` | Auto-approve permissions + pre-commit hooks |

## Blockers
- None. All systems green, all tests passing, GitHub up to date.

## Lessons Learned This Session
- `package:web` (`dart:js_interop`) is web-only — fails in Flutter test VM. Use `url_launcher` instead.
- Import route tests need encrypted tokens pre-populated in test DB before calling the handler.
- Beatport API v4 requires OAuth — apply for access early before building the client.
- SoundCloud is migrating to OAuth 2.1 and AAC HLS — use `urn` field not `id` field (deadline June 2025).
