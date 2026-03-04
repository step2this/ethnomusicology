import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/setlist_track.dart';
import '../services/audio_service.dart';
// Conditional import: uses web implementation on web, no-op stub elsewhere
import '../services/audio_service_stub.dart'
    if (dart.library.js_interop) '../services/audio_service_web.dart';
import 'deezer_provider.dart' show DeezerPreviewState, previewKey;

/// Playback status enum
enum PlaybackStatus { idle, loading, playing, paused, completed, error }

/// State for audio playback
class AudioPlaybackState {
  final PlaybackStatus status;
  final int? currentTrackIndex;
  final String? error;
  final String? statusText;
  final int totalTracks;

  const AudioPlaybackState({
    this.status = PlaybackStatus.idle,
    this.currentTrackIndex,
    this.error,
    this.statusText,
    this.totalTracks = 0,
  });

  bool get isPlaying => status == PlaybackStatus.playing;
  bool get isPaused => status == PlaybackStatus.paused;
  bool get isLoading => status == PlaybackStatus.loading;

  AudioPlaybackState copyWith({
    PlaybackStatus? status,
    int? Function()? currentTrackIndex,
    String? Function()? error,
    String? Function()? statusText,
    int Function()? totalTracks,
  }) {
    return AudioPlaybackState(
      status: status ?? this.status,
      currentTrackIndex: currentTrackIndex != null
          ? currentTrackIndex()
          : this.currentTrackIndex,
      error: error != null ? error() : this.error,
      statusText: statusText != null ? statusText() : this.statusText,
      totalTracks: totalTracks != null ? totalTracks() : this.totalTracks,
    );
  }
}

/// Notifier for managing audio playback state
class AudioPlaybackNotifier extends Notifier<AudioPlaybackState> {
  late final AudioPlaybackService _audioService;

  // Stored references for auto-advance callbacks
  List<SetlistTrack>? _tracks;
  DeezerPreviewState? _deezerState;

  @override
  AudioPlaybackState build() {
    _audioService = createAudioService();
    ref.onDispose(() {
      _tracks = null;
      _deezerState = null;
      _audioService.dispose();
    });
    return const AudioPlaybackState();
  }

  /// Play from a specific track index, setting up the auto-advance chain.
  /// Skips tracks without Deezer URLs and stops cleanly at the end.
  Future<void> playFromIndex(
    int index,
    List<SetlistTrack> tracks,
    DeezerPreviewState deezerState,
  ) async {
    if (index < 0 || index >= tracks.length) return;

    _tracks = tracks;
    _deezerState = deezerState;

    state = state.copyWith(
      status: PlaybackStatus.loading,
      currentTrackIndex: () => index,
      totalTracks: () => tracks.length,
      error: () => null,
      statusText: () => null,
    );

    try {
      await _playCurrentTrack(tracks, deezerState);
    } catch (e) {
      state = state.copyWith(
        status: PlaybackStatus.error,
        error: () => 'Failed to play: $e',
      );
    }
  }

  /// Internal: play the track at the current index and wire up the
  /// auto-advance callback with a race-condition guard.
  Future<void> _playCurrentTrack(
    List<SetlistTrack> tracks,
    DeezerPreviewState deezerState,
  ) async {
    final currentIndex = state.currentTrackIndex;
    if (currentIndex == null || currentIndex < 0 || currentIndex >= tracks.length) {
      return;
    }

    final track = tracks[currentIndex];
    final previewUrl = deezerState.getPreviewUrl(previewKey(track));

    if (previewUrl == null) {
      state = state.copyWith(
        error: () => 'No preview available for this track',
        status: PlaybackStatus.error,
      );
      return;
    }

    // Capture the expected index at the time this track starts playing.
    // The callback uses this to guard against races when the user manually
    // jumps to a different track before this one ends (M7).
    final expectedIndex = currentIndex;
    _audioService.onTrackEnded = () => _handleTrackEnded(expectedIndex, tracks, deezerState);

    await _audioService.loadAndPlay(previewUrl);
    state = state.copyWith(
      status: PlaybackStatus.playing,
      statusText: () => '${track.title} by ${track.artist}',
      error: () => null,
    );
  }

  /// Called when the current track finishes playing.
  /// Advances to the next playable track, or marks the set complete.
  /// [endedIndex] is a race-condition guard — stale callbacks are no-ops (M7).
  void _handleTrackEnded(int endedIndex, List<SetlistTrack> tracks, DeezerPreviewState deezerState) {
    if (state.currentTrackIndex != endedIndex) return;
    final nextIndex = _findNextPlayableTrack(endedIndex, tracks, deezerState);
    if (nextIndex == null) {
      state = state.copyWith(status: PlaybackStatus.completed, statusText: () => 'Set complete');
      return;
    }
    state = state.copyWith(
      status: PlaybackStatus.loading,
      currentTrackIndex: () => nextIndex,
      statusText: () => 'Loading track ${nextIndex + 1}...',
    );
    _playCurrentTrack(tracks, deezerState).catchError((e) {
      state = state.copyWith(status: PlaybackStatus.error, statusText: () => 'Playback error');
    });
  }

