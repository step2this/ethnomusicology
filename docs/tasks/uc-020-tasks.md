# UC-020 Task Decomposition: Purchase Links for Tracks

## Dependency Graph

```
T1 (backend service) ──► T3 (backend route + wiring) ──► T5 (backend tests)
                                    │
T2 (config: affiliate tags) ──► T3  │
                                    ▼
                         T4 (frontend: ApiClient method)
                                    │
                                    ▼
                         T6 (frontend: PurchaseLinkPanel widget)
                                    │
                                    ▼
                         T7 (frontend: wire into SetlistTrackTile)
                                    │
                                    ▼
                         T8 (frontend tests)
```

## Architecture Decisions

- **No migration needed** — URLs computed from title + artist at request time
- **No external API calls** — pure URL construction (~1ms server-side)
- **Lazy-loaded** — frontend calls endpoint only when user expands the panel
- **Affiliate tags server-side** — env vars in `/etc/ethnomusicology/env`, not hardcoded in frontend
- **Panel collapsed by default** — avoids visual clutter, 1 API call per expand action

---

## Tasks

---

### T1: Purchase link service — URL construction logic
**Module**: `backend/src/services/purchase_links.rs` (new)
**Size**: S | **Risk**: Low | **Agent**: Backend Builder
**Depends on**: Nothing
**Files**: `backend/src/services/purchase_links.rs` (new), `backend/src/services/mod.rs` (edit)

Create a service module that constructs purchase search URLs from title + artist.

1. Define response structs:
   ```rust
   #[derive(Debug, Serialize)]
   pub struct PurchaseLink {
       pub store: String,       // "beatport", "bandcamp", "juno", "traxsource"
       pub name: String,        // "Beatport", "Bandcamp", "Juno Download", "Traxsource"
       pub url: String,         // Full search URL
       pub icon: String,        // Same as store slug — frontend maps to icons
   }

   #[derive(Debug, Serialize)]
   pub struct PurchaseLinkResponse {
       pub links: Vec<PurchaseLink>,
   }
   ```

2. Implement `build_purchase_links(title: &str, artist: &str, affiliate_config: &AffiliateConfig) -> PurchaseLinkResponse`:
   - URL-encode `{artist} {title}` as the query string
   - Construct URLs using validated templates from SP-009:
     - Beatport: `https://www.beatport.com/search?q={encoded_query}`
     - Bandcamp: `https://bandcamp.com/search?q={encoded_query}`
     - Juno Download: `https://www.junodownload.com/search/?q%5Ball%5D%5B0%5D={encoded_query}`
     - Traxsource: `https://www.traxsource.com/search?term={encoded_query}`
   - Append affiliate tags from config if present (e.g., `&affiliate_id=XXX`)
   - Return links in fixed order: Beatport > Bandcamp > Juno > Traxsource
   - If both title and artist are empty, return empty links vec

3. Handle partial input: if only title OR only artist is provided, use whichever is available as the query

4. Add `pub mod purchase_links;` to `services/mod.rs`

**Acceptance**:
- Unit tests for: full title+artist, title-only, artist-only, both-empty, special characters (parentheses, &, +)
- Store order is always Beatport > Bandcamp > Juno > Traxsource
- URL encoding handles remix parenthetical info correctly

---

### T2: Affiliate tag configuration
**Module**: `backend/src/services/purchase_links.rs` (same file as T1)
**Size**: XS | **Risk**: Low | **Agent**: Backend Builder (same as T1)
**Depends on**: Nothing (can be done alongside T1)
**Files**: `backend/src/services/purchase_links.rs`

1. Define config struct:
   ```rust
   #[derive(Debug, Clone, Default)]
   pub struct AffiliateConfig {
       pub beatport_affiliate_id: Option<String>,
       pub juno_affiliate_id: Option<String>,
   }
   ```

2. Implement `AffiliateConfig::from_env()` — reads `BEATPORT_AFFILIATE_ID` and `JUNO_AFFILIATE_ID` from environment

3. In `build_purchase_links`, append affiliate params to Beatport and Juno URLs when config values are present

**Acceptance**:
- Unit test: with affiliate IDs set, URLs include affiliate params
- Unit test: without affiliate IDs, URLs have no extra params
- No compile error if env vars are unset

**NOTE**: T1 and T2 can be done by the same builder in one pass since they share the same file.

---

### T3: Purchase links route + wiring into main.rs
**Module**: `backend/src/routes/purchase_links.rs` (new), `backend/src/main.rs` (edit)
**Size**: S | **Risk**: Low | **Agent**: Backend Builder
**Depends on**: T1, T2
**Files**: `backend/src/routes/purchase_links.rs` (new), `backend/src/routes/mod.rs` (edit), `backend/src/main.rs` (edit)

