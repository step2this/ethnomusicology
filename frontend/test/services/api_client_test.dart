import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ethnomusicology_frontend/services/api_client.dart';

/// Interceptor that captures requests for assertion without hitting network.
class RequestCaptureInterceptor extends Interceptor {
  RequestOptions? lastRequest;

  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    lastRequest = options;
    // Return a fake successful response so the call completes
    handler.resolve(Response(
      requestOptions: options,
      statusCode: 200,
      data: _fakeSetlistResponse(),
    ));
  }

  static Map<String, dynamic> _fakeSetlistResponse() => {
        'id': 'test-id',
        'prompt': 'test prompt',
        'model': 'claude-sonnet-4-20250514',
        'tracks': [],
      };
}

void main() {
  late Dio dio;
  late RequestCaptureInterceptor interceptor;
  late ApiClient client;

  setUp(() {
    dio = Dio(BaseOptions(baseUrl: 'http://localhost:3001/api'));
    interceptor = RequestCaptureInterceptor();
    dio.interceptors.add(interceptor);
    client = ApiClient(dio: dio);
  });

  group('generateSetlist', () {
    test('includes all new params when provided', () async {
      await client.generateSetlist(
        'deep house vibes',
        trackCount: 20,
        energyProfile: 'peak-time',
        sourcePlaylistId: 'import-123',
        seedTracklist: 'Track A - Artist A\nTrack B - Artist B',
        creativeMode: true,
        bpmMin: 120.0,
        bpmMax: 130.0,
      );

      final body = interceptor.lastRequest!.data as Map<String, dynamic>;
      expect(body['prompt'], 'deep house vibes');
      expect(body['track_count'], 20);
      expect(body['energy_profile'], 'peak-time');
      expect(body['source_playlist_id'], 'import-123');
      expect(body['seed_tracklist'],
          'Track A - Artist A\nTrack B - Artist B');
      expect(body['creative_mode'], true);
      expect(body['bpm_range'], {'min': 120.0, 'max': 130.0});
    });

    test('omits new params when null', () async {
      await client.generateSetlist('minimal request');

      final body = interceptor.lastRequest!.data as Map<String, dynamic>;
      expect(body['prompt'], 'minimal request');
      expect(body.containsKey('track_count'), isFalse);
      expect(body.containsKey('energy_profile'), isFalse);
      expect(body.containsKey('source_playlist_id'), isFalse);
      expect(body.containsKey('seed_tracklist'), isFalse);
      expect(body.containsKey('creative_mode'), isFalse);
      expect(body.containsKey('bpm_range'), isFalse);
    });

    test('omits bpm_range when only bpmMin provided', () async {
      await client.generateSetlist('test', bpmMin: 120.0);

      final body = interceptor.lastRequest!.data as Map<String, dynamic>;
      expect(body.containsKey('bpm_range'), isFalse);
    });

    test('omits bpm_range when only bpmMax provided', () async {
      await client.generateSetlist('test', bpmMax: 130.0);

      final body = interceptor.lastRequest!.data as Map<String, dynamic>;
      expect(body.containsKey('bpm_range'), isFalse);
    });

    test('includes trackCount without other new params', () async {
      await client.generateSetlist('test', trackCount: 10);

      final body = interceptor.lastRequest!.data as Map<String, dynamic>;
      expect(body['track_count'], 10);
      expect(body.containsKey('energy_profile'), isFalse);
    });
  });

  group('arrangeSetlist', () {
    test('sends energy_profile when provided', () async {
      await client.arrangeSetlist('set-1', energyProfile: 'journey');

      final body = interceptor.lastRequest!.data as Map<String, dynamic>;
      expect(body['energy_profile'], 'journey');
    });

    test('sends null data when no energy_profile', () async {
      await client.arrangeSetlist('set-1');

      expect(interceptor.lastRequest!.data, isNull);
    });

    test('posts to correct URL', () async {
      await client.arrangeSetlist('set-abc');

      expect(interceptor.lastRequest!.path,
          contains('/setlists/set-abc/arrange'));
    });
  });

  group('searchDeezerPreview', () {
    test('returns proxied URL on successful search', () async {
      final dio = Dio(BaseOptions(baseUrl: 'http://localhost:3001/api'));
      dio.interceptors.add(_DeezerSuccessInterceptor());
      final client = ApiClient(dio: dio);

      final result = await client.searchDeezerPreview('Lovely Day', 'Bill Withers');

      expect(result, isNotNull);
      expect(result, contains('/api/audio/proxy?url='));
      expect(result, contains('https%3A%2F%2F')); // URL-encoded https://
    });

    test('returns null on empty results', () async {
      final dio = Dio(BaseOptions(baseUrl: 'http://localhost:3001/api'));
      dio.interceptors.add(_DeezerEmptyInterceptor());
      final client = ApiClient(dio: dio);

      final result =
          await client.searchDeezerPreview('NonexistentTrack', 'NonexistentArtist');

      expect(result, isNull);
    });

    test('returns null on error', () async {
      final dio = Dio(BaseOptions(baseUrl: 'http://localhost:3001/api'));
      dio.interceptors.add(_DeezerErrorInterceptor());
      final client = ApiClient(dio: dio);

      final result = await client.searchDeezerPreview('AnyTrack', 'AnyArtist');

      expect(result, isNull);
    });

    test('passes correct query parameters', () async {
      final dio = Dio(BaseOptions(baseUrl: 'http://localhost:3001/api'));
      final interceptor = _DeezerQueryCaptureInterceptor();
      dio.interceptors.add(interceptor);
      final client = ApiClient(dio: dio);

      await client.searchDeezerPreview('Song Title', 'Artist Name');

      expect(interceptor.lastQueryParams?['q'], 'Artist Name Song Title');
      expect(interceptor.lastQueryParams?['limit'], '1');
    });
  });
}

class _DeezerSuccessInterceptor extends Interceptor {
  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    handler.resolve(Response(
      requestOptions: options,
      statusCode: 200,
      data: {
        'data': [
          {'preview': 'https://cdns-files.dzcdn.net/stream/abc123.preview.mp3'}
        ]
      },
    ));
  }
}

class _DeezerEmptyInterceptor extends Interceptor {
  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    handler.resolve(Response(
      requestOptions: options,
      statusCode: 200,
      data: {'data': []},
    ));
  }
}

class _DeezerErrorInterceptor extends Interceptor {
  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    handler.reject(DioException(
      requestOptions: options,
      error: 'Network error',
      type: DioExceptionType.connectionTimeout,
    ));
  }
}

class _DeezerQueryCaptureInterceptor extends Interceptor {
  Map<String, dynamic>? lastQueryParams;

  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    lastQueryParams = options.queryParameters;
    handler.resolve(Response(
      requestOptions: options,
      statusCode: 200,
      data: {
        'data': [
          {'preview': 'https://cdns-files.dzcdn.net/stream/test.preview.mp3'}
        ]
      },
    ));
  }
}
