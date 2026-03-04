import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/setlist_track.dart';
import 'api_provider.dart';

/// Consistent key for looking up a track's Deezer preview URL.
/// Handles both catalog tracks (have trackId) and LLM suggestions (trackId is null).
String previewKey(SetlistTrack track) =>
    track.trackId ?? 'unknown-${track.position}';

// State for Deezer preview URLs
class DeezerPreviewState {
  final Map<String, String?> previewUrls;
  final bool isLoading;

  const DeezerPreviewState({
    this.previewUrls = const {},
    this.isLoading = false,
  });

  DeezerPreviewState copyWith({
    Map<String, String?>? previewUrls,
    bool? isLoading,
  }) {
    return DeezerPreviewState(
      previewUrls: previewUrls ?? this.previewUrls,
      isLoading: isLoading ?? this.isLoading,
    );
  }

  /// Get preview URL for a specific track by ID, or null if not found/loading
  String? getPreviewUrl(String trackId) => previewUrls[trackId];

  /// Check if we have a preview URL cached for a track
  bool hasPreview(String trackId) => previewUrls.containsKey(trackId);
}

class DeezerPreviewNotifier extends Notifier<DeezerPreviewState> {
  @override
  DeezerPreviewState build() => const DeezerPreviewState();

  /// Prefetch Deezer preview URLs for all tracks in a setlist in parallel
  Future<void> prefetchForSetlist(List<SetlistTrack> tracks) async {
    if (tracks.isEmpty) return;

    final client = ref.read(apiClientProvider);
    state = state.copyWith(isLoading: true);

    try {
      // Build a map of trackId -> (title, artist) for fetching
      final trackMap = <String, (String, String)>{};
      for (final track in tracks) {
        final trackId = track.trackId ?? 'unknown-${track.position}';
        trackMap[trackId] = (track.title, track.artist);
      }

      // Fetch all Deezer URLs in parallel
      final results = await Future.wait(
        trackMap.entries.map((entry) async {
          final trackId = entry.key;
          final (title, artist) = entry.value;
          try {
            final previewUrl = await client.searchDeezerPreview(title, artist);
            return MapEntry(trackId, previewUrl);
          } on Exception catch (_) {
            return MapEntry(trackId, null);
          }
        }),
      );

      // Update state with all results
      final newUrls = Map<String, String?>.fromEntries(results);
      state = state.copyWith(
        previewUrls: {...state.previewUrls, ...newUrls},
        isLoading: false,
      );
    } on Exception catch (_) {
      state = state.copyWith(isLoading: false);
    }
  }

  /// Clear all cached preview URLs
  void reset() {
    state = const DeezerPreviewState();
  }
}

final deezerPreviewProvider =
    NotifierProvider<DeezerPreviewNotifier, DeezerPreviewState>(
        DeezerPreviewNotifier.new);
