# Spike: SP-008 Search Hit Rate Fallback Strategies

## Hypothesis

When the primary combined artist+title search fails across Deezer/iTunes/SoundCloud, additional
fallback strategies (artist-only verification, title-only search, freeform combined search, artist
discography lookup) can distinguish "track genuinely absent from catalog" from "track exists but
didn't match due to artist name variation or title formatting". This distinction matters for UX:
a user should know whether a preview is unavailable because the track is hallucinated vs. because
the DJ alias doesn't match the catalog name.

## Timebox

- **Maximum Hours**: 2h (spike only — no implementation)
- **Start Date**: 2026-03-05
- **Status**: Complete

## Questions to Answer

1. Which fallback strategy recovers the most valid previews without introducing false positives?
2. Does `is_acceptable_match()` in `match_scoring.rs` reject valid results that the APIs return?
3. For hallucinated tracks, can artist-only verification confirm "artist real, track fake"?
4. What is the right threshold for each fallback strategy to avoid matching garbage?
5. Is the current Deezer field search (`artist:"X" track:"Y"`) the right primary query, or should it be reordered?

## Method

Run each strategy as live curl commands against Deezer and iTunes APIs across 5 test cases spanning:
hallucinated tracks, real tracks, artist alias issues, obscure real tracks, and common tracks.
Simulate `is_acceptable_match()` scoring in Python to predict what the Rust code would do.

## Test Cases

| # | Query | Status | Notes |
|---|-------|--------|-------|
| TC1 | Quantic - Brownswood Basement | Hallucinated | Artist exists (93 albums), track does not |
| TC2 | Quantic - Time Is The Enemy | Real | Small title variation ("The" vs "the") |
| TC3 | Rhythim Is Rhythim - Strings of Life | Real / Alias | Deezer has both alias and real name |
| TC4 | DJ Hell - Mind Games | Hallucinated | Artist is real, track was fabricated by LLM (see SP-007) |
| TC5 | Goldie - Inner City Life | Real | Easy case, should hit immediately |

---

## Results by Strategy

### Strategy 1: Artist-only search (verify artist exists)

**Deezer**: `GET /search/artist?q={artist}&limit=3`
**iTunes**: `GET /search?term={artist}&entity=musicArtist&limit=3`

| TC | Deezer | iTunes |
|----|--------|--------|
| TC1 Quantic | Found (id=8098, 93 albums, 93k fans) | Found (3 Quantic artists) |
| TC3 Rhythim Is Rhythim | Not tested (alias issue different) | Found ("Rhythim Is Rhythim" artist) |
| TC4 DJ Hell | Found ("DJ Hell") | Found ("DJ Hell") |

**Finding**: Artist-only search reliably confirms whether an artist exists in the catalog. For TC1
and TC4, the artist exists but the track is hallucinated — this is the key signal. If the artist
is real but the track can't be found via any title search, it's a hallucination.

**False positive risk**: Low. Artist search is checking existence, not matching tracks.

---

### Strategy 2: Title-only search

**Deezer**: `GET /search?q={title}&limit=5`
**iTunes**: `GET /search?term={title}&media=music&entity=song&limit=5`

| TC | Deezer results | iTunes results |
|----|----------------|----------------|
| TC1 "Brownswood Basement" | **0 results** | 2 results — BUT: "On the Other Sea (Live Edit from Brownswood Basement)" and "MESTIZX (Brownswood Basement Live)" — parenthetical content |
| TC2 "Time Is The Enemy" | Not tested (field search found it) | Not tested |
| TC3 "Strings of Life" | 5 results including "Strings Of Life - Derrick May" and others | Multiple results |
| TC4 "Mind Games" | "Fortunate - The Game", "Eazy - The Game" (wrong track entirely) | Returns Ava Max, AC/DC (garbage) |
| TC5 "Inner City Life" | Not tested (field search found it) | Not tested |

**Finding for TC1 (hallucinated Brownswood Basement)**:
- iTunes returns 2 tracks that contain "Brownswood Basement" — but both are in parenthetical
  content, e.g. "(Live Edit from Brownswood Basement)".
