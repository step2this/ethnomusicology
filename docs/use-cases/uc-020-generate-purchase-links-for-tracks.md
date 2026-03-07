# Use Case: UC-020 Generate Purchase Links for Tracks

## Classification
- **Goal Level**: Subfunction
- **Scope**: System (black box)
- **Priority**: P1 Important
- **Complexity**: Low

## Actors
- **Primary Actor**: App User (DJ)
- **Supporting Actors**: None (pure URL construction, no external API calls)
- **Stakeholders & Interests**:
  - DJ User: Wants one-tap access to buy tracks from DJ stores — especially LLM suggestions not in their catalog
  - Business: Purchase links drive user value and future affiliate revenue (Beatport, Juno)

## Conditions
- **Preconditions** (must be true before starting):
  1. Track has title and artist fields (from catalog, LLM generation, or saved setlist)

- **Success Postconditions** (true when done right):
  1. Each track displays a purchase link panel with links to up to 4 DJ stores (Beatport, Bandcamp, Juno Download, Traxsource)
  2. Links are search-URL templates constructed from title + artist — no platform IDs required
  3. Links open in a new browser tab via `url_launcher`
  4. Links are computed on-demand at display time (not persisted) — works for fresh and saved setlists
  5. Purchase link panel is visually SEPARATE from source attribution icons (Spotify, SoundCloud, Deezer)
  6. Affiliate tags are appended to Beatport and Juno URLs (when registered)
  7. Store order is consistent: Beatport > Bandcamp > Juno Download > Traxsource

- **Failure Postconditions** (true when it fails gracefully):
  1. If title or artist is empty/null, purchase panel shows "No purchase links available"
  2. Dead links (store removed the track or no results) are handled by the destination store, not us

- **Invariants** (must remain true throughout):
  1. Links are pure URL construction — no API calls, no server-side verification
  2. Links open externally via `url_launcher` (not in-app webview)
  3. Source attribution icons (existing Spotify/SoundCloud/Deezer links) are unchanged

## Main Success Scenario
1. User views a track in a setlist (fresh, saved, or crate)
2. User expands/opens the purchase link panel for that track
3. System constructs search URLs from track title + artist for each store:
   - Beatport: `https://www.beatport.com/search?q={encoded(artist + " " + title)}`
   - Bandcamp: `https://bandcamp.com/search?q={encoded(artist + " " + title)}`
   - Juno Download: `https://www.junodownload.com/search/?q%5Ball%5D%5B0%5D={encoded(artist + " " + title)}`
   - Traxsource: `https://www.traxsource.com/search?term={encoded(artist + " " + title)}`
4. System displays store icons/buttons with store names, each linking to the search URL
5. User taps a store link — opens in new browser tab

## Extensions (What Can Go Wrong)

- **2a. Track has empty/null title or artist**:
  1. If title is present but artist is missing (or vice versa), construct URLs with available field only
  2. If both are empty, show "No purchase links available" message

- **3a. Special characters in title/artist**:
  1. URL-encode all query parameters (handles &, +, /, etc.)
  2. Parenthetical remix info (e.g., "(Villalobos Remix)") included in search query — helps find correct version

- **5a. `url_launcher` fails**:
  1. Copy link to clipboard as fallback
  2. Show toast: "Link copied — open in your browser"

- **5b. Track is very underground / vinyl-only**:
  1. Store search page may return no results — this is expected (SP-009: 30% miss rate even on Beatport/Bandcamp)
  2. No special handling needed — user sees the store's "no results" page
  3. This is the standard pattern used by other music platforms

## Design Notes (from SP-009)

### Source Attribution vs Purchase Links — DIFFERENT concerns
- **Source attribution icons** (existing): Show where we found the track (Spotify logo, SoundCloud logo, Deezer preview indicator). These use platform IDs/URIs. Already implemented.
- **Purchase link panel** (this UC): Show where to BUY the track. Uses search URLs to DJ stores. New feature.
- These MUST be visually distinct — different section, different interaction pattern.

### Store Coverage (SP-009 findings)
- Beatport: 60% hit rate, best DJ metadata (BPM/key in results), affiliate available
- Bandcamp: 70% hit rate, best for underground/independent, no affiliate but highest user value
- Juno Download: Unverified (403 server-side), wide catalog, affiliate available
- Traxsource: Unverified (403 server-side), strong house/disco reputation

### Empty State Design
- Underground tracks (re:ni, DJ Stingray, etc.) may not be on ANY digital store
- The purchase panel always shows all 4 store links — the store's search page handles "not found"
- No server-side hit verification — just provide the search links

## Variations

- **V1. Affiliate Tags**: Append Beatport and Juno affiliate parameters once registered. Implementation: config-driven affiliate tag appended to base URL.
- **V2. Bulk Purchase Links**: User selects entire setlist and gets consolidated list of purchase links grouped by store (post-MVP).
- **V3. Store Preferences**: User can reorder or hide stores based on their preferences (post-MVP).

## Agent Execution Notes
- **Verification Command**: `cd backend && cargo test` then `cd frontend && flutter test`
- **Depends On**: SP-009 (store viability spike — COMPLETE)
- **Blocks**: None (leaf feature)
- **Estimated Complexity**: S-M

### Key Implementation Details
- **Backend endpoint**: `GET /api/purchase-links?title=X&artist=Y` returns ordered list of store links
  - Pure URL construction, no external API calls
  - Response: `{ "links": [{ "store": "beatport", "name": "Beatport", "url": "https://...", "icon": "beatport" }, ...] }`
  - Affiliate tags appended server-side (configurable, not hardcoded in frontend)
- **Frontend**: `PurchaseLinkPanel` widget — expandable section below track info, separate from source attribution icons
- **URL templates** (validated in SP-009):
  ```
  beatport:     https://www.beatport.com/search?q={encoded_query}
  bandcamp:     https://bandcamp.com/search?q={encoded_query}
  juno:         https://www.junodownload.com/search/?q%5Ball%5D%5B0%5D={encoded_query}
  traxsource:   https://www.traxsource.com/search?term={encoded_query}
  ```
- **No migration needed** — URLs computed from title + artist at request time
- **No platform IDs needed** — works for all tracks (catalog, LLM suggestions, saved setlists)

## Acceptance Criteria (for grading)
- [ ] Purchase link panel displays for tracks in setlists
- [ ] All 4 stores (Beatport, Bandcamp, Juno Download, Traxsource) have working search URLs
- [ ] URLs are correctly encoded (special characters, remix info in parens)
- [ ] Links open in external browser via url_launcher
- [ ] Purchase panel is visually separate from source attribution icons
- [ ] Backend endpoint returns ordered store links from title + artist
- [ ] No API calls to external services — pure URL construction
- [ ] Empty/null title or artist handled gracefully
- [ ] Affiliate tag support is configurable (even if not yet registered)
