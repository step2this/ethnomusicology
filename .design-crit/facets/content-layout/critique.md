# Content Layout — Round 1 Critique

## Option A: Centered Catalog

### WHY THIS OPTION
Classic web app approach. 12-column grid, max-width 1200px, centered on the viewport. Toolbar above with Import button. Enriched track tiles in a comfortable vertical list. Generous margins on widescreen. Simple, ships immediately, gracefully responsive.

### WORKS WELL FOR
- Fastest to implement. Standard Flutter web layout. No custom split panes.
- Comfortable reading width — track tiles aren't stretched across ultra-wide monitors.
- Trivially responsive to tablet: just reduce margins.
- Clean, modern web app feel. Familiar to anyone who's used a SaaS product.

### WATCH OUT FOR
- Doesn't feel like DJ software. DJs expect edge-to-edge density, not centered web content.
- Wastes horizontal space on widescreen monitors (120px+ margins each side at 1440px).
- No obvious home for a future browser pane — would need a layout restructure.
- Track tiles as enriched cards are bulkier than the table rows DJs are used to scanning.

---

## Option B: Full-Width Crate

### WHY THIS OPTION
Bets on DJ software density. Full-width track table with sortable column headers (Title, Artist, BPM, Key, Genre, Source). A 48px collapsed icon rail on the left reserves the slot for a future browser pane that expands to 240px. No max-width — the catalog uses every pixel on a widescreen monitor. This is the Rekordbox collection view translated to the web.

### WORKS WELL FOR
- DJs who already think in terms of sortable columns. BPM column, key column — this is how Rekordbox/Traktor display tracks.
- Maximizes tracks visible on screen (5-6 rows in the wireframe = ~20+ in a real viewport).
- The icon rail is a lightweight commitment — 48px of screen for future extensibility, invisible to users who don't need it yet.
- Table layout is naturally sortable (click column headers) — anticipates sort/filter features in v2.

### WATCH OUT FOR
- Table rows on ultra-wide monitors may feel stretched. Long gaps between columns at 2560px.
- The collapsed rail is a visible affordance for a feature that doesn't exist yet. Users might click it and find nothing useful.
- Table-style layout is harder to make beautiful — it's functional first, visual second.
- Responsive degradation from a full-width table to mobile is non-trivial.

---

## Option C: Browser + Crate

### WHY THIS OPTION
The full Rekordbox layout from day one. Left browser pane (220px, resizable) showing source filters (Beatport/Spotify/All) in v1, with placeholder sections for Genres, Artists, Labels in v2. Main area is a full-width track table identical to Option B. The browser pane is where the DJ slices and dices the collection — this is THE pattern every DJ knows.

### WORKS WELL FOR
- Most familiar to any DJ who has used Rekordbox, Traktor, or Serato. Zero learning curve.
- Source filtering is immediately useful in v1 (filter by Beatport vs Spotify).
- The browser pane grows naturally over time — each v2/v3 feature fills a section.
- Resizable split pane lets the DJ balance browser vs catalog based on their workflow.

### WATCH OUT FOR
- "Coming in v2" placeholders in the browser pane might feel half-baked in v1.
- 220px browser on top of a full-width table means a LOT of screen real estate to manage.
- Resizable split pane is more complex to implement in Flutter (draggable divider, state persistence).
- The browser pane only has one useful section in v1 (Sources with 2-3 items). It may feel empty.

---

## My Take

I'd go with **B (Full-Width Crate)** or **C (Browser + Crate)** — they're close, and both are clearly better than A for a DJ tool.

The real question is: **build the browser pane now or later?**

**C is the right destination.** Every DJ knows this layout. But in v1, the browser pane only shows 3 source filters (All, Beatport, Spotify). That's thin content for a permanent 220px panel. The "Coming in v2" placeholders might feel like an unfinished product rather than a designed architecture.

**B is the right v1.** The icon rail is a subtle 48px reservation that doesn't promise anything. When you're ready for the browser pane, the rail expands. In the meantime, the track table gets maximum space. It's disciplined: ship what works now, expand when the content justifies it.

That said — if the source filter (All/Beatport/Spotify) is genuinely useful to you in v1, then C earns its place immediately. If you find yourself wanting to quickly see "just my Beatport tracks" vs "just my Spotify tracks," the browser pane is worth having even with only 3 items in it.

**My lean: B for v1 discipline, with the understanding that C is the v2 layout.** The icon rail in B is the bridge.