- `strip_noise()` removes ALL parenthetical content before scoring, so these score 0.0 on title
  similarity against query "Brownswood Basement". **This is correct behavior** — the real track
  title is "On the Other Sea", not "Brownswood Basement". The system correctly rejects these.
- Deezer returns 0 results for "Brownswood Basement" — the track truly does not exist.

**Finding for TC4 ("Mind Games")**:
- Deezer freeform "Mind Games" returns "The Game" tracks (rap artist named The Game). "Game" and
  "Mind Games" share the word "games" — word overlap scoring would give a non-zero score, but the
  artist would fail entirely.
- iTunes returns Ava Max and AC/DC — noise. Zero word overlap.

**Critical finding**: Title-only search produces high noise for short 2-word titles like "Mind
Games". The word "game" is too common. This strategy should NOT be implemented without a minimum
title length or minimum title word overlap requirement.

---

### Strategy 3: Freeform combined search (drop field-specific syntax)

**Deezer**: `GET /search?q={artist}+{title}&limit=5`

| TC | Results |
|----|---------|
| TC1 "Quantic Brownswood Basement" | **0 results** |
| TC4 "DJ Hell Mind Games" | "Fortunate - The Game", "Eazy - The Game" — same garbage as title-only |

**Finding**: Freeform combined search for these non-existent tracks produces either 0 results or
misleading results (The Game discography pollutes "Mind Games" queries). This strategy does not
recover hits that field-specific search misses for hallucinated tracks, as expected.

