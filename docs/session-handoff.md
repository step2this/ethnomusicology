# Session Handoff — 2026-03-04

## Active Work: Flutter Architecture Refactor

### Branch: `feature/flutter-arch-refactor` (branched from main @ 319c498)

### What This Session Is Doing
- Removing 9 unused Flutter dependencies
- Migrating all 6 providers: StateNotifier → Notifier (Riverpod 2.x Notifier API)
- Decomposing 712-line god widget (SetlistGenerationScreen) into 3 focused widgets
- Fixing DI inconsistencies, error handling, theme colors, route constants
- Adding missing test coverage (4 new test files)

### Files Being Modified (DO NOT TOUCH)
**Frontend only — backend is completely unaffected.**
- `frontend/pubspec.yaml`
- `frontend/lib/providers/*.dart` (all 6 providers)
- `frontend/lib/screens/setlist_generation_screen.dart`
- `frontend/lib/widgets/` (new files: setlist_input_form.dart, transport_controls.dart, setlist_result_view.dart)
- `frontend/lib/config/routes.dart`, `frontend/lib/config/constants.dart` (new)
- `frontend/lib/services/api_client.dart` (minor)
- `frontend/lib/models/track.dart` (minor), `frontend/lib/models/occasion.dart` (delete)
- `frontend/test/providers/*.dart`, `frontend/test/helpers/` (new)
- `frontend/web/index.html`, `frontend/web/manifest.json`

### What Other Claudes CAN Work On (non-overlapping)
- **Docs**: `docs/use-cases/`, `docs/steel-threads/`, `docs/spikes/` (not session-handoff.md)
- **CI/CD**: `.github/workflows/`, Playwright tests in `e2e/`
- **Design**: `.design-crit/`
- **Infrastructure**: AWS, deployment, monitoring

### Parallel Session: ST-007 Backend (Conversational Refinement)
- **Branch**: `feature/st-007-conversational-refinement` (worktree, from main)
- **Scope**: Backend-only — 6 new files in `backend/src/`, 1 migration, 1 integration test
- **Plan**: `docs/tasks/st-007-conversational-refinement-backend.md`
- **DO NOT TOUCH**: `backend/src/api/claude.rs` (being modified for `converse()` method)

### What Is BLOCKED Until This Completes
- Any Flutter frontend work (we're touching most files)
- Any work that depends on Riverpod provider API (changing from StateNotifier to Notifier)

### Current Phase
Phase 0 complete. Executing Phases 1-5 with builder agents.

### Git Strategy
- Branch: `feature/flutter-arch-refactor` from main
- Will create PR when complete
- No rebase needed (branched from clean main after PR #4 merge)
