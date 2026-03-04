# Spike: SP-006 SoundCloud API Feasibility

## Hypothesis
SoundCloud's API still provides usable audio preview URLs after the Nov 2025 MP3→AAC HLS migration.

## Time Box
2 hours

## Status: PARTIALLY COMPLETE — blocked on app registration

## Questions Answered

### 1. Is `preview_mp3_128_url` still present in track API responses?
**YES — confirmed.** SoundCloud's official blog post ([Moving to Modern Streaming](https://developers.soundcloud.com/blog/api-streaming-urls/)) explicitly states: "`preview_mp3_128_url` will remain available for preview use cases." This field was NOT part of the deprecation. This is our primary audio path.

### 2. What audio fields are available?
Post-deprecation (Dec 31, 2025), the available fields are:
- **`preview_mp3_128_url`** — 30s MP3 preview, NOT deprecated, available for all streamable tracks
- **`hls_aac_160_url`** — full-track AAC HLS stream (new, replacing MP3/Opus)
- **`hls_aac_96_url`** — lower-quality AAC HLS (optional fallback)

**Deprecated/removed** (as of Dec 31, 2025):
- `http_mp3_128_url` — was progressive HTTP MP3 download
- `hls_mp3_128_url` — was HLS MP3 stream
- `hls_opus_64_url` — was HLS Opus stream

Note: The rollout was delayed. As of Nov 2025, AAC HLS was "not on all tracks yet" but actively rolling out ([GitHub issue #466](https://github.com/soundcloud/api/issues/466)).

### 3. Can we get audio from `stream_url` with just client_id or do we need user OAuth?
**Client Credentials flow is sufficient** for public access (search, playback). No user OAuth needed. However, `client_id` as a URL parameter is deprecated since July 2021 — must use `Authorization: OAuth {access_token}` header.

### 4. What format is the audio?
- `preview_mp3_128_url` → direct MP3 file URL (128kbps). No HLS complexity. Web Audio API compatible.
- `hls_aac_160_url` → HLS m3u8 playlist. NOT directly compatible with Web Audio API (needs HLS.js or segment fetching). Too complex for our use case.

### 5. Can our backend proxy handle the audio format?
**YES for `preview_mp3_128_url`** — it's a direct MP3 file, same as Deezer. Our existing proxy handles this perfectly. The `Content-Type` would be `audio/mpeg`. No changes needed to the proxy for this path.

**NO for HLS** — would require HLS segment fetching and reassembly, or an HLS.js client library. Not worth the complexity for MVP.

### 6. What does SoundCloud app registration look like in March 2026?
- Go to `soundcloud.com/you/apps` (requires SoundCloud account login)
- Or use the chatbot "Otto" at developers.soundcloud.com
- Provides `client_id` and `client_secret`
- Client Credentials flow: `POST https://api.soundcloud.com/oauth2/token` with grant_type=client_credentials
- **Registration is manual — requires user action, cannot be automated**

## Additional findings

### Numeric ID → URN migration (April 2025)
SoundCloud is replacing numeric track IDs with string URNs (e.g., `soundcloud:tracks:123456`). New integrations should use URNs. The API still accepts numeric IDs for backward compatibility but this may be removed.

### Rate limits
- 15,000 stream requests per 24 hours
- Standard API rate limiting applies (undocumented exact numbers)

## Success Criteria Evaluation
- [ ] Can acquire OAuth token — **BLOCKED: requires manual app registration by user**
- [x] Can search and find tracks — **CONFIRMED: `/tracks?q={query}` endpoint documented and active**
- [x] Can obtain a streamable audio URL — **CONFIRMED: `preview_mp3_128_url` is NOT deprecated**
- [ ] Audio plays in browser — **NOT TESTED: blocked on credentials**

## Decision

**CONDITIONAL PASS** — proceed with ST-009 design using `preview_mp3_128_url` as the audio path.

### Remaining manual steps before ST-009 implementation:
1. **User must register a SoundCloud app** at `soundcloud.com/you/apps` to obtain `client_id` and `client_secret`
2. Set `SOUNDCLOUD_CLIENT_ID` and `SOUNDCLOUD_CLIENT_SECRET` env vars
3. Verify token acquisition works with actual credentials
4. Test `preview_mp3_128_url` plays in browser (expected to work — it's standard MP3)

### Architecture decision:
- **Use `preview_mp3_128_url`** (NOT HLS) — direct MP3, proxy-compatible, Web Audio compatible
- **Do NOT use `hls_aac_160_url`** — HLS is too complex for MVP (would need HLS.js or server-side segment fetching)
- **Backend proxy works as-is** for MP3 content from SoundCloud CDN
- **Graceful degradation** when credentials not configured — skip SoundCloud in fallback chain

## Sources
- [SoundCloud Moving to Modern Streaming (Blog)](https://developers.soundcloud.com/blog/api-streaming-urls/)
- [GitHub: MP3/Opus Deprecation → AAC HLS (Issue #441)](https://github.com/soundcloud/api/issues/441)
- [GitHub: AAC HLS Not Available (Issue #466)](https://github.com/soundcloud/api/issues/466)
- [SoundCloud API Introduction](https://developers.soundcloud.com/docs/api/introduction)
- [SoundCloud API Guide](https://developers.soundcloud.com/docs/api/guide)
- [SoundCloud API Releases](https://github.com/soundcloud/api/releases)
