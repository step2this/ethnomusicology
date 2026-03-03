import 'audio_service.dart';

/// Factory for creating the platform-appropriate AudioPlaybackService.
/// This stub returns NoOp — overridden by audio_service_web.dart on web.
AudioPlaybackService createAudioService() => NoOpAudioPlaybackService();
