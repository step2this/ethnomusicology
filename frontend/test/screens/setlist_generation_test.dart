import 'package:dio/dio.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:ethnomusicology_frontend/models/setlist.dart';
import 'package:ethnomusicology_frontend/models/setlist_track.dart';
import 'package:ethnomusicology_frontend/providers/setlist_provider.dart';
import 'package:ethnomusicology_frontend/providers/api_provider.dart';
import 'package:ethnomusicology_frontend/screens/setlist_generation_screen.dart';
import 'package:ethnomusicology_frontend/services/api_client.dart';

class MockApiClient extends ApiClient {
  MockApiClient()
      : super(
            dio: Dio(
                BaseOptions(baseUrl: 'http://localhost:3001/api')));
}

// Helper to build a testable widget with overridden providers
Widget buildTestWidget({SetlistState? initialState}) {
  final mockApi = MockApiClient();

  return ProviderScope(
    overrides: [
      apiClientProvider.overrideWithValue(mockApi),
      if (initialState != null)
        setlistProvider.overrideWith((ref) {
          final notifier = SetlistNotifier(mockApi);
          // ignore: invalid_use_of_protected_member
          notifier.state = initialState;
          return notifier;
        }),
    ],
    child: const MaterialApp(
      home: SetlistGenerationScreen(),
    ),
  );
}

Setlist sampleSetlist({bool arranged = false}) {
  return Setlist(
    id: 'test-id',
    prompt: 'chill house vibes',
    model: 'claude-sonnet-4-20250514',
    tracks: [
      const SetlistTrack(
        position: 1,
        title: 'Desert Rose',
        artist: 'Sting',
        bpm: 102.0,
        key: 'A minor',
        camelot: '8A',
        energy: 4,
        transitionNote: 'Open with atmospheric pads',
        transitionScore: null,
        originalPosition: 1,
        source: 'catalog',
        trackId: 't1',
      ),
      SetlistTrack(
        position: 2,
        title: 'Habibi',
        artist: 'Amr Diab',
        bpm: 128.0,
        key: 'B minor',
        camelot: '9A',
        energy: 7,
        transitionNote: 'Build energy',
        transitionScore: arranged ? 0.87 : null,
        originalPosition: 2,
        source: 'suggestion',
      ),
    ],
    notes: 'A chill set',
    harmonicFlowScore: arranged ? 82.0 : null,
    createdAt: '2026-03-02T12:00:00Z',
  );
}

void main() {
  testWidgets('Prompt input renders', (tester) async {
    await tester.pumpWidget(buildTestWidget());

    expect(find.byType(TextField), findsOneWidget);
    expect(find.text('Generate'), findsOneWidget);
    expect(find.text('Describe your ideal set'), findsOneWidget);
  });

  testWidgets('Loading indicator shows during generation', (tester) async {
    await tester.pumpWidget(buildTestWidget(
      initialState: const SetlistState(isGenerating: true),
    ));

    expect(find.byType(CircularProgressIndicator), findsWidgets);
    expect(find.text('Generating your setlist...'), findsOneWidget);
  });

  testWidgets('Setlist renders with track fields', (tester) async {
    await tester.pumpWidget(buildTestWidget(
      initialState: SetlistState(setlist: sampleSetlist()),
    ));

    expect(find.text('Desert Rose'), findsOneWidget);
    expect(find.text('Sting'), findsOneWidget);
    expect(find.text('Habibi'), findsOneWidget);
    expect(find.text('2 tracks'), findsOneWidget);
    expect(find.text('Catalog'), findsOneWidget);
    expect(find.text('Suggestion'), findsOneWidget);
  });

  testWidgets('Error state shows message and try again', (tester) async {
    await tester.pumpWidget(buildTestWidget(
      initialState: const SetlistState(
        error: 'No tracks in your catalog. Import music first.',
      ),
    ));

    expect(
      find.text('No tracks in your catalog. Import music first.'),
      findsOneWidget,
    );
    expect(find.text('Try Again'), findsOneWidget);
  });

  testWidgets('Arrange button appears for unarranged setlist', (tester) async {
    await tester.pumpWidget(buildTestWidget(
      initialState: SetlistState(setlist: sampleSetlist()),
    ));

    expect(find.text('Arrange'), findsOneWidget);
  });

  testWidgets('Transition scores show after arrange', (tester) async {
    await tester.pumpWidget(buildTestWidget(
      initialState: SetlistState(setlist: sampleSetlist(arranged: true)),
    ));

    // After arrangement, arrange button should be gone
    expect(find.text('Arrange'), findsNothing);
    // Flow score chip should appear (header + track-level chips)
    expect(find.textContaining('Flow:'), findsWidgets);
  });

  testWidgets('Empty catalog guidance in error state', (tester) async {
    await tester.pumpWidget(buildTestWidget(
      initialState: const SetlistState(
        error: 'No tracks in your catalog. Import music first.',
      ),
    ));

    expect(find.byIcon(Icons.error_outline), findsOneWidget);
    expect(find.text('Try Again'), findsOneWidget);
  });
}
