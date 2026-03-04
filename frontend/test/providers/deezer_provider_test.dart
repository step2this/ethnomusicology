import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ethnomusicology_frontend/models/setlist_track.dart';
import 'package:ethnomusicology_frontend/providers/api_provider.dart';
import 'package:ethnomusicology_frontend/providers/deezer_provider.dart';
import '../helpers/mock_api_client.dart';

void main() {
  group('DeezerPreviewNotifier', () {
    late ProviderContainer container;
    late MockInterceptor interceptor;

    setUp(() {
      final mock = createMockApiClient();
      interceptor = mock.interceptor;
      container = ProviderContainer(
        overrides: [
          apiClientProvider.overrideWithValue(mock.client),
        ],
      );
    });

    tearDown(() => container.dispose());

    test('initial state is empty', () {
      final state = container.read(deezerPreviewProvider);
      expect(state.previewUrls, isEmpty);
      expect(state.isLoading, false);
    });

    test('prefetchForSetlist fetches URLs for tracks', () async {
      // Mock Deezer search response
      interceptor.responseOverride = {
        'data': [
          {'preview': 'https://cdns-preview.dzcdn.net/stream/track1.mp3'},
        ],
      };

      final tracks = [
        const SetlistTrack(
          position: 0,
          title: 'Test Track',
          artist: 'Test Artist',
          originalPosition: 0,
          source: 'catalog',
          trackId: 'track-1',
        ),
      ];

      await container.read(deezerPreviewProvider.notifier).prefetchForSetlist(tracks);

      final state = container.read(deezerPreviewProvider);
      expect(state.isLoading, false);
      expect(state.previewUrls['track-1'], isNotNull);
    });

    test('prefetchForSetlist handles empty tracks list', () async {
      await container.read(deezerPreviewProvider.notifier).prefetchForSetlist([]);

      final state = container.read(deezerPreviewProvider);
      expect(state.previewUrls, isEmpty);
      expect(state.isLoading, false);
    });

    test('prefetchForSetlist handles API errors gracefully', () async {
      // API returns empty results — track has null preview URL
      interceptor.responseOverride = {'data': []};

      final tracks = [
        const SetlistTrack(
          position: 0,
          title: 'Missing Track',
          artist: 'Unknown',
          originalPosition: 0,
          source: 'suggestion',
        ),
      ];

      await container.read(deezerPreviewProvider.notifier).prefetchForSetlist(tracks);

      final state = container.read(deezerPreviewProvider);
      expect(state.isLoading, false);
      // Track should have null preview URL (not found)
      expect(state.previewUrls['unknown-0'], isNull);
    });

    test('reset clears all state', () async {
      interceptor.responseOverride = {
        'data': [
          {'preview': 'https://example.com/preview.mp3'},
        ],
      };

      final tracks = [
        const SetlistTrack(
          position: 0,
          title: 'Track',
          artist: 'Artist',
          originalPosition: 0,
          source: 'catalog',
          trackId: 'track-x',
        ),
      ];

      await container.read(deezerPreviewProvider.notifier).prefetchForSetlist(tracks);
      container.read(deezerPreviewProvider.notifier).reset();

      final state = container.read(deezerPreviewProvider);
      expect(state.previewUrls, isEmpty);
      expect(state.isLoading, false);
    });

    test('getPreviewUrl returns URL for known track', () {
      const state = DeezerPreviewState(
        previewUrls: {'track-1': '/api/audio/proxy?url=test'},
      );
      expect(state.getPreviewUrl('track-1'), '/api/audio/proxy?url=test');
    });

    test('getPreviewUrl returns null for unknown track', () {
      const state = DeezerPreviewState();
      expect(state.getPreviewUrl('unknown'), isNull);
    });

    test('previewKey uses trackId when available', () {
      const track = SetlistTrack(
        position: 0,
        title: 'T',
        artist: 'A',
        originalPosition: 0,
        source: 'catalog',
        trackId: 'my-track',
      );
      expect(previewKey(track), 'my-track');
    });

    test('previewKey falls back to position for suggestions', () {
      const track = SetlistTrack(
        position: 3,
        title: 'T',
        artist: 'A',
        originalPosition: 3,
        source: 'suggestion',
      );
      expect(previewKey(track), 'unknown-3');
    });
  });
}
