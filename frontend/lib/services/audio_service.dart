import 'package:flutter/foundation.dart';

/// Abstract audio playback service for cross-platform compatibility.
/// Web implementation uses package:web (Web Audio API).
/// Non-web platforms return a no-op stub.
abstract class AudioPlaybackService {
  /// Load and play a single audio URL.
  /// The URL should be a backend-proxied path (e.g., /api/audio/proxy?url=...).
  Future<void> loadAndPlay(String proxyUrl);

  /// Play crossfade between two audio URLs.
  /// Uses linear gain ramps (equal-power deferred) per UC-019 spec.
  /// [fadeDuration] in seconds (1-8).
  Future<void> playCrossfade(
    String proxyUrlA,
    String proxyUrlB,
    double fadeDuration,
  );

  /// Stop all current playback and release audio buffers.
  void stop();

  /// Pause playback without stopping or releasing resources.
  /// The pause is tied to the audio clock (respects AudioContext.suspend()).
  Future<void> pause();

  /// Resume playback from a paused state.
  /// No-op if not currently paused.
  Future<void> resume();

  /// Whether audio is currently playing.
  bool get isPlaying;

  /// Whether audio is currently paused.
  bool get isPaused;

  /// Set a callback that fires when the current track finishes playing.
  /// Used for auto-advance. The callback is tied to the audio clock
  /// (pauses when AudioContext is suspended).
  set onTrackEnded(VoidCallback? callback);

  /// Clean up AudioContext and all resources.
  void dispose();
}

/// Stub implementation for non-web platforms.
class NoOpAudioPlaybackService implements AudioPlaybackService {
  @override
  Future<void> loadAndPlay(String proxyUrl) async {}

  @override
  Future<void> playCrossfade(
    String proxyUrlA,
    String proxyUrlB,
    double fadeDuration,
  ) async {}

  @override
  void stop() {}

  @override
  Future<void> pause() async {}

  @override
  Future<void> resume() async {}

  @override
  bool get isPlaying => false;

  @override
  bool get isPaused => false;

  @override
  set onTrackEnded(VoidCallback? callback) {}

  @override
  void dispose() {}
}
