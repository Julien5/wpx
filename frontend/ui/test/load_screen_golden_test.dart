// ignore_for_file: avoid_print

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wpx/src/screens/shell/screen_shell.dart';
import 'package:wpx/src/rust/api/bridge.dart' as bridge;
import 'package:go_router/go_router.dart';

import 'golden_test_helpers.dart';

/// Golden test for the load screen using the real Rust bridge.
void main() {
  print("[0a]");
  
  setUpAll(() async {
    await initializeGoldenTest();
  });
  print("[0b]");
  

  testWidgets('Load screen golden test', (WidgetTester tester) async {
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
        GoRoute(path: '/load', builder: (context, state) => const ScreenShell()),
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
    print("[4] Home screen should be visible");

    // Find and tap the "Load sample" button
    final loadSampleButton = find.text('Load sample');
    expect(loadSampleButton, findsOneWidget);
    print("[5] Found 'Load sample' button, tapping it");
    
    await tester.tap(loadSampleButton);
    print("[6] Tapped 'Load sample' button");
    
    // Pump to process the navigation
    await tester.pumpAndSettle();
    print("[7] After pumpAndSettle - should be on load screen");

    // Wait for the load screen to initialize and complete processing
    // The LoadScreen starts jobs automatically in didChangeDependencies
    // We need to wait until the OK button exists and is enabled
    // Note: EventModel stream may not deliver events in tests, so we check for data presence instead
    print("[8] Waiting for OK button to be enabled and data to load");
    
    // Wait for OK button to be enabled and data to appear (with timeout)
    final deadline = DateTime.now().add(const Duration(seconds: 30));
    bool allConditionsMet = false;
    
    while (DateTime.now().isBefore(deadline) && !allConditionsMet) {
      // Use runAsync to allow real async operations (for FFI stream events)
      await tester.runAsync(() async {
        await Future.delayed(const Duration(milliseconds: 100));
      });
      // Pump with duration to advance fake async clock (for Future.delayed in job sequencing)
      await tester.pump(const Duration(milliseconds: 300));
      
      // Debug: Print all visible texts
      final allTexts = find.byType(Text);
      final textWidgets = allTexts.evaluate().map((e) => e.widget as Text);
      final textStrings = textWidgets.map((t) => t.data ?? t.textSpan?.toPlainText() ?? '').where((s) => s.isNotEmpty).toList();
      print("  Visible texts: ${textStrings.join(', ')}");
      
      final okButton = find.widgetWithText(ElevatedButton, 'OK');
      // Check if there's at least one "done" text (indicating a completed job)
      final hasDoneText = textStrings.any((text) => text == 'done');
      
      print("  Has done text: $hasDoneText, OK button exists: ${okButton.evaluate().isNotEmpty}");
      
      if (okButton.evaluate().isNotEmpty && hasDoneText) {
        final ElevatedButton buttonWidget = tester.widget(okButton);
        if (buttonWidget.onPressed != null) {
          allConditionsMet = true;
          print("[9] OK button is enabled and done label is visible");
          break;
        } else {
          print("  OK button exists but is disabled");
        }
      }
    }
    
    expect(allConditionsMet, isTrue, reason: 'OK button should be enabled and done label should be visible within timeout');
    
    // Pump and settle to ensure all pending timers and async operations complete
    print("[9a] Pumping frames to complete all pending operations");
    await tester.pumpAndSettle();

    // Compare against golden file
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/load_screen.png'),
    );
    print("[10] Golden comparison done");

    // Cleanup
    await cleanupGoldenTest(tester);
    print("[11] Cleanup done");
  });
}
