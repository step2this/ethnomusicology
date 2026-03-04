import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ethnomusicology_frontend/providers/api_provider.dart';
import 'package:ethnomusicology_frontend/providers/track_catalog_provider.dart';
import 'package:ethnomusicology_frontend/services/api_client.dart';

/// Mock interceptor for catalog tests
class _MockInterceptor extends Interceptor {
  RequestOptions? lastRequest;
  Object? responseOverride;
  DioException? errorOverride;

  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    lastRequest = options;
    if (errorOverride != null) {
      handler.reject(errorOverride!);
      return;
    }
    handler.resolve(Response(
      requestOptions: options,
      statusCode: 200,
      data: responseOverride ?? _emptyTrackResponse(),
    ));
  }

  static Map<String, dynamic> _emptyTrackResponse() => {
        'data': <Map<String, dynamic>>[],
        'page': 1,
        'per_page': 25,
        'total': 0,
        'total_pages': 0,
      };
}

void main() {
  group('TrackCatalogNotifier', () {
    late ProviderContainer container;
    late _MockInterceptor interceptor;

    setUp(() {
      final dio = Dio(BaseOptions(baseUrl: 'http://localhost:3001/api'));
      interceptor = _MockInterceptor();
      dio.interceptors.add(interceptor);
      final apiClient = ApiClient(dio: dio);

      container = ProviderContainer(
        overrides: [
          apiClientProvider.overrideWithValue(apiClient),
        ],
      );
    });

    tearDown(() => container.dispose());

    test('initial state has empty tracks', () {
      final state = container.read(trackCatalogProvider);
      expect(state.tracks, isEmpty);
      expect(state.isLoading, false);
      expect(state.error, isNull);
      expect(state.currentPage, 0);
      expect(state.totalPages, 0);
    });

    test('loadFirstPage populates tracks from API', () async {
      interceptor.responseOverride = {
        'data': [
          {
            'id': 't1',
            'title': 'Track One',
            'artist': 'Artist A',
            'source': 'spotify',
            'date_added': '2026-03-01T00:00:00Z',
          },
          {
            'id': 't2',
            'title': 'Track Two',
            'artist': 'Artist B',
            'source': 'spotify',
            'date_added': '2026-03-02T00:00:00Z',
          },
        ],
        'page': 1,
        'per_page': 25,
        'total': 2,
        'total_pages': 1,
      };

      await container.read(trackCatalogProvider.notifier).loadFirstPage();

      final state = container.read(trackCatalogProvider);
      expect(state.tracks, hasLength(2));
      expect(state.tracks[0].title, 'Track One');
      expect(state.tracks[1].title, 'Track Two');
      expect(state.currentPage, 1);
      expect(state.total, 2);
      expect(state.isLoading, false);
      expect(state.error, isNull);
    });

    test('loadFirstPage sets error on failure', () async {
      interceptor.errorOverride = DioException(
        requestOptions: RequestOptions(path: '/test'),
        message: 'Network error',
      );

      await container.read(trackCatalogProvider.notifier).loadFirstPage();

      final state = container.read(trackCatalogProvider);
      expect(state.isLoading, false);
      expect(state.error, isNotNull);
      expect(state.tracks, isEmpty);
    });

    test('loadNextPage appends tracks', () async {
      // Set up initial page
      interceptor.responseOverride = {
        'data': [
          {
            'id': 't1',
            'title': 'Page 1 Track',
            'artist': 'Artist',
            'source': 'spotify',
            'date_added': '2026-03-01T00:00:00Z',
          },
        ],
        'page': 1,
        'per_page': 1,
        'total': 2,
        'total_pages': 2,
      };
      await container.read(trackCatalogProvider.notifier).loadFirstPage();

      // Load next page
      interceptor.responseOverride = {
        'data': [
          {
            'id': 't2',
            'title': 'Page 2 Track',
            'artist': 'Artist',
            'source': 'spotify',
            'date_added': '2026-03-02T00:00:00Z',
          },
        ],
        'page': 2,
        'per_page': 1,
        'total': 2,
        'total_pages': 2,
      };
      await container.read(trackCatalogProvider.notifier).loadNextPage();

      final state = container.read(trackCatalogProvider);
      expect(state.tracks, hasLength(2));
      expect(state.tracks[0].title, 'Page 1 Track');
      expect(state.tracks[1].title, 'Page 2 Track');
      expect(state.currentPage, 2);
    });

    test('loadNextPage does nothing when no more pages', () async {
      interceptor.responseOverride = {
        'data': [
          {
            'id': 't1',
            'title': 'Only Track',
            'artist': 'Artist',
            'source': 'spotify',
            'date_added': '2026-03-01T00:00:00Z',
          },
        ],
        'page': 1,
        'per_page': 25,
        'total': 1,
        'total_pages': 1,
      };
      await container.read(trackCatalogProvider.notifier).loadFirstPage();

      // Try to load next — should be a no-op
      await container.read(trackCatalogProvider.notifier).loadNextPage();

      final state = container.read(trackCatalogProvider);
      expect(state.tracks, hasLength(1));
      expect(state.currentPage, 1);
    });

    test('setSort resets and reloads', () async {
      interceptor.responseOverride = {
        'data': [],
        'page': 1,
        'per_page': 25,
        'total': 0,
        'total_pages': 0,
      };

      await container.read(trackCatalogProvider.notifier).setSort('title', 'asc');

      final state = container.read(trackCatalogProvider);
      expect(state.sort, 'title');
      expect(state.order, 'asc');

      // Check that the API was called with correct params
      expect(interceptor.lastRequest?.queryParameters['sort'], 'title');
      expect(interceptor.lastRequest?.queryParameters['order'], 'asc');
    });

    test('retry calls loadFirstPage', () async {
      interceptor.responseOverride = {
        'data': [],
        'page': 1,
        'per_page': 25,
        'total': 0,
        'total_pages': 0,
      };

      await container.read(trackCatalogProvider.notifier).retry();

      final state = container.read(trackCatalogProvider);
      expect(state.isLoading, false);
      expect(state.error, isNull);
    });

    test('hasMore is true when more pages available', () async {
      interceptor.responseOverride = {
        'data': [
          {
            'id': 't1',
            'title': 'T',
            'artist': 'A',
            'source': 'spotify',
            'date_added': '2026-03-01T00:00:00Z',
          },
        ],
        'page': 1,
        'per_page': 1,
        'total': 5,
        'total_pages': 5,
      };
      await container.read(trackCatalogProvider.notifier).loadFirstPage();

      expect(container.read(trackCatalogProvider).hasMore, true);
    });
  });
}
