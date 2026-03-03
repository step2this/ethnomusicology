# Task Decomposition: ST-006 Multi-Input Seeding + Enhanced Generation

**Source**: `docs/steel-threads/st-006-multi-input-enhanced-generation.md`
**Review Status**: Devil's advocate review complete on steel thread (3C+5H resolved). Task plan reviewed (3C+5H+5M found, all resolved below).
**Total Tasks**: 12 implementation tasks + 1 wiring task (lead)
**Team**: Lead (coordinator/wirer) + 6 Builders

---

## Dependency Graph

```
Phase 1 (Parallel — no dependencies):
  T1 (migration + DB layer)
  T2 (energy profiles + arrangement)
  T3 (Claude API + prompt engineering)
  T4 (frontend models + API client)

Phase 2 (After their deps):
  T5 (auto-enrich + import linkage) ──── depends on T1
  T6 (generate service extension) ────── depends on T1, T2, T3
  T7 (arrange service extension) ──────── depends on T1, T2

Phase 3 (After Phase 2):
  T8 (route layer: generate/arrange/get) ── depends on T6, T7
  T9 (quality validation) ──────────────── depends on T6

Phase 4 (After Phase 3):
  T10 (frontend provider + state) ──────── depends on T4, T8
  T11 (frontend Create Set screen) ─────── depends on T10

Phase 5 (Lead wiring + final):
  T12 (main.rs wiring) ─────────────────── depends on T3, T5, T8
  T13 (backward compat + integration tests) ── depends on T12
```

**Critical path**: T1 → T6 → T8 → T10 → T11
**Parallel tracks**:
- DB Builder: T1
- Energy Builder: T2 → T7
- Claude Builder: T3
- Frontend Builder: T4 → T10 → T11
- Generation Builder: T6 → T8 → T9
- Import Builder: T5
- Lead: T12 → T13 (wiring + final integration)

---

## Agent Assignment (Non-Overlapping Files)

| Builder | Files Owned | Tasks |
|---------|-------------|-------|
| **db-builder** | `db/migrations/006_*.sql`, `db/mod.rs`, `db/models.rs`, `db/setlists.rs`, `db/imports.rs` | T1 |
| **energy-builder** | `services/camelot.rs`, `services/arrangement.rs` | T2, T7 |
| **claude-builder** | `api/claude.rs` | T3 |
| **generation-builder** | `services/setlist.rs`, `routes/setlist.rs` | T6, T8, T9 |
| **import-builder** | `services/import.rs`, `routes/import.rs` | T5 |
| **frontend-builder** | `frontend/lib/**` (all frontend files) | T4, T10, T11 |
| **Lead** | `main.rs`, `routes/mod.rs`, `services/mod.rs` | T12, T13 |

---

## Tasks

### T1: Migration 006 + DB Layer Extensions

**Module**: `backend/src/db/`
**Covers**: Postconditions 6, 14; Critic #1, #3
**Size**: M | **Risk**: Low | **Agent**: db-builder

**Description**:
Create migration 006 and extend DB operations for ST-006.

**Subtasks**:
1. Create `backend/src/db/migrations/006_import_tracks.sql`:
   ```sql
   CREATE TABLE IF NOT EXISTS import_tracks (
       import_id TEXT NOT NULL REFERENCES spotify_imports(id),
       track_id TEXT NOT NULL REFERENCES tracks(id),
       PRIMARY KEY (import_id, track_id)
   );
   CREATE INDEX IF NOT EXISTS idx_import_tracks_import_id ON import_tracks(import_id);
   CREATE INDEX IF NOT EXISTS idx_import_tracks_track_id ON import_tracks(track_id);
   ```

2. Update `db/models.rs` — Add `energy_profile: Option<String>` to `SetlistRow`. Use explicit column list in `FromRow` derive (not `SELECT *`).

3. Update `db/setlists.rs` — Update `insert_setlist()` to accept and persist `energy_profile`. Update `get_setlist()` / `get_setlist_with_tracks()` to return `energy_profile`. Use explicit column lists.

4. Add to `db/imports.rs`:
   - `insert_import_tracks(pool, import_id, track_ids: &[String])` — bulk insert into junction table
   - `get_tracks_by_import_id(pool, import_id) -> Vec<TrackRow>` — JOIN query for playlist filtering

