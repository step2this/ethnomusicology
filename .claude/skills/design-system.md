---
description: Design system tokens and component patterns for the Ethnomusicology app
---

# Ethnomusicology Design System

This document defines the design tokens and component patterns for the Ethnomusicology app. All Flutter UI code MUST reference these tokens via the theme — never hardcode visual values.

## Color Palette

### Core Colors

| Token | Light Mode | Dark Mode | Usage |
|-------|-----------|-----------|-------|
| `primary` | Navy #1A237E | Light Navy #5C6BC0 | App bar, primary buttons, active states |
| `onPrimary` | White #FFFFFF | White #FFFFFF | Text/icons on primary color |
| `primaryContainer` | Light Blue #E8EAF6 | Dark Navy #283593 | Selected states, chips, badges |
| `onPrimaryContainer` | Navy #1A237E | Light Blue #C5CAE9 | Text on primary container |
| `secondary` | Gold #F9A825 | Warm Gold #FFD54F | Accent buttons, highlights, ratings |
| `onSecondary` | Black #000000 | Black #000000 | Text/icons on secondary color |
| `secondaryContainer` | Light Gold #FFF8E1 | Dark Gold #F57F17 | Occasion badges, secondary selections |
| `onSecondaryContainer` | Dark Gold #E65100 | White #FFFFFF | Text on secondary container |
| `tertiary` | Teal #00695C | Light Teal #4DB6AC | Tags, traditions, region indicators |
| `onTertiary` | White #FFFFFF | Black #000000 | Text/icons on tertiary |
| `surface` | White #FFFFFF | Charcoal #1E1E1E | Cards, sheets, dialogs |
| `onSurface` | Near Black #1C1B1F | Off White #E6E1E5 | Body text, icons |
| `surfaceContainerHighest` | Light Grey #F5F5F5 | Dark Grey #2C2C2C | Subtle backgrounds, dividers |
| `error` | Red #B3261E | Light Red #F2B8B5 | Error states, validation |
| `onError` | White #FFFFFF | Red #601410 | Text/icons on error |
| `outline` | Grey #79747E | Grey #938F99 | Borders, dividers |
| `shadow` | Black | Black | Elevation shadows |

### Occasion-Specific Palettes

Each occasion has a theme that overlays the core palette. Applied via `ThemeExtension<OccasionTheme>`.

| Occasion | Primary | Accent | Surface Tint | Icon |
|----------|---------|--------|-------------|------|
| **Nikah** (Wedding) | Rose Gold #B76E79 | Champagne #F7E7CE | Blush #FFF0F0 | Rings |
| **Eid al-Fitr** | Emerald #2E7D32 | Silver #C0C0C0 | Mint #F0FFF0 | Crescent |
| **Eid al-Adha** | Deep Green #1B5E20 | Gold #F9A825 | Sage #F0F5E8 | Star |
| **Mawlid** | Royal Blue #1565C0 | White #FFFFFF | Sky #F0F4FF | Lantern |
| **Sufi Gathering** | Deep Purple #4A148C | Amber #FFA000 | Lavender #F3E5F5 | Whirl |
| **General Celebration** | Navy #1A237E | Gold #F9A825 | Off White #FAFAFA | Music note |

### Sacred/Devotional Flag Colors

When content is flagged as sacred or devotional, use a muted, respectful visual treatment:

| Token | Value | Usage |
|-------|-------|-------|
| `devotional.surface` | Warm Ivory #FDF6EC | Card background for devotional content |
| `devotional.accent` | Muted Gold #C9A96E | Border accent, icon tint |
| `devotional.text` | Dark Brown #3E2723 | Primary text on devotional surfaces |

## Typography

### Font Families

| Script | Font | Fallback | Weight Range |
|--------|------|----------|-------------|
| Latin | **Noto Sans** | system-ui, sans-serif | 300 (Light) — 700 (Bold) |
| Arabic | **Noto Sans Arabic** | Tahoma, sans-serif | 300 (Light) — 700 (Bold) |
| Display/Headings | **Playfair Display** | Georgia, serif | 400 (Regular) — 700 (Bold) |

