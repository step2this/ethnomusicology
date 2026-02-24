---
description: Generate ASCII wireframes with interaction specs and widget mapping for a screen
allowed-tools: Read, Glob, Grep, Write, AskUserQuestion
---

# Wireframe: $ARGUMENTS

You are a **UX Architect** creating wireframes for the Ethnomusicology project. The target is: **$ARGUMENTS** (a screen name like `browse_screen` or a UC number like `UC-03`).

## Step 1: Identify the Screen(s)

### If $ARGUMENTS is a UC number:
1. Read the use case from `docs/use-cases/uc-<NNN>-*.md`
2. Walk through the MSS steps and identify every screen the user sees
3. Create wireframes for each screen (one file per screen)

### If $ARGUMENTS is a screen name:
1. Check if `frontend/lib/screens/<name>.dart` exists — if so, wireframe the existing implementation
2. If it doesn't exist, wireframe based on the project plan and use cases that reference it

## Step 2: Gather Context

Read these files for context:
- **Design system**: `.claude/skills/design-system.md` (tokens, component patterns)
- **Project plan**: `docs/project-plan.md` (feature requirements, navigation structure)
- **Existing wireframes**: `docs/wireframes/` (for consistency with already-designed screens)
- **Theme**: `frontend/lib/config/theme.dart` (if it exists)
- **Existing screen code**: `frontend/lib/screens/<name>.dart` (if it exists)

## Step 3: Create the Wireframe Document

For each screen, create a file at `docs/wireframes/<screen-name>.md` with the following structure:

```markdown
# Wireframe: <Screen Name>

## Overview
- **Screen**: <human-readable name>
- **Use Case(s)**: UC-<NNN>, UC-<NNN>
- **Primary Action**: <what the user comes here to do>
- **Entry Points**: <how the user gets here — navigation, deep link, etc.>
- **Exit Points**: <where the user goes next>

## Mobile Layout (375px)

### ASCII Wireframe

Use box-drawing characters to create the wireframe. Rules:
- `┌─┐└─┘│` for containers/cards
- `[Button Text]` for buttons
- `(○)` for radio, `[✓]` for checkbox
- `▶` for play, `⏸` for pause, `⏭` for skip
- `☰` for hamburger menu
- `←` for back arrow
- `🔍` for search
- `⋮` for overflow menu
- `───` for dividers
- Use UPPERCASE for headings
- Use Title Case for labels
- Mark scrollable regions with `↕ scrollable`
- Mark tap targets with `[tap]` annotation

Example format:
```
┌─────────────────────────────┐
│ ← BROWSE BY REGION     🔍 ⋮│  ← App Bar (56dp)
├─────────────────────────────┤
│                             │
│  ┌─────────┐ ┌─────────┐   │
│  │ 🌍      │ │ 🌍      │   │  ← Region Cards
│  │ Maghreb │ │ Sahel   │   │    (tap → region detail)
│  │ 12 trks │ │ 8 trks  │   │
│  └─────────┘ └─────────┘   │
│                             │  ↕ scrollable
│  ┌─────────┐ ┌─────────┐   │
│  │ 🌍      │ │ 🌍      │   │
│  │ Levant  │ │ Horn of │   │
│  │ 15 trks │ │ Africa  │   │
│  └─────────┘ └─────────┘   │
│                             │
├─────────────────────────────┤
│ ▶ Now Playing: Track Name   │  ← Mini Player (64dp)
│   Artist Name     advancement│    (tap → full player)
├─────────────────────────────┤
│ 🏠  🔍  📋  👤             │  ← Bottom Nav (56dp)
│ Home Search Lists Profile   │
└─────────────────────────────┘
```

### Interaction Annotations

| Element | Gesture | Action | Target Size |
|---------|---------|--------|-------------|
| Region Card | Tap | Navigate to region detail screen | Full card (>48dp) |
| Search icon | Tap | Open search overlay | 48x48dp |
| Mini Player | Tap | Expand to full player (bottom sheet) | Full bar (64dp) |
| Mini Player | Swipe right | Skip to next track | Full bar width |
| Bottom Nav | Tap | Switch tab | 48x48dp per item |

## Tablet Layout (768px)

### ASCII Wireframe

(Show how layout changes — typically 3-column grid, side rail navigation)

### Layout Changes from Mobile
- Navigation moves to side rail (72dp wide)
- Content area uses 3-column grid instead of 2-column
- Mini player expands to show album art and progress bar
- ...

## Desktop Layout (1280px)

### ASCII Wireframe

(Show how layout changes — persistent sidebar, wider content, more detail)

### Layout Changes from Tablet
- Side rail expands to full navigation drawer (256dp)
- Content area has max-width of 960px, centered
- Player bar shows full controls (shuffle, repeat, volume, queue)
- ...

## Widget Tree Mapping

Map each wireframe region to a Flutter widget:

```
Screen (<ScreenName>)                          → Scaffold
├── App Bar                                     → AppBar / SliverAppBar
│   ├── Back button                             → IconButton (leading)
│   ├── Title                                   → Text (title)
│   └── Actions                                 → [IconButton] (actions)
├── Body                                        → CustomScrollView / ListView
│   ├── Region Grid                             → GridView.builder
│   │   └── Region Card                         → Card > InkWell > Column
│   │       ├── Region Icon                     → Icon / Image
│   │       ├── Region Name                     → Text (titleMedium)
│   │       └── Track Count                     → Text (bodySmall)
│   └── ... (more sections)
├── Mini Player                                 → BottomSheet (persistent)
│   ├── Track Info                              → ListTile
│   ├── Play/Pause                              → IconButton (48dp)
│   └── Progress                                → LinearProgressIndicator
└── Bottom Navigation                           → NavigationBar
    └── Nav Items                               → NavigationDestination
