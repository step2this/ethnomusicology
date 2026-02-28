# Use Case: UC-015 Detect BPM and Musical Key for Track

## Classification
- **Goal Level**: 🌊 User Goal
- **Scope**: System (black box)
- **Priority**: P0 Critical
- **Complexity**: 🟠 High

## Actors
- **Primary Actor**: System (background worker, triggered by import or schedule)
- **Secondary Actor**: App User (views analysis results, can see progress)
- **Supporting Actors**:
  - essentia Analysis Service (Python HTTP sidecar)
  - Audio source APIs (Spotify preview URLs, SoundCloud stream URLs)
  - Database (SQLite/PostgreSQL via SQLx)
- **Stakeholders & Interests**:
  - DJ User: Needs accurate BPM and key data for harmonic mixing; expects results within minutes of import, not hours
  - Developer: Wants a clean analysis pipeline that works across audio sources and caches results permanently
  - System: Needs to handle tracks with no audio URL gracefully and not waste resources re-analyzing

## Conditions
- **Preconditions** (must be true before starting):
  1. Tracks exist in the database with `needs_analysis = true` (set by UC-014 SoundCloud import, or UC-001 Spotify import for tracks lacking BPM/key)
  2. essentia Python sidecar service is running and reachable at configured URL
  3. Tracks have a potential audio source (SoundCloud stream URL, Spotify preview URL, or local file). Tracks with no audio source are skipped with `analysis_error = 'no_audio_source'`
  4. Database migrations 001-003 applied (tracks table has bpm, musical_key, camelot_key columns)

- **Success Postconditions** (true when done right):
  1. Analyzed tracks have `bpm` (float, e.g., 124.5), `musical_key` (e.g., "A minor"), and `camelot_key` (e.g., "8B") populated in the `tracks` table
  2. `needs_analysis` flag is set to `false` for successfully analyzed tracks
  3. Analysis timestamp is recorded (`analyzed_at` column)
  4. Tracks that could not be analyzed (no audio URL, analysis failure) have `needs_analysis` set to `false` with `analysis_error` populated
  5. Beatport tracks with existing BPM/key are never re-analyzed (they already have DJ-grade data)
  6. Analysis results are permanent — re-importing the same track does not trigger re-analysis unless the user explicitly requests it

- **Failure Postconditions** (true when it fails gracefully):
  1. If the analysis service is unreachable, tracks remain `needs_analysis = true` for retry on next cycle
  2. If audio download fails for a specific track, that track is marked with `analysis_error` and skipped; other tracks continue
  3. The background worker does not crash — individual track failures are isolated

- **Invariants** (must remain true throughout):
  1. Audio files are downloaded to a temp directory and deleted after analysis — never persisted permanently
  2. Analysis happens server-side only — no audio processing in the browser
  3. The essentia sidecar is stateless — all state is in the database
  4. Existing BPM/key values from Beatport are never overwritten by analysis (source-of-truth hierarchy: Beatport native > essentia analysis)

## Main Success Scenario
1. Background worker polls for tracks with `needs_analysis = true`, ordered by import timestamp (oldest first), batch size of 10
2. For each track in the batch, worker determines the best audio source URL: SoundCloud stream URL (preferred, longer audio) → Spotify preview URL (fallback, 30s clip)
3. Worker downloads the audio to a temporary file via HTTP
4. Worker sends the audio file as binary data in the HTTP request body (multipart/form-data), NOT a file path. The essentia sidecar and the Rust backend may not share a filesystem. Request format: `POST /analyze` with `Content-Type: multipart/form-data`, field `audio` containing the file bytes
5. essentia service runs BPM detection (RhythmExtractor2013) and key detection (KeyExtractor) on the audio
6. essentia service returns JSON: `{ "bpm": 124.5, "key": "A", "scale": "minor", "confidence": { "bpm": 0.92, "key": 0.85 } }`
7. Worker converts the key+scale to Camelot notation using the Camelot module (from UC-013): "A minor" → "8B"
8. Worker updates the track in the database: sets `bpm`, `musical_key`, `camelot_key`, `analyzed_at`, `needs_analysis = false`
9. Worker deletes the temporary audio file
10. Worker moves to the next track in the batch
11. When batch completes, worker sleeps for configured interval (default 30s) then polls again
12. User sees BPM and key appear in their catalog as tracks are analyzed (reactive UI update or poll)

## Extensions (What Can Go Wrong)

