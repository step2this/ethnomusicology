import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/track.dart';
import 'api_provider.dart';

// State for the track catalog
class TrackCatalogState {
  final List<Track> tracks;
  final int currentPage;
  final int totalPages;
  final int total;
  final String sort;
  final String order;
  final bool isLoading;
  final String? error;

  const TrackCatalogState({
    this.tracks = const [],
    this.currentPage = 0,
    this.totalPages = 0,
    this.total = 0,
    this.sort = 'date_added',
    this.order = 'desc',
    this.isLoading = false,
    this.error,
  });

  TrackCatalogState copyWith({
    List<Track>? tracks,
    int? currentPage,
    int? totalPages,
    int? total,
    String? sort,
    String? order,
    bool? isLoading,
    String? error,
  }) {
    return TrackCatalogState(
      tracks: tracks ?? this.tracks,
      currentPage: currentPage ?? this.currentPage,
      totalPages: totalPages ?? this.totalPages,
      total: total ?? this.total,
      sort: sort ?? this.sort,
      order: order ?? this.order,
      isLoading: isLoading ?? this.isLoading,
      error: error,
    );
  }

  bool get hasMore => currentPage < totalPages;
}

class TrackCatalogNotifier extends Notifier<TrackCatalogState> {
  @override
  TrackCatalogState build() => const TrackCatalogState();

  Future<void> loadFirstPage() async {
    state = state.copyWith(isLoading: true, error: null, tracks: []);
    try {
      final response = await ref.read(apiClientProvider).listTracks(
        page: 1,
        sort: state.sort,
        order: state.order,
      );
      state = state.copyWith(
        tracks: response.data,
        currentPage: response.page,
        totalPages: response.totalPages,
        total: response.total,
        isLoading: false,
      );
    } on Exception catch (_) {
      state = state.copyWith(
        isLoading: false,
        error: 'Failed to load tracks. Please try again.',
      );
    }
  }

  Future<void> loadNextPage() async {
    if (!state.hasMore || state.isLoading) return;

    state = state.copyWith(isLoading: true, error: null);
    try {
      final response = await ref.read(apiClientProvider).listTracks(
        page: state.currentPage + 1,
        sort: state.sort,
        order: state.order,
      );
      state = state.copyWith(
        tracks: [...state.tracks, ...response.data],
        currentPage: response.page,
        totalPages: response.totalPages,
        total: response.total,
        isLoading: false,
      );
    } on Exception catch (_) {
      state = state.copyWith(
        isLoading: false,
        error: 'Failed to load more tracks.',
      );
    }
  }

  Future<void> setSort(String sort, String order) async {
    state = TrackCatalogState(sort: sort, order: order);
    await loadFirstPage();
  }

  Future<void> retry() => loadFirstPage();
}

final trackCatalogProvider =
    NotifierProvider<TrackCatalogNotifier, TrackCatalogState>(
        TrackCatalogNotifier.new);
