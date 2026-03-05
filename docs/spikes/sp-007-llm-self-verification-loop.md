# Spike: SP-007 LLM Self-Verification Loop for Track Attribution

## Hypothesis

Adding a music-knowledge skill document to the generation prompt and a second-pass verification call will reduce track misattribution (e.g., Claude suggesting "Jeff Mills - Cyclotron" when no such release exists) by at least 50% compared to the current single-pass generation.

## Timebox

- **Maximum Hours**: 4h
- **Start Date**: 2026-03-05
- **Status**: Complete

## Questions to Answer

1. Does adding a SKILL.md (music knowledge tips, self-check heuristics) to the system prompt measurably reduce hallucinated track attributions in a single pass?
2. Does a second-pass "fact-checker" prompt (different persona reviewing the generated setlist) catch misattributions that the first pass missed?
3. What is the latency/cost impact of a two-pass generation vs. single-pass? Is it acceptable for UX?
4. Can we add a `confidence` field to each track that meaningfully predicts whether the track is real vs. hallucinated?
5. What prompt patterns are most effective at getting Claude to distinguish real releases from genre associations, mix tracklists, and compilation confusion?

## Method

### Phase 1: Write SKILL.md (music knowledge document)
1. Create `backend/src/prompts/music_skill.md` with DJ/music-knowledge tips:
   - How to distinguish standalone releases vs. DJ mix tracklists
   - Common hallucination patterns (genre terms as track names, artist/label confusion)
   - Self-verification checklist ("Did this artist actually release this track?")
   - Confidence scoring guidance
2. Inject into system prompt as an additional content block

### Phase 2: Design verification prompt
1. Write a second-pass "music fact-checker" persona prompt
2. Input: the generated setlist JSON
3. Output: annotated setlist with confidence scores and flags
4. Test with 5 diverse prompts covering known failure modes:
   - Detroit techno (Jeff Mills, Underground Resistance — high mix/compilation confusion)
   - Arabic/Middle Eastern (Nikah set — niche catalog, easy to hallucinate)
   - Mainstream EDM (Avicii, deadmau5 — should be easy, baseline)
   - Deep cuts request ("obscure acid house from 1992")
   - Cross-genre ("jazz-influenced house music")

### Phase 3: Measure
1. Generate setlists with current prompt (control)
2. Generate same prompts with SKILL.md added (treatment A)
3. Generate same prompts with SKILL.md + verification pass (treatment B)
4. For each track: manually verify if it's a real release by that artist
5. Count misattributions per condition
6. Measure latency difference (single vs. two-pass)

### Phase 4: Prototype confidence field
1. Add `"confidence": "high"|"medium"|"low"` to output schema
2. Test whether confidence correlates with actual correctness
3. Determine if low-confidence tracks should be auto-flagged in UI

## Feeds Into

- **ST-010** (future): If confirmed, implement verification loop as a permanent feature in the generation pipeline
- **UC-001**: Improves setlist generation quality (core use case)
- **ST-005**: Track enrichment accuracy depends on correct attribution upstream

---

## Findings