  /// Find the next playable track starting from [fromIndex].
  /// Returns null if no playable tracks remain.
  int? _findNextPlayableTrack(
    int fromIndex,
    List<SetlistTrack> tracks,
    DeezerPreviewState deezerState,
  ) {
    for (int i = fromIndex + 1; i < tracks.length; i++) {
      final previewUrl = deezerState.getPreviewUrl(previewKey(tracks[i]));
      if (previewUrl != null) {
        return i;
      }
    }
    return null;
  }

  /// Find the previous playable track before [fromIndex].
  /// Returns null if no playable tracks exist before [fromIndex].
  int? _findPreviousPlayableTrack(
    int fromIndex,
    List<SetlistTrack> tracks,
    DeezerPreviewState deezerState,
  ) {
    for (int i = fromIndex - 1; i >= 0; i--) {
      final previewUrl = deezerState.getPreviewUrl(previewKey(tracks[i]));
      if (previewUrl != null) {
        return i;
      }
    }
    return null;
  }

  /// Skip to the next playable track
  Future<void> next(
    List<SetlistTrack> tracks,
    DeezerPreviewState deezerState,
  ) async {
    _tracks = tracks;
    _deezerState = deezerState;

    final currentIndex = state.currentTrackIndex;
    if (currentIndex == null) return;

    final nextIndex = _findNextPlayableTrack(currentIndex, tracks, deezerState);
    if (nextIndex == null) {
      return;
    }

    state = state.copyWith(
      status: PlaybackStatus.loading,
      currentTrackIndex: () => nextIndex,
      error: () => null,
    );

    try {
      await _playCurrentTrack(tracks, deezerState);
    } catch (e) {
      state = state.copyWith(
        status: PlaybackStatus.error,
        error: () => 'Failed to play next: $e',
      );
    }
  }

  /// Go back to the previous track
  Future<void> previous(
    List<SetlistTrack> tracks,
    DeezerPreviewState deezerState,
  ) async {
    _tracks = tracks;
    _deezerState = deezerState;

    final currentIndex = state.currentTrackIndex;
    if (currentIndex == null) return;

    final prevIndex = _findPreviousPlayableTrack(currentIndex, tracks, deezerState);
    if (prevIndex == null) return;

    state = state.copyWith(
      status: PlaybackStatus.loading,
      currentTrackIndex: () => prevIndex,
      error: () => null,
    );

    try {
      await _playCurrentTrack(tracks, deezerState);
    } catch (e) {
      state = state.copyWith(
        status: PlaybackStatus.error,
        error: () => 'Failed to play previous: $e',
      );
    }
  }

  /// Toggle pause/resume
  Future<void> togglePause() async {
    if (state.status == PlaybackStatus.playing) {
      await _audioService.pause();
      state = state.copyWith(status: PlaybackStatus.paused);
    } else if (state.status == PlaybackStatus.paused) {
      await _audioService.resume();
      state = state.copyWith(status: PlaybackStatus.playing);
    }
  }

  /// Stop all playback and reset to idle
  void stop() {
    _audioService.stop();
    _audioService.onTrackEnded = null;
    _tracks = null;
    _deezerState = null;
    state = AudioPlaybackState(
      status: PlaybackStatus.idle,
      currentTrackIndex: null,
      error: null,
      statusText: null,
      totalTracks: state.totalTracks,
    );
  }

  /// Trigger the track-ended logic for testing purposes.
  /// Simulates the audio service firing its onTrackEnded callback for
  /// [endedIndex], allowing tests to verify auto-advance state transitions
  /// without needing a real audio service.
  @visibleForTesting
  void triggerTrackEndedForTest(int endedIndex) {
    final tracks = _tracks;
    final deezerState = _deezerState;
    if (tracks == null || deezerState == null) return;
    _handleTrackEnded(endedIndex, tracks, deezerState);
  }

  /// Set state directly for testing purposes.
  @visibleForTesting
  void setStateForTest(AudioPlaybackState newState) {
    state = newState;
  }
}

/// Riverpod provider for audio playback
final audioPlaybackProvider =
    NotifierProvider<AudioPlaybackNotifier, AudioPlaybackState>(AudioPlaybackNotifier.new);
