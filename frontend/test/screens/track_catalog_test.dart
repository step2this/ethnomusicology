import 'package:dio/dio.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:ethnomusicology_frontend/models/track.dart';
import 'package:ethnomusicology_frontend/models/track_list_response.dart';
import 'package:ethnomusicology_frontend/providers/track_catalog_provider.dart';
import 'package:ethnomusicology_frontend/screens/track_catalog_screen.dart';
import 'package:ethnomusicology_frontend/services/api_client.dart';

// Mock API client that bypasses Dio entirely
class MockApiClient extends ApiClient {
  TrackListResponse? _response;
  bool _shouldThrow = false;
  int _callCount = 0;
  List<TrackListResponse>? _pagedResponses;

  MockApiClient()
      : super(
            dio: Dio(BaseOptions(
                baseUrl: 'http://localhost:3001/api')));

  void setResponse(TrackListResponse response) {
    _response = response;
    _shouldThrow = false;
  }

  void setPagedResponses(List<TrackListResponse> responses) {
    _pagedResponses = responses;
    _shouldThrow = false;
  }

  void setShouldThrow(bool value) {
    _shouldThrow = value;
  }

  @override
  Future<TrackListResponse> listTracks({
    int page = 1,
    int perPage = 25,
    String sort = 'date_added',
    String order = 'desc',
  }) async {
    if (_shouldThrow) {
      throw Exception('Network error');
    }
    if (_pagedResponses != null && _callCount < _pagedResponses!.length) {
      return _pagedResponses![_callCount++];
    }
    return _response!;
  }
}

Track _makeTrack({
  String id = 't1',
  String title = 'Test Track',
  String artist = 'Test Artist',
  double? bpm,
  String? key,
  String? albumArtUrl,
}) {
  return Track(
    id: id,
    title: title,
    artist: artist,
    bpm: bpm,
    key: key,
    albumArtUrl: albumArtUrl,
    source: 'spotify',
    dateAdded: DateTime(2026, 3, 1),
  );
}

Widget _buildTestApp(MockApiClient mockApi) {
  return ProviderScope(
    overrides: [
      apiClientProvider.overrideWithValue(mockApi),
    ],
    child: const MaterialApp(
      home: TrackCatalogScreen(),
    ),
  );
}

void main() {
  testWidgets('renders track list with all fields', (tester) async {
    final mockApi = MockApiClient();
    mockApi.setResponse(TrackListResponse(
      data: [
        _makeTrack(
            id: 't1',
            title: 'Gnawa Blues',
            artist: 'Hassan Hakmoun',
            bpm: 128,
            key: '8A'),
        _makeTrack(
            id: 't2',
            title: 'Desert Rose',
            artist: 'Unknown',
            bpm: null,
            key: null),
      ],
      page: 1,
      perPage: 25,
      total: 2,
      totalPages: 1,
    ));

    await tester.pumpWidget(_buildTestApp(mockApi));
    await tester.pumpAndSettle();

    expect(find.text('Gnawa Blues'), findsOneWidget);
    expect(find.text('Hassan Hakmoun'), findsOneWidget);
    expect(find.text('Desert Rose'), findsOneWidget);
    // BPM: "128" for first, "--" for second
    expect(find.text('128'), findsOneWidget);
    // Key: "8A" for first, "--" for second
    expect(find.text('8A'), findsOneWidget);
    expect(find.text('--'), findsWidgets); // null fields
  });

  testWidgets('renders empty state when no tracks', (tester) async {
    final mockApi = MockApiClient();
    mockApi.setResponse(const TrackListResponse(
      data: [],
      page: 1,
      perPage: 25,
      total: 0,
      totalPages: 0,
    ));

    await tester.pumpWidget(_buildTestApp(mockApi));
    await tester.pumpAndSettle();

    expect(find.text('No tracks yet'), findsOneWidget);
    expect(find.text('Import from Spotify to get started'), findsOneWidget);
    expect(find.text('Import from Spotify'), findsOneWidget);
  });

  testWidgets('renders error state on API failure', (tester) async {
    final mockApi = MockApiClient();
    mockApi.setShouldThrow(true);

    await tester.pumpWidget(_buildTestApp(mockApi));
    await tester.pumpAndSettle();

    expect(find.text('Failed to load tracks. Please try again.'),
        findsOneWidget);
    expect(find.text('Retry'), findsOneWidget);
  });

  testWidgets('renders multi-artist track correctly', (tester) async {
    final mockApi = MockApiClient();
    mockApi.setResponse(TrackListResponse(
      data: [
        _makeTrack(
          id: 't1',
          title: 'Collab Track',
          artist: 'Hassan Hakmoun, Gnawa Diffusion',
        ),
      ],
      page: 1,
      perPage: 25,
      total: 1,
      totalPages: 1,
    ));

    await tester.pumpWidget(_buildTestApp(mockApi));
    await tester.pumpAndSettle();

    expect(find.text('Hassan Hakmoun, Gnawa Diffusion'), findsOneWidget);
  });
}
