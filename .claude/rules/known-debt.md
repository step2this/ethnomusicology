---
paths:
  - "docs/**"
  - "backend/**"
---

# Known Debt

| Item | Source | Priority | Notes |
|------|--------|----------|-------|
| ~~Auto-enrich trigger after import~~ | ~~ST-005 retro #6~~ | ~~High~~ | ~~Resolved in ST-006 T5~~ |
| Retry path for errored tracks | ST-005 retro #7 | Medium | Errored tracks permanently stuck (needs_enrichment=0, enrichment_error set). Need "retry errored" endpoint. |
| Concurrency guard on enrich endpoint | ST-005 critic HIGH-3 | Medium | Two simultaneous POST /api/tracks/enrich calls double-process same tracks. Add AtomicBool or mutex. |
| Claude API error path untested | ST-005 grade | Medium | MockClaude always returns Ok. No test exercises the error variant. |
| Cost cap allows overshoot | ST-005 critic HIGH-2 | Low | Cap checked once before processing; doesn't subtract already-used from fetch limit. |
| API info endpoint has pre-pivot description | Audit | Low | `main.rs` api_info() still says "occasions, African and Middle Eastern traditions" instead of DJ-first. |
| ST-004 retrospective not written | Audit | Low | ST-003 and ST-005 have retros but ST-004 does not. |
| `build_enhanced_system_prompt` uses string not EnergyProfile enum | ST-006 critic MEDIUM-2 | Medium | Takes `Option<&str>` and matches string literals instead of `Option<&EnergyProfile>`. Works but bypasses compiler enforcement. |
| No HTTP integration test for source_playlist_id filtering | ST-006 critic MEDIUM-3 | Medium | Service-level tests exist but no full HTTP round-trip test for import → generate with source_playlist_id. |
| `score_breakdown` not returned from `get_setlist` | ST-006 critic MEDIUM-4 | Medium | After arrangement, refreshing the page loses score_breakdown (not persisted to DB). Recomputation or new columns needed. |
| Duplicate BPM warning functions | ST-006 critic MEDIUM-5 | Low | `compute_bpm_warnings` (SetlistTrackRow) and `compute_bpm_warnings_from_responses` (SetlistTrackResponse) duplicate logic. Could share a generic helper. |
| Energy profile selector lacks visual mini-curve | ST-006 critic LOW-1 | Low | Plan says "visual mini-curve" but implementation uses text-only ChoiceChips. Functional, UX polish deferred. |
| Daily generation limits not enforced | ST-006 steel thread | Low | user_usage.generation_count column exists but not checked during generation. Explicitly deferred from ST-006. |
