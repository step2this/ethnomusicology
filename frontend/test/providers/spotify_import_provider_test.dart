import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ethnomusicology_frontend/providers/api_provider.dart';
import 'package:ethnomusicology_frontend/providers/spotify_import_provider.dart';
import '../helpers/mock_api_client.dart';

void main() {
  group('SpotifyImportNotifier', () {
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

    test('initial state is idle', () {
      final state = container.read(spotifyImportProvider);
      expect(state.status, ImportStatus.idle);
      expect(state.importId, isNull);
      expect(state.errorMessage, isNull);
    });

    test('importPlaylist success updates state to completed', () async {
      interceptor.responseOverride = {
        'import_id': 'import-abc',
        'total': 50,
        'inserted': 45,
        'updated': 5,
        'failed': 0,
      };

      await container.read(spotifyImportProvider.notifier).importPlaylist(
        'https://open.spotify.com/playlist/abc123',
      );

      final state = container.read(spotifyImportProvider);
      expect(state.status, ImportStatus.completed);
      expect(state.importId, 'import-abc');
      expect(state.progress.total, 50);
      expect(state.progress.inserted, 45);
      expect(state.progress.updated, 5);
      expect(state.progress.failed, 0);
    });

    test('importPlaylist error sets error state', () async {
      interceptor.errorOverride = DioException(
        requestOptions: RequestOptions(path: '/test'),
        message: 'Network error',
      );

      await container.read(spotifyImportProvider.notifier).importPlaylist(
        'https://open.spotify.com/playlist/bad',
      );

      final state = container.read(spotifyImportProvider);
      expect(state.status, ImportStatus.error);
      expect(state.errorMessage, isNotNull);
    });

    test('reset returns to idle state', () async {
      interceptor.responseOverride = {
        'import_id': 'import-xyz',
        'total': 10,
        'inserted': 10,
        'updated': 0,
        'failed': 0,
      };

      await container.read(spotifyImportProvider.notifier).importPlaylist('url');
      container.read(spotifyImportProvider.notifier).reset();

      final state = container.read(spotifyImportProvider);
      expect(state.status, ImportStatus.idle);
      expect(state.importId, isNull);
    });
  });

  group('SpotifyConnectionNotifier', () {
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

    test('initial state is disconnected', () {
      final state = container.read(spotifyConnectionProvider);
      expect(state.status, SpotifyConnectionStatus.disconnected);
      expect(state.errorMessage, isNull);
    });

    test('checkConnection sets connected on success', () async {
      interceptor.responseOverride = {'connected': true};

      await container.read(spotifyConnectionProvider.notifier).checkConnection('dev-user');

      final state = container.read(spotifyConnectionProvider);
      expect(state.status, SpotifyConnectionStatus.connected);
    });

    test('checkConnection sets disconnected when not connected', () async {
      interceptor.responseOverride = {'connected': false};

      await container.read(spotifyConnectionProvider.notifier).checkConnection('dev-user');

      final state = container.read(spotifyConnectionProvider);
      expect(state.status, SpotifyConnectionStatus.disconnected);
    });

    test('checkConnection sets error on failure', () async {
      interceptor.errorOverride = DioException(
        requestOptions: RequestOptions(path: '/test'),
        message: 'Connection refused',
      );

      await container.read(spotifyConnectionProvider.notifier).checkConnection('dev-user');

      final state = container.read(spotifyConnectionProvider);
      expect(state.status, SpotifyConnectionStatus.error);
      expect(state.errorMessage, isNotNull);
    });

    test('setConnected updates state', () {
      container.read(spotifyConnectionProvider.notifier).setConnected();

      final state = container.read(spotifyConnectionProvider);
      expect(state.status, SpotifyConnectionStatus.connected);
    });
  });
}