**Why Noto Sans**: Covers Latin + Arabic in a single family with consistent metrics. Noto Sans Arabic has proper ligature and diacritic rendering. Google Fonts hosted, free.

**Why Playfair Display**: Elegant serif for headings that evokes tradition while remaining readable. Used sparingly for screen titles and occasion names.

### Type Scale

Based on Material 3 type scale. All sizes in logical pixels (dp).

| Token | Font | Size | Weight | Line Height | Letter Spacing | Usage |
|-------|------|------|--------|-------------|----------------|-------|
| `displayLarge` | Playfair Display | 57 | 400 | 64 | -0.25 | Hero headings (landing page) |
| `displayMedium` | Playfair Display | 45 | 400 | 52 | 0 | Section titles (occasion name on playlist screen) |
| `displaySmall` | Playfair Display | 36 | 400 | 44 | 0 | Screen titles |
| `headlineLarge` | Noto Sans | 32 | 600 | 40 | 0 | Screen headers |
| `headlineMedium` | Noto Sans | 28 | 600 | 36 | 0 | Section headers |
| `headlineSmall` | Noto Sans | 24 | 600 | 32 | 0 | Card titles, dialog titles |
| `titleLarge` | Noto Sans | 22 | 500 | 28 | 0 | App bar title |
| `titleMedium` | Noto Sans | 16 | 500 | 24 | 0.15 | List item title, card title |
| `titleSmall` | Noto Sans | 14 | 500 | 20 | 0.1 | Tab labels, chip text |
| `bodyLarge` | Noto Sans | 16 | 400 | 24 | 0.5 | Primary body text |
| `bodyMedium` | Noto Sans | 14 | 400 | 20 | 0.25 | Secondary body text |
| `bodySmall` | Noto Sans | 12 | 400 | 16 | 0.4 | Captions, timestamps, metadata |
| `labelLarge` | Noto Sans | 14 | 500 | 20 | 0.1 | Button text |
| `labelMedium` | Noto Sans | 12 | 500 | 16 | 0.5 | Navigation labels, badge text |
| `labelSmall` | Noto Sans | 11 | 500 | 16 | 0.5 | Overlines, fine print |

### Arabic Typography Notes

- Arabic text typically renders ~20% larger than Latin at the same font size — test both scripts at each scale
- Line height should be increased by 4dp for Arabic body text to accommodate descenders and diacritics
- Never truncate Arabic text mid-word — use `TextOverflow.fade` instead of `TextOverflow.ellipsis` when possible
- Mixed-direction text (Arabic label with Latin artist name) should use `Unicode.bidi` markers if alignment breaks

## Spacing Scale

Base unit: **4dp**. All spacing values MUST be multiples of 4.

| Token | Value | Usage |
|-------|-------|-------|
| `spacing.xxs` | 2dp | Inline icon-to-text gap (exception to 4dp rule) |
| `spacing.xs` | 4dp | Tight padding (inside chips, badges) |
| `spacing.sm` | 8dp | Standard inner padding, grid gap |
| `spacing.md` | 16dp | Card padding, section gap |
| `spacing.lg` | 24dp | Screen horizontal margin, section separator |
| `spacing.xl` | 32dp | Large section gap |
| `spacing.xxl` | 48dp | Screen top/bottom safe area padding |
| `spacing.xxxl` | 64dp | Hero section spacing |

### Layout Margins

| Context | Margin |
|---------|--------|
| Screen horizontal padding (mobile) | 16dp |
| Screen horizontal padding (tablet) | 24dp |
| Screen horizontal padding (desktop) | 32dp (content max-width 960dp, centered) |
| Bottom padding (above nav bar) | 8dp |
| Bottom padding (above mini player + nav) | 120dp (64dp player + 56dp nav) |

## Elevation / Shadow Levels

