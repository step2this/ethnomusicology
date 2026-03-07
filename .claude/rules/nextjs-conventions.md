---
paths:
  - "frontend-next/**"
---

# Next.js Conventions

## Stack
- Next.js 16 (App Router, Turbopack) with bun runtime
- TypeScript strict mode, Tailwind CSS 4, shadcn/ui (Radix primitives)
- TanStack Query v5 (server state), Zustand v5 (client state)
- Vitest + React Testing Library + MSW for testing, Playwright for e2e

## State Architecture
- **Server state** (API data): TanStack Query hooks in `src/hooks/use-*.ts`
- **Client state** (UI, playback, ephemeral): Zustand stores in `src/stores/*.ts`
- Never mix: TanStack Query manages cache/refetch, Zustand manages local UI state

## Audio Singleton
- Plain TypeScript module (`src/lib/audio-service.ts`) outside React tree
- Zustand store (`src/stores/playback-store.ts`) exposes state to components
- Survives React strict mode double-mount — no useEffect for audio lifecycle

## File Structure
```
src/
  app/           # Next.js App Router pages
  components/    # Reusable UI components
  hooks/         # TanStack Query hooks (use-setlist.ts, use-tracks.ts)
  stores/        # Zustand stores (playback-store.ts)
  lib/           # Utilities, API client, audio service
  types/         # TypeScript interfaces
```

## Component Patterns
- All pages are `'use client'` (SPA pattern — no SSR for this app)
- Use shadcn/ui components; customize via Tailwind, not inline styles
- Theme: dark mode default, gold (#D4AF37) + navy (#1B2A4A) brand tokens

## API Client
- Typed fetch wrapper in `src/lib/api-client.ts`
- Per-endpoint timeouts via AbortController (120s generate, 10s default)
- Error parsing for `{"error": {"code", "message"}}` format

## Testing
- Tests in `__tests__/` or colocated `*.test.ts(x)` files
- MSW handlers in `src/__mocks__/handlers.ts` for API mocking
- `vitest` for unit/component, `playwright` for e2e
- Use `@testing-library/react` `render` + `screen` patterns

## Scripts (bun)
```json
"dev": "bun --bun next dev",
"build": "bun --bun next build",
"start": "bun --bun next start",
"test": "vitest",
"test:e2e": "playwright test"
```
