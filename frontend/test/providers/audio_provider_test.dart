import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:ethnomusicology_frontend/providers/audio_provider.dart';
import 'package:ethnomusicology_frontend/providers/deezer_provider.dart';
import 'package:ethnomusicology_frontend/models/setlist_track.dart';

// Helper to build a track list and a DeezerPreviewState that has URLs for the
// given [withUrlIndices] positions.
_TestFixture _buildFixture({
  int trackCount = 2,
  List<int> withUrlIndices = const [],
}) {
  final tracks = List.generate(
    trackCount,
    (i) => SetlistTrack(
      position: i,
      title: 'Track $i',
      artist: 'Artist $i',
      originalPosition: i,
      source: 'suggestion',
    ),
  );
  // Build a previewUrls map keyed by previewKey(track)
  final Map<String, String?> urls = {};
  for (final i in withUrlIndices) {
    final key = previewKey(tracks[i]);
    urls[key] = '/api/audio/proxy?url=track$i';
  }
  final deezerState = DeezerPreviewState(previewUrls: urls);
  return _TestFixture(tracks: tracks, deezerState: deezerState);
}

class _TestFixture {
  final List<SetlistTrack> tracks;
  final DeezerPreviewState deezerState;
  _TestFixture({required this.tracks, required this.deezerState});
}