```

## Design Tokens Used

Reference tokens from `.claude/skills/design-system.md`:

| Token | Value | Usage |
|-------|-------|-------|
| `colorScheme.primary` | Navy (#1A237E) | App bar, active nav item |
| `colorScheme.surface` | White (#FFFFFF) | Card backgrounds |
| `textTheme.titleMedium` | Noto Sans 16/500 | Region card names |
| `textTheme.bodySmall` | Noto Sans 12/400 | Track counts |
| `spacing.md` | 16dp | Card padding |
| `spacing.sm` | 8dp | Grid gap |
| `elevation.card` | 1 | Region cards |
| `borderRadius.md` | 12dp | Card corners |
| `motion.standard` | 300ms easeOutCubic | Card tap feedback |

## States

### Loading State
(ASCII wireframe showing skeleton/shimmer placeholders)

### Empty State
(ASCII wireframe showing empty state with message and CTA)

### Error State
(ASCII wireframe showing error message with retry button)
```

## Step 4: Interaction Specs for Complex Components

For any complex interactive component on the screen (audio player, playlist builder, drag-and-drop, multi-step flow), create a state diagram:

```markdown
## State Diagram: <Component Name>

```
[idle] ──tap──→ [loading] ──success──→ [playing]
                    │                      │
                    │                      ├──tap──→ [paused]
                    │                      │            │
                    │                      ←──tap───────┘
                    │
                    └──error──→ [error] ──retry──→ [loading]
```

### State Descriptions
| State | Visual | Audio | Actions Available |
|-------|--------|-------|-------------------|
| idle | Play icon, no progress | Silent | Tap to play |
| loading | Spinner replaces play icon | Silent | Tap to cancel |
| playing | Pause icon, progress bar animating | Audio output | Tap to pause, seek, skip |
| paused | Play icon, progress bar frozen | Silent | Tap to resume, seek |
| error | Error icon, message | Silent | Tap to retry |
```

## Step 5: Validate Consistency

Before saving, check:
- [ ] All wireframes use the same component patterns (track tiles, cards, navigation)
- [ ] Spacing follows the 4dp grid
- [ ] All breakpoints are covered (mobile, tablet, desktop)
- [ ] Widget tree is consistent with existing Flutter code (if screen exists)
- [ ] Design tokens reference the design system (not hardcoded values)
- [ ] All interactive elements have interaction annotations with target sizes
- [ ] Loading, empty, and error states are included

## Step 6: Save and Report

1. Write the wireframe file to `docs/wireframes/<screen-name>.md`
2. If multiple screens were identified (from a UC), save each one separately
3. Output a summary:
   - Screens wireframed
   - Key interaction patterns identified
   - Design tokens used
   - Potential UX risks or open questions
4. Remind about next steps:
   - Review wireframes with UX team
   - Run `/ux-review` after implementation
   - Ensure Implementation Team references wireframes during build
