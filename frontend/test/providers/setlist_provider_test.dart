import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ethnomusicology_frontend/providers/api_provider.dart';
import 'package:ethnomusicology_frontend/providers/setlist_provider.dart';
import 'package:ethnomusicology_frontend/services/api_client.dart';

/// Interceptor that captures requests and returns configurable responses.
class MockInterceptor extends Interceptor {
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
      data: responseOverride ?? _defaultSetlistResponse(),
    ));
  }

  static Map<String, dynamic> _defaultSetlistResponse() => {
        'id': 'set-1',
        'prompt': 'test',
        'model': 'claude-sonnet-4-20250514',
        'tracks': [],
        'energy_profile': 'journey',
        'catalog_percentage': 80.0,
        'bpm_warnings': [],
      };
}

void main() {
  late Dio dio;
  late MockInterceptor interceptor;
  late ApiClient apiClient;
  late ProviderContainer container;
  late SetlistNotifier notifier;

  setUp(() {
    dio = Dio(BaseOptions(baseUrl: 'http://localhost:3001/api'));
    interceptor = MockInterceptor();
    dio.interceptors.add(interceptor);
    apiClient = ApiClient(dio: dio);

    container = ProviderContainer(
      overrides: [
        apiClientProvider.overrideWithValue(apiClient),
      ],
    );
    notifier = container.read(setlistProvider.notifier);
  });

  tearDown(() {
    container.dispose();
  });

  group('generateSetlist', () {
    test('passes energy_profile to API client', () async {
      await notifier.generateSetlist(
        'deep house',
        energyProfile: 'peak-time',
      );

      final body = interceptor.lastRequest!.data as Map<String, dynamic>;
      expect(body['energy_profile'], 'peak-time');
    });

    test('passes source_playlist_id to API client', () async {
      await notifier.generateSetlist(
        'chill vibes',
        sourcePlaylistId: 'import-abc',
      );

      final body = interceptor.lastRequest!.data as Map<String, dynamic>;
      expect(body['source_playlist_id'], 'import-abc');
    });

    test('passes all new params to API client', () async {
      await notifier.generateSetlist(
        'test prompt',
        trackCount: 20,
        energyProfile: 'journey',
        sourcePlaylistId: 'import-1',
        seedTracklist: 'Track A\nTrack B',
        creativeMode: true,
        bpmMin: 120.0,
        bpmMax: 135.0,
      );

      final body = interceptor.lastRequest!.data as Map<String, dynamic>;
      expect(body['track_count'], 20);
      expect(body['energy_profile'], 'journey');
      expect(body['source_playlist_id'], 'import-1');
      expect(body['seed_tracklist'], 'Track A\nTrack B');
      expect(body['creative_mode'], true);
      expect(body['bpm_range'], {'min': 120.0, 'max': 135.0});
    });

    test('state includes new response fields after generation', () async {
      interceptor.responseOverride = {
        'id': 'set-2',
        'prompt': 'test',
        'model': 'claude-sonnet-4-20250514',
        'tracks': [],
        'energy_profile': 'warm-up',
        'catalog_percentage': 65.0,
        'catalog_warning': null,
        'bpm_warnings': [
          {'from_position': 1, 'to_position': 2, 'bpm_delta': 8.0},
        ],
      };

      await notifier.generateSetlist('test', energyProfile: 'warm-up');

      expect(container.read(setlistProvider).setlist, isNotNull);
      expect(container.read(setlistProvider).setlist!.energyProfile, 'warm-up');
      expect(container.read(setlistProvider).setlist!.catalogPercentage, 65.0);
      expect(container.read(setlistProvider).setlist!.bpmWarnings, hasLength(1));
      expect(container.read(setlistProvider).energyProfile, 'warm-up');
    });

    test('preserves energyProfile and creativeMode in state during generation',
        () async {
      await notifier.generateSetlist(
        'test',
        energyProfile: 'peak-time',
        creativeMode: true,
        sourcePlaylistId: 'import-x',
      );

      expect(container.read(setlistProvider).energyProfile, 'peak-time');
      expect(container.read(setlistProvider).creativeMode, true);
      expect(container.read(setlistProvider).sourcePlaylistId, 'import-x');
    });
  });

  group('arrangeSetlist', () {
    test('passes energy profile from state to API', () async {
      // First generate to populate state
      await notifier.generateSetlist('test', energyProfile: 'journey');

      // Then arrange
      await notifier.arrangeSetlist();

      // The arrange call should have sent the energy profile
      final body = interceptor.lastRequest!.data as Map<String, dynamic>;
      expect(body['energy_profile'], 'journey');
    });

    test('sends null data when no energy profile in state', () async {
      // Generate without energy profile
      await notifier.generateSetlist('test');

      // Then arrange
      await notifier.arrangeSetlist();

      // No energy profile should be sent
      expect(interceptor.lastRequest!.data, isNull);
    });
  });

  group('error handling', () {
    DioException makeError(String code) => DioException(
          requestOptions: RequestOptions(path: '/test'),
          response: Response(
            requestOptions: RequestOptions(path: '/test'),
            statusCode: 400,
            data: {
              'error': {'code': code, 'message': 'test error'}
            },
          ),
          message: code,
        );

    test('INVALID_ENERGY_PROFILE error', () async {
      interceptor.errorOverride = makeError('INVALID_ENERGY_PROFILE');
      await notifier.generateSetlist('test', energyProfile: 'bad');

      expect(container.read(setlistProvider).error, 'Invalid energy profile selected.');
    });

    test('PLAYLIST_NOT_FOUND error', () async {
      interceptor.errorOverride = makeError('PLAYLIST_NOT_FOUND');
      await notifier.generateSetlist('test', sourcePlaylistId: 'bad-id');

      expect(container.read(setlistProvider).error, contains('Playlist not found'));
    });

    test('INVALID_BPM_RANGE error', () async {
      interceptor.errorOverride = makeError('INVALID_BPM_RANGE');
      await notifier.generateSetlist('test', bpmMin: 200.0, bpmMax: 100.0);

      expect(container.read(setlistProvider).error, contains('Invalid BPM range'));
    });

    test('EMPTY_CATALOG error', () async {
      interceptor.errorOverride = makeError('EMPTY_CATALOG');
      await notifier.generateSetlist('test');

      expect(container.read(setlistProvider).error, contains('No tracks in your catalog'));
    });
  });

  group('backward compatibility', () {
    test('default state has null energyProfile and false creativeMode', () {
      expect(container.read(setlistProvider).energyProfile, isNull);
      expect(container.read(setlistProvider).creativeMode, false);
      expect(container.read(setlistProvider).sourcePlaylistId, isNull);
    });

    test('generateSetlist without new params works', () async {
      await notifier.generateSetlist('simple prompt');

      final body = interceptor.lastRequest!.data as Map<String, dynamic>;
      expect(body['prompt'], 'simple prompt');
      expect(body.containsKey('energy_profile'), isFalse);
      expect(body.containsKey('creative_mode'), isFalse);
      expect(container.read(setlistProvider).setlist, isNotNull);
    });
  });
}
