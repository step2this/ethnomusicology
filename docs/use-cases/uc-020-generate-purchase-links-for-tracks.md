# Use Case: UC-020 Generate Purchase Links for Tracks

## Classification
- **Goal Level**: 🐟 Subfunction
- **Scope**: System (black box)
- **Priority**: P1 Important
- **Complexity**: 🟢 Low

## Actors
- **Primary Actor**: App User (authenticated, DJ)
- **Supporting Actors**:
  - Database (track metadata with platform IDs)
- **Stakeholders & Interests**:
  - DJ User: Wants one-tap access to buy/stream tracks — especially suggestions from UC-016 that aren't in their catalog yet
  - Business: Purchase links drive user value and potential future affiliate revenue

## Conditions
- **Preconditions** (must be true before starting):
  1. Track exists in database with at least one platform identifier (beatport_id, soundcloud_urn, spotify_uri) OR track is an LLM suggestion with title + artist

- **Success Postconditions** (true when done right):
  1. Each track displays clickable links to its source platforms (Beatport, SoundCloud, Spotify)
  2. Links open in a new browser tab to the correct track/release page
  3. LLM-suggested tracks (not in catalog) have search-based links constructed from title + artist
  4. Links are generated at display time (not stored) — always fresh URLs

- **Failure Postconditions** (true when it fails gracefully):
  1. If a track has no platform identifiers, show "Search on Beatport" with a search query link
  2. Dead links (platform removed the track) are handled by the destination platform, not us

- **Invariants** (must remain true throughout):
  1. Links are direct URL construction — no API calls needed
  2. Links open externally via `url_launcher` (not in-app webview)

## Main Success Scenario
1. User views a track in their catalog or a setlist
2. System checks the track's platform identifiers (beatport_id, soundcloud_urn, spotify_uri)
3. For each available identifier, system constructs a direct URL:
   - Beatport: `https://www.beatport.com/track/{slug}/{beatport_id}`
   - SoundCloud: `https://soundcloud.com/{permalink}` (stored as permalink_url)
   - Spotify: `https://open.spotify.com/track/{spotify_id}` (extracted from URI)
4. System displays platform icons/buttons next to the track, each linking to the respective platform
5. User taps a platform icon — link opens in a new browser tab via `url_launcher`

## Extensions (What Can Go Wrong)

- **2a. Track has no platform identifiers (manually added or data gap)**:
  1. System generates search-based fallback links:
     - Beatport: `https://www.beatport.com/search?q={url_encode(artist + " " + title)}`
     - SoundCloud: `https://soundcloud.com/search?q={url_encode(artist + " " + title)}`
     - Spotify: `https://open.spotify.com/search/{url_encode(artist + " " + title)}`
  2. Links labeled "Search on [Platform]" instead of direct links

- **2b. Track is an LLM suggestion (from UC-016, not in catalog)**:
  1. System constructs search-based links using title + artist from LLM response
  2. If LLM provided `find_on` field (e.g., "beatport"), that platform is shown first
  3. All three platform search links shown as options

- **3a. Beatport track slug is unknown (only ID stored)**:
  1. Construct a search URL instead: `https://www.beatport.com/search?q={url_encode(title + " " + artist)}`
  2. Note: Beatport track URLs follow the pattern `https://www.beatport.com/track/{slug}/{beatport_id}`. The `slug` must be stored during import (UC-013) from the API's `slug` field. If slug is missing at display time, fall back to the search URL above.

- **5a. `url_launcher` fails (platform not supported)**:
  1. Copy link to clipboard as fallback
  2. Show toast: "Link copied — open in your browser"

## Variations

- **V1. Bulk Purchase Links**: User selects multiple tracks (or entire setlist) and gets a consolidated list of purchase links, grouped by platform.
- **V2. Price Display**: If Beatport API provides pricing, show price alongside link (future enhancement, requires API call).
- **V3. "Buy This Setlist"**: Aggregate all suggestion tracks from a setlist into a shopping list with links.

## Agent Execution Notes
- **Verification Command**: `cd backend && cargo test --test purchase_links && cd ../frontend && flutter test test/purchase_links_test.dart`
- **Test File**: `backend/tests/purchase_links.rs`, `frontend/test/purchase_links_test.dart`
- **Depends On**: UC-001 (Spotify URIs), UC-013 (Beatport IDs), UC-014 (SoundCloud URNs/permalinks), UC-016 (suggestion tracks)
- **Blocks**: None (leaf feature)
- **Estimated Complexity**: S (~800 tokens implementation budget)
- **Agent Assignment**:
  - Teammate:Backend — URL construction utility (pure function, no API calls), expose via track metadata endpoint
  - Teammate:Frontend — Platform icon buttons, url_launcher integration, search fallback for suggestions

### Key Implementation Details
- **URL templates** (pure string construction):
  ```
  beatport:    https://www.beatport.com/track/{beatport_slug}/{beatport_id}
  soundcloud:  {permalink_url}  (stored directly)
  spotify:     https://open.spotify.com/track/{id}  (extracted from spotify:track:{id})
  search:      https://www.beatport.com/search?q={encoded_query}
  ```
- **Frontend**: `url_launcher` package (already used in UC-001), `launchUrl()` with `LaunchMode.externalApplication`
- **No migration needed** — uses existing platform ID columns (UC-013 migration must include `beatport_slug` column)
- **No API calls** — pure URL construction from stored metadata
- **Beatport slug**: UC-013 import must store the Beatport API's `slug` field alongside `beatport_id` to enable direct track URLs

## Acceptance Criteria (for grading)
- [ ] Catalog tracks with platform IDs generate correct direct links
- [ ] LLM suggestion tracks generate search-based links
- [ ] Links open in external browser via url_launcher
- [ ] All three platforms (Beatport, SoundCloud, Spotify) supported
- [ ] Tracks with no identifiers fall back to search links
- [ ] URL encoding handles special characters in track/artist names
- [ ] No API calls required for link generation
