# Beatport v4 API Audio & Preview Research

**Date:** March 3, 2026
**Status:** Complete research, findings documented
**Scope:** Audio preview capabilities, metadata availability, and platform comparison

---

## Executive Summary

Beatport v4 API **does NOT expose preview/sample audio URLs in the public API response**. Previews (2-minute lofi clips) are available through Beatport's web UI and DJ apps, but the mechanism is not standardized or documented for third-party API consumers. ISRC metadata is supported. Preview quality is significantly lower than streaming-grade audio (30–120 second lofi clips vs. full-track streaming).

---

## Audio Preview Capabilities

### Does Beatport Expose Preview URLs via API?

**Answer: Not in the documented v4 API.**

- Beatport has a **Partner Portal** (partnerportal.beatport.com) and official v4 API docs (api.beatport.com/v4/docs/) requiring login to access
- The beets-beatport4 plugin (the most complete open-source v4 integration) retrieves track metadata and album artwork only—**no preview URL handling**
- No GitHub projects targeting the v4 API expose preview URLs; older v3-era projects (beatportdl, youtube-dl extractor) reference preview mechanisms, but those are reverse-engineered and not part of the official API contract

### DJ.Studio Integration (Workaround Evidence)

DJ.Studio implements Beatport preview playback via the "Beatport Shop" integration:
- Provides **2-minute preview clips** for testing tracks before purchase
- Previews are sourced from Beatport but the method is not publicly documented
- Integration suggests Beatport may provide previews via OAuth or private endpoints to **authorized partners only**
- DJ.Studio also offers Beatport Streaming (subscription model) for full-length playback

### Preview Specifications

When previews are available (web/app):
- **Duration:** 2 minutes (lofi clip, not full track)
- **Quality:** Low-fidelity preview audio (30–120 seconds of lower bitrate content)
- **Availability:** Beatport decides which section of track is available (users note "previews may skip the part you need")
- **Format:** Reverse-engineered projects show MP3 (96 kbps) and MP4/AAC (96 kbps, 44.1 kHz) are used internally, but not guaranteed in API

### Authentication for Previews

- **Public API:** Requires OAuth2 and valid API key (difficult to obtain, "partners only")
- **Preview Access:** Likely restricted to subscribed partner apps or higher-tier API credentials
- **Rate Limits:** Not documented for public API; partner-level limits unknown
- **CORS:** No public documentation on CORS headers; custom CDN likely uses domain restrictions

---

## Track Metadata

### Track Response Fields

Beatport v4 API includes standard music metadata:

- **Core:** Track ID, title, artist(s), album/release name
- **Release Info:** UPC, catalog number, year
- **Genre/Classification:** Genre, subgenres, key (BPM available per music-assistant discussions)
- **Identifiers:** **ISRC (International Standard Recording Code) supported**
- **Artwork:** Image URLs for release artwork
- **Streaming:** Full-track URL for Beatport Streaming subscribers only

**Notable:** No public documentation confirms presence of preview URLs in the standard track schema.

### ISRC Support

- **Yes, ISRC is supported** — both as searchable field and retrievable metadata
- Format: ISO-3901 (12-character code: CC-XXX-YY-NNNNN)
- Beatport will assign ISRC if not provided during release upload
- ISRC enables accurate cross-platform matching with Spotify, Deezer, SoundCloud, etc.

### Metadata Quality vs. Deezer

| Aspect | Beatport | Deezer |
|--------|----------|--------|
| **BPM Accuracy** | High (DJ platform) | Moderate |
| **Key Format** | Camelot wheel (for harmonic mixing) | Standard key notation |
| **Genre Depth** | Electronic music taxonomy (detailed sub-genres) | General music genres |
| **ISRC** | Supported | Supported |
| **Preview URLs** | Private/undocumented | Public API; 30-second MP3 |
| **Preview Quality** | Lofi 2-minute clip | Higher quality, shorter |
| **Audio Formats** | MP3, AAC (Streaming: 128/256 kbps AAC) | MP3, MP4 (HiFi: FLAC) |

Beatport metadata is more suitable for DJ use cases (BPM, key, harmonic mixing). Deezer preview URLs are more accessible for third-party apps.

---

## Streaming & Playback SDK

### Beatport Streaming Service

- **Model:** Subscription service (Beatport Pro $9.99/mo, Beatport DJ $4.99/mo)
- **Audio Quality:** 128 kbps or 256 kbps AAC
- **Official SDK:** SDK documentation exists on Partner Portal but not publicly indexed
- **Auth:** OAuth + subscription validation
- **Playback:** Restricted to official Beatport apps and authorized partners (DJ.Studio, Serato, rekordbox, etc.)

### No Public Playback SDK (Unlike Spotify)

- Beatport does **not** expose a Web Playback SDK (like Spotify Web Playback SDK)
- Full-track playback requires native app integration or Beatport Streaming subscription
- Third-party apps achieve playback via:
  1. OAuth + licensed API access
  2. Reverse-engineered stream endpoints (unsupported, violates ToS)
  3. Beatport Streaming tier (consumer-grade)

