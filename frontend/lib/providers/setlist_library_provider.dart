import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'api_provider.dart';

class SetlistSummary {
  final String id;
  final String prompt;
  final String? name;
  final int trackCount;
  final String createdAt;

  const SetlistSummary({
    required this.id,
    required this.prompt,
    this.name,
    required this.trackCount,
    required this.createdAt,
  });

  factory SetlistSummary.fromJson(Map<String, dynamic> json) => SetlistSummary(
        id: json['id'] as String,
        prompt: json['prompt'] as String,
        name: json['name'] as String?,
        trackCount: json['track_count'] as int,
        createdAt: json['created_at'] as String,
      );
}

class SetlistLibraryState {
  final List<SetlistSummary> setlists;
  final bool isLoading;
  final String? error;

  const SetlistLibraryState({
    this.setlists = const [],
    this.isLoading = false,
    this.error,
  });

  SetlistLibraryState copyWith({
    List<SetlistSummary>? setlists,
    bool? isLoading,
    String? Function()? error,
  }) {
    return SetlistLibraryState(
      setlists: setlists ?? this.setlists,
      isLoading: isLoading ?? this.isLoading,
      error: error != null ? error() : this.error,
    );
  }
}

class SetlistLibraryNotifier extends Notifier<SetlistLibraryState> {
  @override
  SetlistLibraryState build() => const SetlistLibraryState();

  Future<void> loadSetlists() async {
    state = state.copyWith(isLoading: true, error: () => null);
    try {
      final items = await ref.read(apiClientProvider).listSetlists();
      state = state.copyWith(setlists: items, isLoading: false);
    } on Exception catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: () => 'Failed to load setlists: $e',
      );
    }
  }

  Future<void> deleteSetlist(String id) async {
    try {
      await ref.read(apiClientProvider).deleteSetlist(id);
      state = state.copyWith(
        setlists: state.setlists.where((s) => s.id != id).toList(),
      );
    } on Exception catch (e) {
      state = state.copyWith(error: () => 'Failed to delete: $e');
    }
  }

  Future<void> renameSetlist(String id, String name) async {
    try {
      await ref.read(apiClientProvider).updateSetlist(id, name: name);
      state = state.copyWith(
        setlists: state.setlists.map((s) {
          return s.id == id
              ? SetlistSummary(
                  id: s.id,
                  prompt: s.prompt,
                  name: name,
                  trackCount: s.trackCount,
                  createdAt: s.createdAt,
                )
              : s;
        }).toList(),
      );
    } on Exception catch (e) {
      state = state.copyWith(error: () => 'Failed to rename: $e');
    }
  }

  Future<void> duplicateSetlist(String id) async {
    try {
      await ref.read(apiClientProvider).duplicateSetlist(id);
      await loadSetlists();
    } on Exception catch (e) {
      state = state.copyWith(error: () => 'Failed to duplicate: $e');
    }
  }
}

final setlistLibraryProvider =
    NotifierProvider<SetlistLibraryNotifier, SetlistLibraryState>(
        SetlistLibraryNotifier.new);