5. Update `db/mod.rs` — Add migration 006 to `create_test_pool()` so all tests can use the `import_tracks` table. **Without this, every test that touches the DB will fail once any code references `import_tracks`.**

6. Add `insert_import_track_link(import_id, track_id)` to `ImportRepository` trait (keeps abstraction clean for mocking in import tests, avoids mixing trait + raw pool patterns).

**Tests**:
- Migration applies cleanly on fresh DB
- `insert_import_tracks` + `get_tracks_by_import_id` round-trip
- `insert_setlist` with energy_profile persists and retrieves correctly
- `insert_setlist` with `None` energy_profile (backward compat)
- `create_test_pool()` includes migration 006

**Acceptance**: All subtask functions compile and pass tests. Migration idempotent. Test pool includes 006.

---

### T2: EnergyProfile Enum + Parameterized Scoring

**Module**: `backend/src/services/camelot.rs`, `backend/src/services/arrangement.rs`
**Covers**: Postconditions 8-10; Critic #6
**Size**: M | **Risk**: Low | **Agent**: energy-builder

**Description**:
Add `EnergyProfile` enum and parameterized energy arc scoring. This is a pure-function module — ideal for isolated subagent work.

**Subtasks**:
1. Add to `services/camelot.rs`:
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
   #[serde(rename_all = "kebab-case")]
   pub enum EnergyProfile {
       WarmUp,    // 3→7: gentle build
       PeakTime,  // 7→9→7: high energy plateau
       Journey,   // 3→9→4: full arc
       Steady,    // 6→6: consistent energy
   }
   ```

2. Add `energy_arc_score_with_profile(energy: i32, position: usize, total: usize, profile: EnergyProfile) -> f64` that computes ideal energy at each position based on profile target curves:
   - WarmUp: linear 3→7
   - PeakTime: 7→9 at 40%, 9→7 from 60%
   - Journey: 3→9 at 50%, 9→4 from 50%
   - Steady: constant 6

3. Add `EnergyProfile::from_str()` / `TryFrom<&str>` for API parsing.

4. Update `arrange_tracks()` in `services/arrangement.rs` to accept `Option<EnergyProfile>`:
   - If `Some(profile)`, use `energy_arc_score_with_profile()`
   - If `None`, use existing `energy_arc_score()` (backward compat)

**Tests**:
- Each profile produces correct ideal energy at start/middle/end positions
- `energy_arc_score_with_profile` scores match expected curves
- `arrange_tracks` with profile produces different orderings than without
- `arrange_tracks` with `None` produces identical results to current behavior
- `EnergyProfile` serialization: "warm-up" ↔ `WarmUp`, etc.
- `from_str` rejects invalid strings

**Acceptance**: 10+ new tests. All existing arrangement tests pass unchanged.

---

### T3: Claude API Extension + Enhanced System Prompt

**Module**: `backend/src/api/claude.rs`
**Covers**: Postconditions 7, 16-17; Critic #2
**Size**: L | **Risk**: Medium | **Agent**: claude-builder

**Description**:
Extend `ClaudeClientTrait` with a structured content block method for prompt caching, and build the enhanced DJ system prompt.

**IMPORTANT**: The existing `api/claude.rs` already defines a `ContentBlock` struct for *deserializing* responses. The new type for *serializing* request blocks MUST use a different name to avoid collision.

**Subtasks**:
1. Define request content block types (named `RequestContentBlock` to avoid collision with existing response `ContentBlock`):
   ```rust
   #[derive(Debug, Clone, Serialize)]
   #[serde(tag = "type")]
   pub enum RequestContentBlock {
       #[serde(rename = "text")]
       Text { text: String, #[serde(skip_serializing_if = "Option::is_none")] cache_control: Option<CacheControl> },
   }

   #[derive(Debug, Clone, Serialize)]
   pub struct CacheControl {
       #[serde(rename = "type")]
       pub control_type: String, // "ephemeral"
   }
   ```

2. Add new method to `ClaudeClientTrait`:
   ```rust
   async fn generate_with_blocks(
       &self,
       system_blocks: Vec<RequestContentBlock>,
       user_blocks: Vec<RequestContentBlock>,
       model: &str,
       max_tokens: u32,
   ) -> Result<(String, CacheMetrics), ClaudeError>;
   ```
   Where `CacheMetrics` captures `cache_creation_input_tokens` and `cache_read_input_tokens` from the API response.

3. Implement `generate_with_blocks()` on `ClaudeClient` — sends structured `system` array and `content` array to the Messages API. Reuses existing retry logic.

4. **Update ALL MockClaude implementations** — both `MockClaudeClient` in this file and the `MockClaude` in `services/setlist.rs::test_utils` must implement `generate_with_blocks()`. Return canned response + zero cache metrics. **Without this, every test that uses MockClaude will fail to compile.**

5. Keep existing `generate_setlist()` method on the trait — enrichment still depends on it. Do NOT remove it.

6. Build `build_enhanced_system_prompt()` function that returns `Vec<RequestContentBlock>`:
   - Block 1 (cached): DJ expert persona + Camelot rules + transition techniques + output format
   - Block 2 (cached): Catalog context with real BPM/key/energy metadata
   - Energy profile instructions inserted into Block 1 based on profile parameter
   - Creative mode instructions appended when enabled

7. Build `build_enhanced_user_prompt()` that returns `Vec<RequestContentBlock>`:
   - User's prompt text
   - Seed tracklist (if provided)
   - BPM range constraint (if provided)

**Tests**:
- `build_enhanced_system_prompt()` includes energy profile instructions when profile provided
- `build_enhanced_system_prompt()` includes creative mode text when enabled
- `build_enhanced_system_prompt()` includes catalog in second cached block
- `build_enhanced_user_prompt()` includes seed_tracklist when provided
- `build_enhanced_user_prompt()` includes bpm_range constraint when provided
- Content blocks have correct `cache_control` annotations
- `generate_with_blocks()` sends correct JSON structure (unit test with mock HTTP)
- `MockClaudeClient` implements both methods

**Acceptance**: Enhanced prompt includes DJ expertise, energy guidance, Camelot rules. Both trait methods implemented + mocked. 8+ tests.

---

### T4: Frontend Models + API Client Extensions

**Module**: `frontend/lib/models/`, `frontend/lib/services/api_client.dart`
**Covers**: Postconditions 18-24 (data layer); Critic #8
**Size**: M | **Risk**: Low | **Agent**: frontend-builder

**Description**:
Extend frontend data models and API client to support ST-006 request/response shapes. Can start immediately using the API contract from the steel thread doc.

**Subtasks**:
1. Update `models/setlist.dart`:
   - Add `energyProfile: String?` field
   - Add `catalogPercentage: double?` field
   - Add `catalogWarning: String?` field
   - Add `bpmWarnings: List<BpmWarning>` field
   - Add `BpmWarning` class: `{fromPosition, toPosition, bpmDelta}`
   - Update `fromJson()` to parse new fields

2. Update `models/setlist_track.dart`:
   - Add `bpmWarning: bool` computed getter (check if this track position appears in parent's bpmWarnings)

3. Update `services/api_client.dart` — Extend `generateSetlist()`:
   ```dart
   Future<Setlist> generateSetlist(
     String prompt, {
     int? trackCount,
     String? energyProfile,
     String? sourcePlaylistId,
     String? seedTracklist,
     bool? creativeMode,
     double? bpmMin,
     double? bpmMax,
   }) async
   ```
   - Build request body with optional fields (only include non-null)
   - Extend `arrangeSetlist()` to accept optional `energyProfile`

**Tests**:
- `Setlist.fromJson()` parses all new fields
- `Setlist.fromJson()` handles missing new fields (backward compat with old responses)
- `BpmWarning.fromJson()` round-trip
- `ApiClient.generateSetlist()` includes new params in request body when provided
- `ApiClient.generateSetlist()` omits new params when null

**Acceptance**: Models parse extended response. API client sends extended request. All existing model tests pass.

---

### T5: Auto-Enrich Trigger + Import-Track Linkage

**Module**: `backend/src/services/import.rs`, `backend/src/routes/import.rs`
**Covers**: Postconditions 14-15; Critic #9; Retro action item #6
**Size**: M | **Risk**: Medium | **Agent**: import-builder
**Depends on**: T1, T3 (soft — T3 updates MockClaude which import tests use)

**Description**:
After `import_playlist()` completes, fire enrichment in the background via `tokio::spawn`. Also record import-track linkage for playlist filtering.

**Subtasks**:
1. Update `import_playlist()` to call `repo.insert_import_track_link()` (from T1's `ImportRepository` trait extension) after each track upsert — records the import_id ↔ track_id linkage in the junction table via the trait, not raw SQL.

2. Update `ImportState` in `routes/import.rs` to include `claude: Arc<dyn ClaudeClientTrait>`:
   ```rust
   pub struct ImportState {
       pub spotify: SpotifyClient,
       pub repo: Arc<dyn ImportRepository>,
       pub pool: SqlitePool,
       pub encryption_key: [u8; 32],
       pub claude: Arc<dyn ClaudeClientTrait>,
   }
   ```

3. In `import_spotify()` handler, after successful import, spawn background enrichment:
   ```rust
   let pool = state.pool.clone();
   let claude = state.claude.clone();
   let user_id_owned = user_id.to_string(); // Note: user_id is &str, must own it for 'static closure
   tokio::spawn(async move {
       if let Err(e) = enrich_tracks(&pool, claude.as_ref(), &user_id_owned).await {
           tracing::warn!("Auto-enrich after import failed: {e}");
       }
   });
   ```

4. Import response returns immediately — enrichment is fire-and-forget.

**Tests**:
- `import_playlist()` records linkage in `import_tracks` table
- `get_tracks_by_import_id()` returns tracks from that import
- Import handler returns 200 before enrichment completes (test with slow mock Claude)
- Auto-enrich failure doesn't affect import response
- `ImportState` compiles with new `claude` field

**Acceptance**: Import records track linkage. Auto-enrich fires in background. Import response unaffected by enrichment. 5+ tests.

---

### T6: Extended Generate Service

**Module**: `backend/src/services/setlist.rs`
**Covers**: Postconditions 1-7, 11-13; Extensions 3c, 24b; Critic #4, #18
**Size**: XL | **Risk**: High | **Agent**: generation-builder
**Depends on**: T1, T2, T3

**NOTE**: Generation-builder should complete T7 (arrange extension) BEFORE T6, since T7 has fewer deps (T1+T2 only) and is smaller. T6 requires T3 which may take longer.

**Description**:
This is the core service change. Extend `generate_setlist()` to accept all new parameters and implement catalog filtering and quality validation. (Daily generation limits are explicitly deferred per steel thread "Does NOT Prove" section.)

**Subtasks**:
1. Create `GenerateSetlistRequest` struct:
   ```rust
   pub struct GenerateSetlistRequest {
       pub user_id: String,
       pub prompt: String,
       pub track_count: Option<u32>,
       pub energy_profile: Option<EnergyProfile>,
       pub source_playlist_id: Option<String>,
       pub seed_tracklist: Option<String>,
       pub creative_mode: Option<bool>,
       pub bpm_range: Option<BpmRange>,
   }

   pub struct BpmRange {
       pub min: f64,
       pub max: f64,
   }
   ```

2. Refactor `generate_setlist()` to accept `GenerateSetlistRequest` instead of individual params.

3. Add input validation:
   - `energy_profile`: validated by `EnergyProfile` deserialization (from T2)
   - `bpm_range`: min >= 60.0, max <= 200.0, min <= max → 400 `INVALID_BPM_RANGE`
   - `source_playlist_id`: check exists in DB → 404 `PLAYLIST_NOT_FOUND`

4. Catalog filtering: if `source_playlist_id` is set, load tracks via `get_tracks_by_import_id()` (from T1) instead of full catalog. If filtered catalog is empty → 400 `EMPTY_CATALOG`.

5. **Extension 3c**: If `source_playlist_id` playlist has tracks but 0 enriched tracks (all `needs_enrichment=1`), fall back to full catalog with warning in notes: "Source playlist tracks are not yet enriched. Using full catalog."

6. **Extension 24b**: When generating with `source_playlist_id`, check if any tracks have `needs_enrichment=1`. If so, include note: "Some tracks are still being enriched. Generate again after enrichment completes for better results."

7. Switch from `claude.generate_setlist()` to `claude.generate_with_blocks()` (from T3) — use `build_enhanced_system_prompt()` and `build_enhanced_user_prompt()`.

8. Post-generation quality validation:
   - `catalog_percentage`: count tracks with `source == "catalog"` / total × 100
   - `catalog_warning`: if catalog_percentage < 30%, set warning message
   - `bpm_warnings`: scan adjacent tracks, flag where |bpm_delta| > 6.0 → `Vec<BpmWarning>`
   - `seed_match_count`: if seed_tracklist provided, count matches (backend computation, not LLM)

9. Persist `energy_profile` on setlist via updated `insert_setlist()` (from T1).

10. Add new error variants to `SetlistError`:
    - `InvalidEnergyProfile(String)`
    - `InvalidBpmRange(String)`
    - `PlaylistNotFound(String)`

**Tests**:
- Generate with energy_profile stores it on setlist
- Generate with source_playlist_id filters catalog
- Generate with empty filtered catalog → 400 EMPTY_CATALOG
- Generate with nonexistent playlist → 404 PLAYLIST_NOT_FOUND
- Generate with source_playlist_id where 0 tracks enriched → falls back to full catalog with warning (ext 3c)
- Generate with source_playlist_id where some tracks still enriching → includes partial enrichment warning (ext 24b)
- Generate with seed_tracklist passes it to LLM context
- Generate with creative_mode enables creative prompt
- Generate with bpm_range constrains prompt
- Generate with invalid bpm_range → 400 INVALID_BPM_RANGE
- BPM warnings correctly flag >±6 transitions
- Catalog percentage computed correctly
- Catalog warning at <30%
- Generate without new params works identically (backward compat)

**Acceptance**: All new request params validated and functional. Quality validation produces correct warnings. Extension paths 3c and 24b handled. 14+ tests.

---

### T7: Extended Arrange Service

**Module**: `backend/src/services/setlist.rs` (arrange portion)
**Covers**: Postconditions 8-10; Critic #5
**Size**: S | **Risk**: Low | **Agent**: generation-builder (same file as T6)
**Depends on**: T1, T2

**Description**:
Extend `arrange_setlist()` to accept an optional energy profile, falling back to the stored profile on the setlist.

**Subtasks**:
1. Update `arrange_setlist()` signature:
   ```rust
   pub async fn arrange_setlist(
       pool: &SqlitePool,
       id: &str,
       energy_profile: Option<EnergyProfile>,
   ) -> Result<SetlistResponse, SetlistError>
   ```

2. If `energy_profile` is `None`, read from `SetlistRow.energy_profile` (from T1).

3. Pass resolved profile to `arrange_tracks()` (from T2).

4. If no profile anywhere (pre-ST-006 setlists), use `None` → existing default behavior.

**Tests**:
- Arrange with explicit profile uses that profile
- Arrange without profile reads from stored setlist
- Arrange on pre-ST-006 setlist (no stored profile) uses default
- Different profiles produce different arrangements on same data

**Acceptance**: Profile parameter flows through arrangement. Backward compat maintained. 4+ tests.

---

### T8: Extended Route Layer

**Module**: `backend/src/routes/setlist.rs`
**Covers**: Postconditions 1-6, 8, 11-13; Critic #8
**Size**: M | **Risk**: Low | **Agent**: generation-builder (same file as T6)
**Depends on**: T6, T7

**Description**:
Extend request/response structs and route handlers to expose all new parameters.

**Subtasks**:
1. Extend `GenerateRequest`:
   ```rust
   #[derive(Deserialize)]
   pub struct GenerateRequest {
       pub prompt: String,
       pub track_count: Option<u32>,
       #[serde(default)]
       pub energy_profile: Option<String>,
       #[serde(default)]
       pub source_playlist_id: Option<String>,
       #[serde(default)]
       pub seed_tracklist: Option<String>,
       #[serde(default)]
       pub creative_mode: Option<bool>,
       #[serde(default)]
       pub bpm_range: Option<BpmRangeRequest>,
   }

   #[derive(Deserialize)]
   pub struct BpmRangeRequest {
       pub min: f64,
       pub max: f64,
   }
   ```

2. Extend `SetlistResponse`:
   ```rust
   pub struct SetlistResponse {
       // ... existing fields ...
       pub energy_profile: Option<String>,
       pub catalog_percentage: Option<f64>,
       pub catalog_warning: Option<String>,
       pub bpm_warnings: Vec<BpmWarning>,
   }

   #[derive(Serialize)]
   pub struct BpmWarning {
       pub from_position: i32,
       pub to_position: i32,
       pub bpm_delta: f64,
   }
   ```

3. Add `ArrangeRequest`:
   ```rust
   #[derive(Deserialize)]
   pub struct ArrangeRequest {
       #[serde(default)]
       pub energy_profile: Option<String>,
   }
   ```

4. Update `generate_setlist_handler()` to map route request → service request struct.

5. Update `arrange_setlist_handler()` to parse optional body with energy_profile. **IMPORTANT**: Use `Option<Json<ArrangeRequest>>` extractor (not `Json<ArrangeRequest>`) so that clients sending POST with no body (pre-ST-006 behavior) don't get 400 errors. This is required for backward compatibility.

6. Update `get_setlist_handler()` to return energy_profile and compute BPM warnings on-the-fly.

7. Map new `SetlistError` variants to HTTP status codes:
   - `InvalidEnergyProfile` → 400 `INVALID_ENERGY_PROFILE`
   - `InvalidBpmRange` → 400 `INVALID_BPM_RANGE`
   - `PlaylistNotFound` → 404 `PLAYLIST_NOT_FOUND`

**Tests**:
- POST /generate with all new params returns 201
- POST /generate with invalid energy_profile returns 400
- POST /generate with invalid bpm_range returns 400
- POST /generate with nonexistent playlist returns 404
- POST /arrange with energy_profile returns 200
- POST /arrange with empty body (no JSON) returns 200 (backward compat)
- GET /setlist includes energy_profile and bpm_warnings
- POST /generate without new params returns 201 (backward compat)

**Acceptance**: All new params flow through routes. Error codes match steel thread. 7+ tests.

---

### T9: Quality Validation Module

**Module**: `backend/src/services/setlist.rs` (validation functions)
**Covers**: Postconditions 11-13
**Size**: S | **Risk**: Low | **Agent**: generation-builder
**Depends on**: T6

**Description**:
Extract quality validation into testable pure functions.

**Subtasks**:
1. `compute_bpm_warnings(tracks: &[SetlistTrackRow]) -> Vec<BpmWarning>` — scan adjacent pairs, flag |delta| > 6.0
2. `compute_catalog_percentage(tracks: &[SetlistTrackRow]) -> f64` — catalog count / total × 100
3. `compute_catalog_warning(percentage: f64) -> Option<String>` — warning if < 30%
4. `compute_seed_match_count(seed_text: &str, tracks: &[SetlistTrackRow]) -> u32` — fuzzy match seed entries against track titles/artists

**Tests**:
- BPM warnings: no warnings when all deltas ≤ 6
- BPM warnings: correct flag when delta = 8.5
- BPM warnings: handles tracks with missing BPM (skip)
- Catalog percentage: 100% when all catalog, 0% when all suggestions
- Catalog warning triggers at 29%, not at 30%
- Seed match count: exact match, partial match, no match

**Acceptance**: Pure functions with comprehensive edge case tests. 6+ tests.

---

### T10: Frontend Provider + State Extensions

**Module**: `frontend/lib/providers/setlist_provider.dart`
**Covers**: Postconditions 18-24 (state layer)
**Size**: M | **Risk**: Low | **Agent**: frontend-builder
**Depends on**: T4, T8

**Description**:
Extend `SetlistState` and `SetlistNotifier` to handle all new generation parameters and response fields.

**Subtasks**:
1. Extend `SetlistState`:
   ```dart
   class SetlistState {
     // ... existing fields ...
     final String? energyProfile;
     final bool creativeMode;
     final String? sourcePlaylistId;
   }
   ```

2. Extend `SetlistNotifier.generateSetlist()` to pass new params:
   ```dart
   Future<void> generateSetlist(
     String prompt, {
     int? trackCount,
     String? energyProfile,
     String? sourcePlaylistId,
     String? seedTracklist,
     bool? creativeMode,
     double? bpmMin,
     double? bpmMax,
   }) async
   ```

3. Handle new error codes: `INVALID_ENERGY_PROFILE`, `PLAYLIST_NOT_FOUND`, `INVALID_BPM_RANGE`, `EMPTY_CATALOG`.

4. Extend `arrangeSetlist()` to pass energy profile from state.

**Tests**:
- generateSetlist passes energy_profile to API client
- generateSetlist passes source_playlist_id to API client
- State includes new response fields after generation
- Error handling for new error codes

**Acceptance**: Provider passes all new params, handles all new errors. 4+ tests.

---

### T11: Frontend Create Set Screen

**Module**: `frontend/lib/screens/setlist_generation_screen.dart`, new widgets
**Covers**: Postconditions 18-24 (UI layer)
**Size**: XL | **Risk**: Medium | **Agent**: frontend-builder
**Depends on**: T10

**Description**:
Rebuild the setlist generation screen with three input tabs and new controls.

**Subtasks**:
1. **Tab layout**: Replace single-prompt screen with `TabBar` + `TabBarView`:
   - Tab 1: "Describe a Vibe" (existing prompt input)
   - Tab 2: "From Spotify Playlist" (URL input → import → generate)
   - Tab 3: "From Tracklist" (free-text tracklist input)

2. **Energy profile selector**: Row of 4 `ChoiceChip` widgets:
   - Warm-Up, Peak-Time, Journey, Steady
   - Visual mini-curve icon per chip (can be simple text label for now; polish deferred)

3. **Creative mode toggle**: `SwitchListTile` — "Creative Mode: Unexpected but compatible combinations"

4. **Set length slider**: `Slider` widget, 5-30 tracks, default 15. Shows current value.

5. **BPM range inputs** (optional): Two `TextFormField` widgets for min/max BPM. Only shown when user expands "Advanced" section.

6. **Spotify playlist tab**:
   - URL input field with validation
   - "Import & Generate" button
   - Calls import → then generate with `sourcePlaylistId`
   - Shows import progress inline

7. **Tracklist tab**:
   - Multi-line `TextField` for pasting tracklists
   - "Generate from Tracklist" button
   - Passes text as `seedTracklist`

8. **BPM warning badges**: In `SetlistTrackTile`, show warning icon when track position appears in `bpmWarnings`.

9. **Catalog percentage indicator**: Show percentage badge on setlist result card. Warning color if < 30%.

**Tests**:
- Widget test: all three tabs render
- Widget test: energy profile chips are tappable
- Widget test: creative mode toggle changes state
- Widget test: set length slider in range 5-30
- Widget test: BPM warning badge renders for flagged transitions
- Widget test: catalog percentage displays correctly

**Acceptance**: All three input modes work. Controls affect generation params. Warnings display correctly. 6+ tests.

---

### T12: Main.rs Wiring (Lead)

**Module**: `backend/src/main.rs`, `backend/src/routes/mod.rs`
**Covers**: Invariants 1-4; Critic #9
**Size**: S | **Risk**: Low | **Agent**: Lead
**Depends on**: T3, T5, T8

**Description**:
Wire new components into the application. This is integration/wiring code that the lead handles.

**Subtasks**:
1. Add migration 006 to the migration list in `main.rs`.
2. Update `ImportState` construction to include `claude: Arc<dyn ClaudeClientTrait>`.
3. Verify all route state types compile with new fields.
4. Ensure `mod.rs` files export new types.

**Tests**: Compilation + existing tests pass. No new tests needed (covered by T13).

---

### T13: Backward Compatibility + Integration Tests

**Module**: Cross-cutting
**Covers**: Invariant 1 (backward compat); Integration Assertions 1-8
**Size**: M | **Risk**: Medium | **Agent**: Lead (or dedicated test builder)
**Depends on**: T12

**Description**:
Verify backward compatibility and cross-layer integration.

**Subtasks**:
1. Run ALL existing tests unchanged — they must pass without modification (all new params are `Option<T>` with `#[serde(default)]`).
2. Add integration test: generate without new params → response shape matches pre-ST-006.
3. Add integration test: generate with energy_profile → arrange reads stored profile.
4. Add integration test: import → get_tracks_by_import_id returns linked tracks.
5. Verify prompt caching: check that `build_enhanced_system_prompt()` annotates blocks with `cache_control`.

**Tests**: 4 integration tests + full regression suite.

---

## Summary

| Metric | Value |
|--------|-------|
| Total tasks | 13 |
| Phase 1 (parallel) | T1, T2, T3, T4 |
| Phase 2 | T5, T6, T7 |
| Phase 3 | T8, T9 |
| Phase 4 | T10, T11 |
| Phase 5 (lead) | T12, T13 |
| Estimated new tests | 70+ |
| Estimated new/modified files | ~14 backend + ~8 frontend |
| Estimated lines | 1,500-2,500 new/changed |
| Critical path length | 5 phases |
| Max parallelism | 4 builders (Phase 1) |

## Pre-Implementation Checklist

- [ ] Feature branch created: `feature/st-006-enhanced-generation`
- [ ] Task decomposition reviewed (devil's advocate)
- [ ] Agent team plan confirmed (`/agent-team-plan`)
- [ ] API contract finalized (`/api-contract` if needed)
- [ ] All Phase 1 tasks have zero dependencies — can start immediately
- [ ] Builder file ownership confirmed — no overlaps

## Post-Implementation Checklist

- [ ] All quality gates pass: `cargo fmt --check && cargo clippy -- -D warnings && cargo test`
- [ ] Frontend gates pass: `flutter analyze && flutter test`
- [ ] Critic agent review complete (fresh context, reads diff cold)
- [ ] All backward compat tests pass (no existing test modified)
- [ ] `/verify-uc` passes all postconditions
- [ ] `/grade-work` scores ≥ 80%

---

## Devil's Advocate Review Findings (Task Plan)

Review found 3 CRITICAL, 5 HIGH, 5 MEDIUM, 3 LOW. All CRITICAL and HIGH resolved in-place above.

| # | Severity | Finding | Resolution |
|---|----------|---------|------------|
| 3 | CRITICAL | `ContentBlock` name collision — existing response struct vs new request enum | Renamed to `RequestContentBlock` in T3 |
| 12 | CRITICAL | `create_test_pool()` in `db/mod.rs` needs migration 006 — file was unassigned | Added `db/mod.rs` to db-builder ownership; added subtask 5 to T1 |
| 15 | CRITICAL | T6 subtask 8 (daily generation limit) contradicts steel thread "Does NOT Prove" | Removed from T6. Generation limits explicitly deferred. |
| 4 | HIGH | `generate_with_blocks()` trait addition breaks all MockClaude implementations | T3 subtask 4 now requires updating ALL mocks; T5 added T3 as soft dependency |
| 5 | HIGH | T7 and T6 ordering not explicit (same builder, same file) | Added note: generation-builder does T7 before T6 |
| 6 | HIGH | `SetlistRow` energy_profile addition is cross-builder compile dependency | Already handled by T6 → T1 dependency (confirmed) |
| 7 | HIGH | `import_playlist()` mixing trait + raw pool patterns | T1 adds `insert_import_track_link` to `ImportRepository` trait instead |
| 9 | HIGH | `arrange_setlist_handler` adding `Json<ArrangeRequest>` breaks no-body clients | T8 uses `Option<Json<ArrangeRequest>>` extractor |
| 1 | MEDIUM | Extension 3c (0 enriched tracks fallback) not in tasks | Added subtask 5 + test to T6 |
| 2 | MEDIUM | Extension 24b (partial enrichment warning) not in tasks | Added subtask 6 + test to T6 |
| 8 | MEDIUM | T5 pseudocode `user_id.clone()` vs `user_id.to_string()` on `&str` | Fixed to `user_id.to_string()` |
| 10 | MEDIUM | Generation usage DB functions not in any builder's file ownership | Moot — generation limits removed per Finding 15 |
| 11 | LOW | `generate_setlist()` must not be removed from trait (enrichment depends on it) | Added subtask 5 to T3 documenting this |
| 13 | LOW | T9 dependency on T6 unnecessarily strict (pure functions) | No action — same builder serializes naturally |
| 14 | LOW | Error code strings match steel thread | Confirmed correct |
