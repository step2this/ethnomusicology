import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/setlist_track.dart';
import 'api_provider.dart';

/// Consistent key for looking up a track's preview URL.
/// Handles both catalog tracks (have trackId) and LLM suggestions (trackId is null).
String previewKey(SetlistTrack track) =>
    track.trackId ?? 'unknown-${track.position}';

enum PreviewSearchStatus { loading, found, notFound, error }

class PreviewTrackInfo {
  final String? previewUrl;
  final PreviewSearchStatus status;
  final List<String> searchQueries;
  final String? source;
  final String? externalUrl;
  final String? uploaderName;

  const PreviewTrackInfo({
    this.previewUrl,
    required this.status,
    required this.searchQueries,
    this.source,
    this.externalUrl,
    this.uploaderName,
  });
}

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

  /// Prefetch preview URLs for all tracks in a setlist in parallel
  Future<void> prefetchForSetlist(List<SetlistTrack> tracks) async {
    if (tracks.isEmpty) return;

    final client = ref.read(apiClientProvider);

    // Mark all tracks as loading first
    final loadingInfo = <String, PreviewTrackInfo>{};
    for (final track in tracks) {
      final key = track.trackId ?? 'unknown-${track.position}';
      loadingInfo[key] = PreviewTrackInfo(
        status: PreviewSearchStatus.loading,
        searchQueries: ['artist:"${track.artist}" track:"${track.title}"'],
      );
    }
    state = state.copyWith(
      trackInfo: {...state.trackInfo, ...loadingInfo},
      isLoading: true,
    );

    try {
      // Build a map of trackId -> (title, artist) for fetching
      final trackEntries = <String, (String, String)>{};
      for (final track in tracks) {
        final trackId = track.trackId ?? 'unknown-${track.position}';
        trackEntries[trackId] = (track.title, track.artist);
      }

      // Fetch all preview URLs in parallel via unified search endpoint
      final results = await Future.wait(
        trackEntries.entries.map((entry) async {
          final trackId = entry.key;
          final (title, artist) = entry.value;
          try {
            final result = await client.searchPreview(title, artist);
            return MapEntry(
              trackId,
              PreviewTrackInfo(
                previewUrl: result.previewUrl,
                status: result.previewUrl != null
                    ? PreviewSearchStatus.found
                    : PreviewSearchStatus.notFound,
                searchQueries: result.searchQueries,
                source: result.source,
                externalUrl: result.externalUrl,
                uploaderName: result.uploaderName,
              ),
            );
          } on Exception catch (_) {
            return MapEntry(
              trackId,
              const PreviewTrackInfo(
                status: PreviewSearchStatus.error,
                searchQueries: [],
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