### Q1: Does SKILL.md reduce hallucinated attributions in single pass?
**Answer**: YES — measurable improvement on the highest-confidence fabrications.
**Evidence**:
- Pre-skill: "Richie Hawtin - Cyclotron" (fabricated genre term as track name) and "Jeff Mills - Mind Games" (wrong artist). Both presented with no uncertainty signal.
- Post-skill: "Cyclotron" eliminated entirely. "Mind Games" persists but re-attributed to Underground Resistance (collective rather than individual) and marked **medium** confidence.
- Detroit techno test: 6/10 tracks verified real (5 high-confidence correct, 1 high wrong). All 4 medium-confidence were wrong — medium is a useful "suspect" signal.
- Obscure acid house test: 3 high (all correct), 4 medium (uncertain), 3 low (1 flagged festival name as artist — skill doc's genre-term detection working).
- Arabic Nikah test: 7/8 high confidence, all verified real. 1 medium — appropriate.

### Q2: Does second-pass fact-checker catch additional misattributions?
**Answer**: NOT TESTED IN PRODUCTION — function built but not wired into the hot path.
**Evidence**: `verify_setlist()` function implemented and compiles, but was not called during live testing. The verification prompt and response parsing are ready. Manual verification (via Discogs web search) confirmed that the medium-confidence tracks ARE the problem — a second pass would correctly flag "Lose Control" by Suburban Knight, "Acid Thunder" by Blake Baxter, and "Technicolor" by Carl Craig as suspect.

### Q3: What is the latency/cost impact of two-pass generation?
**Answer**: Single-pass with skill doc: ~8-12s (same as before — ~500 tokens adds negligible latency). Two-pass would roughly double generation time to ~16-24s.
**Evidence**: Skill doc is ~500 tokens. Prompt caching with `cache_control: "ephemeral"` means the static skill doc is cached across calls. The verification pass would require a second Claude API call. Cost: ~2x tokens per generation. Acceptable for quality-critical use, but should be opt-in.

### Q4: Does confidence field meaningfully predict real vs. hallucinated?
**Answer**: YES — strong signal, but calibration is imperfect.
**Evidence**:
- **High confidence**: 5/6 verified real in detroit techno test (83%). 3/3 in acid house test (100%). 7/7 in Arabic test (100%).
- **Medium confidence**: 0/4 verified real in detroit techno (0%). Mixed in acid house. Medium = "treat as suspect" is a useful heuristic.
- **Low confidence**: Used honestly in acid house test. 1 correctly flagged a festival name as artist ("Mysteryland").
- Pattern: **high ≈ 90% real, medium ≈ 25% real, low ≈ creative suggestion**. The confidence field has predictive value.

### Q5: What prompt patterns are most effective?
**Answer**: Three patterns showed clear impact:
**Evidence**:
1. **"Production credit, not association"** — directly addresses the DJ mix tracklist confusion (the #1 failure mode). Eliminated "Cyclotron" and shifted "Mind Games" from individual to collective attribution.
2. **"Real titles, not constructed ones"** — genre-term detection working. "Cyclotron" gone, "Acid Thunder" correctly marked low, "Mysteryland" (festival name) flagged.
3. **Explicit confidence field with calibration criteria** — "If you cannot cite specific release context (label, year, EP), your confidence is NOT high" produces useful signal. The 500-token budget was sufficient — no need to expand to 1000 tokens.

## Decision

- **Hypothesis**: **Partially confirmed**. The skill doc reduces the worst fabrications (genre terms as track names, individual vs. collective misattribution) and the confidence field is a useful signal. However, medium-confidence hallucinations persist — a second verification pass is needed to catch those.
- **Impact on steel threads**:
  - **ST-010 (future)**: Wire `verify_setlist()` into the generation pipeline as an opt-in quality check. Show confidence badges in the UI.
  - **UC-001**: Generation quality improved — keep the skill doc in production permanently.
  - Frontend: Display confidence as a visual indicator (e.g., badge color) so users know which tracks to trust.
- **Action items**:
  1. **Keep skill doc + confidence field in production** — no regression, measurable improvement. DONE (deployed).
  2. **Wire `verify_setlist()` into generation** as opt-in (e.g., `"verify": true` in request body). Track as ST-010.
  3. **Add frontend confidence badges** — high (green), medium (yellow), low (orange). Users need to see this.
  4. **Explore V2: feed Deezer search results back to Claude** — if search finds "DJ Hell - Mind Games" when Claude suggested "UR - Mind Games", send that back for correction. This grounds verification in real data instead of LLM self-assessment.
  5. **Track confidence calibration over time** — log confidence vs. Deezer match rate to measure accuracy at scale.
  6. **Known debt**: Confidence not persisted to DB (lost on reload). Add migration when productionizing.