1. Create route handler:
   ```rust
   #[derive(Deserialize)]
   pub struct PurchaseLinkQuery {
       pub title: Option<String>,
       pub artist: Option<String>,
   }

   pub async fn get_purchase_links(
       State(state): State<Arc<PurchaseLinkRouteState>>,
       Query(params): Query<PurchaseLinkQuery>,
   ) -> Json<PurchaseLinkResponse> { ... }
   ```

2. Define `PurchaseLinkRouteState`:
   ```rust
   pub struct PurchaseLinkRouteState {
       pub affiliate_config: AffiliateConfig,
   }
   ```

3. Create router function:
   ```rust
   pub fn purchase_link_router(state: Arc<PurchaseLinkRouteState>) -> Router {
       Router::new()
           .route("/purchase-links", get(get_purchase_links))
           .with_state(state)
   }
   ```

4. Wire into `main.rs`:
   - Load `AffiliateConfig::from_env()` during startup
   - Create `PurchaseLinkRouteState`
   - Add `.nest("/api", routes::purchase_links::purchase_link_router(purchase_link_state))`

5. Add `pub mod purchase_links;` to `routes/mod.rs`

**Acceptance**:
- `GET /api/purchase-links?title=Strings+of+Life&artist=Derrick+May` returns 4 store links
- `GET /api/purchase-links` with no params returns `{"links": []}`
- Response is JSON with correct content-type

---

### T4: Frontend ApiClient method for purchase links
**Module**: `frontend/lib/services/api_client.dart` (edit)
**Size**: XS | **Risk**: Low | **Agent**: Frontend Builder
**Depends on**: T3 (needs endpoint to exist)
**Files**: `frontend/lib/services/api_client.dart` (edit)

1. Add a `PurchaseLink` model class (can be in a new file `frontend/lib/models/purchase_link.dart` or inline):
   ```dart
   class PurchaseLink {
     final String store;
     final String name;
     final String url;
     final String icon;

     PurchaseLink({required this.store, required this.name, required this.url, required this.icon});

     factory PurchaseLink.fromJson(Map<String, dynamic> json) => PurchaseLink(
       store: json['store'] as String,
       name: json['name'] as String,
       url: json['url'] as String,
       icon: json['icon'] as String,
     );
   }
   ```

2. Add method to `ApiClient`:
   ```dart
   Future<List<PurchaseLink>> getPurchaseLinks({
     required String title,
     required String artist,
   }) async {
     final response = await _dio.get('/purchase-links', queryParameters: {
       'title': title,
       'artist': artist,
     });
     final links = (response.data['links'] as List)
         .map((e) => PurchaseLink.fromJson(e as Map<String, dynamic>))
         .toList();
     return links;
   }
   ```

**Acceptance**:
- Method compiles, returns `List<PurchaseLink>`
- `flutter analyze` passes

---

### T5: Backend unit + integration tests
**Module**: `backend/src/services/purchase_links.rs` (tests section)
**Size**: S | **Risk**: Low | **Agent**: Backend Builder
**Depends on**: T1, T2, T3
**Files**: `backend/src/services/purchase_links.rs` (edit — add `#[cfg(test)]` module)

1. Unit tests in the service module:
   - `test_build_purchase_links_full_query` — title + artist → 4 links in correct order
   - `test_build_purchase_links_title_only` — only title → 4 links with title as query
   - `test_build_purchase_links_artist_only` — only artist → 4 links with artist as query
   - `test_build_purchase_links_both_empty` — empty title + artist → empty links
   - `test_url_encoding_special_chars` — parentheses, ampersand, plus sign encoded correctly
   - `test_affiliate_tags_appended` — with affiliate config → URLs include affiliate params
   - `test_no_affiliate_tags_when_empty` — default config → no extra URL params
   - `test_store_order_consistent` — links always in Beatport > Bandcamp > Juno > Traxsource order

2. (Optional) Integration test for the route handler via axum test utilities — verify JSON response shape

**Acceptance**:
- All tests pass with `cargo test`
- `cargo clippy -- -D warnings` passes
- `cargo fmt --check` passes

---

### T6: PurchaseLinkPanel widget
**Module**: `frontend/lib/widgets/purchase_link_panel.dart` (new)
**Size**: M | **Risk**: Medium | **Agent**: Frontend Builder
**Depends on**: T4
**Files**: `frontend/lib/widgets/purchase_link_panel.dart` (new)

