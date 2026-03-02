# Spike: SP-001 Probe Beatport v4 API for DJ Track Import

## Hypothesis

Beatport v4 API provides authenticated access to track metadata including BPM, musical key, and genre, with sufficient rate limits for batch import workflows.

## Timebox

- **Maximum Hours**: 4h
- **Start Date**: 2026-03-02
- **Status**: Complete

## Questions to Answer

1. What is the Beatport v4 API authentication flow? (OAuth2, API key, or other?)
2. Does the track search/detail endpoint return BPM and musical key fields natively?
3. What format are BPM and key returned in? (numeric BPM, Camelot notation, Open Key, or raw key name?)
4. What are the rate limits? Can we batch-import 500+ tracks without hitting limits?
5. Is there a playlist/chart endpoint we can import from (similar to Spotify playlist import)?

## Method

- Review Beatport v4 API documentation (developer portal)
- Register for API access if needed
- Use curl to test authentication flow
- Fetch a known track and inspect the full JSON response
- Test rate limits with a burst of 10 rapid requests
- Document the complete data shape of a track response

## Feeds Into

- **ST-002**: Beatport Import steel thread — API client design, auth flow, data mapping (planned)
- **UC-013**: Import Tracks from External Sources — Beatport as additional source
- **UC-015**: Analyze Track Audio Properties — BPM/key from Beatport vs essentia analysis

---

## Findings

