import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:ethnomusicology_frontend/main.dart';

void main() {
  testWidgets('App renders home screen', (WidgetTester tester) async {
    await tester.pumpWidget(
      const ProviderScope(child: SalamicVibesApp()),
    );
    await tester.pumpAndSettle();

    expect(find.text('Salamic Vibes'), findsOneWidget);
    expect(find.text('Welcome to Salamic Vibes'), findsOneWidget);
  });
}
