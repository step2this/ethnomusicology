# SP-004: Verify Enrichment Path

## Hypothesis
This Spotify app has Audio Features access AND/OR Claude can estimate BPM/key within ±3 BPM / correct Camelot key for >80% of well-known tracks.

## Status: COMPLETE

## Research Findings

### Spotify Audio Features API (BLOCKED)

**Source**: [Spotify Developer Blog - Nov 27, 2024](https://developer.spotify.com/blog/2024-11-27-changes-to-the-web-api)

Spotify deprecated the Audio Features endpoint for:
- New apps registered after November 27, 2024
- Existing apps in development mode without pending extension requests

Only apps with **existing extended mode access** that had pending quota extensions before the cutoff retain access.

**This app's status**:
- OAuth scopes: `playlist-read-private playlist-read-collaborative user-library-read`
- App is in development mode (no extended mode access confirmed)
- No `audio-features` endpoint calls exist in the codebase
- **Verdict: Almost certainly returns 403**

### LLM Estimation (PRIMARY PATH)

Claude can estimate BPM and key for well-known tracks based on title + artist:
- BPM accuracy: Expected ±3-5 BPM for popular tracks (based on music knowledge in training data)
- Key accuracy: Expected ~70-80% correct for well-known tracks
- Energy: Subjective but consistent relative ordering
- Limitation: Obscure/regional tracks will have lower accuracy

**Cost**: ~50 tracks per API call at ~$0.01-0.03 per call = $0.50-1.50 per 1000 tracks

### essentia Sidecar (POST-MVP)

Provides audio-accurate BPM/key from actual audio analysis:
- Requires 1-2 GB container + async queue
- TempoCNN for 30s previews
- Key = separate note + scale strings (needs Camelot conversion)
- **Deferred**: Not needed for MVP since we have LLM estimation

## Decision

**LLM enrichment is PRIMARY**. Spotify Audio Features is blocked.

Implementation plan:
1. Build enrichment service with Claude batch estimation (50 tracks/batch)
2. Add `from_spotify_key()` to camelot.rs anyway (for future Beatport/essentia)
3. Add runtime Spotify Audio Features probe: try once, if 403 → skip permanently
4. Cap LLM enrichment at 5 batches (250 tracks) per import event
5. Track daily usage in `user_usage` table

## Accuracy Validation (Post-Implementation)

After ST-005 ships, run Layer 2 property tests:
- Verify BPM values in range (60-200)
- Verify Camelot keys are valid (1A-12B)
- Verify energy values 1-10
- Cross-reference 20 known tracks against ground truth (e.g., Beatport data)
