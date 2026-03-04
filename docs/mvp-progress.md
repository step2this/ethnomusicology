# MVP Progress: UC Postcondition Matrix

| UC | Postcondition | Status | Covered By |
|----|--------------|--------|-----------|
| UC-015 | BPM/key populated on tracks | ✅ | ST-005 (LLM estimation, not essentia) |
| UC-015 | essentia sidecar | ⬜ | Post-MVP |
| UC-016 | Setlist from prompt | ✅ | ST-003 |
| UC-016 | Energy arc variation | ✅ | ST-006 (4 energy profiles: warm-up, peak-time, journey, steady) |
| UC-016 | BPM transition flagging | ✅ | ST-006 (compute_bpm_warnings, >±6 BPM threshold) |
| UC-016 | <30% catalog warning | ✅ | ST-006 (compute_catalog_warning) |
| UC-016 | Daily usage limits | ✅ | ST-005 (user_usage table + cap logic) |
| UC-016 | Prompt caching | ✅ | ST-006 (cache_control: ephemeral on system prompt) |
| UC-017 | Harmonic arrangement | ✅ | ST-003 |
| UC-017 | Held-Karp for n<=20 | ⬜ | Post-MVP |
| UC-017 | Energy arc parameterized | ✅ | ST-006 (EnergyProfile enum, energy_arc_score_with_profile) |
| UC-018 | mood_tags, enriched_at | ✅ | ST-005 (enriched_at column + update; mood_tags post-MVP) |
| UC-019 | Sequential preview playback | ✅ | Deezer 30s previews + Web Audio API + backend proxy. Crossfade removed (too complex for 30s clips). |
| UC-019 | Transport controls (prev/next/pause) | ✅ | Phase 3: auto-advance, PlaybackStatus enum. PR #4 merged. |
| UC-019 | Playback debugging infrastructure | 🔄 | Per-track Deezer search status indicators + search query tooltips |
| UC-019 | Track attribution links | 🔄 | Clickable title/artist → Google search, Spotify links for catalog tracks |
| UC-019 | Waveform visualization | ⬜ | Deferred — post-MVP polish |
| UC-019 | Purchase link fallback | ⬜ | Deferred — needs UC-020 |
| UC-023 | Multi-turn refinement | ✅ | ST-007 backend + frontend. PR #5 merged. |
| UC-023 | Version history + undo | ✅ | ST-007 backend + frontend. PR #5 merged. |
| UC-023 | >50% change guard | ✅ | ST-007 backend (change_warning field). PR #5 merged. |

Status: ⬜ backlog, 🔄 doing, ✅ done