Using Material 3 elevation system. Tonal elevation (surface tint) in light mode; shadow-based in dark mode.

| Level | Elevation | Usage | Surface Tint Opacity |
|-------|-----------|-------|---------------------|
| 0 | 0dp | Flat surfaces, backgrounds | 0% |
| 1 | 1dp | Cards at rest, navigation rail | 5% |
| 2 | 3dp | Cards on hover/focus, bottom sheets | 8% |
| 3 | 6dp | FAB at rest, modal bottom sheets, dialogs | 11% |
| 4 | 8dp | FAB on hover, elevated navigation | 12% |
| 5 | 12dp | Dragged cards, top app bar (scrolled) | 14% |

## Border Radius

| Token | Value | Usage |
|-------|-------|-------|
| `radius.none` | 0dp | Dividers, full-width containers |
| `radius.xs` | 4dp | Chips, badges, small buttons |
| `radius.sm` | 8dp | Text fields, small cards |
| `radius.md` | 12dp | Standard cards, dialogs |
| `radius.lg` | 16dp | Large cards, bottom sheets |
| `radius.xl` | 24dp | FAB, rounded buttons |
| `radius.full` | 999dp | Circular avatars, pill shapes |

## Component Patterns

### Track Tile

The primary repeating element. Used in browse lists, playlists, search results.

```
┌──────────────────────────────────────────┐
│ ┌──────┐                                 │
│ │Album │  Track Title          ▶  ⋮      │  56dp height
│ │ Art  │  Artist Name • 3:42             │
│ │48x48 │  [Gnawa] [Morocco]             │  Tags optional
│ └──────┘                                 │
└──────────────────────────────────────────┘
```

- Height: 56dp minimum (72dp with tags)
- Album art: 48x48dp, `radius.sm` (8dp)
- Play button: 48x48dp touch target, icon 24dp
- Overflow menu: 48x48dp touch target
- Title: `titleMedium` (Noto Sans 16/500)
- Subtitle: `bodySmall` (Noto Sans 12/400), `onSurface` at 60% opacity
- Tags: `labelSmall` chips, `radius.xs`, `primaryContainer` background
- Left padding: 16dp, right padding: 8dp
- Spacing between art and text: 12dp

### Playlist Card

Used in browse grids, home screen, occasion recommendations.

```
┌─────────────────────┐
│                     │
│   Cover Mosaic      │  Aspect ratio: 1:1
│   (4 album arts)    │  radius.md (12dp)
│                     │
├─────────────────────┤
│ Playlist Name       │  titleMedium
│ 12 tracks • 45 min  │  bodySmall
│ [Nikah] [Celebratory]  labelSmall chips
└─────────────────────┘
```

- Width: fills grid cell
- Card: `radius.md` (12dp), elevation level 1
- Cover mosaic: 2x2 grid of album arts, clipped to top radius
- Padding: 12dp horizontal, 8dp vertical (text area)
- Occasion badge: positioned top-right of cover, `secondaryContainer` background

### Mini Player Bar

Persistent bar at bottom of screen, above navigation.

```
┌──────────────────────────────────────────┐
│ ┌────┐                                   │
│ │Art │ Track Title        ▶  ⏭          │  64dp height
│ │40  │ Artist Name                       │
│ └────┘                                   │
│ ════════════════════░░░░░░░░░░░░░░░░░░░  │  Progress bar (2dp)
└──────────────────────────────────────────┘
```

- Height: 64dp
- Album art: 40x40dp, `radius.xs` (4dp)
- Play/pause: 48x48dp touch target
- Skip: 48x48dp touch target
- Progress: `LinearProgressIndicator`, 2dp height, full width at bottom
- Tap anywhere (except buttons): expand to full player sheet
- Swipe right: skip to next track
- Elevation: level 2

### Bottom Sheet (Full Player)

Expands from mini player.

