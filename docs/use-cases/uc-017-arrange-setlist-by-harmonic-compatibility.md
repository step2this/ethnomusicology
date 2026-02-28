# Use Case: UC-017 Arrange Setlist by Harmonic Compatibility

## Classification
- **Goal Level**: 🌊 User Goal
- **Scope**: System (black box)
- **Priority**: P0 Critical
- **Complexity**: 🟡 Medium

## Actors
- **Primary Actor**: App User (authenticated, DJ)
- **Supporting Actors**:
  - Database (SQLite/PostgreSQL via SQLx — setlist and track data)
- **Stakeholders & Interests**:
  - DJ User: Wants their setlist reordered so transitions sound smooth — keys are compatible, BPM shifts are gradual, and energy builds naturally. Doesn't want to do this manually with a Camelot chart.
  - Developer: Wants a pure algorithmic module (no external dependencies, no LLM call) that can be tested deterministically

## Conditions
- **Preconditions** (must be true before starting):
  1. User has a generated setlist (from UC-016) with at least 2 tracks
  2. Tracks in the setlist have BPM and Camelot key data (at least partially — tracks missing data are handled gracefully)
  3. Camelot conversion module exists (from UC-013)

- **Success Postconditions** (true when done right):
  1. The setlist tracks are reordered to maximize a weighted transition score: Camelot key compatibility (50%), BPM proximity (30%), energy flow (20%)
  2. Each track-to-track transition has a computed `transition_score` (0.0 to 1.0) stored in the setlist
  3. The overall setlist has an aggregate `harmonic_flow_score` (average of all transition scores)
  4. The original LLM-generated order is preserved as `original_position` so the user can toggle back
  5. Tracks missing BPM or key are placed at the end of the setlist in their original order (not discarded)
  6. The arrangement is persisted (updated setlist_tracks positions in DB)

- **Failure Postconditions** (true when it fails gracefully):
  1. If arrangement cannot improve the score (already optimal or all tracks lack data), the original order is preserved and user is informed: "This setlist is already well-arranged (Harmonic Flow Score: X/100)."
  2. If the algorithm fails (bug), original order is preserved — never leave the setlist in a broken state

- **Invariants** (must remain true throughout):
  1. Arrangement is a pure reordering — no tracks are added or removed
  2. The algorithm is deterministic — same input always produces same output
  3. No LLM calls — this is pure Rust computation
  4. Original order is always recoverable

## Main Success Scenario
1. User views a generated setlist (from UC-016) in the Flutter app
2. User taps "Arrange by Key" (or "Optimize Transitions") button
3. System loads the setlist tracks with their BPM, Camelot key, and energy level data
4. System separates tracks into two groups: those with complete DJ data (BPM + key) and those with incomplete data
5. System applies the arrangement algorithm to the complete-data group:
   a. Compute a transition score matrix for all track pairs using weighted formula:
      - **Key compatibility (50%)**: 1.0 for same Camelot key, 0.9 for compatible (±1 number or same-number different-letter), 0.5 for ±2, 0.0 for incompatible
      - **BPM proximity (30%)**: 1.0 for ±0-2 BPM, 0.8 for ±3-4, 0.6 for ±5-6, 0.3 for ±7-10, 0.0 for >10
      - **Energy flow (20%)**: Energy scoring is position-aware — it evaluates how well each track fits the expected energy arc, not just adjacent track similarity. Default arc: gradual build (positions 1-40% → energy 3-5), peak (40-75% → energy 6-8), cooldown (75-100% → energy 4-6). The energy score for a track at position P is: `1.0 - |actual_energy - expected_energy(P)| / 8.0`
   b. Find the best ordering using a greedy nearest-neighbor heuristic starting from the track with the lowest energy level (opener)
   c. Apply a local optimization pass (2-opt swaps) to improve the greedy solution
6. System appends incomplete-data tracks at the end in their original relative order
7. System computes the overall `harmonic_flow_score` (average of all transition scores, scaled 0-100)
8. System persists the new order to `setlist_tracks` (updated positions) and stores the `harmonic_flow_score` on the setlist
9. System displays the rearranged setlist with:
   - Transition score badges between tracks (green ≥0.8, yellow ≥0.5, red <0.5)
   - BPM flow line showing gradual transitions
   - Camelot key labels on each track
   - Energy arc visualization
   - Overall Harmonic Flow Score (0-100) with component breakdown: Key Compatibility (50%), BPM Continuity (30%), Energy Arc (20%). Example: "Harmonic Flow Score: 82 (Key: 90, BPM: 78, Energy: 70)"
10. User can toggle between "Harmonic Order" and "Original Order" with a single tap

## Extensions (What Can Go Wrong)

- **1a. Setlist has only 1 track**:
  1. "Arrange" button is disabled
  2. Tooltip: "Need at least 2 tracks to arrange"

- **Xa. Setlist has only 2 tracks**:
  1. Arrangement is trivial (only one possible order to evaluate, plus the reverse)
  2. System still computes the Harmonic Flow Score
  3. UI shows: "Only 2 tracks — try adding more for a richer set"

- **3a. Setlist tracks fail to load from database**:
  1. System displays "Couldn't load setlist data. Please try again."
  2. Returns to step 1

- **4a. All tracks are missing BPM and/or key data**:
  1. System displays "These tracks don't have BPM or key data yet. Run audio analysis first, or import from Beatport for instant DJ metadata."
  2. Original order preserved
  3. Harmonic Flow Score: N/A

- **4b. Only 1 track has complete data (rest incomplete)**:
  1. That track becomes the opener, incomplete tracks follow in original order
  2. Harmonic Flow Score reflects only the complete-data portion
  3. Note: "Only 1 of X tracks had key data. Import more analyzed tracks for better arrangement."

