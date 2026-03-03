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

Setlist sampleSetlist({
  bool arranged = false,
  double? catalogPercentage,
  String? catalogWarning,
  List<BpmWarning>? bpmWarnings,
}) {
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
    catalogPercentage: catalogPercentage,
    catalogWarning: catalogWarning,
    bpmWarnings: bpmWarnings ?? const [],
  );
}

void main() {
  // -----------------------------------------------------------------------
  // Existing tests (adapted for new tab-based layout)
  // -----------------------------------------------------------------------

  testWidgets('Prompt input renders in Describe a Vibe tab', (tester) async {
    await tester.pumpWidget(buildTestWidget());
    await tester.pumpAndSettle();

    // Tab bar should be visible
    expect(find.text('Describe a Vibe'), findsOneWidget);
    expect(find.text('From Spotify'), findsOneWidget);
    expect(find.text('From Tracklist'), findsOneWidget);

    // First tab should show prompt input and generate button
    expect(find.text('Describe your ideal setlist...'), findsOneWidget);
    expect(find.text('Generate'), findsOneWidget);
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

  // -----------------------------------------------------------------------
  // T11 new tests
  // -----------------------------------------------------------------------

  testWidgets('All three tabs render', (tester) async {
    await tester.pumpWidget(buildTestWidget());
    await tester.pumpAndSettle();

    expect(find.text('Describe a Vibe'), findsOneWidget);
    expect(find.text('From Spotify'), findsOneWidget);
    expect(find.text('From Tracklist'), findsOneWidget);

    // Tap From Spotify tab
    await tester.tap(find.text('From Spotify'));
    await tester.pumpAndSettle();
    expect(find.text('Spotify Playlist URL'), findsOneWidget);
    expect(find.text('Import & Generate'), findsOneWidget);

    // Tap From Tracklist tab
    await tester.tap(find.text('From Tracklist'));
    await tester.pumpAndSettle();
    expect(find.text('Generate from Tracklist'), findsOneWidget);
  });

  testWidgets('Energy profile chips are tappable', (tester) async {
    await tester.pumpWidget(buildTestWidget());
    await tester.pumpAndSettle();

    // All 4 profiles should be visible
    expect(find.text('Warm-Up'), findsOneWidget);
    expect(find.text('Peak-Time'), findsOneWidget);
    expect(find.text('Journey'), findsOneWidget);
    expect(find.text('Steady'), findsOneWidget);

    // Tap Journey chip
    await tester.tap(find.text('Journey'));
    await tester.pumpAndSettle();

    // The chip should be selected (ChoiceChip changes appearance)
    final chip = tester.widget<ChoiceChip>(
      find.ancestor(
        of: find.text('Journey'),
        matching: find.byType(ChoiceChip),
      ),
    );
    expect(chip.selected, isTrue);
  });

  testWidgets('Creative mode toggle changes state', (tester) async {
    await tester.pumpWidget(buildTestWidget());
    await tester.pumpAndSettle();

    // Should find the switch tile
    expect(find.text('Creative Mode'), findsOneWidget);
    expect(find.text('Unexpected but compatible combinations'),
        findsOneWidget);

    // Toggle it on
    await tester.tap(find.byType(SwitchListTile));
    await tester.pumpAndSettle();

    final switchTile = tester.widget<SwitchListTile>(
      find.byType(SwitchListTile),
    );
    expect(switchTile.value, isTrue);
  });

  testWidgets('Set length slider in range 5-30', (tester) async {
    await tester.pumpWidget(buildTestWidget());
    await tester.pumpAndSettle();

    // Default should show 15 tracks
    expect(find.text('15 tracks'), findsOneWidget);
    expect(find.text('Set Length'), findsOneWidget);

    final slider = tester.widget<Slider>(find.byType(Slider));
    expect(slider.min, 5);
    expect(slider.max, 30);
    expect(slider.value, 15);
  });

  testWidgets('BPM warning badge renders for flagged transitions',
      (tester) async {
    await tester.pumpWidget(buildTestWidget(
      initialState: SetlistState(
        setlist: sampleSetlist(
          bpmWarnings: [
            const BpmWarning(
                fromPosition: 1, toPosition: 2, bpmDelta: 26.0),
          ],
        ),
      ),
    ));

    // Both positions 1 and 2 should show the warning
    expect(find.text('Large BPM jump'), findsNWidgets(2));
    expect(find.byIcon(Icons.warning_amber), findsNWidgets(2));
  });

  testWidgets('Catalog percentage displays correctly', (tester) async {
    await tester.pumpWidget(buildTestWidget(
      initialState: SetlistState(
        setlist: sampleSetlist(catalogPercentage: 75.0),
      ),
    ));

    expect(find.text('Catalog: 75%'), findsOneWidget);
  });

  testWidgets('Catalog percentage shows warning color when low',
      (tester) async {
    await tester.pumpWidget(buildTestWidget(
      initialState: SetlistState(
        setlist: sampleSetlist(
          catalogPercentage: 20.0,
          catalogWarning: 'Low catalog match',
        ),
      ),
    ));

    expect(find.text('Catalog: 20%'), findsOneWidget);
    expect(find.text('Low catalog match'), findsOneWidget);
  });
}