```
┌──────────────────────────────────────────┐
│ ─── (drag handle, 4dp × 32dp)           │
│                                          │
│         ┌──────────────────┐             │
│         │                  │             │
│         │   Album Art      │             │  240x240dp max
│         │                  │             │
│         └──────────────────┘             │
│                                          │
│        Track Title                       │  headlineSmall
│        Artist Name                       │  bodyLarge, 60% opacity
│        Album Name • 2024                 │  bodySmall
│                                          │
│  ═══════════════════░░░░░░░░░░░░░░░░░░  │  Seek bar
│  1:23                          3:42      │  bodySmall
│                                          │
│      ⏮     ◀◀     ▶⏸     ▶▶     ⏭    │  Controls (48dp each)
│                                          │
│  [Gnawa] [Morocco] [Devotional]          │  Tags/chips
│                                          │
│  Source: Spotify Preview                 │  bodySmall, 40% opacity
└──────────────────────────────────────────┘
```

### Occasion Selector

Card-based grid for choosing occasion type.

```
┌──────────────┐
│     🕌       │  Icon: 32dp
│              │
│    Nikah     │  titleMedium, centered
│   Wedding    │  bodySmall, 60% opacity
│  12 playlists│  labelSmall
└──────────────┘
```

- Card: `radius.md`, elevation 1
- Background: occasion-specific `Surface Tint` color
- Border: 2dp, occasion-specific `Primary` color (when selected)
- Grid: 2 columns (mobile), 3 (tablet), 4 (desktop)
- Card min height: 120dp

### Filter Chip Bar

Horizontal scrollable row of filter chips.

```
[✕ Gnawa] [Maghreb] [+ Add Filter]
```

- Chip height: 32dp
- Chip padding: 8dp horizontal
- Active chip: `primaryContainer` background, `onPrimaryContainer` text
- Inactive chip: `outline` border, transparent background
- Remove (✕): 18dp icon, part of chip touch target
- Add filter: dashed border, `outline` color
- Horizontal scroll with `ListView(scrollDirection: Axis.horizontal)`
- Gap between chips: 8dp

## Motion / Animation Specs

### Duration Tokens

| Token | Duration | Usage |
|-------|----------|-------|
| `motion.instant` | 0ms | No animation (layout changes) |
| `motion.micro` | 100ms | Button press feedback, icon swap |
| `motion.fast` | 150ms | Chip select, toggle, checkbox |
| `motion.standard` | 300ms | Card expand/collapse, page transition, list item appear |
| `motion.emphasis` | 500ms | Bottom sheet open/close, hero transition, player expand |
| `motion.dramatic` | 800ms | Splash/intro, occasion theme transition |

### Easing Curves

| Token | Curve | Usage |
|-------|-------|-------|
| `motion.easeOut` | `Curves.easeOutCubic` | Default for entering elements (appear, expand) |
| `motion.easeIn` | `Curves.easeInCubic` | Exiting elements (disappear, collapse) |
| `motion.easeInOut` | `Curves.easeInOutCubic` | Moving/transforming elements (reorder, slide) |
| `motion.spring` | `Curves.elasticOut` | Playful interactions (like button, bounce back) |
| `motion.linear` | `Curves.linear` | Progress bars, continuous animations |

### Specific Animation Specs

| Animation | Duration | Curve | Property | Notes |
|-----------|----------|-------|----------|-------|
| Page push transition | 300ms | easeOut | transform (slide from right) | Shared axis (Material motion) |
| Page pop transition | 300ms | easeIn | transform (slide to right) | Reverse of push |
| Mini player → full player | 500ms | easeOut | transform (expand from bottom) | Hero animation on album art |
| Track tile appear (list) | 300ms | easeOut | opacity + translateY (16dp up) | Stagger: +50ms per item, max 10 |
| Card press feedback | 100ms | easeOut | scale (1.0 → 0.98) | Release: 150ms easeOut back to 1.0 |
| Playlist reorder (drag) | 300ms | easeInOut | translateY | Other items slide to make room |
| Filter chip toggle | 150ms | easeOut | background color + scale (0.95 → 1.0) | |
| Play/pause icon swap | 100ms | easeOut | opacity cross-fade | |
| Progress bar seek | 150ms | easeOut | progress value | |
| Occasion theme transition | 800ms | easeInOut | color scheme (all tokens) | Use `AnimatedTheme` |
| Error shake | 300ms | spring | translateX (±4dp, 3 cycles) | On validation error |
| Skeleton shimmer | 1500ms | linear | gradient position (loop) | Loading placeholder |

