# Chore: SoundCloud API Terms Compliance Review

## Context

Before implementing ST-009 (SoundCloud preview integration), we reviewed the [SoundCloud API Terms of Use](https://developers.soundcloud.com/docs/api/terms-of-use). Several requirements affect our architecture and must be addressed in ST-009's design — and some affect the existing codebase too (Apple/Deezer attribution patterns we should follow consistently).

## Terms Analysis

### BLOCKER: AI Input Restriction

> "Copy or reproduce any User Content for the purposes of informing, training, developing (or as input to) artificial intelligence"

**Our risk**: We pass track metadata (title, artist) through our system. However, we use SoundCloud purely as a **preview playback source** — search → play → link back. We do NOT:
- Feed SoundCloud content into LLM prompts
- Use SoundCloud tracks in catalog for generation
- Train models on SoundCloud data

**Decision**: SoundCloud is PLAYBACK-ONLY. Never include SoundCloud metadata in LLM prompts or catalog. The LLM generates track suggestions from its own knowledge; SoundCloud is just one source for hearing those suggestions. This is analogous to how Shazam identifies songs — it uses audio fingerprinting as input but doesn't train on the content.

### BLOCKER: Aggregation Restriction

> "Cannot create competing services aggregating multiple creators' content"

**Our risk**: We aggregate tracks from multiple artists into setlists.

**Decision**: We are a DJ setlist planning tool, not a streaming/discovery platform competing with SoundCloud. We link back to SoundCloud for every track. Our setlists are ephemeral planning tools, not playlists users consume as a listening experience. The 30s preview is for identification, not consumption. This is similar to how Beatport charts or 1001Tracklists aggregate track references.

### MUST-DO: Attribution Requirements

Per the terms, every SoundCloud-sourced track must display:
1. **Uploader credit** — "credits the Uploader as the creator"
2. **SoundCloud source credit** — "credits SoundCloud as the source"
3. **Backlink** — "clearly visible backlinks from the relevant sounds to the URL on soundcloud.com"

### MUST-DO: Session-Only Caching

> "cached content must cease to be available at the end of that session"

Our `PreviewState` is in-memory Riverpod state — cleared on page refresh. This is compliant. We must NOT persist SoundCloud preview URLs to the database (unlike Deezer where we store `deezer_preview_url` in the tracks table for batch enrichment).

### MUST-DO: Branding

- Display as "SoundCloud" (capital S, capital C)
- Cannot use in app name or suggest endorsement
- Cannot modify SoundCloud design assets

### NOT APPLICABLE (for our use case)

- Privacy/GDPR data handling — we don't store SoundCloud user data
- No ad placement — we have no ads
- No content modification — we play audio as-is
- No offline downloads — session-only streaming

## Changes Required

### Update ST-009 Steel Thread
Add compliance section with the decisions above. Ensure acceptance criteria include attribution.

### Update ST-009 Task Decomposition
- T3 (frontend) must include: uploader name display, "via SoundCloud" label, backlink to soundcloud.com URL
- Backend search response must return uploader name + permalink URL from SoundCloud API

### Update Existing Attribution Pattern (Deezer + iTunes)
For consistency, we should follow the same attribution pattern across all sources:
- Deezer tracks: show "via Deezer" + link (we already have external_url)
- iTunes tracks: show "via Apple Music" + link (we already have external_url + Apple icon)
- SoundCloud tracks: show "via SoundCloud" + uploader name + link

This is already partially done — the track tile shows source-specific icons and external URLs. We just need to add the "via [Source]" text label for SoundCloud compliance, and it makes sense to do it for all sources.

### Add Compliance Notes to Known Debt
Document the AI input restriction decision and aggregation risk assessment.

## Task List

### T1: Update ST-009 steel thread with compliance requirements (~10 min, lead direct)
- Add "Compliance" section to ST-009 steel thread
- Update acceptance criteria
- Update task decomposition T3 to include attribution UI

### T2: Update known-debt.md with compliance decisions (~5 min, lead direct)
- Document AI input restriction decision
- Document aggregation risk assessment
- Note session-only caching requirement

### T3: Add "via [Source]" attribution label to track tile (~20 lines, can be part of ST-009)
- When source="soundcloud": show "via SoundCloud" + uploader name + backlink
- When source="itunes": show "via Apple Music" + backlink (Apple ToS also requires badge proximity)
- When source="deezer": show "via Deezer" + backlink
- This is a frontend-only change to `setlist_track_tile.dart`

## Dependency

T1 and T2 are docs-only, do now. T3 ships with ST-009.