### Q1: What is the Beatport v4 API authentication flow? (OAuth2, API key, or other?)
**Answer**: OAuth 2.0 with authorization_code grant type. Requires username/password credentials (not API key). Official client_id/client_secret are not easily obtainable; current workaround uses the public client_id scraped from Beatport's own Swagger UI documentation.
**Evidence**:
- Beatport killed API v3 and currently does not offer traditional OAuth client credential setup for third-party developers
- Multiple production implementations (beets-beatport4, music-assistant) use a workaround: scrape the public `client_id` from https://api.beatport.com/v4/docs/ and authenticate with Beatport username/password via authorization_code grant
- beets-beatport4 supports both username/password flow (stored in config) and token-based auth (manual browser token capture)
- References: [beets-beatport4 GitHub](https://github.com/Samik081/beets-beatport4), [Beatport API access discussion](https://groups.google.com/g/beatport-api/c/3qR1Uj1HnUk)

### Q2: Does the track search/detail endpoint return BPM and musical key fields natively?
**Answer**: YES. Both BPM and musical key are natively returned in track responses. Available as facets/returnFacets for API queries.
**Evidence**:
- Research doc (docs/research/dj-platform-research.md) confirms "BPM, musical key, genre, sub-genre, label, remixer, ISRC" are returned as native fields
- beets plugin implementation shows `data.get("bpm")` and `data.get("key")` are directly accessible from track response
- Multiple sources confirm these are available facets for filtering and return data
- References: [beets-beatport4 implementation](https://github.com/Samik081/beets-beatport4), [fedegiust Beatport API JSON feed](https://github.com/fedegiust/Beatport-API-JSON-feed)

### Q3: What format are BPM and key returned in? (numeric BPM, Camelot notation, Open Key, or raw key name?)
**Answer**:
- **BPM**: Numeric integer (e.g., `128`)
- **Musical Key**: Raw key name notation (e.g., `"A♭ min"`, `"E♭ min"`, `"F major"`). Returned in nested object structure with `"shortName"` property. NOT in Camelot format natively.
**Evidence**:
- beets plugin code parses BPM as string via `data.get("bpm")` and key via nested `data.get("key").get("shortName")`
- Research doc lists track fields as: "bpm" (numeric), "key" (string shortName format)
- Camelot notation (e.g., "1A", "1B") is NOT returned by Beatport API; must be mapped client-side using standard key-to-Camelot lookup table
- Example key mappings per existing Camelot wheel research: F Major → 7A, A♭ min → 4A, E♭ min → 2A, etc.
- References: [GitHub Camelot wheel implementations](https://github.com/regorxxx/Camelot-Wheel-Notation), [beaTunes BeatportTrack API docs](https://www.beatunes.com/apidocs/com/tagtraum/ubermusic/beatport/BeatportTrack.html)

### Q4: What are the rate limits? Can we batch-import 500+ tracks without hitting limits?
**Answer**: **UNKNOWN. No publicly documented rate limits found.** Community discussions and GitHub issues acknowledge rate limiting exists but provide no official specifications. Conservative recommendation: assume standard API rate limits (~100-1000 req/min) and implement exponential backoff + distributed import.
**Evidence**:
- Official API documentation at https://api.beatport.com/v4/docs/ does not publicly disclose rate limit policy
- Multiple searches and GitHub implementations (beets-beatport4, Beatporter, music-assistant discussions) show no explicit rate limit documentation
- Beatporter project README notes "room for better timeout and rate-limit handling" suggesting rate limits are a real constraint
- Community sentiment (Google Groups beatport-api) indicates "Beatport doesn't care about the community" — official support is minimal
- **Recommendation**: Implement request throttling, retry-backoff strategy, and per-user daily quotas. Test with 500-track import in production before assuming safe limits
- References: [Beatport API community discussions](https://groups.google.com/g/beatport-api/), [Beatporter GitHub](https://github.com/rootshellz/Beatporter)

### Q5: Is there a playlist/chart endpoint we can import from (similar to Spotify playlist import)?
**Answer**: **LIKELY YES, but details not publicly documented.** Beatport API likely supports charts and playlists (Beatport has public DJ Charts UI), but exact endpoint paths and parameters are not in accessible documentation. Would require direct API exploration.
**Evidence**:
- Research doc lists track fields including "bpm_range" and mentions API v4 at https://api.beatport.com/v4/docs/ (implying broader endpoint support)
- Beatport publicly hosts DJ Charts at https://www.beatport.com/charts (e.g., genre-specific "top 100" charts)
- beaTunes documentation references Beatport API "Tracks", "Releases", "Playlists", "Charts", "Labels", "Artists" endpoint categories
- Tools like Beatporter and Beatport-Spotify exist that scrape charts, but they use web scraping (Selenium) rather than API — suggests API chart endpoints may not be easily accessible or may have stricter access controls
- **Recommendation**: Post-implementation, directly test `/charts` or similar endpoints using the OAuth credentials. May require explicit API access grant from Beatport for chart data.
- References: [beaTunes Beatport API docs](https://www.beatunes.com/apidocs/com/tagtraum/ubermusic/beatport/Beatport.html), [Beatporter GitHub](https://github.com/rootshellz/Beatporter)

## Decision

- **Hypothesis**: **Partially confirmed**
  - ✅ Confirmed: OAuth2 auth flow exists (though public client_id workaround required)
  - ✅ Confirmed: BPM and key natively returned
  - ✅ Confirmed: BPM numeric, key as shortName string (not Camelot)
  - ⚠ Unknown: Rate limits — no public docs, assume conservative limits
  - ⚠ Unknown: Playlist/chart endpoint details — likely exists but not documented

- **Impact on steel threads**:
  - **ST-001 (Serve Paginated Track Catalog)**: No impact — focuses on SQLite catalog serving, not live imports
  - **ST-002 (Beatport Import)**: MAJOR — requires custom OAuth client_id scraping; rate-limit handling critical; key-to-Camelot mapping client-side. No official dev support.
  - **UC-013+ (Import Tracks)**: Beatport is viable but requires conservative approach; playlist/chart import may not be available without special access

- **Action items**:
  1. **Before implementing ST-002**: Manually test Beatport API v4 with public client_id. Measure actual rate limits (run 500-track burst, observe response headers for rate-limit info).
  2. **Design ST-002 Beatport client**: Implement req throttling (e.g., 5 req/sec max) and exponential backoff. Store client_id in `.env` (not code).
  3. **Key mapping**: Build Camelot wheel lookup table in Rust. Map Beatport shortName → Camelot notation (8A ↔ C major, 8B ↔ A minor, etc.). Cache in DB.
  4. **Playlist import**: Post-MVP. Attempt `/charts` endpoint first; if blocked, escalate to Beatport or use web scraping fallback (Selenium).
  5. **Documentation**: Log public client_id source and OAuth workaround in code comments. Flag for future migration if Beatport opens official API access.
