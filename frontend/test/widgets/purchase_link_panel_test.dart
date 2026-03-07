import 'package:dio/dio.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:ethnomusicology_frontend/providers/api_provider.dart';
import 'package:ethnomusicology_frontend/widgets/purchase_link_panel.dart';

import '../helpers/mock_api_client.dart';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

final _sampleLinks = {
  'links': [
    {
      'store': 'beatport',
      'name': 'Beatport',
      'url': 'https://beatport.com/search?q=Test',
      'icon': '🎧',
    },
    {
      'store': 'bandcamp',
      'name': 'Bandcamp',
      'url': 'https://bandcamp.com/search?q=Test',
      'icon': '🎵',
    },
    {
      'store': 'juno',
      'name': 'Juno Download',
      'url': 'https://juno.co.uk/search?q=Test',
      'icon': '💿',
    },
    {
      'store': 'traxsource',
      'name': 'Traxsource',
      'url': 'https://traxsource.com/search?q=Test',
      'icon': '🎶',
    },
  ],
};

Widget _buildPanel({
  required MockInterceptor interceptor,
  required client,
  String title = 'Test Track',
  String artist = 'Test Artist',
}) {
  return ProviderScope(
    overrides: [
      apiClientProvider.overrideWithValue(client),
    ],
    child: MaterialApp(
      home: Scaffold(
        body: PurchaseLinkPanel(title: title, artist: artist),
      ),
    ),
  );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

void main() {
  group('PurchaseLinkPanel', () {
    testWidgets('renders collapsed by default with Buy button', (tester) async {
      final (:client, :interceptor) = createMockApiClient();
      interceptor.responseOverride = _sampleLinks;

      await tester.pumpWidget(_buildPanel(
        interceptor: interceptor,
        client: client,
      ));

      // Buy button visible
      expect(find.text('Buy'), findsOneWidget);
      expect(find.byIcon(Icons.shopping_bag_outlined), findsOneWidget);

      // Store chips not visible (collapsed)
      expect(find.text('Beatport'), findsNothing);
      expect(find.text('Bandcamp'), findsNothing);
    });

    testWidgets('expands and shows store links on tap', (tester) async {
      final (:client, :interceptor) = createMockApiClient();
      interceptor.responseOverride = _sampleLinks;

      await tester.pumpWidget(_buildPanel(
        interceptor: interceptor,
        client: client,
      ));

      // Tap the Buy button to expand
      await tester.tap(find.text('Buy'));
      await tester.pumpAndSettle();

      // All 4 store chips appear
      expect(find.text('Beatport'), findsOneWidget);
      expect(find.text('Bandcamp'), findsOneWidget);
      expect(find.text('Juno Download'), findsOneWidget);
      expect(find.text('Traxsource'), findsOneWidget);
    });

    testWidgets('store order matches API response order', (tester) async {
      final (:client, :interceptor) = createMockApiClient();
      interceptor.responseOverride = _sampleLinks;

      await tester.pumpWidget(_buildPanel(
        interceptor: interceptor,
        client: client,
      ));

      await tester.tap(find.text('Buy'));
      await tester.pumpAndSettle();

      // Find all ActionChip widgets and verify order
      final chips = tester.widgetList<ActionChip>(find.byType(ActionChip)).toList();
      expect(chips.length, 4);
      expect((chips[0].label as Text).data, 'Beatport');
      expect((chips[1].label as Text).data, 'Bandcamp');
      expect((chips[2].label as Text).data, 'Juno Download');
      expect((chips[3].label as Text).data, 'Traxsource');
    });

    testWidgets('hides panel when title and artist are empty', (tester) async {
      final (:client, :interceptor) = createMockApiClient();

      await tester.pumpWidget(_buildPanel(
        interceptor: interceptor,
        client: client,
        title: '',
        artist: '',
      ));

      // Should render nothing
      expect(find.text('Buy'), findsNothing);
      expect(find.byIcon(Icons.shopping_bag_outlined), findsNothing);
    });

    testWidgets('handles API error gracefully', (tester) async {
      final (:client, :interceptor) = createMockApiClient();
      interceptor.errorOverride = DioException(
        requestOptions: RequestOptions(path: '/purchase-links'),
        type: DioExceptionType.badResponse,
        response: Response(
          requestOptions: RequestOptions(path: '/purchase-links'),
          statusCode: 500,
        ),
      );

      await tester.pumpWidget(_buildPanel(
        interceptor: interceptor,
        client: client,
      ));

      await tester.tap(find.text('Buy'));
      await tester.pumpAndSettle();

      expect(find.text('Failed to load purchase links'), findsOneWidget);
    });

    testWidgets('shows loading indicator while fetching', (tester) async {
      final (:client, :interceptor) = createMockApiClient();
      interceptor.responseOverride = _sampleLinks;

      await tester.pumpWidget(_buildPanel(
        interceptor: interceptor,
        client: client,
      ));

      await tester.tap(find.text('Buy'));
      // pump once without settling to catch loading state
      await tester.pump();

      expect(find.byType(CircularProgressIndicator), findsOneWidget);

      // Let async complete to avoid pending timer assertion
      await tester.pumpAndSettle();
    });

    testWidgets('shows empty links message when API returns empty list',
        (tester) async {
      final (:client, :interceptor) = createMockApiClient();
      interceptor.responseOverride = {'links': []};

      await tester.pumpWidget(_buildPanel(
        interceptor: interceptor,
        client: client,
      ));

      await tester.tap(find.text('Buy'));
      await tester.pumpAndSettle();

      expect(find.text('No purchase links'), findsOneWidget);
    });

    testWidgets('collapses on second tap', (tester) async {
      final (:client, :interceptor) = createMockApiClient();
      interceptor.responseOverride = _sampleLinks;

      await tester.pumpWidget(_buildPanel(
        interceptor: interceptor,
        client: client,
      ));

      // Expand
      await tester.tap(find.text('Buy'));
      await tester.pumpAndSettle();
      expect(find.text('Beatport'), findsOneWidget);

      // Collapse
      await tester.tap(find.text('Buy'));
      await tester.pumpAndSettle();
      expect(find.text('Beatport'), findsNothing);
    });
  });
}
