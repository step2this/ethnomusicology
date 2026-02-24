---
description: Review a Flutter screen implementation against UX criteria, wireframes, and design system
allowed-tools: Read, Glob, Grep, Bash
---

# UX Review: $ARGUMENTS

You are a **UX Architect** performing a structured UX review for the Ethnomusicology project. The target is: **$ARGUMENTS** (a UC number like `UC-03` or a screen name like `browse_screen`).

## Step 1: Load Context

### 1a. Identify the target
- If `$ARGUMENTS` is a UC number: find the use case at `docs/use-cases/uc-<NNN>-*.md` and identify all screens it touches
- If `$ARGUMENTS` is a screen name: find the Flutter file at `frontend/lib/screens/<name>.dart`

### 1b. Load supporting files
- **Use case**: `docs/use-cases/uc-<NNN>-*.md` (for MSS steps and postconditions)
- **Wireframes**: `docs/wireframes/<screen-name>.md` (if they exist)
- **Design system**: `.claude/skills/design-system.md`
- **Theme file**: `frontend/lib/config/theme.dart`
- **Screen implementation**: `frontend/lib/screens/<name>.dart`
- **Related widgets**: `frontend/lib/widgets/` (any widgets used by the screen)

If wireframes don't exist, note this as a CRITICAL finding — UX specs should precede implementation.

## Step 2: Visual Hierarchy & Information Density

Review the screen layout for:

- [ ] **Clear visual hierarchy**: Primary action is most prominent, secondary actions are visually subordinate
- [ ] **Information density**: Not too sparse (wasted space) or too dense (overwhelming)
- [ ] **Content grouping**: Related items are visually grouped with consistent spacing
- [ ] **Typography hierarchy**: Headings, body, captions use the correct scale from design system
- [ ] **Whitespace**: Adequate breathing room between sections (minimum 16dp between groups)
- [ ] **Alignment**: Elements align to a consistent grid (4dp base unit)
- [ ] **Card/surface usage**: Proper elevation levels from design system for content containers

For each issue found, reference the specific widget and line number in the Flutter code.

## Step 3: Touch Targets & Interaction Areas

- [ ] **Minimum touch target**: All interactive elements are at least 48x48dp
  - Check: `IconButton`, `InkWell`, `GestureDetector`, `TextButton`, custom tap handlers
  - Common violations: small icons without padding, tight list item actions
- [ ] **Target spacing**: At least 8dp between adjacent touch targets
- [ ] **Tap feedback**: All tappable elements provide visual feedback (`InkWell`, `InkResponse`, ripple)
- [ ] **Hit testing**: No overlapping touch targets that could cause mis-taps
- [ ] **Gesture conflicts**: No conflicting gestures (e.g., horizontal swipe on a vertically scrolling list)

Scan for widgets smaller than 48x48:
```
grep -rn "SizedBox\|Container\|IconButton\|InkWell" frontend/lib/screens/ frontend/lib/widgets/
```

## Step 4: Loading States

Every screen and async operation must have a loading state:

- [ ] **Initial load**: Screen shows skeleton/shimmer or centered `CircularProgressIndicator`
- [ ] **Pull-to-refresh**: Where applicable, `RefreshIndicator` wraps scrollable content
- [ ] **Inline loading**: Actions (add to playlist, play track) show inline progress, not full-screen
- [ ] **Pagination loading**: Bottom-of-list loader for paginated content
- [ ] **No flash of empty**: Loading state appears before data, not after a blank flash

Check for `FutureBuilder`, `StreamBuilder`, `AsyncValue` (Riverpod) — each must handle `loading` state.

## Step 5: Empty States

- [ ] **Empty list/grid**: Shows illustration or icon + message + primary action (not just blank space)
- [ ] **No search results**: Helpful message with suggestions ("Try broadening your filters")
- [ ] **Empty playlist**: Clear CTA to add tracks
- [ ] **First-time user**: Onboarding hint or guided action
- [ ] **Offline/no connection**: Meaningful message with retry action

Check that every `ListView`, `GridView`, or data-driven widget handles the empty case.