- **1a. No tracks need analysis (queue empty)**:
  1. Worker sleeps for the configured interval
  2. Returns to step 1

- **1b. Worker fails to connect to database**:
  1. Worker logs error
  2. Sleeps for backoff interval (60s)
  3. Returns to step 1

- **2a. Track has no audio source URL (no preview URL, no stream URL)**:
  1. Worker sets `needs_analysis = false` and `analysis_error = 'no_audio_source'`
  2. Continues to next track

- **2b. Track source is Beatport with existing BPM/key**:
  1. Worker skips analysis (Beatport data is DJ-grade)
  2. Sets `needs_analysis = false` (should not have been flagged)
  3. Continues to next track

- **3a. Audio download fails (404 — URL expired or track removed)**:
  1. Worker sets `analysis_error = 'audio_download_failed'` and `needs_analysis = false`
  2. Continues to next track

- **3b. Audio download fails (network timeout)**:
  1. Worker retries up to 2 times with 2s backoff
  2. If all retries fail, leaves `needs_analysis = true` for retry on next cycle
  3. Continues to next track

- **3c. Audio download fails (403 — access revoked)**:
  1. Worker sets `analysis_error = 'audio_access_denied'` and `needs_analysis = false`
  2. Continues to next track

- **3e. Audio URL has expired (SoundCloud token expiry)**:
  1. Download returns 403 or signed URL error
  2. System refreshes the stream URL via SoundCloud API
  3. Retries download with fresh URL
  4. If refresh fails, marks track `analysis_error = 'url_expired'` and retries on next cycle

- **3d. Downloaded audio is too short (<5 seconds)**:
  1. Worker attempts analysis anyway (essentia can handle short clips with reduced accuracy)
  2. If essentia returns low confidence (<0.5), stores results with a `low_confidence` flag
  3. Continues to next track

- **4a. essentia sidecar is unreachable (connection refused)**:
  1. Worker logs critical error
  2. Leaves all remaining tracks as `needs_analysis = true`
  3. Sleeps for backoff interval (5 minutes)
  4. Returns to step 1

