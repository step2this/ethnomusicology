import 'package:dio/dio.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:ethnomusicology_frontend/models/refinement.dart';
import 'package:ethnomusicology_frontend/providers/api_provider.dart';
import 'package:ethnomusicology_frontend/services/api_client.dart';
import 'package:ethnomusicology_frontend/widgets/conversation_history.dart';
import 'package:ethnomusicology_frontend/widgets/refinement_chat_input.dart';
import 'package:ethnomusicology_frontend/widgets/version_history_panel.dart';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Wrap a plain widget in a MaterialApp+Scaffold for rendering.
Widget wrapInMaterial(Widget child) {
  return MaterialApp(
    home: Scaffold(body: child),
  );
}

/// Wrap a widget in a ProviderScope with a real-but-isolated ApiClient so that
/// Riverpod providers (refinementProvider) resolve without hitting the network.
Widget wrapInProviders(Widget child) {
  final dio = Dio(BaseOptions(baseUrl: 'http://localhost:3001/api'));
  // No-op interceptor: immediately resolves every request with an empty body
  // so that `refineSetlist` would fail gracefully (we only test UI state here).
  dio.interceptors.add(_NoOpInterceptor());
  return ProviderScope(
    overrides: [
      apiClientProvider.overrideWithValue(ApiClient(dio: dio)),
    ],
    child: MaterialApp(
      home: Scaffold(body: child),
    ),
  );
}

/// Opens the VersionHistoryPanel end-drawer.
///
/// Uses a fixed pump duration rather than pumpAndSettle because a
/// CircularProgressIndicator keeps the animation loop alive and would cause
/// pumpAndSettle to time out when isLoading is true.
Future<void> openVersionPanel(WidgetTester tester) async {
  await tester.tap(find.text('Open'));
  // Drive enough frames for the drawer slide-in animation to complete.
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 350));
}

// ---------------------------------------------------------------------------
// No-op Dio interceptor — never lets requests reach the network.
// ---------------------------------------------------------------------------
class _NoOpInterceptor extends Interceptor {
  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    handler.resolve(Response(
      requestOptions: options,
      statusCode: 200,
      data: <String, dynamic>{},
    ));
  }
}

// ---------------------------------------------------------------------------
// Sample fixtures
// ---------------------------------------------------------------------------

const testMessages = [
  ConversationMessage(
    id: '1',
    setlistId: 's1',
    role: 'user',
    content: 'Make it energetic',
  ),
  ConversationMessage(
    id: '2',
    setlistId: 's1',
    role: 'assistant',
    content: 'Added high-energy tracks',
  ),
];

final testVersions = [
  const SetlistVersion(
    id: 'v0',
    setlistId: 's1',
    versionNumber: 0,
    actionSummary: 'Initial',
  ),
  const SetlistVersion(
    id: 'v1',
    setlistId: 's1',
    versionNumber: 1,
    action: 'refine',
    actionSummary: 'More energy',
  ),
  const SetlistVersion(
    id: 'v2',
    setlistId: 's1',
    versionNumber: 2,
    action: 'refine',
    actionSummary: 'Add bass',
  ),
];

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

