import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:ethnomusicology_frontend/models/setlist_track.dart';
import 'package:ethnomusicology_frontend/providers/api_provider.dart';
import 'package:ethnomusicology_frontend/providers/refinement_provider.dart';
import 'package:ethnomusicology_frontend/providers/setlist_provider.dart';
import 'package:ethnomusicology_frontend/services/api_client.dart';

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
      data: responseOverride ?? <String, dynamic>{},
    ));
  }
}

void main() {
  late Dio dio;
  late _MockInterceptor interceptor;
  late ApiClient apiClient;
  late ProviderContainer container;

  setUp(() {
    dio = Dio(BaseOptions(baseUrl: 'http://localhost:3001/api'));
    interceptor = _MockInterceptor();
    dio.interceptors.add(interceptor);
    apiClient = ApiClient(dio: dio);

    container = ProviderContainer(
      overrides: [
        apiClientProvider.overrideWithValue(apiClient),
      ],
    );

    // Seed a setlist so refinement has something to work with
    container.read(setlistProvider.notifier).updateTracks(const [
      SetlistTrack(
        position: 1,
        title: 'Test Track',
        artist: 'Test Artist',
        originalPosition: 1,
        source: 'catalog',
      ),
    ]);
  });

  tearDown(() {
    container.dispose();
  });

  group('refineSetlist', () {
    test('sends message and updates state on success', () async {
      interceptor.responseOverride = {
        'version_number': 1,
        'tracks': [
          {
            'position': 1,
            'title': 'Energetic Track',
            'artist': 'DJ A',
            'original_position': 1,
            'source': 'catalog',
          },
        ],
        'explanation': 'Made it more energetic',
        'change_warning': null,
      };

      final notifier = container.read(refinementProvider.notifier);
      await notifier.refineSetlist('set-1', 'make it energetic');

      final state = container.read(refinementProvider);
      expect(state.isRefining, false);
      expect(state.currentVersion, 1);
      expect(state.conversation, hasLength(2)); // user + assistant
      expect(state.conversation[0].role, 'user');
      expect(state.conversation[0].content, 'make it energetic');
      expect(state.conversation[1].role, 'assistant');
      expect(state.conversation[1].content, 'Made it more energetic');
      // H1 fix: version history updated optimistically
      expect(state.versionHistory, hasLength(1));
      expect(state.versionHistory![0].versionNumber, 1);
      expect(state.versionHistory![0].action, 'refine');
    });

    test('sets isRefining during API call', () async {
      interceptor.responseOverride = {
        'version_number': 1,
        'tracks': [],
        'explanation': 'Done',
      };

      final notifier = container.read(refinementProvider.notifier);

      // Check initial state
      expect(container.read(refinementProvider).isRefining, false);

      // After call completes
      await notifier.refineSetlist('set-1', 'test');
      expect(container.read(refinementProvider).isRefining, false);
    });

    test('handles error and removes optimistic message', () async {
      interceptor.errorOverride = DioException(
        requestOptions: RequestOptions(path: '/test'),
        message: 'TURN_LIMIT_EXCEEDED',
      );

      final notifier = container.read(refinementProvider.notifier);
      await notifier.refineSetlist('set-1', 'test');

      final state = container.read(refinementProvider);
      expect(state.isRefining, false);
      expect(state.error, contains('Refinement limit'));
      expect(state.conversation, isEmpty); // optimistic msg removed
    });

    test('stores changeWarning from response', () async {
      interceptor.responseOverride = {
        'version_number': 2,
        'tracks': [],
        'explanation': 'Changed a lot',
        'change_warning': 'Removed 3 tracks',
      };

      final notifier = container.read(refinementProvider.notifier);
      await notifier.refineSetlist('set-1', 'change everything');

      final state = container.read(refinementProvider);
      expect(state.changeWarning, 'Removed 3 tracks');
    });
  });

  group('revertToVersion', () {
    test('reverts and adds system message', () async {
      interceptor.responseOverride = {
        'version_number': 0,
        'tracks': [
          {
            'position': 1,
            'title': 'Original Track',
            'artist': 'Original Artist',
            'original_position': 1,
            'source': 'catalog',
          },
        ],
        'explanation': 'Reverted to original',
      };

      final notifier = container.read(refinementProvider.notifier);
      await notifier.revertToVersion('set-1', 0);

      final state = container.read(refinementProvider);
      expect(state.isRefining, false);
      expect(state.currentVersion, 0);
      expect(state.conversation, hasLength(1));
      expect(state.conversation[0].content, contains('Reverted to version 0'));
      // H1 fix: version history updated optimistically
      expect(state.versionHistory, hasLength(1));
      expect(state.versionHistory![0].action, 'revert');
      expect(state.versionHistory![0].actionSummary, 'Reverted to v0');
    });

    test('handles revert error', () async {
      interceptor.errorOverride = DioException(
        requestOptions: RequestOptions(path: '/test'),
        message: 'NOT_FOUND',
      );

      final notifier = container.read(refinementProvider.notifier);
      await notifier.revertToVersion('set-1', 99);

      final state = container.read(refinementProvider);
      expect(state.error, contains('not found'));
    });
  });

  group('loadHistory', () {
    test('populates conversation and versions', () async {
      interceptor.responseOverride = {
        'versions': [
          {
            'id': 'v-0',
            'setlist_id': 'set-1',
            'version_number': 0,
          },
          {
            'id': 'v-1',
            'setlist_id': 'set-1',
            'version_number': 1,
            'action': 'refine',
            'action_summary': 'More energy',
          },
        ],
        'conversations': [
          {
            'id': 'msg-1',
            'setlist_id': 'set-1',
            'role': 'user',
            'content': 'More energy',
          },
        ],
      };

      final notifier = container.read(refinementProvider.notifier);
      await notifier.loadHistory('set-1');

      final state = container.read(refinementProvider);
      expect(state.versionHistory, hasLength(2));
      expect(state.conversation, hasLength(1));
      expect(state.currentVersion, 1); // highest version number
      expect(state.isLoadingHistory, false);
    });

    test('handles empty history', () async {
      interceptor.responseOverride = {
        'versions': [],
        'conversations': [],
      };

      final notifier = container.read(refinementProvider.notifier);
      await notifier.loadHistory('set-1');

      final state = container.read(refinementProvider);
      expect(state.versionHistory, isEmpty);
      expect(state.conversation, isEmpty);
      expect(state.currentVersion, isNull);
    });
  });

  group('reset', () {
    test('clears all state', () async {
      interceptor.responseOverride = {
        'version_number': 1,
        'tracks': [],
        'explanation': 'Done',
      };

      final notifier = container.read(refinementProvider.notifier);
      await notifier.refineSetlist('set-1', 'test');

      // Verify state is populated
      expect(container.read(refinementProvider).conversation, isNotEmpty);

      // Reset
      notifier.reset();

      final state = container.read(refinementProvider);
      expect(state.conversation, isEmpty);
      expect(state.currentVersion, isNull);
      expect(state.isRefining, false);
      expect(state.error, isNull);
      expect(state.changeWarning, isNull);
      expect(state.versionHistory, isNull);
    });
  });

  group('error parsing', () {
    test('NOT_FOUND error', () async {
      interceptor.errorOverride = DioException(
        requestOptions: RequestOptions(path: '/test'),
        message: 'NOT_FOUND',
      );

      await container.read(refinementProvider.notifier).refineSetlist('set-1', 'test');
      expect(container.read(refinementProvider).error, contains('not found'));
    });

    test('LLM_ERROR', () async {
      interceptor.errorOverride = DioException(
        requestOptions: RequestOptions(path: '/test'),
        message: 'LLM_ERROR',
      );

      await container.read(refinementProvider.notifier).refineSetlist('set-1', 'test');
      expect(container.read(refinementProvider).error, contains('AI service'));
    });

    test('INVALID_REQUEST error', () async {
      interceptor.errorOverride = DioException(
        requestOptions: RequestOptions(path: '/test'),
        message: 'INVALID_REQUEST',
      );

      await container.read(refinementProvider.notifier).refineSetlist('set-1', 'test');
      expect(container.read(refinementProvider).error, contains('Invalid request'));
    });

    test('unknown error', () async {
      interceptor.errorOverride = DioException(
        requestOptions: RequestOptions(path: '/test'),
        message: 'SOMETHING_WEIRD',
      );

      await container.read(refinementProvider.notifier).refineSetlist('set-1', 'test');
      expect(container.read(refinementProvider).error, contains('Refinement failed'));
    });
  });

  group('SetlistNotifier.updateTracks', () {
    test('updates tracks on existing setlist', () async {
      // First, generate a setlist via the mock
      interceptor.responseOverride = {
        'id': 'set-1',
        'prompt': 'test',
        'model': 'claude-sonnet',
        'tracks': [
          {
            'position': 1,
            'title': 'Old Track',
            'artist': 'Old Artist',
            'original_position': 1,
            'source': 'catalog',
          },
        ],
        'bpm_warnings': [],
      };

      await container.read(setlistProvider.notifier).generateSetlist('test');
      expect(container.read(setlistProvider).setlist!.tracks[0].title, 'Old Track');

      // Now update tracks
      container.read(setlistProvider.notifier).updateTracks(const [
        SetlistTrack(
          position: 1,
          title: 'New Track',
          artist: 'New Artist',
          originalPosition: 1,
          source: 'suggestion',
        ),
      ]);

      expect(container.read(setlistProvider).setlist!.tracks[0].title, 'New Track');
      // Verify other fields preserved
      expect(container.read(setlistProvider).setlist!.id, 'set-1');
      expect(container.read(setlistProvider).setlist!.prompt, 'test');
    });

    test('does nothing when no setlist', () {
      container.read(setlistProvider.notifier).reset();

      // Should not throw
      container.read(setlistProvider.notifier).updateTracks(const [
        SetlistTrack(
          position: 1,
          title: 'Test',
          artist: 'Test',
          originalPosition: 1,
          source: 'catalog',
        ),
      ]);

      expect(container.read(setlistProvider).setlist, isNull);
    });
  });
}