- **4b. essentia sidecar returns 500 (analysis crash)**:
  1. Worker sets `analysis_error = 'analysis_service_error'` for that track
  2. Sets `needs_analysis = false` (don't retry a track that crashes the analyzer)
  3. Continues to next track

- **5a. essentia cannot determine BPM (e.g., ambient/drone track with no beat)**:
  1. essentia returns `bpm: null` or `bpm: 0` with low confidence
  2. Worker stores `bpm = null`, `analysis_error = 'no_bpm_detected'`
  3. Key detection may still succeed — store key if available
  4. Sets `needs_analysis = false`

- **5b. essentia cannot determine key (atonal or heavily processed audio)**:
  1. essentia returns `key: null` with low confidence
  2. Worker stores `musical_key = null`, `camelot_key = null`
  3. BPM may still succeed — store BPM if available
  4. Sets `needs_analysis = false`

- **6a. essentia returns unexpected response format**:
  1. Worker logs the raw response
  2. Sets `analysis_error = 'invalid_response'` and `needs_analysis = false`
  3. Continues to next track

- **7a. Key-to-Camelot conversion fails (unknown key format)**:
  1. Worker stores raw `musical_key` but sets `camelot_key = null`
  2. Logs warning
  3. Continues to next track

- **8a. Database update fails**:
  1. Worker logs error
  2. Leaves `needs_analysis = true` for retry
  3. Continues to next track

- **9a. Temp file deletion fails**:
  1. Worker logs warning (non-critical)
  2. Temp directory is cleaned up periodically by OS or a cleanup job
  3. Continues to next track

- **12a. User sees "Pending analysis" for a long time (service down)**:
  1. Frontend shows "Analysis service may be delayed" after 10 minutes without updates
  2. User can see analysis status in import history

## Variations

- **V1. Manual Re-analysis**: User selects a track and clicks "Re-analyze". System sets `needs_analysis = true`, clears existing BPM/key, and the track re-enters the queue. (Does NOT override Beatport native data unless user explicitly confirms.)
- **V2. Bulk Re-analysis**: User selects multiple tracks or "all SoundCloud tracks" and triggers re-analysis.
- **V3. Priority Analysis**: When a user is actively building a setlist (UC-016), tracks in the setlist that lack BPM/key are prioritized in the analysis queue.

## Agent Execution Notes
- **Verification Command**: `cd backend && cargo test --test audio_analysis`
- **Test File**: `backend/tests/audio_analysis.rs`
- **Depends On**: UC-013 (Camelot module, migration 003), UC-014 (SoundCloud tracks with needs_analysis flag)
- **Blocks**: UC-016 (setlist generation needs BPM/key), UC-017 (harmonic arrangement needs Camelot keys), UC-018 (enrichment builds on analysis)
- **Estimated Complexity**: H (~3000 tokens implementation budget)
- **Agent Assignment**:
  - Teammate:Backend — Background worker (tokio task), audio download, essentia HTTP client, DB updates
  - Teammate:Python — essentia sidecar service (Flask/FastAPI, /analyze endpoint, Dockerfile)
  - Teammate:Frontend — Analysis status display, "Pending analysis" → BPM/key transition in catalog

### Key Implementation Details
- **essentia sidecar**: Flask/FastAPI app, single endpoint `POST /analyze` accepting audio binary (multipart/form-data), returns JSON `{ bpm, key, scale, confidence }`
- **essentia sidecar Docker**: Base image `python:3.11-slim`, install `essentia` via pip.
  Pin version: `essentia==2.1b6.dev1184`. Expose port 5000.
  Dockerfile: `FROM python:3.11-slim` -> `pip install essentia flask` -> `CMD ["python", "app.py"]`
  Architecture: linux/amd64 (essentia has limited ARM support)
  Health check: `GET /health` returns `{"status": "ok", "version": "2.1b6"}`
- **essentia algorithms**: `RhythmExtractor2013` for BPM, `KeyExtractor` for key detection
- **Background worker**: `tokio::spawn` in the Rust backend, polls DB every 30s, processes batches of 10
- **Audio temp dir**: `/tmp/ethnomusicology-analysis/` — cleaned after each track. Temporary audio files (downloaded for analysis) use a Rust Drop guard (`TempFile` wrapper) to ensure cleanup even if analysis panics or errors. Pattern: `let _guard = TempFile::new(path); // Deleted on drop`
- **Source priority**: SoundCloud stream (full track) > Spotify preview (30s clip) — longer audio = more accurate analysis
- **Confidence threshold**: Analysis results below confidence thresholds are stored but flagged: BPM confidence < 0.7 -> `bpm_confidence = 'low'`, Key confidence < 0.5 -> `key_confidence = 'low'`. Low-confidence results are still used for arrangement but displayed with a warning indicator in the UI
- **Migration addition**: Add `analyzed_at TIMESTAMP`, `analysis_error TEXT`, `needs_analysis BOOLEAN DEFAULT false` to tracks table (part of migration 003 or 004)
- **30-Second Preview Accuracy Limitation**: Spotify preview clips are ~30 seconds. BPM detection from 30s samples has ~85-90% accuracy (vs 95%+ for full tracks). Key detection accuracy drops to ~70-80%. Tracks analyzed from short previews should be flagged with `analysis_confidence = 'low'`. Beatport tracks with native BPM/key data are not affected (they skip analysis entirely).
- **Worker startup health check**: Worker checks essentia sidecar health on startup (`GET /health`). If sidecar is unreachable, worker logs an error and retries every 30 seconds. Worker does not process any tracks until sidecar is confirmed healthy.
- **Queue throughput**: Expected throughput: ~10-15 tracks/minute (download + analysis). A 500-track catalog takes ~30-50 minutes for full analysis. Worker processes tracks in batches of 10, with 5-second cooldown between batches to avoid overwhelming the sidecar.

## Acceptance Criteria (for grading)
- [ ] All success postconditions verified by automated test
- [ ] All extension paths have explicit handling
- [ ] No invariant violations detected
- [ ] Code passes quality gates
- [ ] essentia sidecar service starts and responds to health check
- [ ] BPM detection returns accurate results for test audio files (±2 BPM tolerance)
- [ ] Key detection returns correct key for test audio files
- [ ] Camelot conversion produces correct notation for all 24 keys
- [ ] Background worker processes queued tracks without manual intervention
- [ ] Tracks with no audio URL are gracefully skipped with error recorded
- [ ] Beatport tracks with existing BPM/key are never re-analyzed
- [ ] Temporary audio files are cleaned up after analysis
- [ ] Individual track failures don't crash the worker or block other tracks
- [ ] Frontend transitions from "Pending analysis" to actual BPM/key values
- [ ] Analysis results persist across re-imports (not overwritten)
