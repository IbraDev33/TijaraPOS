import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tijarapos_mobile/app.dart';
import 'package:tijarapos_mobile/core/constants/app_constants.dart';

void main() {
  testWidgets('renders the Phase 1 scaffold placeholder', (
    WidgetTester tester,
  ) async {
    await tester.pumpWidget(const TijaraPosApp());
    await tester.pumpAndSettle();

    expect(find.text(AppConstants.appName), findsOneWidget);
    expect(find.byType(Scaffold), findsOneWidget);
  });
}
