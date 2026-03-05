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
| Duplicate `create_test_pool()` in integration tests | ST-006 retro #5 | High | `tests/setlist_api_test.rs` has its own `create_test_pool()` that diverges from `db/mod.rs`. Bitten twice (ST-005, ST-006). Refactor to single canonical pool builder, `pub` exported for integration tests. |
| `build_enhanced_system_prompt` uses string not EnergyProfile enum | ST-006 critic MEDIUM-2 | Medium | Takes `Option<&str>` and matches string literals instead of `Option<&EnergyProfile>`. Works but bypasses compiler enforcement. |
| No HTTP integration test for source_playlist_id filtering | ST-006 critic MEDIUM-3 | Medium | Service-level tests exist but no full HTTP round-trip test for import → generate with source_playlist_id. |
| `score_breakdown` not returned from `get_setlist` | ST-006 critic MEDIUM-4 | Medium | After arrangement, refreshing the page loses score_breakdown (not persisted to DB). Recomputation or new columns needed. |
| Duplicate BPM warning functions | ST-006 critic MEDIUM-5 | Low | `compute_bpm_warnings` (SetlistTrackRow) and `compute_bpm_warnings_from_responses` (SetlistTrackResponse) duplicate logic. Could share a generic helper. |
| Energy profile selector lacks visual mini-curve | ST-006 critic LOW-1 | Low | Plan says "visual mini-curve" but implementation uses text-only ChoiceChips. Functional, UX polish deferred. |
| Daily generation limits not enforced | ST-006 steel thread | Low | user_usage.generation_count column exists but not checked during generation. Explicitly deferred from ST-006. |
| `CorsLayer::permissive()` in main.rs | AWS deploy plan T1 | Medium | Unconditional permissive CORS. Will be fixed in T1 alongside DEV_MODE and graceful shutdown. |
| No graceful shutdown on SIGTERM | AWS deploy plan review | Medium | `axum::serve` runs bare — `systemctl restart` drops in-flight requests. Wire `with_graceful_shutdown(tokio::signal)` in T1. |
| Migration versioning needed before ALTER TABLE | AWS deploy plan H4 | Medium | Current migrations use `CREATE TABLE IF NOT EXISTS` (re-runs safe). First `ALTER TABLE` migration will require version tracking or it will fail on re-run. |
| `sst-deployer` IAM has AdministratorAccess | AWS deploy plan C1 | High | Must scope down to S3-only before storing any credentials in GitHub Actions secrets. |
| Dead `apply_*` functions in quick_commands.rs | ST-007 critic L1 | Low | `apply_shuffle`, `apply_sort_by_bpm`, `apply_reverse` operate on `SetlistTrackRow` but service uses `VersionTrackRow`. Only called from their own tests. Remove or unify with generic trait. |
| `SortByBpm` always ascending | ST-007 critic L2 | Low | Plan specified `SortByBpm { ascending: bool }` but implemented as always ascending. Add descending option post-MVP. |
| `Timeout`/`ServiceBusy` error variants missing from `RefinementError` | ST-007 critic L3 | Low | Plan listed these variants. Currently `ClaudeError::Timeout` maps to `LlmError`. Clients can't distinguish timeout from other LLM errors. |
| `parent_version_id` not set on LLM-refined versions | ST-007 critic L4 | Low | Only set on reverts. Normal refinements create versions with `parent_version_id: None`, so version lineage chain is incomplete. History still works via version_number ordering. |
| No test for undo-with-only-v0 edge case | ST-007 critic L5 | Low | `handle_quick_command` checks `versions.len() < 2` for undo but no explicit test covers bootstrap → immediate undo. |
| Inconsistent `catch` patterns in Flutter | Frontend critic L1 | Low | 4 catch blocks use `catch (e)` while refactored code uses `on Exception catch (e)`. Functionally fine (broader catch is safer for async). `setlist_input_form.dart:354`, `audio_provider.dart:99,265,295`. |
| Duplicate `MockInterceptor` in provider tests | Frontend critic L2 | Low | `setlist_provider_test.dart` and `track_catalog_provider_test.dart` define local `MockInterceptor` while `mock_api_client.dart` provides a shared one. Unify. |
| `_InitialStateNotifier` test workaround | Frontend critic L3 | Low | `setlist_generation_test.dart` subclasses `SetlistNotifier` to inject initial state, bypassing `build()`. Provider unit tests cover that path. |
| Crossfade removed (intentional) | Playback simplification | Low | Crossfade was too complex for 30s Deezer previews. Can re-add post-MVP when full tracks are available. |
| ~~Deezer search fallback strategies~~ | ~~Playback debugging~~ | ~~Medium~~ | ~~Resolved in ST-008: field-specific search + iTunes fallback.~~ |
| Admin wipe endpoint basic auth | Data cleanup | Low | Token-based auth via `X-Admin-Token` header + `ADMIN_TOKEN` env var. No role-based access control. |
| SoundCloud AI input restriction | Compliance | Medium | SC terms prohibit using content as "input to AI." Decision: SC is playback-only source — metadata MUST NOT enter LLM prompts or catalog. Documented in ST-009 steel thread + chore-soundcloud-compliance.md. Review if we ever want to use SC for track discovery/recommendation. |
| SoundCloud aggregation risk | Compliance | Low | SC terms prohibit "competing services aggregating content." We're a DJ planning tool linking back to SC, not a streaming platform. Analogous to 1001Tracklists or Beatport charts. Monitor if SC's enforcement evolves. |
| Source attribution consistency | Compliance | Low | SoundCloud requires uploader credit + source label + backlink. Apple ToS requires store badge proximity. Deezer has no explicit requirement but we show links for consistency. All three should have "via [Source]" labels — currently only icons. Ships with ST-009. |
| Deploy script `mv -Tf` fix | Infra | Resolved | Fixed `mv -f` → `mv -Tf` in deploy.sh. Root cause: `mv -f` follows symlinks to directories. Both repo and production copies updated. |
| Deezer search fallback (RESOLVED) | Playback | Resolved | Replaced with field-specific search `artist:"X" track:"Y" strict=on` + iTunes fallback (ST-008). Old freeform search was ~20-30% miss rate. |
| ~~confidence not persisted to DB~~ | ~~SP-007 debt~~ | ~~Resolved~~ | ~~Resolved in ST-010: migration 009_verification.sql adds confidence + verification_notes columns.~~ |
| ST-008 retrospective not written | Audit | Low | ST-008 (iTunes preview fallback) completed in a parallel session. No retrospective doc exists at `docs/retrospectives/st-008-*.md`. |
| ST-009 retrospective not written | Audit | Low | ST-009 (SoundCloud preview) completed. No retrospective doc exists at `docs/retrospectives/st-009-*.md`. |
| Prompt caching for verification call | ST-010 critic | Low | `verify_setlist()` makes a second LLM call but does not apply cache_control to the verification_prompt.md content block. Cost overhead on every verified generation. Wire prompt caching in a follow-up. |
