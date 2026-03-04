import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ethnomusicology_frontend/models/setlist_track.dart';
import 'package:ethnomusicology_frontend/providers/api_provider.dart';
import 'package:ethnomusicology_frontend/providers/deezer_provider.dart';
import '../helpers/mock_api_client.dart';

void main() {
  group('PreviewNotifier', () {
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
      final state = container.read(previewProvider);
      expect(state.trackInfo, isEmpty);
      expect(state.isLoading, false);
    });

    test('prefetchForSetlist fetches URLs for tracks', () async {
      // Mock unified search response
      interceptor.responseOverride = {
        'source': 'deezer',
        'preview_url': '/api/audio/proxy?url=https%3A%2F%2Fcdns-preview.dzcdn.net%2Ftrack1.mp3',
        'external_url': 'https://www.deezer.com/track/123',
        'search_queries': ['artist:"Test Artist" track:"Test Track"'],
        'deezer_id': 123,
        'itunes_id': null,
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

      await container.read(previewProvider.notifier).prefetchForSetlist(tracks);

      final state = container.read(previewProvider);
      expect(state.isLoading, false);
      expect(state.trackInfo['track-1']?.previewUrl, isNotNull);
      expect(state.trackInfo['track-1']?.status, PreviewSearchStatus.found);
      expect(state.trackInfo['track-1']?.source, 'deezer');
      expect(state.trackInfo['track-1']?.externalUrl, 'https://www.deezer.com/track/123');
    });

    test('prefetchForSetlist handles iTunes fallback', () async {
      interceptor.responseOverride = {
        'source': 'itunes',
        'preview_url': '/api/audio/proxy?url=https%3A%2F%2Faudio-ssl.itunes.apple.com%2Ftrack.m4a',
        'external_url': 'https://music.apple.com/track/456',
        'search_queries': ['artist:"Artist" track:"Track"', 'iTunes: Artist Track'],
        'deezer_id': null,
        'itunes_id': 456,
      };

      final tracks = [
        const SetlistTrack(
          position: 0,
          title: 'Track',
          artist: 'Artist',
          originalPosition: 0,
          source: 'catalog',
          trackId: 'track-itunes',
        ),
      ];

      await container.read(previewProvider.notifier).prefetchForSetlist(tracks);

      final state = container.read(previewProvider);
      expect(state.trackInfo['track-itunes']?.source, 'itunes');
      expect(state.trackInfo['track-itunes']?.externalUrl, contains('apple.com'));
      expect(state.trackInfo['track-itunes']?.searchQueries, hasLength(2));
    });

    test('prefetchForSetlist handles empty tracks list', () async {
      await container.read(previewProvider.notifier).prefetchForSetlist([]);

      final state = container.read(previewProvider);
      expect(state.trackInfo, isEmpty);
      expect(state.isLoading, false);
    });

    test('prefetchForSetlist handles no match', () async {
      interceptor.responseOverride = {
        'source': null,
        'preview_url': null,
        'external_url': null,
        'search_queries': ['artist:"Unknown" track:"Missing"', 'iTunes: Unknown Missing'],
        'deezer_id': null,
        'itunes_id': null,
      };

      final tracks = [
        const SetlistTrack(
          position: 0,
          title: 'Missing Track',
          artist: 'Unknown',
          originalPosition: 0,
          source: 'suggestion',
        ),
      ];

      await container.read(previewProvider.notifier).prefetchForSetlist(tracks);

      final state = container.read(previewProvider);
      expect(state.isLoading, false);
      expect(state.trackInfo['unknown-0']?.previewUrl, isNull);
      expect(state.trackInfo['unknown-0']?.status, PreviewSearchStatus.notFound);
      expect(state.trackInfo['unknown-0']?.source, isNull);
    });

    test('reset clears all state', () async {
      interceptor.responseOverride = {
        'source': 'deezer',
        'preview_url': '/api/audio/proxy?url=test',
        'external_url': null,
        'search_queries': ['query'],
        'deezer_id': 1,
        'itunes_id': null,
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

      await container.read(previewProvider.notifier).prefetchForSetlist(tracks);
      container.read(previewProvider.notifier).reset();

      final state = container.read(previewProvider);
      expect(state.trackInfo, isEmpty);
      expect(state.isLoading, false);
    });

    test('getPreviewUrl returns URL for known track', () {
      const state = PreviewState(
        trackInfo: {
          'track-1': PreviewTrackInfo(
            previewUrl: '/api/audio/proxy?url=test',
            status: PreviewSearchStatus.found,
            searchQueries: ['artist:"Artist" track:"Track"'],
            source: 'deezer',
          ),
        },
      );
      expect(state.getPreviewUrl('track-1'), '/api/audio/proxy?url=test');
    });

    test('getPreviewUrl returns null for unknown track', () {
      const state = PreviewState();
      expect(state.getPreviewUrl('unknown'), isNull);
    });

    test('hasPreview returns true when previewUrl is non-null', () {
      const state = PreviewState(
        trackInfo: {
          'track-1': PreviewTrackInfo(
            previewUrl: '/api/audio/proxy?url=test',
            status: PreviewSearchStatus.found,
            searchQueries: ['artist:"Artist" track:"Track"'],
          ),
        },
      );
      expect(state.hasPreview('track-1'), isTrue);
    });

    test('hasPreview returns false for notFound track', () {
      const state = PreviewState(
        trackInfo: {
          'track-1': PreviewTrackInfo(
            status: PreviewSearchStatus.notFound,
            searchQueries: ['artist:"Artist" track:"Track"'],
          ),
        },
      );
      expect(state.hasPreview('track-1'), isFalse);
    });

    test('status transitions: loading → found when URL returned', () async {
      interceptor.responseOverride = {
        'source': 'deezer',
        'preview_url': '/api/audio/proxy?url=test',
        'external_url': null,
        'search_queries': ['artist:"Artist" track:"Track"'],
        'deezer_id': 1,
        'itunes_id': null,
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

      final notifier = container.read(previewProvider.notifier);
      final future = notifier.prefetchForSetlist(tracks);

      expect(container.read(previewProvider).isLoading, isTrue);
      expect(
        container.read(previewProvider).trackInfo['track-found']?.status,
        PreviewSearchStatus.loading,
      );

      await future;

      final state = container.read(previewProvider);
      expect(state.isLoading, false);
      expect(state.trackInfo['track-found']?.status, PreviewSearchStatus.found);
      expect(state.trackInfo['track-found']?.searchQueries, ['artist:"Artist" track:"Track"']);
    });

    test('status transitions: loading → notFound when no URL returned', () async {
      interceptor.responseOverride = {
        'source': null,
        'preview_url': null,
        'external_url': null,
        'search_queries': ['artist:"Nobody" track:"Missing"'],
        'deezer_id': null,
        'itunes_id': null,
      };

      final tracks = [
        const SetlistTrack(
          position: 0,
          title: 'Missing',
          artist: 'Nobody',
          originalPosition: 0,
          source: 'suggestion',
        ),
      ];

      await container.read(previewProvider.notifier).prefetchForSetlist(tracks);

      final state = container.read(previewProvider);
      expect(state.trackInfo['unknown-0']?.status, PreviewSearchStatus.notFound);
      expect(state.trackInfo['unknown-0']?.searchQueries, ['artist:"Nobody" track:"Missing"']);
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