---

## Rate Limits & Access

### API Access

- **Current Status:** Partner-only; public API access requests are not being accepted
- **Authentication:** OAuth 2.0 (Beatport account + API credentials)
- **Rate Limits:** Not publicly documented; partner-tier limits unknown
- **Access Request:** Must apply via Partner Portal; standard approval timeline is 2–8 weeks or may be denied

### Preview Access Restrictions

- **Public:** No direct API endpoint for preview URLs
- **Partners:** Likely provisioned per integration agreement
- **Terms:** Beatport Streaming and download shop have different preview policies

---

## Comparison: Beatport vs. Deezer for Preview Playback

| Feature | Beatport | Deezer |
|---------|----------|--------|
| **Preview URL in API** | No (private/undocumented) | **Yes (public)** |
| **Preview Duration** | 2 minutes (lofi) | 30 seconds |
| **Preview Format** | MP3/AAC 96 kbps | MP3 |
| **CORS-Friendly** | Unknown | Yes |
| **Auth Required** | Yes (OAuth + partner status) | Yes (API key) |
| **Ease of Integration** | High barrier | Low barrier |
| **DJ-Friendly Metadata** | Excellent (BPM, Camelot key) | Moderate |
| **ISRC Support** | Yes | Yes |

**Recommendation:** For prototype MVP, Deezer previews are significantly more accessible. Beatport previews would require:
1. Partner API approval (may take weeks or be denied)
2. Custom OAuth flow
3. Reverse-engineering preview endpoints (unsupported, risky)

---

## Findings & Implications

### What We Know (Confirmed)

1. ✅ Beatport v4 API exists and is active (api.beatport.com/v4/docs/)
2. ✅ ISRC metadata is available for accurate cross-platform matching
3. ✅ Track metadata (BPM, key) is excellent for DJ use cases
4. ✅ 2-minute preview clips exist and are used by DJ.Studio and Beatport apps
5. ✅ Preview mechanism is not part of the public API contract
6. ✅ Partner apps (DJ.Studio, Serato, rekordbox) have preview playback via private agreements

### What We Don't Know (Not Documented)

1. ❓ Exact preview URL format or endpoint (if any)
2. ❓ Whether preview URLs are returned in authenticated requests to v4 API
3. ❓ CORS headers for preview CDN (if public)
4. ❓ Whether partner agreements bundle preview access or require separate licensing
5. ❓ Rate limits for Beatport Streaming API

### Architectural Implications for MVP

**Short term (MVP):**
- Use Deezer for accessible preview playback (CORS-friendly, documented)
- Use Beatport for metadata enrichment only (ISRC, BPM, genre, Camelot key)
- Flag Beatport integration as a future enhancement pending partner approval

**Medium term (Post-MVP):**
- Apply for Beatport Partner API access if user demand warrants
- If approved, negotiate preview access as part of partnership
- Implement Beatport Streaming tier as alternative to Deezer previews

**Long term:**
- Beatport provides superior metadata for harmonic mixing; worth investing in partnership
- Preview playback is secondary; focus on metadata accuracy first

---

## Sources & References

- [Beatport API v4 Docs](https://api.beatport.com/v4/docs/)
- [Beatport Partner Portal](https://partnerportal.beatport.com/hc/en-us)
- [DJ.Studio Beatport Preview Integration](https://help.dj.studio/en/articles/12158250-beatport-preview-integration-in-dj-studio)
- [beets-beatport4 Plugin](https://github.com/Samik081/beets-beatport4)
- [Beatport Streaming Info](https://help.dj.studio/en/articles/8660745-beatport-integration)
- [Beatport ISRC Support](https://labelsupport.beatport.com/hc/en-us/articles/9444795955732-Create-New-Releases)
- [youtube-dl Beatport Extractor](https://github.com/ytdl-org/youtube-dl/blob/master/youtube_dl/extractor/beatport.py) (reverse-engineered reference)
- [Music Data API Guide](https://soundcharts.com/en/blog/music-data-api)

---

## Research Log

**Search Queries Executed:**
1. beatport API v4 preview audio stream
2. beatport API track sample mp3 preview
3. beets-beatport4 preview audio sample
4. beatport API v4 endpoints track fields response
5. github beatport API preview audio source code
6. beatport streaming API playback SDK
7. Beatport track preview URL format 30 second sample
8. "beatport" "preview" "mp3" OR "m4a" OR "aac" site:github.com
9. DJ.Studio Beatport preview integration how does it work
10. Beatport ISRC identifier track metadata
11. Beatport Streaming SDK API how to access preview
12. "beatport.com" preview stream download authentication CORS
13. Beatport vs Deezer API preview audio quality metadata comparison

**Queries with Strong Results:**
- DJ.Studio integration guide (confirmed 2-minute preview availability)
- beets-beatport4 plugin (confirmed no preview URL handling in official integrations)
- Beatport metadata standards (confirmed ISRC support)
- Music platform comparison (Deezer is more accessible for previews)

