# Component Design — Round 2 Critique

## Round 1 Recap
- **A (Lean Crate):** Cut — right design language but missing C's structural features
- **B (Studio Deck):** Cut — too much for v1, bottom sheet felt mobile, checkbox/batch premature
- **C (Mixer Board):** Kept — right structure (slide-over, detail strip, multi-sort, source pills, accent-border selection)
- **User direction:** Merge A+C — C's structure with A's compact design language

---

## Option A+C: Crate + Board

**14 components. C's structural decisions, A's visual language.**

### What came from C (structure):
- Click-to-select rows with accent left border
- Expandable detail strip below selected row (label, year, album, duration)
- Multi-column sort with priority numbers (Shift+click)
- Source pills (not segmented control or tabs)
- Slide-over panel with scrim + explicit close button

### What came from A (design language):
- Compact pill Camelot chips (3px radius, 20px height)
- Thin vertical energy bars (5px wide, color-coded)
- Flat borders, 6px radius buttons, 34px button height
- 320px panel width (not C's 400px)
- Simple dot status indicators in import results
- Compact toasts, shaped skeleton rows

### Assessment:

1. **14 components** — right count. Everything needed, nothing extra.
2. **State coverage:** Complete for v1 scope. Row: default/hover/selected/focused/skeleton. Buttons: default/hover/disabled/loading. Detail strip: expanded/collapsed.
3. **The full table view** is the most important artifact — it shows the track row, multi-sort headers, detail strip, and skeleton all in context. This is what the DJ sees every day.
4. **Multi-column sort** is clean — priority numbers in circles next to the arrow. Click = primary sort, Shift+click = add secondary. The most impactful interaction pattern in the kit.
5. **Slide-over at 320px** is the right balance — wide enough for URL input and result rows, narrow enough to not feel like it's replacing the catalog. Scrim communicates "temporary focused task."
6. **Source pills** scale cleanly to 4+ sources without crowding. The segmented control from A would've gotten tight.
7. **Import result rows showing BPM + key on success** — nice touch. The DJ immediately sees if the imported data is useful.
8. **Compact pill Camelot chips** keep rows dense. The color does the heavy lifting for harmonic awareness — the shape is just a container.

### What to watch:
- Detail strip metadata (label, year, album, duration) depends on the backend actually having this data. For Spotify imports it's available; for Beatport it varies. Edge state: show "—" for missing fields.
- Multi-column sort is a backend concern too — the API needs to support multi-field ordering.

### Downstream implications:
- **Typography:** Must work at 13px artist (bold), 12px title, 13px mono BPM, 11px chip text, 10px genre pill, 9px source badge. 6 distinct sizes/weights.
- **Color system:** Needs Camelot tint palette (24 keys), energy bar gradient (green→yellow→red), accent for selection border and sort priority, scrim opacity.
- **Elevation:** Mostly flat — slide-over panel is the main layered surface. Detail strip uses background tint, not elevation.
- **Density:** Row height ~48px, detail strip ~40px. Column spacing already defined by the grid template.
- **Accessibility:** Multi-sort needs keyboard support (Enter to sort, Shift+Enter to add). Detail strip needs aria-expanded. Focus management when slide-over opens/closes.
