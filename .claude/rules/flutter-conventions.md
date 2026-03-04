---
description: Flutter/Dart conventions for the ethnomusicology frontend
globs: frontend/**
---

# Flutter Conventions

## Riverpod 2.x Patterns
- Use `Notifier` + `NotifierProvider` (not StateNotifier, not code-gen)
- State classes use `copyWith()` with nullable function params for nullable fields: `String? Function()? error`
- Provider declarations at file bottom: `final xProvider = NotifierProvider<XNotifier, XState>(XNotifier.new);`
- Access: `ref.watch(provider)` in build, `ref.read(provider.notifier).method()` for actions
- Cross-provider reads via `ref.read(otherProvider)` or `ref.read(otherProvider.notifier)`

## Model Conventions
- Models in `lib/models/`, one class per concept (can share file if tightly coupled)
- `fromJson` factory constructors, snake_case JSON keys
- Nullable num parsing: `(json['field'] as num?)?.toDouble()` or `.toInt()`
- No `toJson` unless needed for serialization
- Import shared types (e.g., `SetlistTrack`) rather than duplicating

## Widget Conventions
- Widgets in `lib/widgets/`, screens in `lib/screens/`
- `ConsumerWidget` when reading providers, `ConsumerStatefulWidget` when needing controllers
- Pass data via constructor, not by reading providers in leaf widgets (exception: action callbacks)
- Use Material 3 theme tokens (`Theme.of(context).colorScheme.*`)

## API Client Pattern
- Single `ApiClient` class in `lib/services/api_client.dart` using Dio
- Methods return typed models, throw DioException on error
- Provided via `apiClientProvider` (see `lib/providers/api_provider.dart`)

## Test Conventions
- Tests in `test/` mirroring `lib/` structure
- Use `createMockApiClient()` from `test/helpers/mock_api_client.dart` for API tests
- Provider tests: create `ProviderContainer` with overrides, don't use widget testing
- Group tests by method/feature with `group()`

## Error Handling
- Parse API errors in provider `_parseError()` methods
- Backend error format: `{"error": {"code": "...", "message": "..."}}`
- Map error codes to user-friendly messages
