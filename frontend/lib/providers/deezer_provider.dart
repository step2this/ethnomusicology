import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/setlist_track.dart';
import 'api_provider.dart';

/// Consistent key for looking up a track's Deezer preview URL.
/// Handles both catalog tracks (have trackId) and LLM suggestions (trackId is null).
String previewKey(SetlistTrack track) =>
    track.trackId ?? 'unknown-${track.position}';

enum PreviewSearchStatus { loading, found, notFound, error }

class PreviewTrackInfo {
  final String? previewUrl;
  final PreviewSearchStatus status;
  final String searchQuery;

  const PreviewTrackInfo({
    this.previewUrl,
    required this.status,
    required this.searchQuery,
  });
}

// State for Deezer preview URLs
class PreviewState {
  final Map<String, PreviewTrackInfo> trackInfo;
  final bool isLoading;

  const PreviewState({
    this.trackInfo = const {},
    this.isLoading = false,
  });

  PreviewState copyWith({
    Map<String, PreviewTrackInfo>? trackInfo,
    bool? isLoading,
  }) {
    return PreviewState(
      trackInfo: trackInfo ?? this.trackInfo,
      isLoading: isLoading ?? this.isLoading,
    );
  }

  /// Get preview URL for a specific track key, or null if not found/loading
  String? getPreviewUrl(String key) => trackInfo[key]?.previewUrl;

  /// Check if we have a non-null preview URL for a track
  bool hasPreview(String key) => trackInfo[key]?.previewUrl != null;
}

class PreviewNotifier extends Notifier<PreviewState> {
  @override
  PreviewState build() => const PreviewState();

  /// Prefetch Deezer preview URLs for all tracks in a setlist in parallel
  Future<void> prefetchForSetlist(List<SetlistTrack> tracks) async {
    if (tracks.isEmpty) return;

    final client = ref.read(apiClientProvider);

    // Mark all tracks as loading first
    final loadingInfo = <String, PreviewTrackInfo>{};
    for (final track in tracks) {
      final key = track.trackId ?? 'unknown-${track.position}';
      final query = 'artist:"${track.artist}" track:"${track.title}"';
      loadingInfo[key] = PreviewTrackInfo(
        status: PreviewSearchStatus.loading,
        searchQuery: query,
      );
    }
    state = state.copyWith(
      trackInfo: {...state.trackInfo, ...loadingInfo},
      isLoading: true,
    );

    try {
      // Build a map of trackId -> (title, artist, query) for fetching
      final trackEntries = <String, (String, String, String)>{};
      for (final track in tracks) {
        final trackId = track.trackId ?? 'unknown-${track.position}';
        final query = 'artist:"${track.artist}" track:"${track.title}"';
        trackEntries[trackId] = (track.title, track.artist, query);
      }

      // Fetch all Deezer URLs in parallel
      final results = await Future.wait(
        trackEntries.entries.map((entry) async {
          final trackId = entry.key;
          final (title, artist, query) = entry.value;
          try {
            final previewUrl = await client.searchDeezerPreview(title, artist);
            return MapEntry(
              trackId,
              PreviewTrackInfo(
                previewUrl: previewUrl,
                status: previewUrl != null
                    ? PreviewSearchStatus.found
                    : PreviewSearchStatus.notFound,
                searchQuery: query,
              ),
            );
          } on Exception catch (_) {
            return MapEntry(
              trackId,
              PreviewTrackInfo(
                status: PreviewSearchStatus.error,
                searchQuery: query,
              ),
            );
          }
        }),
      );

      final newInfo = Map<String, PreviewTrackInfo>.fromEntries(results);
      state = state.copyWith(
        trackInfo: {...state.trackInfo, ...newInfo},
        isLoading: false,
      );
    } on Exception catch (_) {
      state = state.copyWith(isLoading: false);
    }
  }

  /// Clear all cached preview URLs
  void reset() {
    state = const PreviewState();
  }
}

final previewProvider =
    NotifierProvider<PreviewNotifier, PreviewState>(
        PreviewNotifier.new);