**Note**: The current implementation already uses freeform as a fallback (Step 2 in `audio_search`
is `deezer_field_search` with `strict=false` which uses the same `artist:"X" track:"Y"` syntax but
without Deezer's strict=on mode). A true freeform without field qualifiers would add noise.

---

### Strategy 4: Artist discography lookup (Deezer artist/top endpoint)

**Deezer**: `GET /artist/{id}/top?limit=5` (requires knowing artist ID first)

For TC1 Quantic (artist id=8098), top tracks include:
- Feeling Good (feat. Alice Russell) — preview=YES
- Cumbia Sobre El Mar — preview=YES
- Time Is My Enemy — preview=YES
- Pelota (Cut a Rug Mix) — preview=YES
- Ojos Vicheros — preview=YES

**Finding**: Once the artist ID is known, we can serve a real track from that artist as a
"representative sample" when the specific requested track doesn't exist. This is useful for
reducing "no preview" states in the UI: instead of nothing, play a real track by the same artist
as a proxy. This is a different use case than finding the exact track.

**Implementation complexity**: Requires a two-step lookup (1. get artist ID from search, 2. fetch
top tracks). Also requires a new API endpoint or DB column to surface the substituted track
prominently — the UI needs to communicate "playing a different track by this artist".

---

## TC3: Alias Issue Deep Dive (Rhythim Is Rhythim)

**Problem**: The LLM generates "Rhythim Is Rhythim - Strings of Life". Deezer has this track
attributed to "Derrick May" (real name), "Rhythim Is Rhythim" (alias), and "Derrick May" with full
credits. iTunes has it as "Derrick May, Mayday & Rhythim Is Rhythim".

**Current code behavior** (field-specific search, limit=5):

Deezer returns these 2 results for `artist:"Rhythim Is Rhythim" track:"Strings of Life"`:
1. "Strings Of Life" — artist: "Derrick May" → `is_acceptable_match("Strings of Life", "Rhythim Is Rhythim", "Strings Of Life", "Derrick May")` → title=1.0, artist=0.0 → **FAIL**
2. "Strings Of Life" — artist: "Rhythim Is Rhythim" → title=1.0, artist=1.0 → **PASS**

The code uses `.find()` to scan all returned results, not just the first — so it would correctly
pick result #2 and succeed.

**iTunes behavior** for the same query:
- First result: "Strings Of Life" — artist: "Derrick May, Mayday & Rhythim Is Rhythim"
- `is_acceptable_match("Strings of Life", "Rhythim Is Rhythim", "Strings Of Life", "Derrick May, Mayday & Rhythim Is Rhythim")`
- Title: 1.0 ✓. Artist: "rhythim is rhythim" is a substring of "derrick may, mayday & rhythim is rhythim" → length ratio = 18/40 = 0.45 → score = 0.8 * 0.45 + 0.4 = **0.76** ✓
- Result: **PASS** — the multi-artist credit string correctly matches.

**Conclusion**: TC3 is NOT a bug. The current code handles this case correctly because:
1. Deezer returns the alias-named entry within the same search result batch
2. iTunes multi-artist credits contain the alias as a substring, which scores 0.76
The alias case is already handled.

---

## TC4: DJ Hell - Mind Games (Hallucinated Track)

This track was documented in SP-007 as a known hallucination. "Mind Games" does not exist in DJ
Hell's catalog on Deezer or iTunes.

**DJ Hell discography on both services includes**: "The Game Changer", "Cold Song 2013", "This Is
for You", "Electronic Germany", "Wonderland", "U Can Dance" — no "Mind Games".

**Verification**: Searched Deezer and iTunes with all strategies. Zero results for "Mind Games"
by "DJ Hell" on any strategy. Title-only "Mind Games" returns noise (The Game discography on
Deezer, unrelated pop on iTunes).

**The correct behavior for TC4 is**: return `source: null, preview_url: null`. The current
system does exactly this. No improvement needed here — the track is hallucinated, so no preview
is the right outcome.

---

## is_acceptable_match() Analysis

**Threshold**: 0.5 for both title and artist.

**Cases tested**:

| Query | Result | Title score | Artist score | Verdict | Correct? |
|-------|--------|-------------|--------------|---------|----------|
| "Strings of Life" / "Rhythim Is Rhythim" | "Strings Of Life" / "Derrick May" | 1.0 | 0.0 | FAIL | Yes — different attribution |
| "Strings of Life" / "Rhythim Is Rhythim" | "Strings Of Life" / "Derrick May, Mayday & Rhythim Is Rhythim" | 1.0 | 0.76 | PASS | Yes — alias in credits |
| "Strings of Life" / "Rhythim Is Rhythim" | "Strings Of Life" / "Rhythim Is Rhythim" | 1.0 | 1.0 | PASS | Yes |
| "Mind Games" / "DJ Hell" | "Tragic Picture Show" / "DJ Hell" | 0.0 | 1.0 | FAIL | Yes — wrong track |
| "Brownswood Basement" / "Quantic" | "On the Other Sea" / "Jeremiah Chiu" | 0.0 | 0.0 | FAIL | Yes — hallucinated |
| "Time Is The Enemy" / "Quantic" | "Time Is the Enemy" / "Quantic" | 1.0 | 1.0 | PASS | Yes — case difference handled |

**strip_noise() edge case found**: For iTunes results like "On the Other Sea (Live Edit from
Brownswood Basement)", `strip_noise()` removes the parenthetical, leaving "on the other sea".
This means "Brownswood Basement" is stripped out before comparison — the function correctly rejects
it as a title match even though the query term appears in the result. **This is the correct
behavior.**

**Known weakness**: The artist similarity function treats artist credit strings like "Derrick May,
Mayday & Rhythim Is Rhythim" as single strings. The substring check happens to work here because
"rhythim is rhythim" appears verbatim in the credit string. But if the alias had different word
order or punctuation, it would fall through to word overlap scoring. This is fragile but works in
practice for comma-separated credit lines.

**No changes needed to `is_acceptable_match()`** for the test cases examined. The 0.5 threshold
is appropriate.

---

## Summary: Which Fallbacks Are Worth Building?

### Strategy Assessment

| Strategy | TC1 Hallucinated | TC2 Real | TC3 Alias | TC4 Hallucinated | TC5 Easy | Worth Building? |
|----------|-----------------|----------|-----------|-----------------|----------|-----------------|
| Artist-only (existence check) | ✓ confirms artist real | N/A | N/A | ✓ confirms artist real | N/A | **YES** — diagnostic value |
| Title-only | False negatives (strip_noise saves us), noise on TC4 | N/A | N/A | Noise (The Game) | N/A | **NO** — too noisy |
| Freeform combined | 0 results | N/A | N/A | Noise | N/A | **NO** — already exists as fuzzy mode |
| Artist top tracks | Gets real tracks by artist | N/A | N/A | Gets real tracks by artist | N/A | **YES** — for "representative sample" UX |

