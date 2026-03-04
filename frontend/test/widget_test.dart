import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:ethnomusicology_frontend/main.dart';

void main() {
  testWidgets('App renders home screen', (WidgetTester tester) async {
    await tester.pumpWidget(
      const ProviderScope(child: TarabStudioApp()),
    );
    await tester.pumpAndSettle();

    expect(find.text('Tarab Studio'), findsOneWidget);
    expect(find.text('Welcome to Tarab Studio'), findsOneWidget);
  });
}
