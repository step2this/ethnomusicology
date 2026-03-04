import 'dart:js_interop';
import 'dart:async';
import 'package:flutter/foundation.dart';
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
  bool _isPaused = false;

  web.AudioBufferSourceNode? _sourceA;
  web.GainNode? _gainA;

  VoidCallback? _onTrackEnded;
  int _playGeneration = 0;

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
      if (!response.ok) {
        throw Exception('HTTP ${response.status} fetching audio from $proxyUrl');
      }
      final arrayBuffer = await response.arrayBuffer().toDart;
      final audioBuffer = await _audioCtx.decodeAudioData(arrayBuffer).toDart;
      return audioBuffer;
    } catch (e) {
      throw Exception('Failed to load audio from $proxyUrl: $e');
    }
  }

  /// Stop all currently playing sources.
  void _stopSources() {
    _onTrackEnded = null; // prevent stale ended events before stopping
    try { _sourceA?.stop(); } catch (_) {}
    try { _sourceA?.disconnect(); } catch (_) {}
    try { _gainA?.disconnect(); } catch (_) {}
    _sourceA = null;
    _gainA = null;
  }

  @override
  Future<void> loadAndPlay(String proxyUrl) async {
    _stopSources();
    final gen = ++_playGeneration;
    _isPlaying = true;
    _isPaused = false;

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

      // Set up onended callback for track finish detection
      _sourceA!.addEventListener(
        'ended',
        (web.Event event) {
          if (gen != _playGeneration) return;
          _isPlaying = false;
          _onTrackEnded?.call();
        }.toJS,
      );
    } catch (e) {
      _isPlaying = false;
      rethrow;
    }
  }

  @override
  void stop() {
    _stopSources();
    _isPlaying = false;
    _isPaused = false;
  }

  @override
  Future<void> pause() async {
    if (_audioCtxInitialized) {
      await _audioCtx.suspend().toDart;
      _isPaused = true;
    }
  }

  @override
  Future<void> resume() async {
    if (_audioCtxInitialized) {
      await _audioCtx.resume().toDart;
      _isPaused = false;
    }
  }

  @override
  bool get isPlaying => _isPlaying;

  @override
  bool get isPaused => _isPaused;

  @override
  set onTrackEnded(VoidCallback? callback) {
    _onTrackEnded = callback;
  }

  @override
  void dispose() {
    _stopSources();
    if (_audioCtxInitialized) {
      _audioCtx.close();
      _audioCtxInitialized = false;
    }
    _isPlaying = false;
    _isPaused = false;
    _onTrackEnded = null;
  }
}
