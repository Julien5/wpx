// ignore_for_file: avoid_print

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wpx/src/screens/shell/screen_shell.dart';
import 'package:wpx/src/rust/api/bridge.dart' as bridge;
import 'package:go_router/go_router.dart';

import 'golden_test_helpers.dart';

/// Golden test for the home screen using the real Rust bridge.
void main() {
  print("[0a]");
  
  setUpAll(() async {
    await initializeGoldenTest();
  });
  print("[0b]");
  

  testWidgets('Home screen golden test', (WidgetTester tester) async {
    setupGoldenTestWindowSize(tester);
    
    // Create real backend
    print("[1]");
    final backend = bridge.Bridge.make();
    print("[2]");
    
    // Create mock package info
    final packageInfo = createMockPackageInfo();

    // Create a simple router for testing
    final router = GoRouter(
      initialLocation: '/',
      routes: [
        GoRoute(path: '/', builder: (context, state) => const ScreenShell()),
      ],
    );
    print("[3]");
    
    // Build the widget with all necessary providers
    await tester.pumpWidget(
      buildTestApp(
        backend: backend,
        packageInfo: packageInfo,
        router: router,
      ),
      duration: Duration.zero,
    );

    // Pump once more to let the layout settle
    await tester.pump(Duration.zero);

    // Compare against golden file
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/home_screen.png'),
    );

    // Cleanup
    await cleanupGoldenTest(tester);
  });
}