### Reduced Motion

When `MediaQuery.of(context).disableAnimations` is true:
- All durations become `motion.instant` (0ms)
- Stagger delays removed
- Spring/elastic curves become `Curves.easeOut`
- Skeleton shimmer uses static placeholder (no animation)

## Responsive Breakpoints

| Breakpoint | Width | Navigation | Grid Columns | Player |
|------------|-------|-----------|-------------|--------|
| **Mobile** | < 600dp | Bottom nav bar (56dp) | 2 | Mini bar (64dp) |
| **Tablet** | 600–1023dp | Navigation rail (72dp, left) | 3 | Expanded mini bar |
| **Desktop** | ≥ 1024dp | Navigation drawer (256dp, left) | 4 | Full player sidebar or bar |

### Navigation Component by Breakpoint

| Breakpoint | Component | Behavior |
|------------|-----------|----------|
| Mobile | `NavigationBar` (Material 3) | Fixed bottom, 4-5 destinations |
| Tablet | `NavigationRail` | Fixed left, icons + labels on selection |
| Desktop | `NavigationDrawer` | Persistent left, icons + labels always |

### Content Layout by Breakpoint

| Breakpoint | Max Content Width | Card Behavior | Side Panels |
|------------|------------------|--------------|-------------|
| Mobile | 100% - 32dp margins | Full width stacked | None (sheets only) |
| Tablet | 100% - 48dp margins | Grid (2-3 columns) | Optional (detail pane) |
| Desktop | 960dp centered | Grid (3-4 columns) | Persistent (detail, queue, player) |

## Implementation in Flutter

### Theme Setup

```dart
// Reference implementation — frontend/lib/config/theme.dart
ThemeData buildAppTheme({
  Brightness brightness = Brightness.light,
  OccasionType? occasion,
}) {
  final colorScheme = ColorScheme.fromSeed(
    seedColor: const Color(0xFF1A237E), // Navy primary
    brightness: brightness,
  );

  // Apply occasion overlay if specified
  final effectiveColors = occasion != null
      ? colorScheme.copyWith(/* occasion-specific overrides */)
      : colorScheme;

  return ThemeData(
    useMaterial3: true,
    colorScheme: effectiveColors,
    textTheme: _buildTextTheme(brightness),
    // ... component themes
  );
}
```

### Accessing Tokens in Widgets

```dart
// Colors — ALWAYS use theme, never hardcode
final primary = Theme.of(context).colorScheme.primary;
final surface = Theme.of(context).colorScheme.surface;

// Typography — ALWAYS use theme text styles
final title = Theme.of(context).textTheme.titleMedium;
final body = Theme.of(context).textTheme.bodySmall;

// Spacing — use the spacing constants
const spacing = AppSpacing(); // or extension on BuildContext
padding: EdgeInsets.all(spacing.md), // 16dp

// Responsive — check breakpoint
final width = MediaQuery.of(context).size.width;
final isMobile = width < 600;
final isTablet = width >= 600 && width < 1024;
final isDesktop = width >= 1024;
```

### File Organization

Design system implementation files in the Flutter project:

```
frontend/lib/config/
  theme.dart              # ThemeData builder, color schemes, occasion themes
  typography.dart         # Text theme with Noto Sans + Playfair Display
  spacing.dart            # Spacing constants (AppSpacing class)
  breakpoints.dart        # Responsive breakpoint helpers
  motion.dart             # Animation duration and curve constants
  occasions.dart          # OccasionTheme extension and palette definitions
```