- **5a. Multiple tracks share the same Camelot key**:
  1. Algorithm groups same-key tracks and orders by BPM within the group (ascending)
  2. Normal behavior — actually ideal for DJs

- **5b. All tracks are in incompatible keys (worst case)**:
  1. Algorithm still finds the least-bad ordering
  2. Transition scores will be low; Harmonic Flow Score will reflect this
  3. System suggests: "These tracks have challenging key relationships. Consider swapping some tracks for key-compatible alternatives."

- **5c. Algorithm produces a worse score than the original order**:
  1. System keeps the original order
  2. Displays: "The original order is already well-arranged (Harmonic Flow Score: X/100). No changes made."

- **6a. Many incomplete-data tracks (>50% of setlist)**:
  1. System proceeds but warns: "X of Y tracks lack BPM/key data. Arrangement quality is limited."
  2. Suggests: "Analyze missing tracks via the catalog for better results."

- **8a. Database update fails**:
  1. System displays the rearranged setlist in the UI (in memory)
  2. Shows warning: "Arrangement couldn't be saved. Changes will be lost if you leave this screen."
  3. Offers retry button

- **10a. User toggles back to original order**:
  1. System restores `original_position` ordering
  2. Transition scores and visualization update to reflect original order
  3. User can toggle back to harmonic order at any time

## Variations

- **V1. Auto-Arrange**: A future setting allows users to enable automatic arrangement after every setlist generation (always applies UC-017 after UC-016). Off by default.
- **V2. Partial Arrange**: User selects a subset of tracks in the setlist to rearrange, keeping others pinned in place (e.g., "I want this opener and closer, optimize the middle").
- **V3. BPM-Priority Mode**: User selects "Sort by BPM" instead of harmonic arrange — simple ascending BPM sort, useful for tempo-progression sets.
- **V4. Manual Drag-and-Drop with Scoring**: User manually reorders tracks, system updates transition scores in real-time as feedback.

## Agent Execution Notes
- **Verification Command**: `cd backend && cargo test --test harmonic_arrangement`
- **Test File**: `backend/tests/harmonic_arrangement.rs`
- **Depends On**: UC-013 (Camelot module), UC-016 (setlists to arrange)
- **Blocks**: UC-019 (crossfade preview uses arranged order), UC-024 (export uses final arrangement)
- **Estimated Complexity**: M (~2000 tokens implementation budget)
- **Agent Assignment**:
  - Teammate:Backend — Arrangement algorithm (scoring matrix, greedy + 2-opt), Axum endpoint `POST /setlists/{id}/arrange`, DB updates
  - Teammate:Frontend — Arrange button, transition score badges, BPM flow visualization, original/harmonic order toggle, Harmonic Flow Score display

### Key Implementation Details
- **Algorithm**: For setlists with n ≤ 20 tracks, use the Held-Karp dynamic programming algorithm (O(2^n × n^2)) for optimal arrangement. For n > 20, fall back to greedy nearest-neighbor + 2-opt local search (heuristic). The threshold of 20 keeps Held-Karp under ~1 second on modern hardware. O(n²) for scoring matrix, O(n²) for greedy, O(n² × iterations) for 2-opt. Fine for n ≤ 50 tracks.
- **Scoring function**: `score(a, b) = 0.5 * camelot_score(a.key, b.key) + 0.3 * bpm_score(a.bpm, b.bpm) + 0.2 * energy_score(a.energy, b.energy)`
- **Camelot scoring**: Reuse module from UC-013. `same_key → 1.0`, `compatible → 0.9`, `±2 → 0.5`, `else → 0.0`
- **BPM scoring**: `abs(a.bpm - b.bpm)` mapped to score via threshold table
- **Energy scoring**: Position-aware — evaluates how well each track fits the expected energy arc. Default arc: gradual build (positions 1-40% → energy 3-5), peak (40-75% → energy 6-8), cooldown (75-100% → energy 4-6). Score: `1.0 - |actual_energy - expected_energy(P)| / 8.0`
- **Greedy start**: Pick track with lowest energy as opener (natural set opening)
- **2-opt**: Try swapping pairs of tracks, keep swap if total score improves. Max 100 iterations or until no improvement.
- **API endpoint**: `POST /api/setlists/{id}/arrange` → returns rearranged setlist with scores
- **No new migration needed** — uses existing setlist_tracks (adds harmonic_flow_score to setlists table, transition_score to setlist_tracks; can be part of migration 004 with UC-016)

## Acceptance Criteria (for grading)
- [ ] All success postconditions verified by automated test
- [ ] All extension paths have explicit handling
- [ ] No invariant violations detected
- [ ] Code passes quality gates
- [ ] Arrangement algorithm produces deterministic output (same input → same output)
- [ ] Camelot-compatible transitions score higher than incompatible ones
- [ ] BPM transitions are smooth (adjacent tracks within ±6 BPM preferred)
- [ ] Energy flow shows natural arc (gradual build, not random jumps)
- [ ] Tracks missing DJ data are placed at end without crashing algorithm
- [ ] Original order is always recoverable (toggle between original and arranged)
- [ ] Harmonic Flow Score accurately reflects transition quality (0-100 scale) with component breakdown (Key, BPM, Energy)
- [ ] 2-opt optimization improves greedy solution on test cases
- [ ] Held-Karp produces optimal arrangement for setlists with n ≤ 20 tracks
- [ ] Setlist with all same-key tracks scores near 100/100
- [ ] Setlist with all incompatible keys scores low but doesn't crash
- [ ] Frontend displays transition score badges (green/yellow/red) and BPM flow
- [ ] Algorithm completes in <100ms for setlists up to 50 tracks
