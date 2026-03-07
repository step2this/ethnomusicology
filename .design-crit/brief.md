# Design Brief: Purchase Link Panel (UC-020)

> An expandable panel on each track tile that shows search links to DJ stores (Beatport, Bandcamp, Juno Download, Traxsource), enabling DJs to buy tracks discovered through LLM-generated setlists.

## Target Users and Context of Use
DJs at a laptop during set prep. They've just generated or reviewed a setlist and want to buy tracks -- especially LLM suggestions not in their catalog. The purchase action is secondary to browsing/listening but critical for converting discovery into acquisition. Desktop-first, occasionally tablet.

## Core Interaction Loop
See track in setlist -> want to buy it -> expand purchase panel -> tap store link -> store opens in new tab with search results. One-tap-per-store, collapsed by default to avoid clutter.

## Differentiator
Multi-store search links from a single panel. Most DJ tools link to one store (usually their own). This gives DJs choice across Beatport, Bandcamp, Juno Download, and Traxsource -- covering different catalog strengths (Beatport for mainstream electronic, Bandcamp for underground/independent).

## Platform and Constraints
- **Platform:** Web (Flutter Web, Chrome primary)
- **Tech stack:** Flutter/Dart, Riverpod, Material 3, url_launcher
- **Device targets:** Desktop-first, responsive to tablet
- **Hard constraints:** Links are pure URL construction (no external API calls). Panel must be visually SEPARATE from existing source attribution icons (Spotify/SoundCloud/Deezer). Underground tracks may not be on any store -- the "empty result" is the store's search page, not our UI problem.

## Scope
### In v1
- Expandable purchase panel on each track tile (collapsed by default)
- 4 store links: Beatport, Bandcamp, Juno Download, Traxsource (fixed order)
- Lazy-loaded on expand (calls `GET /api/purchase-links?title=X&artist=Y`)
- Store icon/button treatment per store
- url_launcher to open in external browser tab
- Clipboard fallback if url_launcher fails
- Empty state when both title and artist are missing

### Out of v1
- Hit verification (checking if a store actually has the track)
- Store preference reordering
- Bulk purchase links for entire setlist
- Affiliate tag registration (infrastructure ready, no active affiliates yet)
- Custom store icons/logos (use Material icons or text initially)

## Existing Design Language
Full design system from previous crit cycle (locked):
- **Color:** Ember Crate palette -- amber (#E8963A) primary, copper (#C06030) secondary, warm blacks (#0F0D0B)
- **Typography:** Inter Tight + JetBrains Mono, 7-level scale
- **Spacing:** 4px base unit, 40px track rows, Studio density
- **Elevation:** Wire Lift -- flat table, shadows only on floating elements
- **Components:** Existing track tile with metadata chips, source attribution row, preview status dots, confidence indicators

## Accessibility Requirements
Keyboard accessible (Enter/Space to expand, Tab through store links). Store links must have descriptive labels for screen readers. Touch targets follow 48dp Material minimum. Will inherit locked accessibility decisions from previous crit.

---

## Design Questions to Resolve

| # | Question | Design Impact |
|---|----------|---------------|
| 1 | **Expandable inline panel vs bottom sheet?** UC specifies collapsed-by-default expandable panel. Should it expand inline (pushing content down) or slide up as a bottom sheet? Inline keeps context; bottom sheet avoids layout shift. | Layout, animation, mobile behavior |
| 2 | **Store icon treatment** -- Material icons (shopping_cart, storefront), first-letter avatars ("B", "J"), or simple text buttons? No official store logos available as Flutter icons. | Component design, visual density |
| 3 | **Trigger button placement** -- Where does the "Buy" button sit in the existing track tile? Options: end of metadata chip row, in the action area (near play button), or as a new row below metadata. | Hierarchy, scannability |

---

## Design Decisions (from brief)

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Interaction pattern | Collapsed-by-default expandable panel | Avoids visual clutter. DJs only expand when they want to buy. Lazy-loads API call on expand (1ms server-side). |
| Store count | 4 stores, fixed order | Beatport > Bandcamp > Juno Download > Traxsource. Covers mainstream electronic + underground. More would add clutter for diminishing returns. |
| Visual separation | Separate section from source attribution | Source attribution (Spotify/SoundCloud/Deezer) shows WHERE we found the track. Purchase links show WHERE to BUY. Different concerns, different UI sections. |
| Empty state | Store search page handles "not found" | Underground tracks may not be on any store. We always show all 4 links -- the store's search page handles empty results. No client-side "not found" needed. |
| Link opening | url_launcher + clipboard fallback | External browser tab via url_launcher. If it fails, copy URL to clipboard and show SnackBar. |
