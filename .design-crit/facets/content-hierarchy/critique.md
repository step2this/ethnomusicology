# Content Hierarchy — Round 1 Critique

Building on locked decisions:
- **Layout:** Full-Width Crate — edge-to-edge track table, sortable columns, 48px icon rail, DJ software density.
- **Screen Inventory:** Catalog-First — the track table is the product, import is a secondary action.
- **Edge States:** Context-Rich — tailored per situation.

---

## Option A: DJ Data Forward

**Reading order:** Art → BPM → Key → Title → Artist → Genre → Source

**The 5-second test:** "This is a track list and the first thing I notice is tempo and key data." Passes for a DJ tool. Fails for anyone who doesn't already know what BPM and Camelot keys mean.

**Assessment:**

1. User sees BPM first (16px mono bold, biggest element in row), then Key chip (color draws the eye), then title/artist.
2. Page purpose is clear — it's a track catalog. Primary action (Import) is top-right.
3. Uses 4 emphasis levels — appropriate for a data-dense table.
4. No competing emphasis — BPM and Key are both Primary but differentiated (number vs. chip).
5. Artist at tertiary (12px, muted) feels too low given that DJs think in terms of artist as much as BPM. **This is the main weakness** — it buries the "who" to promote the "how fast/what key."
6. Reading order follows a "can I mix this?" → "what is it?" sequence — logical for set building, but awkward for browsing.
7. **Missing energy level column** — an important DJ data point flagged by the brief.

**Downstream implications:** 4-level hierarchy → type scale needs 4 distinct steps.

---

## Option B: Identity First

**Reading order:** Art → Title → Artist → BPM → Key → Genre → Source

**The 5-second test:** "This is a music library and I should browse by title/artist." Passes as a general music app. Loses DJ-specific character — could be any media library.

**Assessment:**

1. User sees title first (14px/600, heaviest text), then artist, then BPM/Key.
2. Page purpose is clear. Primary action visible.
3. Uses 4 emphasis levels.
4. BPM and Key rely on visual treatment (monospace, color chip) rather than size/position for attention. This is elegant but depends on the DJ having learned these visual cues.
5. Title getting top emphasis feels slightly wrong for DJ context — DJs think "that Fisher track" not "that Losing It track." Title matters but artist matters more.
6. Reading order follows "what is it?" → "can I mix it?" — comfortable but not DJ-optimized.
7. **Missing energy level column.**

**Downstream implications:** 4-level hierarchy → type scale needs 4 distinct steps.

---

## Option C: Flat Five

**Reading order:** Art → Artist → Title → BPM → Key → Energy → Genre → Source

**The 5-second test:** "This is a DJ catalog and I can see who made it, what it's called, how fast it is, what key it's in, and how intense it is — all at a glance." Passes strongly for the target user.

**Assessment:**

1. User's eye enters at Artist (bold, left), then flows across the "identity zone" (artist + title) and into the "mix zone" (BPM + Key + Energy). Two reading zones, not six individual stops.
2. Page purpose clear. Import CTA top-right.
3. Uses only 3 emphasis levels (co-primary, secondary, ambient). Flat hierarchy means less visual drama but higher information density — more like Rekordbox/Traktor's column-based scanning.
4. No competing emphasis within the co-primary tier because each field uses a completely different visual voice: bold text, regular text, monospace number, color chip, bar graph. The differentiation comes from treatment variety, not size hierarchy.
5. **Energy level included** as a mini bar graph — immediately scannable without taking up much horizontal space. Color-coded: green (low/chill), yellow (mid), red (high energy).
6. Artist before Title reflects how DJs actually think about their collection.
7. Reading order is task-flexible: when browsing, the eye hits artist → title; when set-building, the eye jumps to BPM → Key → Energy. The flat hierarchy supports both modes.

**Downstream implications:** 3-level hierarchy → type scale needs only 3 distinct steps. More visual coding work for the color system (energy colors + Camelot colors).

---

## My Take

**I'd go with C (Flat Five)**, and here's why:

The user told us DJs think in terms of artist, title, BPM, key, AND energy — all five matter. Options A and B both try to pick a winner between identity and DJ data, but the real answer is neither dominates. The DJ's task determines what they scan for: browsing mode hits artist/title, set-building mode hits BPM/key/energy.

A flat hierarchy with rich visual coding is exactly how Rekordbox and Traktor work — every column is equally accessible, and you scan to whatever matters right now. The visual coding approach (bold vs. regular vs. mono vs. chip vs. bars) gives each field its own "shape" so the eye can learn to jump directly to what it needs without a rigid reading order.

Option A's approach of making BPM/Key physically larger is heavy-handed — it pre-decides the DJ's task. Option B's traditional library ordering demotes the mix data that makes this product special.

Option C also nails the energy column, which A and B both miss. Energy is a core part of set building — you need to shape the night's arc, not just match BPM/key.

The trade-off: a flat hierarchy gives the typography and color facets less room to create dramatic visual hierarchy. But for a dense, scannable catalog, that's the right trade. This is a power-user tool, not a consumer landing page.
