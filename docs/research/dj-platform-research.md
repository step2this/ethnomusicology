# DJ Platform Research Notes

## Date: 2026-02-28

## 1. Platform API Assessment

### Beatport (Primary DJ Source)
- **API**: Public v4 at https://api.beatport.com/v4/docs/ (v3 deprecated)
- **Auth**: OAuth
- **DJ Metadata**: Native BPM, musical key, genre, sub-genre, label, remixer, ISRC
- **Track fields**: id, name, mix_name, artists, remixers, key, bpm, genre, subgenre, label, release_date, length, isrc, bpm_range
- **Affiliate Program**: Discontinued (~2008). Contact directly for partnership.
- **Key advantage**: Only platform with native DJ-grade metadata (BPM + key)
- **Format**: Downloads in FLAC, AAC

### SoundCloud (Discovery + Streaming)
- **API**: Public, maintained, regular updates
- **Auth**: OAuth 2.1 (migrated from 2.0). Tokens transitioning to JWT format.
- **Migrations in progress**:
  - By June 30, 2025: use `urn` field instead of `id` field
  - By Dec 31, 2025: streaming migrating to AAC HLS format
- **Track access**: Upload, retrieve metadata, stream URLs, playlists
- **Stream access levels**: `playable`, `preview`, `blocked`
- **DJ Metadata**: NONE natively (no BPM, no key). Must run audio analysis.
- **Search**: Across tracks, users, playlists
- **All requests require**: `Authorization: OAuth ACCESS_TOKEN`

### Bandcamp — SKIPPED
- No public API (partner-only, restricted)
- No BPM/key metadata
- Decision: Skip entirely, focus on Spotify + Beatport + SoundCloud

### Spotify (Already Integrated)
- UC-001 complete with OAuth, import, retry/resilience
- No native BPM/key — will need audio analysis for DJ metadata

## 2. Audio Analysis

### Recommended: essentia (C++/Python)
- Comprehensive audio analysis framework from Music Technology Group (Barcelona)
- **BPM**: RhythmExtractor2013 algorithm, LoopBpmEstimator for clips, TempoCNN for detailed analysis
- **Key Detection**: Via chroma features and spectral analysis
- **Integration approach**: Python sidecar service or CLI subprocess called from Rust via `tokio::process::Command`
- **Strategy**: Analyze server-side, cache results in DB. Don't analyze in browser.

### Alternative: librosa (Python)
- Industry standard for music information retrieval
- Used by Spotify, YouTube Music
- Provides: beat tracking, tempo estimation, chroma/tonal features, onset detection
- Active, well-documented

### Alternative: Realtime BPM Analyzer (TypeScript)
- WebAudioAPI-based, real-time processing
- Could be useful for browser-based analysis later

### Open-source DJ Software
- **Mixxx**: Open-source DJ software with built-in key and BPM detection
- Can potentially use its analysis algorithms

## 3. Harmonic Mixing — Camelot Wheel

### System Overview
- Developed by Mark Davis (Mixed In Key)
- 24 keys: 1-12 with A (major) or B (minor) suffix
- Adapted from the circle of fifths for DJ use

### Compatibility Rules
1. **Same number, different letter**: 8A ↔ 8B (relative major/minor)
2. **Adjacent numbers, same letter**: 8A ↔ 7A or 9A (semitone shift)
3. **Circle wraps**: 12A ↔ 1A (adjacent)

### Key-to-Camelot Mapping (for reference)
| Camelot | Major Key | Minor Key |
|---------|-----------|-----------|
| 1A / 1B | B major | G# minor |
| 2A / 2B | F# major | Eb minor |
| 3A / 3B | Db major | Bb minor |
| 4A / 4B | Ab major | F minor |
| 5A / 5B | Eb major | C minor |
| 6A / 6B | Bb major | G minor |
| 7A / 7B | F major | D minor |
| 8A / 8B | C major | A minor |
| 9A / 9B | G major | E minor |
| 10A / 10B | D major | B minor |
| 11A / 11B | A major | F# minor |
| 12A / 12B | E major | C# minor |

### Implementation
- Pure algorithmic logic in Rust — no external dependencies needed
- Data structure: enum with 24 variants
- `compatible_keys(key: CamelotKey) -> Vec<CamelotKey>` returns 3 compatible keys
- Transition scoring: 1.0 for same key, 0.9 for compatible, 0.5 for ±2, 0.0 for incompatible

