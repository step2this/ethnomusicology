import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/setlist_track.dart';
import '../services/audio_service.dart';
// Conditional import: uses web implementation on web, no-op stub elsewhere
import '../services/audio_service_stub.dart'
    if (dart.library.js_interop) '../services/audio_service_web.dart';
import 'deezer_provider.dart' show DeezerPreviewState, previewKey;

/// State for audio playback
class AudioPlaybackState {
  final bool isPlaying;
  final bool isLoading;
  final int? currentTrackIndex;
  final double crossfadeDuration;
  final String? error;
  final String? statusText;

  const AudioPlaybackState({
    this.isPlaying = false,
    this.isLoading = false,
    this.currentTrackIndex,
    this.crossfadeDuration = 4.0,
    this.error,
    this.statusText,
  });

  AudioPlaybackState copyWith({
    bool? isPlaying,
    bool? isLoading,
    int? Function()? currentTrackIndex,
    double? crossfadeDuration,
    String? Function()? error,
    String? Function()? statusText,
  }) {
    return AudioPlaybackState(
      isPlaying: isPlaying ?? this.isPlaying,
      isLoading: isLoading ?? this.isLoading,
      currentTrackIndex: currentTrackIndex != null
          ? currentTrackIndex()
          : this.currentTrackIndex,
      crossfadeDuration: crossfadeDuration ?? this.crossfadeDuration,
      error: error != null ? error() : this.error,
      statusText: statusText != null ? statusText() : this.statusText,
    );
  }
}

/// Notifier for managing audio playback state
class AudioPlaybackNotifier extends StateNotifier<AudioPlaybackState> {
  late final AudioPlaybackService _audioService;

  AudioPlaybackNotifier() : super(const AudioPlaybackState()) {
    _audioService = createAudioService();
  }

  /// Play a single track with preview URL from Deezer
  Future<void> playTrack(
    int index,
    List<SetlistTrack> tracks,
    DeezerPreviewState deezerState,
  ) async {
    if (index < 0 || index >= tracks.length) return;

    final track = tracks[index];
    final previewUrl = deezerState.getPreviewUrl(previewKey(track));

    if (previewUrl == null) {
      state = state.copyWith(
        error: () => 'No preview available',
        isPlaying: false,
      );
      return;
    }

    state = state.copyWith(
      isLoading: true,
      currentTrackIndex: () => index,
      error: () => null,
    );

    try {
      await _audioService.loadAndPlay(previewUrl);
      state = state.copyWith(
        isPlaying: true,
        isLoading: false,
        statusText: () => '${track.title} by ${track.artist}',
      );
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: () => 'Failed to play: $e',
        isPlaying: false,
      );
    }
  }

  /// Play a crossfade between two tracks
  Future<void> playCrossfade(
    int indexA,
    int indexB,
    List<SetlistTrack> tracks,
    DeezerPreviewState deezerState,
  ) async {
    if (indexA < 0 || indexA >= tracks.length) return;
    if (indexB < 0 || indexB >= tracks.length) return;

    final trackA = tracks[indexA];
    final trackB = tracks[indexB];

    final urlA = deezerState.getPreviewUrl(previewKey(trackA));
    final urlB = deezerState.getPreviewUrl(previewKey(trackB));

    if (urlA == null || urlB == null) {
      state = state.copyWith(
        error: () => 'Preview not available for one or both tracks',
        isPlaying: false,
      );
      return;
    }

    state = state.copyWith(
      isLoading: true,
      currentTrackIndex: () => indexA,
      error: () => null,
    );

    try {
      await _audioService.playCrossfade(
        urlA,
        urlB,
        state.crossfadeDuration,
      );
      state = state.copyWith(
        isPlaying: true,
        isLoading: false,
        statusText: () => '${trackA.title} → ${trackB.title}',
      );
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        error: () => 'Crossfade failed: $e',
        isPlaying: false,
      );
    }
  }

  /// Stop all playback
  void stop() {
    _audioService.stop();
    state = state.copyWith(
      isPlaying: false,
      isLoading: false,
      currentTrackIndex: () => null,
      statusText: () => null,
      error: () => null,
    );
  }

  /// Update crossfade duration
  void setCrossfadeDuration(double duration) {
    state = state.copyWith(crossfadeDuration: duration.clamp(1.0, 8.0));
  }

  /// Dispose audio service resources
  @override
  void dispose() {
    _audioService.dispose();
    super.dispose();
  }
}
/// Riverpod provider for audio playback
final audioPlaybackProvider =
    StateNotifierProvider<AudioPlaybackNotifier, AudioPlaybackState>((ref) {
  return AudioPlaybackNotifier();
});
