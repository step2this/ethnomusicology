# Component Design Critique — Purchase Link Panel

**Facet:** purchase-panel | **Round:** 1 | **Options:** 3
**Building on:** Locked Crate + Board component system (option-ac), Ember Crate palette, Studio density (40px rows), Wire Lift elevation.

---

## Option A: Chip Strip

Horizontal row of pill-shaped store chips that expands inline below the source attribution row.

**State coverage:** Default, hover, focus, active, loading (skeleton chips). All interactive states defined.

**Strengths:**
- Most compact expansion (+32px). Minimal disruption to the track list scanning flow.
- Chip pattern is already in the design vocabulary (Camelot key chips, source pills). Consistent.
- "Buy" label gives the strip semantic meaning at a glance.
- Horizontal layout matches the existing metadata row flow — information scans left to right.

**Weaknesses:**
- 24px chip height is below the 44px Material touch target minimum on mobile/tablet. Would need larger tap areas or a mobile-specific variant.
- First-letter avatars create ambiguity: "B" = Beatport or Bandcamp? Relies on the store name text which is small (11px).
- Four chips in a horizontal row can get tight on narrower viewports. May need horizontal scroll or wrapping.

**Downstream implications:** Color system provides amber hover state. Typography needs 11px weight for chip text. No elevation needed (flat inline).

---

## Option B: Popover Tray

Floating popover menu anchored to the shopping bag icon button. Vertical list of stores.

**State coverage:** Default, hover, focus, loading (spinner). Plus dismiss states (click-outside, ESC). Good overlay state management.

**Strengths:**
- Zero layout shift. Track list stays perfectly stable. Important for a dense DJ table where vertical position matters.
- 48px row height per store item gives excellent touch targets — best of all options for mobile.
- Vertical list with 28px icon squares gives the most space to store names. "Juno Download" fits without truncation.
- Follows the popover/dropdown pattern already established in the locked component system (slide-over, sort menus).
- Dismiss via click-outside/ESC is a well-understood pattern.

**Weaknesses:**
- Overlays track rows below — can occlude tracks the user might want to reference while choosing a store.
- Requires overlay z-index management and dismiss logic. Slightly more complex to implement.
- Only one popover can be open at a time (or you get visual chaos). Need to close others when opening a new one.
- On narrow viewports, popover may clip the right edge — needs repositioning logic.

**Downstream implications:** Elevation system must handle popover shadow (0 8px 24px — consistent with Wire Lift floating elements). Focus trap may be needed for keyboard users.

---

## Option C: Inline Grid

Full-width 4-column grid that expands below the track row, using the full table width.

**State coverage:** Default, hover, focus, loading (skeleton cells). Chevron rotation indicates open state.

**Strengths:**
- Largest touch targets of all options (~150px x 44px per cell). Best for tablet/touch.
- Equal-width cells give visual balance — no store feels privileged over others.
- Full-width grid creates the clearest visual separation from source attribution row.
- 1px gap dividers are elegant and match the table's flat aesthetic.

**Weaknesses:**
- Largest layout shift (+44px). An expanded track nearly doubles its height (60px → 104px). This disrupts the scan flow in a dense track table.
- Grid pattern is unlike anything else in the current component system. No other component uses a sub-row grid. Feels structurally foreign.
- On narrow viewports, 4 columns would need to collapse to 2x2 — more responsive logic.
- The chevron trigger (▾) is less semantically clear than a shopping bag icon. Users may not associate it with "purchase."

**Downstream implications:** Color system needs amber hover per-cell. Elevation: none (flat inline). Grid introduces a new layout pattern not in the existing system.

---

## My Take

**I'd lean toward Option B (Popover Tray).**

The zero-layout-shift property is the killer feature here. In a DJ setlist with 15-20 tracks at 40px rows, the list density is core to the product's "scan fast" promise. Options A and C both push content down — A by 32px (tolerable) and C by 44px (disruptive). The popover keeps the table perfectly stable.

B also has the best touch targets (48px rows), the most room for store names (no truncation), and follows the established overlay pattern from the locked component system (slide-over, sort menus). The implementation complexity (dismiss logic, z-index) is standard Flutter PopupMenuButton territory.

The main risk with B is occlusion — the popover covers tracks below. But since this is a quick "tap → pick store → tab opens → done" interaction (typically <2 seconds), momentary occlusion is acceptable. The user is focused on the popover, not the tracks below it.

If the user wants zero-overlay and maximum density, Option A (Chip Strip) is the runner-up — just needs larger touch targets for mobile.

Option C is the outlier. The full-width grid is visually appealing but structurally foreign to the rest of the component system, and the layout shift is the largest.