1. Create `PurchaseLinkPanel` — a `StatefulWidget` (needs expand/collapse state + async loading):
   - Props: `String title`, `String artist`
   - Collapsed state: shows a "Buy" or shopping cart icon button
   - Expanded state: calls API, shows loading indicator, then list of store links
   - Each link: store icon (can use first letter or generic icon initially) + store name, tappable
   - Tap opens URL via `url_launcher` (`launchUrl` with `LaunchMode.externalApplication`)
   - If `url_launcher` fails, copy to clipboard + show SnackBar "Link copied"
   - Empty state: if both title and artist are empty, show "No purchase links available"
   - Uses theme tokens (`Theme.of(context).colorScheme.*`), no hardcoded colors

2. Visual design:
   - Visually SEPARATE from source attribution icons (different section in track tile)
   - Compact: horizontal row of store buttons when expanded
   - Smooth expand/collapse animation with `AnimatedCrossFade` or similar

**Acceptance**:
- Widget renders in collapsed state by default
- Expanding triggers API call and shows links
- Links open in external browser
- Empty state handled (both fields empty)
- Uses theme tokens, no hardcoded colors
- `flutter analyze` passes

---

### T7: Wire PurchaseLinkPanel into SetlistTrackTile
**Module**: `frontend/lib/widgets/setlist_track_tile.dart` (edit)
**Size**: S | **Risk**: Low | **Agent**: Frontend Builder
**Depends on**: T6
**Files**: `frontend/lib/widgets/setlist_track_tile.dart` (edit)

1. Import `PurchaseLinkPanel`
2. Add `PurchaseLinkPanel(title: track.title, artist: track.artist)` below the existing track info section
3. Ensure it's visually separated from source attribution icons (Spotify/SoundCloud/Deezer)
4. Panel should be below the track metadata row, above or beside the audio controls

**Acceptance**:
- Purchase panel visible on every track tile
- Visually distinct from source attribution row
- Existing functionality (playback, attribution) unchanged
- `flutter analyze` passes

---

### T8: Frontend widget tests
**Module**: `frontend/test/widgets/purchase_link_panel_test.dart` (new)
**Size**: S | **Risk**: Low | **Agent**: Frontend Builder
**Depends on**: T6, T7
**Files**: `frontend/test/widgets/purchase_link_panel_test.dart` (new)

1. Test cases:
   - `renders collapsed by default` — verify buy button visible, no store links shown
   - `expands and shows store links on tap` — mock API response, verify 4 store buttons
   - `store order is correct` — Beatport > Bandcamp > Juno > Traxsource
   - `opens URL via url_launcher on store tap` — mock url_launcher, verify launchUrl called
   - `shows empty state when no title/artist` — verify "No purchase links available" message
   - `handles API error gracefully` — mock error, verify error state shown

2. Use `createMockApiClient()` from `test/helpers/mock_api_client.dart` for API mocking

**Acceptance**:
- All tests pass with `flutter test`
- `flutter analyze` passes
- Tests cover all postconditions from UC-020

---

## File Ownership Matrix

| File | Owner (Task) | Type |
|------|-------------|------|
| `backend/src/services/purchase_links.rs` | T1+T2+T5 | NEW |
| `backend/src/services/mod.rs` | T1 | EDIT (1 line) |
| `backend/src/routes/purchase_links.rs` | T3 | NEW |
| `backend/src/routes/mod.rs` | T3 | EDIT (1 line) |
| `backend/src/main.rs` | T3 | EDIT (~10 lines) |
| `frontend/lib/models/purchase_link.dart` | T4 | NEW |
| `frontend/lib/services/api_client.dart` | T4 | EDIT (~15 lines) |
| `frontend/lib/widgets/purchase_link_panel.dart` | T6 | NEW |
| `frontend/lib/widgets/setlist_track_tile.dart` | T7 | EDIT (~5 lines) |
| `frontend/test/widgets/purchase_link_panel_test.dart` | T8 | NEW |

## Agent Assignment (recommended)

- **Backend Builder**: T1+T2 (same file, do together) → T3 → T5
- **Frontend Builder**: T4 → T6 → T7 → T8
- Backend and frontend builders can work in **parallel** — no shared files
- T4 depends on T3 only for the endpoint to exist (can develop against contract before T3 is done)

## Risk Notes

- **Low overall risk** — no DB changes, no external API calls, pure URL construction
- **Juno/Traxsource 403 risk** — SP-009 found these stores block server-side requests. Our links are opened in user's browser, so this doesn't affect us. But users may see "no results" on those stores for some tracks.
- **url_launcher web compatibility** — already proven in the codebase (used for attribution links)
