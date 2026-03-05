import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:ethnomusicology_frontend/models/setlist_track.dart';
import 'package:ethnomusicology_frontend/widgets/setlist_track_tile.dart';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

SetlistTrack _makeTrack({
  String? confidence,
  String? verificationFlag,
  String? verificationNote,
}) {
  return SetlistTrack(
    position: 1,
    title: 'Test Track',
    artist: 'Test Artist',
    originalPosition: 1,
    source: 'suggestion',
    confidence: confidence,
    verificationFlag: verificationFlag,
    verificationNote: verificationNote,
  );
}

Widget _buildTile(SetlistTrack track) {
  return MaterialApp(
    home: Scaffold(
      body: SetlistTrackTile(track: track),
    ),
  );
}

/// Returns all 8×8 circle Containers with the given color in the widget tree.
Iterable<Container> _findConfidenceDots(
    WidgetTester tester, Color expectedColor) {
  return tester.widgetList<Container>(find.byType(Container)).where((c) {
    final d = c.decoration as BoxDecoration?;
    return d?.color == expectedColor && d?.shape == BoxShape.circle;
  });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

void main() {
  group('confidence dot', () {
    testWidgets('high confidence shows green dot', (tester) async {
      await tester.pumpWidget(_buildTile(_makeTrack(confidence: 'high')));
      expect(_findConfidenceDots(tester, Colors.green), isNotEmpty);
    });

    testWidgets('medium confidence shows amber dot', (tester) async {
      await tester.pumpWidget(_buildTile(_makeTrack(confidence: 'medium')));
      expect(_findConfidenceDots(tester, Colors.amber), isNotEmpty);
    });

    testWidgets('low confidence shows orange dot', (tester) async {
      await tester.pumpWidget(_buildTile(_makeTrack(confidence: 'low')));
      expect(_findConfidenceDots(tester, Colors.orange), isNotEmpty);
    });

    testWidgets('null confidence shows no colored circle dot', (tester) async {
      await tester.pumpWidget(_buildTile(_makeTrack()));
      expect(_findConfidenceDots(tester, Colors.green), isEmpty);
      expect(_findConfidenceDots(tester, Colors.amber), isEmpty);
      expect(_findConfidenceDots(tester, Colors.orange), isEmpty);
    });
  });

  group('verification flag', () {
    testWidgets('flagged track shows verification note text', (tester) async {
      const note = 'Title may be slightly inaccurate';
      await tester.pumpWidget(_buildTile(_makeTrack(
        verificationFlag: 'title_uncertain',
        verificationNote: note,
      )));
      expect(find.text(note), findsOneWidget);
    });

    testWidgets('non-flagged track shows no verification note', (tester) async {
      await tester.pumpWidget(_buildTile(_makeTrack()));
      // No verification note text should appear
      expect(find.text('Title may be slightly inaccurate'), findsNothing);
    });
  });
}
