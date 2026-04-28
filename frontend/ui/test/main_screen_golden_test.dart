// ignore_for_file: avoid_print

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wpx/src/screens/shell/screen_shell.dart';
import 'package:wpx/src/rust/api/bridge.dart' as bridge;
import 'package:go_router/go_router.dart';

import 'golden_test_helpers.dart';

/// Golden test for the main screen (overview) after loading completes.
void main() {
  print("[0a]");
  
  setUpAll(() async {
    await initializeGoldenTest();
  });
  print("[0b]");
  

  testWidgets('Main screen golden test', (WidgetTester tester) async {
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
        GoRoute(path: '/overview', builder: (context, state) => const ScreenShell()),
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

    // Wait for the load screen to complete (OK button enabled + done text appears)
    print("[8] Waiting for OK button to be enabled and done label to appear");
    
    final deadline = DateTime.now().add(const Duration(seconds: 30));
    bool loadComplete = false;
    
    while (DateTime.now().isBefore(deadline) && !loadComplete) {
      // Use runAsync to allow real async operations (for FFI stream events)
      await tester.runAsync(() async {
        await Future.delayed(const Duration(milliseconds: 100));
      });
      // Pump with duration to advance fake async clock (for Future.delayed in job sequencing)
      await tester.pump(const Duration(milliseconds: 300));
      
      final okButton = find.widgetWithText(ElevatedButton, 'OK');
      
      // Debug: Check for done text
      final allTexts = find.byType(Text);
      final textWidgets = allTexts.evaluate().map((e) => e.widget as Text);
      final textStrings = textWidgets.map((t) => t.data ?? t.textSpan?.toPlainText() ?? '').where((s) => s.isNotEmpty).toList();
      final hasDoneText = textStrings.any((text) => text == 'done');
      
      if (okButton.evaluate().isNotEmpty && hasDoneText) {
        final ElevatedButton buttonWidget = tester.widget(okButton);
        if (buttonWidget.onPressed != null) {
          loadComplete = true;
          print("[9] OK button is enabled and done label is visible");
          break;
        }
      }
    }
    
    expect(loadComplete, isTrue, reason: 'Load screen should complete within timeout');
    
    // Pump and settle to ensure all pending operations complete
    await tester.pumpAndSettle();
    print("[10] Load screen completed, tapping OK button");

    // Find the OK button again
    final okButton = find.widgetWithText(ElevatedButton, 'OK');
    expect(okButton, findsOneWidget, reason: 'OK button should exist');
    
    // Tap the OK button to navigate to main screen
    await tester.tap(okButton);
    print("[11] Tapped OK button");
    
    // Pump to process navigation
    await tester.pumpAndSettle();
    print("[12] After pumpAndSettle - checking current screen");
    
    // Debug: Check what's on screen
    final allTexts = find.byType(Text);
    final textWidgets = allTexts.evaluate().map((e) => e.widget as Text);
    final textStrings = textWidgets.map((t) => t.data ?? t.textSpan?.toPlainText() ?? '').where((s) => s.isNotEmpty).toList();
    print("  Current screen texts: ${textStrings.join(', ')}");
    
    // Check if we navigated successfully
    final stillOnLoadScreen = textStrings.contains('Loading...') || textStrings.contains('Loaded');
    if (stillOnLoadScreen) {
      print("  WARNING: Still on load screen after tapping OK");
    }

    // Wait for main screen to be ready (rendering to complete)
    print("[13] Waiting for main screen rendering to complete");
    
    // Use runAsync to allow rendering futures to complete
    await tester.runAsync(() async {
      await Future.delayed(const Duration(seconds: 5));
    });
    
    // Pump multiple times to allow all rendering to complete and UI to update
    for (int i = 0; i < 10; i++) {
      await tester.pump(const Duration(milliseconds: 500));
    }
    
    print("[14] Main screen should be ready");

    // Compare against golden file
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/main_screen.png'),
    );
    print("[15] Golden comparison done");

    // Cleanup
    await cleanupGoldenTest(tester);
    print("[16] Cleanup done");
  });
}
