import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/crate.dart';
import 'api_provider.dart';

class CrateLibraryState {
  final List<Crate> crates;
  final bool isLoading;
  final String? error;

  const CrateLibraryState({
    this.crates = const [],
    this.isLoading = false,
    this.error,
  });

  CrateLibraryState copyWith({
    List<Crate>? crates,
    bool? isLoading,
    String? Function()? error,
  }) {
    return CrateLibraryState(
      crates: crates ?? this.crates,
      isLoading: isLoading ?? this.isLoading,
      error: error != null ? error() : this.error,
    );
  }
}

class CrateLibraryNotifier extends Notifier<CrateLibraryState> {
  @override
  CrateLibraryState build() => const CrateLibraryState();

  Future<void> loadCrates() async {
    state = state.copyWith(isLoading: true, error: () => null);
    try {
      final items = await ref.read(apiClientProvider).listCrates();
      state = state.copyWith(crates: items, isLoading: false);
    } on Exception catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: () => 'Failed to load crates: $e',
      );
    }
  }

  Future<void> createCrate(String name) async {
    try {
      await ref.read(apiClientProvider).createCrate(name);
      await loadCrates();
    } on Exception catch (e) {
      state = state.copyWith(error: () => 'Failed to create crate: $e');
    }
  }

  Future<void> deleteCrate(String id) async {
    try {
      await ref.read(apiClientProvider).deleteCrate(id);
      state = state.copyWith(
        crates: state.crates.where((c) => c.id != id).toList(),
      );
    } on Exception catch (e) {
      state = state.copyWith(error: () => 'Failed to delete crate: $e');
    }
  }

  Future<void> addSetlistToCrate(String crateId, String setlistId) async {
    try {
      await ref.read(apiClientProvider).addSetlistToCrate(crateId, setlistId);
      await loadCrates();
    } on Exception catch (e) {
      state = state.copyWith(error: () => 'Failed to add to crate: $e');
    }
  }
}

final crateLibraryProvider =
    NotifierProvider<CrateLibraryNotifier, CrateLibraryState>(
        CrateLibraryNotifier.new);
