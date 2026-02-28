# Screen Inventory — Round 1 Critique

## Option A: Two-Screen Flow

### WHY THIS OPTION
Follows the proven pattern from the existing Spotify import. Separate Import and Catalog screens, each with a single responsibility. The user imports on one screen, then navigates to browse their catalog. This is the most conventional approach — familiar from Rekordbox (import view vs collection view).

### WORKS WELL FOR
- Teams extending the existing codebase incrementally (keeps spotify_import_screen.dart pattern)
- Clear mental model: "I'm importing" vs "I'm browsing"
- Easier to add more import sources later without complicating the catalog
- Each screen is simpler to build and test independently

### WATCH OUT FOR
- The "View Catalog" navigation step adds friction to the core loop. Every import requires a screen change.
- User can't see their catalog while deciding what to import — no context for "do I already have this?"
- Two separate routes to maintain in GoRouter, two sets of state management
- Feels like a v1 pattern that might need consolidation in v2

---

## Option B: Unified Workspace

### WHY THIS OPTION
The core loop is "source tab → URL → import → enriched track list." A unified screen delivers this as a continuous, zero-navigation flow. Import controls collapse after use, giving maximum space to the catalog. This mirrors the DJ workflow: import is a quick action, browsing is the sustained activity.

### WORKS WELL FOR
- Repeat importers who do URL-paste-import many times in one session
- Desktop-first layout with enough vertical space for all three zones
- "Power user" feel — everything accessible without navigation
- Future features (search, filter) naturally live above the catalog in the same screen

### WATCH OUT FOR
- Single screen carries more complexity — collapsible zones, conditional visibility, compound state
- Import controls take up space even when the user just wants to browse
- More states to manage on one screen (8 combined states vs 10 split across two screens)
- First-use experience is trickier — empty catalog + expanded import needs careful design

---

## Option C: Catalog-First

### WHY THIS OPTION
Makes the strongest bet that the catalog is the product. Import is a task you do to feed the catalog — it's secondary, triggered as needed. The bottom sheet pattern keeps import focused and dismissible while the catalog remains contextually visible behind it. New tracks appear right where the user will browse them.

### WORKS WELL FOR
- Users who import occasionally but browse frequently (catalog is the daily driver)
- The "new tracks appear in context" moment — import results are immediately visible
- Clean information architecture: one primary surface, import is an action on it
- Natural fit for future features — setlist builder, filters, sort all live on the primary surface

### WATCH OUT FOR
- Bottom sheet on desktop can feel like a mobile pattern. Needs careful sizing for laptop screens.
- Import sheet hides/dims the catalog during import — user loses context while the sheet is up
- The sheet needs to be large enough for tabs + URL + progress — could feel cramped
- Extra gesture to start importing (open sheet) vs dedicated import screen where you land ready to go

---

## My Take

I'd lean toward **B (Unified Workspace)**. Here's why:

The brief's core loop is "source tab → URL → import → enriched track list." Option B delivers this as a continuous flow with zero navigation. For DJs doing set prep — importing from multiple sources in one session, then browsing their enriched catalog — the unified screen minimizes friction.

Option A is the safest and most conventional, but the navigation step between import and catalog adds friction that compounds across multiple imports per session. Option C has the right instinct (catalog is the star) but the bottom sheet pattern is awkward on desktop and adds a gesture to start importing.

The real question is: **how often do your DJs import vs browse?** If they import once and browse for an hour, C makes sense. If they import from 3-4 sources in quick succession, B wins. Given that the brief describes multi-source import as the feature, I'm betting on frequent importing.

That said, B and C aren't far apart. B is "import lives on the same page as catalog." C is "catalog is the page, import is an overlay." Both have one primary surface. The difference is whether import is a zone or an overlay. I'd pick B for the zero-friction importing, but C's catalog-first framing is worth keeping in mind for layout decisions.
