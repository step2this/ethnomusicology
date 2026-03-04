# Audio Source Landscape Research — March 2026

## Purpose
Research findings from prototyping Deezer preview playback and investigating alternatives for DJ track discovery and preview.

## Deezer API — Field-Specific Search (KEY FINDING)

Deezer supports structured queries far beyond our current freeform search:

```
# Current (broken for many tracks):
GET /search?q=Stacey Pullen Throw&limit=1

# Fixed (precise):
GET /search?q=artist:"Paperclip People" track:"Throw"&strict=on&limit=1
```

### Supported field prefixes:
- `artist:"..."` — filter by artist name
- `track:"..."` — filter by track title
- `album:"..."` — filter by album name
- `label:"..."` — filter by record label
- `bpm_min:N` / `bpm_max:N` — BPM range
- `dur_min:N` / `dur_max:N` — duration range (seconds)

### ISRC lookup:
```
GET /track/isrc:{ISRC_CODE}
```
Returns single track with full metadata + preview. Most precise method when ISRC available.

### Rate limits:
- ~50 requests/5 seconds per IP (unofficial)
- Error code 4 (ERR_QUOTA) on exceed
- No auth required for search/read

### Recommended fallback chain:
1. ISRC lookup (if available from Spotify metadata)
2. Field-specific strict (`artist:"X" track:"Y"&strict=on`)
3. Field-specific fuzzy (without strict)
4. Freeform (current behavior, last resort)

## iTunes/Apple Music Search API

- **URL**: `https://itunes.apple.com/search?term={query}&media=music&limit=1`
- **Auth**: None required
- **Preview**: 30s AAC via `previewUrl` field
- **Catalog**: 100M+ tracks
- **Affiliate**: Apple Services Performance Partners — append `?at={token}` to links
- **ToS**: Previews must be "streamed only, not saved" and appear near a store badge
- **Assessment**: Best Deezer fallback. Larger catalog, higher quality AAC.

## SoundCloud API

- **Auth**: OAuth 2.1 (Client Credentials flow)
- **Preview**: `preview_mp3_128_url` field (migrating to AAC HLS Nov 2025)
- **Catalog**: Skews underground/independent — ideal for electronic music
- **Rate limit**: 15,000 stream requests/24 hours
- **Registration**: Via chatbot "Otto" at developers.soundcloud.com
- **Assessment**: Good for underground catalog. Requires OAuth setup. Format migration needs attention.

## Beatport API

- **URL**: `api.beatport.com/v4/docs/`
- **Auth**: OAuth (gated access — must apply)
- **Metadata**: BPM, musical key, genre/subgenre, ISRC, label — richest DJ data
- **Preview**: LOFI MP3 samples available
- **Streaming**: Beatport LINK is closed partnership only (Serato, Traktor, etc.)
- **Assessment**: Apply for access. Best for DJ metadata enrichment and purchase links.

## Spotify — NOT VIABLE for previews

- Preview URLs deprecated November 2024
- New/dev-mode apps get null `preview_url` in responses
- Still useful for: metadata, user library import, auth
- DJ software streaming returned Sept 2025 but closed partnership only

## Purchase Link Strategy

| Store | URL Pattern | Affiliate |
|-------|------------|-----------|
| Beatport | `beatport.com/search?q={query}` | No active program |
| Apple Music | `music.apple.com/search?term={query}` | Yes (Performance Partners) |
| Bandcamp | `bandcamp.com/search?q={query}` | No |
| Traxsource | `traxsource.com/search?term={query}` | No public API |
| Juno | `junodownload.com/search/?q={query}` | Yes (affiliate program) |

## Competitive Landscape

- **Tarab Studio**: Only LLM-powered natural language setlist generator. Unique.
- **KADO**: Data from 200K+ DJ sets for track recommendations. Track-level, not setlist-level.
- **Djoid**: Graph-based track mapping by harmony/energy. Compatibility tool.
- **Engine DJ**: Denon hardware firmware with streaming (Apple Music, Amazon, Tidal, Beatport). Not a software competitor.
- **DJ streaming integrations**: All closed partnerships. No public "DJ streaming API" exists.