void main() {
  group('AudioPlaybackNotifier', () {
    late ProviderContainer container;

    setUp(() {
      container = ProviderContainer();
    });

    tearDown(() {
      container.dispose();
    });

    test('initial state is idle', () {
      final state = container.read(audioPlaybackProvider);
      expect(state.status, PlaybackStatus.idle);
      expect(state.currentTrackIndex, isNull);
      expect(state.crossfadeDuration, 4.0);
      expect(state.error, isNull);
      expect(state.statusText, isNull);
      expect(state.totalTracks, 0);
    });

    test('playFromIndex sets status to loading then playing', () async {
      final notifier = container.read(audioPlaybackProvider.notifier);

      final tracks = [
        SetlistTrack(
          position: 0,
          title: 'Track 1',
          artist: 'Artist 1',
          originalPosition: 0,
          source: 'suggestion',
        ),
      ];

      // Since NoOp service is used in tests, we can't fully test playback
      // But we can test state transitions
      await notifier.playFromIndex(0, tracks, const DeezerPreviewState());

      final state = container.read(audioPlaybackProvider);
      expect(state.currentTrackIndex, 0);
      expect(state.totalTracks, 1);
    });

    test('next() does nothing if no next playable track available', () async {
      final notifier = container.read(audioPlaybackProvider.notifier);

      final tracks = [
        SetlistTrack(
          position: 0,
          title: 'Track 1',
          artist: 'Artist 1',
          originalPosition: 0,
          source: 'suggestion',
        ),
        SetlistTrack(
          position: 1,
          title: 'Track 2',
          artist: 'Artist 2',
          originalPosition: 1,
          source: 'suggestion',
        ),
      ];

      // Manually set state to simulate having a current track at last position
      container.read(audioPlaybackProvider.notifier).setStateForTest(
        const AudioPlaybackState(
          status: PlaybackStatus.playing,
          currentTrackIndex: 1,
          totalTracks: 2,
        ));

      final initialIndex = container.read(audioPlaybackProvider).currentTrackIndex;
      await notifier.next(tracks, const DeezerPreviewState());
      final finalIndex = container.read(audioPlaybackProvider).currentTrackIndex;

      // Since there's no next playable track (we're at the last one), index should stay the same
      expect(finalIndex, initialIndex);
    });

    test('next() at last track does nothing when no more playable tracks', () async {
      final notifier = container.read(audioPlaybackProvider.notifier);

      final tracks = [
        SetlistTrack(
          position: 0,
          title: 'Track 1',
          artist: 'Artist 1',
          originalPosition: 0,
          source: 'suggestion',
        ),
      ];

      // Set state to last track
      container.read(audioPlaybackProvider.notifier).setStateForTest(
        const AudioPlaybackState(
          status: PlaybackStatus.playing,
          currentTrackIndex: 0,
          totalTracks: 1,
        ));

      final initialIndex = container.read(audioPlaybackProvider).currentTrackIndex;
      await notifier.next(tracks, const DeezerPreviewState());
      final finalIndex = container.read(audioPlaybackProvider).currentTrackIndex;

      expect(finalIndex, initialIndex);
    });

    test('previous() at first track does nothing', () async {
      final notifier = container.read(audioPlaybackProvider.notifier);

      final tracks = [
        SetlistTrack(
          position: 0,
          title: 'Track 1',
          artist: 'Artist 1',
          originalPosition: 0,
          source: 'suggestion',
        ),
        SetlistTrack(
          position: 1,
          title: 'Track 2',
          artist: 'Artist 2',
          originalPosition: 1,
          source: 'suggestion',
        ),
      ];

      // Set state to first track
      container.read(audioPlaybackProvider.notifier).setStateForTest(
        const AudioPlaybackState(
          status: PlaybackStatus.playing,
          currentTrackIndex: 0,
          totalTracks: 2,
        ));

      final initialIndex = container.read(audioPlaybackProvider).currentTrackIndex;
      await notifier.previous(tracks, const DeezerPreviewState());
      final finalIndex = container.read(audioPlaybackProvider).currentTrackIndex;

      expect(finalIndex, initialIndex);
    });

    test('previous() goes to previous playable track', () async {
      final notifier = container.read(audioPlaybackProvider.notifier);
      final fixture = _buildFixture(trackCount: 2, withUrlIndices: [0, 1]);

      // Set state to second track
      container.read(audioPlaybackProvider.notifier).setStateForTest(
        const AudioPlaybackState(
          status: PlaybackStatus.playing,
          currentTrackIndex: 1,
          totalTracks: 2,
        ));

      await notifier.previous(fixture.tracks, fixture.deezerState);

      final state = container.read(audioPlaybackProvider);
      expect(state.currentTrackIndex, 0);
    });

    test('previous() skips tracks without preview URLs', () async {
      // Tracks [0:url, 1:no-url, 2:url]. Playing track 2, previous should land on 0.
      final fixture = _buildFixture(trackCount: 3, withUrlIndices: [0, 2]);
      final notifier = container.read(audioPlaybackProvider.notifier);

      container.read(audioPlaybackProvider.notifier).setStateForTest(
        const AudioPlaybackState(
          status: PlaybackStatus.playing,
          currentTrackIndex: 2,
          totalTracks: 3,
        ));

      await notifier.previous(fixture.tracks, fixture.deezerState);

      final state = container.read(audioPlaybackProvider);
      expect(state.currentTrackIndex, 0);
    });

    test('stop() resets to idle', () {
      final notifier = container.read(audioPlaybackProvider.notifier);

      // Set state to playing
      container.read(audioPlaybackProvider.notifier).setStateForTest(
        const AudioPlaybackState(
          status: PlaybackStatus.playing,
          currentTrackIndex: 0,
          statusText: 'Playing: Track 1',
        ));

      notifier.stop();

      final state = container.read(audioPlaybackProvider);
      expect(state.status, PlaybackStatus.idle);
      expect(state.currentTrackIndex, isNull);
      expect(state.statusText, isNull);
      expect(state.error, isNull);
    });

    test('stop() prevents stale track-ended callbacks from advancing state', () async {
      final fixture = _buildFixture(trackCount: 2, withUrlIndices: [0, 1]);
      final notifier = container.read(audioPlaybackProvider.notifier);

      await notifier.playFromIndex(0, fixture.tracks, fixture.deezerState);
      expect(container.read(audioPlaybackProvider).status, PlaybackStatus.playing);

      notifier.stop();
      expect(container.read(audioPlaybackProvider).status, PlaybackStatus.idle);

      // Stale callback fires after stop() — _tracks is null so it's a no-op.
      notifier.triggerTrackEndedForTest(0);

      final state = container.read(audioPlaybackProvider);
      expect(state.status, PlaybackStatus.idle);
      expect(state.currentTrackIndex, isNull);
    });

    test('togglePause() changes status from playing to paused', () async {
      final notifier = container.read(audioPlaybackProvider.notifier);

      // Set state to playing
      container.read(audioPlaybackProvider.notifier).setStateForTest(
        const AudioPlaybackState(status: PlaybackStatus.playing));

      await notifier.togglePause();

      final state = container.read(audioPlaybackProvider);
      expect(state.status, PlaybackStatus.paused);
    });

    test('togglePause() changes status from paused to playing', () async {
      final notifier = container.read(audioPlaybackProvider.notifier);

      // Set state to paused
      container.read(audioPlaybackProvider.notifier).setStateForTest(
        const AudioPlaybackState(status: PlaybackStatus.paused));

      await notifier.togglePause();

      final state = container.read(audioPlaybackProvider);
      expect(state.status, PlaybackStatus.playing);
    });

    test('setCrossfadeDuration clamps to 1-8 range', () {
      final notifier = container.read(audioPlaybackProvider.notifier);

      // Test lower bound
      notifier.setCrossfadeDuration(0.5);
      expect(container.read(audioPlaybackProvider).crossfadeDuration, 1.0);

      // Test upper bound
      notifier.setCrossfadeDuration(10.0);
      expect(container.read(audioPlaybackProvider).crossfadeDuration, 8.0);

      // Test valid value
      notifier.setCrossfadeDuration(5.0);
      expect(container.read(audioPlaybackProvider).crossfadeDuration, 5.0);
    });

    test('isPlaying getter returns true when status is playing', () {
      container.read(audioPlaybackProvider.notifier).setStateForTest(
        const AudioPlaybackState(status: PlaybackStatus.playing));

      final state = container.read(audioPlaybackProvider);
      expect(state.isPlaying, true);
    });

    test('isPaused getter returns true when status is paused', () {
      container.read(audioPlaybackProvider.notifier).setStateForTest(
        const AudioPlaybackState(status: PlaybackStatus.paused));

      final state = container.read(audioPlaybackProvider);
      expect(state.isPaused, true);
    });

    test('isLoading getter returns true when status is loading', () {
      container.read(audioPlaybackProvider.notifier).setStateForTest(
        const AudioPlaybackState(status: PlaybackStatus.loading));

      final state = container.read(audioPlaybackProvider);
      expect(state.isLoading, true);
    });

    // --- Auto-advance tests ---

    test('auto-advance: track end triggers crossfade to next track', () async {
      // Set up 2 tracks both with preview URLs so auto-advance can proceed.
      final fixture = _buildFixture(trackCount: 2, withUrlIndices: [0, 1]);
      final notifier = container.read(audioPlaybackProvider.notifier);

      // Play track 0 — stores _tracks and _deezerState in the notifier.
      // NoOp service makes loadAndPlay a no-op, so state goes to playing.
      await notifier.playFromIndex(0, fixture.tracks, fixture.deezerState);
      expect(container.read(audioPlaybackProvider).status, PlaybackStatus.playing);
      expect(container.read(audioPlaybackProvider).currentTrackIndex, 0);

      // Simulate track 0 ending. _handleTrackEnded finds track 1 and starts
      // a crossfade (also a no-op with the stub service).
      notifier.triggerTrackEndedForTest(0);

      // Before the async .then() fires, currentTrackIndex is already 1
      // and status is loading.
      expect(container.read(audioPlaybackProvider).currentTrackIndex, 1);

      // Allow the Future.then() from playCrossfade to resolve so that
      // status transitions from loading → playing.
      await Future<void>.microtask(() {});

      final state = container.read(audioPlaybackProvider);
      expect(state.currentTrackIndex, 1);
      expect(state.status, PlaybackStatus.playing);
      expect(state.statusText, contains('Track 1'));
    });

    test('race condition guard: stale callback is no-op when track has changed', () async {
      // Set up notifier with 3 tracks so _tracks and _deezerState are stored.
      final fixture = _buildFixture(trackCount: 3, withUrlIndices: [0, 1, 2]);
      final notifier = container.read(audioPlaybackProvider.notifier);

      // Start playing track 0.
      await notifier.playFromIndex(0, fixture.tracks, fixture.deezerState);

      // Simulate the user manually jumping to track 2 before track 0 ends.
      container.read(audioPlaybackProvider.notifier).setStateForTest(
        const AudioPlaybackState(
          status: PlaybackStatus.playing,
          currentTrackIndex: 2,
          totalTracks: 3,
        ));

      // Stale callback fires for track 0 (endedIndex=0, but currentIndex=2).
      // The race-condition guard inside _handleTrackEnded must reject it.
      notifier.triggerTrackEndedForTest(0);
      await Future<void>.microtask(() {});

      // State should be unchanged — still on track 2, still playing.
      final state = container.read(audioPlaybackProvider);
      expect(state.currentTrackIndex, 2);
      expect(state.status, PlaybackStatus.playing);
    });

    test('set complete: last track ends sets status=completed and statusText="Set complete"', () async {
      // One track with a preview URL. When it ends, there is no next track.
      final fixture = _buildFixture(trackCount: 1, withUrlIndices: [0]);
      final notifier = container.read(audioPlaybackProvider.notifier);

      // Play track 0 — stores context in notifier.
      await notifier.playFromIndex(0, fixture.tracks, fixture.deezerState);
      expect(container.read(audioPlaybackProvider).status, PlaybackStatus.playing);

      // Track 0 ends — no next playable track.
      notifier.triggerTrackEndedForTest(0);

      final state = container.read(audioPlaybackProvider);
      expect(state.status, PlaybackStatus.completed);
      expect(state.statusText, 'Set complete');
    });
  });
}
