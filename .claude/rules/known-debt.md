---
paths:
  - "docs/**"
  - "backend/**"
---

# Known Debt

| Item | Source | Priority | Notes |
|------|--------|----------|-------|
| Auto-enrich trigger after import | ST-005 retro #6 | High | Plan called for tokio::spawn after import_playlist(). Fold into ST-006. |
| Retry path for errored tracks | ST-005 retro #7 | Medium | Errored tracks permanently stuck (needs_enrichment=0, enrichment_error set). Need "retry errored" endpoint. |
| Concurrency guard on enrich endpoint | ST-005 critic HIGH-3 | Medium | Two simultaneous POST /api/tracks/enrich calls double-process same tracks. Add AtomicBool or mutex. |
| Claude API error path untested | ST-005 grade | Medium | MockClaude always returns Ok. No test exercises the error variant. |
| Cost cap allows overshoot | ST-005 critic HIGH-2 | Low | Cap checked once before processing; doesn't subtract already-used from fetch limit. |
| API info endpoint has pre-pivot description | Audit | Low | `main.rs` api_info() still says "occasions, African and Middle Eastern traditions" instead of DJ-first. |
| ST-004 retrospective not written | Audit | Low | ST-003 and ST-005 have retros but ST-004 does not. |