void main() {
  // =========================================================================
  // ConversationHistory
  // =========================================================================
  group('ConversationHistory', () {
    testWidgets('empty state shows placeholder text', (tester) async {
      await tester.pumpWidget(wrapInMaterial(
        const SizedBox(
          height: 300,
          child: ConversationHistory(messages: [], isRefining: false),
        ),
      ));

      expect(
        find.text('Type a message to refine your setlist'),
        findsOneWidget,
      );
    });

    testWidgets('user messages are right-aligned', (tester) async {
      await tester.pumpWidget(wrapInMaterial(
        const SizedBox(
          height: 300,
          child: ConversationHistory(
            messages: testMessages,
            isRefining: false,
          ),
        ),
      ));

      // Find the Align widget that wraps the user message bubble
      final userAlign = tester.widget<Align>(
        find.ancestor(
          of: find.text('Make it energetic'),
          matching: find.byType(Align),
        ),
      );
      expect(userAlign.alignment, Alignment.centerRight);
    });

    testWidgets('assistant messages are left-aligned', (tester) async {
      await tester.pumpWidget(wrapInMaterial(
        const SizedBox(
          height: 300,
          child: ConversationHistory(
            messages: testMessages,
            isRefining: false,
          ),
        ),
      ));

      final assistantAlign = tester.widget<Align>(
        find.ancestor(
          of: find.text('Added high-energy tracks'),
          matching: find.byType(Align),
        ),
      );
      expect(assistantAlign.alignment, Alignment.centerLeft);
    });

    testWidgets('loading indicator shows when isRefining is true',
        (tester) async {
      await tester.pumpWidget(wrapInMaterial(
        const SizedBox(
          height: 300,
          child: ConversationHistory(
            messages: testMessages,
            isRefining: true,
          ),
        ),
      ));

      // The loading row contains "Refining..." text and a progress indicator
      expect(find.text('Refining...'), findsOneWidget);
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
    });

    testWidgets(
        'user message bubble uses primaryContainer color '
        'and assistant uses surfaceContainerHighest', (tester) async {
      await tester.pumpWidget(wrapInMaterial(
        const SizedBox(
          height: 300,
          child: ConversationHistory(
            messages: testMessages,
            isRefining: false,
          ),
        ),
      ));

      // Helper: finds a Container ancestor of [text] and returns its decoration color.
      Color? bubbleColor(String text) {
        final container = tester.widget<Container>(
          find
              .ancestor(
                of: find.text(text),
                matching: find.byType(Container),
              )
              .first,
        );
        return (container.decoration as BoxDecoration?)?.color;
      }

      final theme = Theme.of(tester.element(find.text('Make it energetic')));

      expect(bubbleColor('Make it energetic'),
          theme.colorScheme.primaryContainer);
      expect(bubbleColor('Added high-energy tracks'),
          theme.colorScheme.surfaceContainerHighest);
    });
  });

  // =========================================================================
  // VersionHistoryPanel
  // =========================================================================
  group('VersionHistoryPanel', () {
    /// Utility: build a Scaffold that has a VersionHistoryPanel as an end-drawer
    /// and a button to open it.
    Widget buildDrawerScaffold({
      List<SetlistVersion>? versions,
      int? currentVersion,
      bool isLoading = false,
      void Function(int)? onRevert,
    }) {
      return MaterialApp(
        home: Scaffold(
          endDrawer: VersionHistoryPanel(
            versions: versions,
            currentVersion: currentVersion,
            isLoading: isLoading,
            onRevert: onRevert ?? (_) {},
          ),
          body: Builder(
            builder: (context) => ElevatedButton(
              onPressed: () => Scaffold.of(context).openEndDrawer(),
              child: const Text('Open'),
            ),
          ),
        ),
      );
    }

    testWidgets('shows "No version history yet" when versions is empty',
        (tester) async {
      await tester.pumpWidget(buildDrawerScaffold(versions: []));
      await openVersionPanel(tester);

      expect(find.text('No version history yet'), findsOneWidget);
    });

    testWidgets('shows loading indicator when isLoading is true',
        (tester) async {
      await tester.pumpWidget(buildDrawerScaffold(isLoading: true));
      await openVersionPanel(tester);

      expect(find.byType(CircularProgressIndicator), findsOneWidget);
      expect(find.text('No version history yet'), findsNothing);
    });

    testWidgets('renders version list with version numbers', (tester) async {
      await tester
          .pumpWidget(buildDrawerScaffold(versions: testVersions, currentVersion: 2));
      await openVersionPanel(tester);

      expect(find.text('v0'), findsOneWidget);
      expect(find.text('v1'), findsOneWidget);
      expect(find.text('v2'), findsOneWidget);
    });

    testWidgets('current version shows "Current" chip instead of Revert',
        (tester) async {
      // currentVersion == 2 means v2 is current
      await tester.pumpWidget(
          buildDrawerScaffold(versions: testVersions, currentVersion: 2));
      await openVersionPanel(tester);

      // There should be exactly one "Current" chip
      expect(find.text('Current'), findsOneWidget);

      // The v2 row should NOT have a Revert button
      // There are 3 versions; 2 non-current → 2 Revert buttons
      expect(find.text('Revert'), findsNWidgets(2));
    });

    testWidgets('non-current versions show Revert button', (tester) async {
      await tester.pumpWidget(
          buildDrawerScaffold(versions: testVersions, currentVersion: 2));
      await openVersionPanel(tester);

      // v0 and v1 are not current
      expect(find.text('Revert'), findsNWidgets(2));
    });

    testWidgets('Revert button calls onRevert with correct version number',
        (tester) async {
      int? revertedVersion;
      await tester.pumpWidget(buildDrawerScaffold(
        versions: testVersions,
        currentVersion: 2,
        onRevert: (v) => revertedVersion = v,
      ));
      await openVersionPanel(tester);

      // Tap the first Revert button (belongs to v0 — rendered first in the list)
      await tester.tap(find.text('Revert').first);
      await tester.pump();

      expect(revertedVersion, 0);
    });
  });

  // =========================================================================
  // RefinementChatInput
  // =========================================================================
  group('RefinementChatInput', () {
    testWidgets('shows all 4 quick-command chips', (tester) async {
      await tester.pumpWidget(wrapInProviders(
        const RefinementChatInput(setlistId: 's1', isRefining: false),
      ));

      expect(find.text('!shuffle'), findsOneWidget);
      expect(find.text('!sort-by-bpm'), findsOneWidget);
      expect(find.text('!reverse'), findsOneWidget);
      expect(find.text('!undo'), findsOneWidget);
    });

    testWidgets('text field is disabled when isRefining is true',
        (tester) async {
      await tester.pumpWidget(wrapInProviders(
        const RefinementChatInput(setlistId: 's1', isRefining: true),
      ));

      final textField = tester.widget<TextField>(find.byType(TextField));
      expect(textField.enabled, isFalse);
    });

    testWidgets('send button shows loading indicator when isRefining',
        (tester) async {
      await tester.pumpWidget(wrapInProviders(
        const RefinementChatInput(setlistId: 's1', isRefining: true),
      ));

      // When isRefining, the icon button renders a CircularProgressIndicator
      // inside itself instead of the send Icon.
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
      expect(find.byIcon(Icons.send), findsNothing);
    });
  });
}