### Recommended Fallback Chain (with additions)

The **current chain** is:
1. Deezer field search (strict=on)
2. Deezer field search (strict=off/fuzzy)
3. iTunes combined
4. SoundCloud combined
5. No preview

**Proposed addition** (after Step 4 fails):

**Step 5: Artist existence verification** — if all preview sources return no match, call
`GET /search/artist?q={artist}&limit=3` on Deezer and score the top result. If `artist_similarity`
score ≥ 0.7 (artist is real and known), set a flag `artist_verified: true` in the response. This
allows the frontend to show a differentiated message:
- Artist verified + track not found → "Preview not available — this track may not have a digital release"
- Artist not found → "Track not found — check artist name" (hallucination signal)

**Step 6 (optional, UI-level decision):** If `artist_verified: true` and the Deezer artist ID is
known, fetch `GET /artist/{id}/top?limit=1` and surface the top track as a "representative sample"
— play a preview of a real song by the same artist as a substitute. The UI must clearly label this
as "Similar by {artist}" not the requested track.

### Explicit Non-Recommendations

- **Do not add title-only fallback**: The noise ratio is too high. "Mind Games" returns The Game
  discography on Deezer. For 2-word generic titles this produces garbage.
- **Do not add freeform (no field syntax)**: The current fuzzy Deezer search already covers this.
  True freeform (dropping field qualifiers) would match "game" in "Mind Games" to "The Game". The
  field syntax acts as a noise filter.

---

## Code Impact

### Files to change for artist verification:

- `backend/src/routes/audio.rs` — add `artist_verified: bool` to `AudioSearchResponse`, add
  `deezer_artist_id: Option<u64>` for top-tracks fallback, add `deezer_artist_verify()` helper
- `backend/src/services/match_scoring.rs` — no changes needed
- `backend/src/services/soundcloud.rs` — no changes needed
- Frontend: `lib/models/audio_search_result.dart` — add `artistVerified` field, update tile UI

### Estimated effort:

- Backend artist verification endpoint: ~1 day
- Frontend badge/message differentiation: ~0.5 days
- Artist top-tracks "representative sample" feature: ~1.5 days additional

---

## Decision

**Hypothesis**: **Partially confirmed**. The four strategies tested have very different value:
- Artist existence check: high value, low cost
- Title-only: low value, high noise risk
- Freeform combined: no additional value (already covered)
- Artist discography: medium value, medium complexity

**Immediate action items**:
1. **Add `artist_verified` flag to `AudioSearchResponse`** — requires adding a Deezer
   `/search/artist` call after all preview sources fail. Low cost, high UX value. Track as ST-010
   or a standalone task.
2. **No changes to `is_acceptable_match()`** — the function handles all test cases correctly
   including the alias case (TC3) and correctly rejects hallucinated tracks (TC1, TC4).
3. **Defer representative sample feature** (artist top tracks) — requires UI design decisions about
   how to communicate "this is not the track you asked for". Track in backlog.
4. **Feeds into SP-007 action item #4**: "Feed Deezer search results back to Claude" — the artist
   verification data (artist_verified + known tracks from that artist) could be used by a second-
   pass LLM call to self-correct the hallucinated track title. This is the highest-leverage
   improvement but requires both SP-008 (artist verification) and SP-007 (verification loop).

## Feeds Into

- **ST-010** (LLM verification loop): Artist verification data enriches the LLM self-correction
  prompt — "We searched Deezer and found no track 'Mind Games' by DJ Hell. Their real tracks are:
  The Game Changer, Cold Song 2013, Wonderland. Please substitute the most appropriate real track."
- **UC-001**: Better no-preview states with explanatory messaging reduce user confusion.
- **MEMORY.md**: Note that `is_acceptable_match()` handles alias cases (TC3) and is not the cause
  of search failures. The real problem is hallucinated track names, not scoring.
