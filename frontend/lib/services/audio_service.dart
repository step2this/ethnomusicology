/// Abstract audio playback service for cross-platform compatibility.
/// Web implementation uses package:web (Web Audio API).
/// Non-web platforms return a no-op stub.
abstract class AudioPlaybackService {
  /// Load and play a single audio URL.
  /// The URL should be a backend-proxied path (e.g., /api/audio/proxy?url=...).
  Future<void> loadAndPlay(String proxyUrl);

  /// Play crossfade between two audio URLs.
  /// Uses equal-power curves (cos/sin) per UC-019 spec.
  /// [fadeDuration] in seconds (1-8).
  Future<void> playCrossfade(
    String proxyUrlA,
    String proxyUrlB,
    double fadeDuration,
  );

  /// Stop all current playback and release audio buffers.
  void stop();

  /// Whether audio is currently playing.
  bool get isPlaying;

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
  bool get isPlaying => false;

  @override
  void dispose() {}
}
