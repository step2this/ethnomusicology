import 'dart:js_interop';
import 'dart:async';
import 'package:web/web.dart' as web;
import 'audio_service.dart';

/// Factory for creating the web AudioPlaybackService.
/// Used by conditional import in audio_provider.dart.
AudioPlaybackService createAudioService() => WebAudioPlaybackService();

/// Web implementation of AudioPlaybackService using Web Audio API.
/// Uses package:web and dart:js_interop for cross-browser compatibility.
class WebAudioPlaybackService implements AudioPlaybackService {
  late web.AudioContext _audioCtx;
  bool _audioCtxInitialized = false;
  bool _isPlaying = false;

  web.AudioBufferSourceNode? _sourceA;
  web.AudioBufferSourceNode? _sourceB;
  web.GainNode? _gainA;
  web.GainNode? _gainB;

  Timer? _autoStopTimer;

  /// Initialize AudioContext on first use (requires user gesture).
  void _initAudioContext() {
    if (_audioCtxInitialized) return;
    _audioCtx = web.AudioContext();
    _audioCtxInitialized = true;
  }

  /// Fetch and decode audio from a backend-proxied URL.
  Future<web.AudioBuffer> _loadAudio(String proxyUrl) async {
    _initAudioContext();

    try {
      final response = await web.window.fetch(proxyUrl.toJS).toDart;
      final arrayBuffer = await response.arrayBuffer().toDart;
      final audioBuffer = await _audioCtx.decodeAudioData(arrayBuffer).toDart;
      return audioBuffer;
    } catch (e) {
      throw Exception('Failed to load audio from $proxyUrl: $e');
    }
  }

  /// Stop all currently playing sources.
  void _stopSources() {
    try {
      _sourceA?.stop();
    } catch (e) {
      // Source may have already stopped
    }
    try {
      _sourceB?.stop();
    } catch (e) {
      // Source may have already stopped
    }

    _sourceA = null;
    _sourceB = null;
    _gainA = null;
    _gainB = null;

    _autoStopTimer?.cancel();
    _autoStopTimer = null;
  }

  @override
  Future<void> loadAndPlay(String proxyUrl) async {
    _stopSources();
    _isPlaying = true;

    try {
      final buffer = await _loadAudio(proxyUrl);

      _initAudioContext();
      _sourceA = _audioCtx.createBufferSource();
      _sourceA!.buffer = buffer;

      _gainA = _audioCtx.createGain();
      _sourceA!.connect(_gainA!);
      _gainA!.connect(_audioCtx.destination);

      final now = _audioCtx.currentTime;
      _sourceA!.start(now);

      // Auto-stop after buffer duration
      final durationMs = (buffer.duration * 1000).toInt();
      _autoStopTimer = Timer(Duration(milliseconds: durationMs), () {
        _isPlaying = false;
      });
    } catch (e) {
      _isPlaying = false;
      rethrow;
    }
  }

  @override
  Future<void> playCrossfade(
    String proxyUrlA,
    String proxyUrlB,
    double fadeDuration,
  ) async {
    _stopSources();
    _isPlaying = true;

    try {
      // Load both audio buffers in parallel
      final [bufferA, bufferB] = await Future.wait([
        _loadAudio(proxyUrlA),
        _loadAudio(proxyUrlB),
      ]);

      _initAudioContext();

      // Create sources and gains for both tracks
      _sourceA = _audioCtx.createBufferSource();
      _sourceA!.buffer = bufferA;
      _gainA = _audioCtx.createGain();
      _sourceA!.connect(_gainA!);
      _gainA!.connect(_audioCtx.destination);

      _sourceB = _audioCtx.createBufferSource();
      _sourceB!.buffer = bufferB;
      _gainB = _audioCtx.createGain();
      _sourceB!.connect(_gainB!);
      _gainB!.connect(_audioCtx.destination);

      final now = _audioCtx.currentTime;

      // Calculate start positions
      // Track A: play last fadeDuration seconds
      final bufferADurationDouble = bufferA.duration;
      final startA = (bufferADurationDouble - fadeDuration - 2).clamp(0.0, bufferADurationDouble);
      final fadeStartTime = now + 2; // 2s of Track A solo, then fade begins

      // Equal-power crossfade using linear gain curves
      // Track A: full volume for 2s, then fade out over fadeDuration
      _gainA!.gain.setValueAtTime(1.0, now);
      _gainA!.gain.setValueAtTime(1.0, fadeStartTime);
      _gainA!.gain.linearRampToValueAtTime(0.0, fadeStartTime + fadeDuration);

      // Track B: silent for 2s, then fade in over fadeDuration
      _gainB!.gain.setValueAtTime(0.0, now);
      _gainB!.gain.setValueAtTime(0.0, fadeStartTime);
      _gainB!.gain.linearRampToValueAtTime(1.0, fadeStartTime + fadeDuration);

      // Start playback
      _sourceA!.start(now, startA);
      _sourceB!.start(fadeStartTime);

      // Calculate total playback time: 2s solo A + fade + 3s solo B
      final totalTime = 2 + fadeDuration + 3;

      // Auto-stop after playback complete
      _autoStopTimer =
          Timer(Duration(milliseconds: (totalTime * 1000).toInt()), () {
        _isPlaying = false;
      });
    } catch (e) {
      _isPlaying = false;
      rethrow;
    }
  }

  @override
  void stop() {
    _stopSources();
    _isPlaying = false;
  }

  @override
  bool get isPlaying => _isPlaying;

  @override
  void dispose() {
    _stopSources();
    if (_audioCtxInitialized) {
      _audioCtx.close();
      _audioCtxInitialized = false;
    }
    _isPlaying = false;
  }
}