## Step 6: Error State Presentation

- [ ] **Network errors**: User-friendly message (not raw exception text), retry button
- [ ] **API errors**: Graceful degradation (e.g., show cached data if available)
- [ ] **Validation errors**: Inline, near the field, in red/error color from theme
- [ ] **Snackbar/toast usage**: Transient errors use `SnackBar`; persistent errors use inline UI
- [ ] **Error recovery**: Every error state has a clear path forward (retry, go back, contact support)

Check for `try/catch`, `catchError`, `AsyncValue.error` — each must render a user-facing message.

## Step 7: Animation & Transition Quality

- [ ] **Page transitions**: Consistent transition type (Material motion: shared axis, fade through)
- [ ] **List animations**: Items animate in (staggered fade, slide) on first load
- [ ] **State transitions**: Play/pause, add/remove, expand/collapse are animated, not instant
- [ ] **Duration compliance**: Animations use design system durations (150ms micro, 300ms standard, 500ms emphasis)
- [ ] **Easing compliance**: Animations use design system curves (`Curves.easeOutCubic` default)
- [ ] **No jank**: Animations don't cause frame drops (no heavy computation during animation)
- [ ] **Reduced motion**: Respects `MediaQuery.disableAnimations` for users who prefer reduced motion

Check for `AnimatedContainer`, `AnimationController`, `Hero`, `PageRouteBuilder` usage.

## Step 8: RTL Layout Correctness

- [ ] **Directionality**: Screen renders correctly when `Directionality` is set to `TextDirection.rtl`
- [ ] **Logical properties**: Uses `EdgeInsetsDirectional` (start/end) not `EdgeInsets` (left/right)
- [ ] **Icon mirroring**: Directional icons (arrows, back) mirror in RTL
- [ ] **Text alignment**: Uses `TextAlign.start`/`TextAlign.end`, not `left`/`right`
- [ ] **Row ordering**: `MainAxisAlignment.start` behaves correctly in both directions
- [ ] **Mixed text**: Arabic labels with Latin content (artist names) render correctly
- [ ] **Number direction**: Numbers and Latin text within Arabic context flow correctly

Scan for RTL violations:
```
grep -rn "EdgeInsets\.\|TextAlign\.left\|TextAlign\.right\|Alignment\.centerLeft\|Alignment\.centerRight" frontend/lib/screens/ frontend/lib/widgets/
```

## Step 9: Responsive Behavior

Test the layout at three breakpoints:

### Mobile (375px)
- [ ] Single-column layout, no horizontal overflow
- [ ] Bottom navigation bar (if applicable)
- [ ] Mini player bar at bottom, above nav bar
- [ ] Cards fill width with horizontal margin

### Tablet (768px)
- [ ] Two-column layout where appropriate (e.g., browse grid)
- [ ] Side navigation or rail (if applicable)
- [ ] Player bar may expand to show more controls
- [ ] Content has max-width constraint, doesn't stretch edge-to-edge

### Desktop (1280px)
- [ ] Three-column layout where appropriate
- [ ] Persistent side navigation
- [ ] Player bar is full-featured
- [ ] Content area has max-width (e.g., 960px) and is centered

Check for `LayoutBuilder`, `MediaQuery`, responsive breakpoint logic.

## Step 10: Accessibility Audit

- [ ] **Semantic labels**: All `Image`, `Icon`, `IconButton` have `semanticLabel` or `Semantics` wrapper
- [ ] **Contrast ratios**: Text on backgrounds meets WCAG AA (4.5:1 normal, 3:1 large)
  - Check all text color / background color combinations against design system
- [ ] **Focus order**: `FocusTraversalGroup` or logical widget order ensures sensible tab navigation
- [ ] **Screen reader**: Interactive elements announce their purpose and state
  - Buttons: "Play track [name]", not just "Play"
  - Toggles: announce on/off state
  - Lists: announce position ("Track 3 of 12")
- [ ] **Sufficient information**: No information conveyed by color alone (icons or text supplement color)
- [ ] **Keyboard navigation**: All interactive elements reachable via Tab key
- [ ] **Live regions**: Dynamic content updates (now playing, loading complete) announced via `Semantics(liveRegion: true)`