## 4. LLM Music Knowledge Strategy

### Approach: Claude API + Rich Prompt Engineering
- **NOT fine-tuning**: Disproportionate cost/effort for solo developer
- **NOT RAG**: Catalog fits in 200K context window (5,000+ tracks with metadata)
- Claude Sonnet already knows Larry Levan, Sound Factory, Paradise Garage, Frankie Knuckles, Chicago house, Detroit techno, NYC underground, Berlin minimal, UK garage, etc.

### Architecture
- **Model**: claude-sonnet-4-20250514 (default), claude-opus-4-20250514 (complex refinements)
- **System prompt** (~2K tokens, cached): DJ/music expert persona, output format spec, Camelot rules
- **Catalog context** (~10-50K tokens, cached): All tracks with DJ metadata serialized
- **User prompt**: Verbatim natural language
- **Output**: Structured JSON — setlist tracks, missing suggestions, transition notes
- **Cost controls**: Prompt caching (~90% cost reduction), per-user daily limits

### Example Prompt Understanding
"I want a setlist of music that captures the deep, dubby, underground sound of NYC house in the early 90s from DJs who played at the Sound Factory"
→ LLM should understand: Sound Factory (NYC, 1989-1995), Junior Vasquez (resident), Larry Levan (Paradise Garage influence), deep house subgenre, dub-influenced production, Roland TB-303 basslines, Chicago house roots, 118-125 BPM range, keys often in minor modes

## 5. DJ Concepts Reference

### Metadata Hierarchy (what DJs care about most)
1. **BPM** — essential for beatmatching
2. **Musical Key** (Camelot notation) — for harmonic mixing
3. **Energy Level** (1-8 scale) — controls dancefloor intensity
4. **Genre/Sub-genre** — set coherence
5. **Mood tags** — emotional arc (dark, euphoric, groovy, melancholic)
6. **Label** — quality signal, sub-genre indicator
7. **Release date** — freshness vs. classic status
8. **Duration** — set planning
9. **Intro/outro length** — transition planning

### Energy Flow in Sets
- Start medium, build gradually
- Peak at climactic moments
- Bring down for breathers (essential pacing)
- "Ebb and flow" — rises and releases, not sustained peak
- Energy levels: 1 (ambient) → 6 (danceable) → 7 (high energy) → 8 (maximum impact)

### Digital Crate Digging
Our LLM replaces manual crate digging:
- Instead of browsing Beatport charts manually → prompt: "deep NYC house early 90s Sound Factory"
- Instead of browsing by genre → prompt: "something that sounds like Kerri Chandler meets Burial"
- Instead of key/BPM filtering → automatic Camelot arrangement

## 6. Feasibility Matrix

| Feature | Feasibility | Sprint Estimate |
|---------|------------|-----------------|
| Beatport import (API v4) | HIGH | 1 sprint |
| SoundCloud import (API) | HIGH | 1 sprint |
| BPM/key detection (essentia server-side) | HIGH | 1-2 sprints |
| LLM setlist generation (Claude API) | HIGH | 1-2 sprints |
| Camelot arrangement (pure Rust) | HIGH | 0.5 sprint |
| Crossfade preview (Web Audio API) | MEDIUM | 1 sprint |
| Purchase links (URL construction) | HIGH | 0.5 sprint |
| DJ metadata enrichment (analysis + LLM) | MEDIUM | 1-2 sprints |
| Conversational refinement | MEDIUM | 1 sprint |
| Full beat-matched mixing | LOW | 3+ sprints (aspirational) |

## Sources
- Beatport API: https://api.beatport.com/v4/docs/
- SoundCloud API: https://developers.soundcloud.com/docs/api/guide
- SoundCloud OAuth 2.1: https://developers.soundcloud.com/blog/oauth-migration/
- essentia: https://essentia.upf.edu/
- librosa: https://librosa.org/doc/latest/
- Mixed In Key (Camelot): https://mixedinkey.com/camelot-wheel/
- DJ.Studio (set structure): https://dj.studio/blog/anatomy-great-dj-mix-structure-energy-flow-transition-logic
