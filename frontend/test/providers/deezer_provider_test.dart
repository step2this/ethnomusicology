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
      expect(state.trackInfo, isEmpty);
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
      expect(state.trackInfo['track-1']?.previewUrl, isNotNull);
      expect(state.trackInfo['track-1']?.status, DeezerSearchStatus.found);
    });

    test('prefetchForSetlist handles empty tracks list', () async {
      await container.read(deezerPreviewProvider.notifier).prefetchForSetlist([]);

      final state = container.read(deezerPreviewProvider);
      expect(state.trackInfo, isEmpty);
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
      // Track should have notFound status with null preview URL
      expect(state.trackInfo['unknown-0']?.previewUrl, isNull);
      expect(state.trackInfo['unknown-0']?.status, DeezerSearchStatus.notFound);
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
      expect(state.trackInfo, isEmpty);
      expect(state.isLoading, false);
    });

    test('getPreviewUrl returns URL for known track', () {
      const state = DeezerPreviewState(
        trackInfo: {
          'track-1': DeezerTrackInfo(
            previewUrl: '/api/audio/proxy?url=test',
            status: DeezerSearchStatus.found,
            searchQuery: 'artist:"Artist" track:"Track"',
          ),
        },
      );
      expect(state.getPreviewUrl('track-1'), '/api/audio/proxy?url=test');
    });

    test('getPreviewUrl returns null for unknown track', () {
      const state = DeezerPreviewState();
      expect(state.getPreviewUrl('unknown'), isNull);
    });

    test('hasPreview returns true when previewUrl is non-null', () {
      const state = DeezerPreviewState(
        trackInfo: {
          'track-1': DeezerTrackInfo(
            previewUrl: '/api/audio/proxy?url=test',
            status: DeezerSearchStatus.found,
            searchQuery: 'artist:"Artist" track:"Track"',
          ),
        },
      );
      expect(state.hasPreview('track-1'), isTrue);
    });

    test('hasPreview returns false for notFound track', () {
      const state = DeezerPreviewState(
        trackInfo: {
          'track-1': DeezerTrackInfo(
            status: DeezerSearchStatus.notFound,
            searchQuery: 'artist:"Artist" track:"Track"',
          ),
        },
      );
      expect(state.hasPreview('track-1'), isFalse);
    });

    test('status transitions: loading → found when URL returned', () async {
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
          trackId: 'track-found',
        ),
      ];

      final notifier = container.read(deezerPreviewProvider.notifier);
      final future = notifier.prefetchForSetlist(tracks);

      // After calling prefetch, state should be isLoading=true
      // (loading markers set synchronously before fetch)
      expect(container.read(deezerPreviewProvider).isLoading, isTrue);
      expect(
        container.read(deezerPreviewProvider).trackInfo['track-found']?.status,
        DeezerSearchStatus.loading,
      );

      await future;

      final state = container.read(deezerPreviewProvider);
      expect(state.isLoading, false);
      expect(state.trackInfo['track-found']?.status, DeezerSearchStatus.found);
      expect(state.trackInfo['track-found']?.searchQuery, 'artist:"Artist" track:"Track"');
    });

    test('status transitions: loading → notFound when no URL returned', () async {
      interceptor.responseOverride = {'data': []};

      final tracks = [
        const SetlistTrack(
          position: 0,
          title: 'Missing',
          artist: 'Nobody',
          originalPosition: 0,
          source: 'suggestion',
        ),
      ];

      await container.read(deezerPreviewProvider.notifier).prefetchForSetlist(tracks);

      final state = container.read(deezerPreviewProvider);
      expect(state.trackInfo['unknown-0']?.status, DeezerSearchStatus.notFound);
      expect(state.trackInfo['unknown-0']?.searchQuery, 'artist:"Nobody" track:"Missing"');
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