## Step 11: Design System Compliance

- [ ] **Colors**: All colors reference theme tokens (`Theme.of(context).colorScheme.*`), no hardcoded `Color(0x...)`
- [ ] **Typography**: All text uses theme text styles (`Theme.of(context).textTheme.*`), no hardcoded `TextStyle`
- [ ] **Spacing**: Padding/margin values use multiples of 4dp from the spacing scale
- [ ] **Elevation**: Card/surface elevation uses design system levels (0, 1, 2, 3, 4, 5)
- [ ] **Border radius**: Uses design system radius tokens (4, 8, 12, 16, 24)
- [ ] **Icons**: Consistent icon set (Material Icons or Material Symbols), consistent size (24dp default)
- [ ] **Component patterns**: Standard components (track tile, playlist card, occasion chip) are reused, not recreated

Scan for hardcoded values:
```
grep -rn "Color(0x\|FontWeight\.\|fontSize:\|EdgeInsets\.all([^4816]" frontend/lib/screens/ frontend/lib/widgets/
```

## Step 12: Wireframe Compliance (if wireframes exist)

If `docs/wireframes/<screen-name>.md` exists:

- [ ] Widget hierarchy matches wireframe regions
- [ ] Content placement matches wireframe layout
- [ ] Responsive behavior matches wireframe breakpoint specs
- [ ] Interactive elements are where the wireframe places them
- [ ] Missing elements from wireframe are flagged

## Step 13: Generate UX Review Report

Output a structured report:

```markdown
# UX Review: <Screen/UC Name>

## Summary
- **Overall Rating**: PASS / CONDITIONAL / FAIL
- **Critical Issues**: <count>
- **Warnings**: <count>
- **Suggestions**: <count>
- **Wireframe Exists**: Yes/No
- **Reviewed Files**: <list of files examined>

## Critical Issues (Must Fix)
### CRIT-1: <Title>
- **Category**: <Visual Hierarchy | Touch Targets | Loading | Empty | Error | Animation | RTL | Responsive | A11y | Design System | Wireframe>
- **Location**: `<file>:<line>`
- **Problem**: <description>
- **Fix**: <specific Flutter code change>

## Warnings (Should Fix)
### WARN-1: <Title>
- **Category**: <...>
- **Location**: `<file>:<line>`
- **Problem**: <description>
- **Fix**: <specific Flutter code change>

## Suggestions (Nice to Have)
### SUGG-1: <Title>
- **Category**: <...>
- **Problem**: <description>
- **Suggestion**: <improvement idea>

## Design System Compliance
| Token Category | Compliant | Violations |
|---------------|-----------|------------|
| Colors | Yes/No | <details> |
| Typography | Yes/No | <details> |
| Spacing | Yes/No | <details> |
| Elevation | Yes/No | <details> |
| Border Radius | Yes/No | <details> |
| Motion | Yes/No | <details> |

## Accessibility Scorecard
| Criterion | Status | Notes |
|-----------|--------|-------|
| Contrast (4.5:1) | PASS/FAIL | |
| Touch targets (48dp) | PASS/FAIL | |
| Semantic labels | PASS/FAIL | |
| Focus order | PASS/FAIL | |
| Screen reader | PASS/FAIL | |
| RTL layout | PASS/FAIL | |
| Reduced motion | PASS/FAIL | |

## Responsive Behavior
| Breakpoint | Status | Issues |
|------------|--------|--------|
| Mobile (375px) | PASS/FAIL | |
| Tablet (768px) | PASS/FAIL | |
| Desktop (1280px) | PASS/FAIL | |

## Next Steps
1. <Prioritized action items>
2. ...
```

## Verdicts

- **PASS**: No critical issues. Warnings and suggestions are tracked but don't block.
- **CONDITIONAL**: 1-2 critical issues that are straightforward to fix. Implementation Team resolves before merge.
- **FAIL**: 3+ critical issues or fundamental layout/accessibility problems. Rework required.
