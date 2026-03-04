# Spike: SP-006 SoundCloud API Feasibility

## Hypothesis
SoundCloud's API still provides usable audio preview URLs after the Nov 2025 MP3→AAC HLS migration.

## Time Box
2 hours

## Questions to Answer
1. Is `preview_mp3_128_url` still present in track API responses?
2. If not, what audio fields are available? (`stream_url`, `hls_aac_160_url`, other?)
3. Can we get audio from `stream_url` with just client_id or do we need user OAuth?
4. What format is the audio? (MP3, AAC, HLS m3u8?)
5. Can our backend proxy handle the audio format? (Web Audio API compatibility)
6. What does SoundCloud app registration look like in March 2026?

## Method
1. Register SoundCloud OAuth app at developers.soundcloud.com (chatbot "Otto")
2. Acquire Client Credentials token
3. Search for a known underground track (e.g., "Throw" by Paperclip People)
4. Inspect full JSON response — document all audio-related fields
5. Try fetching audio from each available URL
6. Test playback in browser via backend proxy

## Success Criteria
- [ ] Can acquire OAuth token
- [ ] Can search and find tracks
- [ ] Can obtain a streamable audio URL (any format)
- [ ] Audio plays in browser (directly or via proxy)

## Failure Criteria
- OAuth registration blocked/unavailable → SoundCloud integration not feasible
- No audio URLs in response → preview not possible
- HLS-only with no direct stream → requires server-side transcoding (too complex for MVP)

## Decision
**If pass**: Proceed with ST-009 implementation using confirmed audio delivery path.
**If fail**: Defer SoundCloud. Rely on Deezer + iTunes. Revisit when SoundCloud API stabilizes.
