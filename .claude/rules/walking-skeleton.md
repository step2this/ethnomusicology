---
paths:
  - "docs/steel-threads/**"
  - "docs/spikes/**"
  - "docs/api/**"
  - "docs/mvp-roadmap.md"
  - "docs/mvp-progress.md"
---

# Walking Skeleton Process

Steel threads and spikes extend the Forge to de-risk frontend-backend integration before full UC implementation.

## PDCA Mapping

| PLAN | DO | CHECK | ACT/ADJUST |
|------|----|-------|------------|
| `/spike-create` (unknowns) | Implement | `/verify-uc ST-NNN` | Update parent UCs |
| `/st-create` | (agent teams, | `/grade-work ST-NNN` | Update API contract |
| `/uc-review ST-NNN` | parallel on contract) | | Update CLAUDE.md |
| `/task-decompose ST-NNN` | | | |
| `/api-contract ST-NNN` | | | |
| `design-crit` (if frontend) | | | |

## Commands

| Command | Purpose |
|---------|---------|
| `/st-create` | Create a Cockburn-format steel thread with cross-cutting fields |
| `/spike-create` | Create a time-boxed spike with hypothesis/findings/decision |
| `/api-contract` | Write/update OpenAPI section for a steel thread's endpoints |

## Artifact Locations

| Artifact | Path |
|----------|------|
| Steel threads | `docs/steel-threads/st-NNN-slug.md` |
| Spikes | `docs/spikes/sp-NNN-slug.md` |
| API contract | `docs/api/openapi.yaml` |

## Workflow

1. **Spike** unknowns first (`/spike-create`) — time-boxed, lightweight
2. **Create steel thread** (`/st-create`) — thin vertical slice across UCs
3. **Define API contract** (`/api-contract ST-NNN`) — OpenAPI single source of truth
4. **Review** (`/uc-review ST-NNN`) — devil's advocate, validates contract against assertions
5. **API Contract Review Gate** — both frontend and backend agents confirm before implementation
6. **Implement** — backend TO the contract, frontend FROM the contract. **Lead delegates, never implements solo.**
7. **Critic Review** — fresh-context agent reviews diff cold (REQUIRED before verify)
8. **Verify** (`/verify-uc ST-NNN`) — integration assertions + contract conformance
9. **Grade** (`/grade-work ST-NNN`) — Integration Proof category (20% weight)
