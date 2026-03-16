# Tarab Studio Frontend

DJ-first music platform frontend built with Next.js.

## Stack

- Next.js 16 + React
- TanStack Query (server state)
- Zustand (client state)
- shadcn/ui + Tailwind 4
- Vitest + happy-dom (unit/component tests)
- Playwright (e2e tests)

## Development

```bash
bun --bun next dev      # Start dev server (port 3000)
bunx vitest run         # Run unit/component tests
bunx playwright test    # Run e2e tests
bun --bun next build    # Production build
```

## API

All `/api/*` requests are rewritten to the backend server (configured in `next.config.ts` via `API_URL` env var, defaults to `http://localhost:3001`).

## Structure

- `src/app/` — App Router pages
- `src/components/` — Reusable components (shadcn/ui based)
- `src/lib/` — API client, utilities
- `src/__tests__/` — Vitest tests
- `e2e/` — Playwright e2e tests
