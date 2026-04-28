import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:wpx/src/screens/shell/screen_shell.dart';
import 'package:wpx/src/models/screen_configuration.dart';
import 'package:wpx/src/models/root.dart';
import 'package:wpx/src/models/kindsmodel.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/models/stackviewscontroller.dart';
import 'package:wpx/src/models/futurerenderer.dart';
import 'package:provider/provider.dart';
import 'package:go_router/go_router.dart';
import 'package:wpx/src/rust/api/bridge.dart' as bridge;
import 'package:wpx/src/rust/frb_generated.dart';
import 'package:wpx/main.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

/// Golden test for the home screen using the real Rust bridge.
///
/// The EventModel is created with enableStream: false to avoid creating
/// the FFI stream which cannot be properly closed and causes test hangs.
/// This is the recommended approach for testing with flutter_rust_bridge streams.
void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async {
    await RustLib.init();
  });

  testWidgets('Home screen golden test', (WidgetTester tester) async {
    // Create real backend
    final backend = bridge.Bridge.make();

    // Create mock package info
    final packageInfo = PackageInfo(
      appName: 'WPX Test',
      packageName: 'com.test.wpx',
      version: '1.0.0',
      buildNumber: '1',
    );

    // Create a simple router for testing
    final router = GoRouter(
      initialLocation: '/',
      routes: [
        GoRoute(path: '/', builder: (context, state) => const ScreenShell()),
      ],
    );

    // Build the widget with all necessary providers
    await tester.pumpWidget(
      MultiProvider(
        providers: [
          ChangeNotifierProvider(create: (_) => RootModel(backend: backend)),
          ChangeNotifierProvider(create: (_) => ScreenConfiguration()),
          ChangeNotifierProvider(
            create: (_) => EventModel(backend: backend, enableStream: false),
          ), // Disable stream in tests
          ChangeNotifierProvider(create: (_) => FociModel()),
          ChangeNotifierProvider(create: (_) => KindsModel()),
          ChangeNotifierProvider(
            create: (_) => ParameterModel(backend: backend),
          ),
          ChangeNotifierProvider(
            create:
                (_) => StackViewsController(
                  exposed: StackViewsController.wmp(),
                  scales: {TrackData.profile: 1.5, TrackData.map: 1.5},
                ),
          ),
          ChangeNotifierProvider(
            create: (_) => PackageModel(packageInfo: packageInfo),
          ),
        ],
        child: MaterialApp.router(
          routerConfig: router,
          theme: ThemeData(
            textTheme: const TextTheme(
              bodyMedium: TextStyle(fontSize: 12.0),
              bodyLarge: TextStyle(fontSize: 12.0),
            ),
          ),
        ),
      ),
      duration: Duration.zero, // Don't wait for animations
    );

    // Pump once more to let the layout settle
    await tester.pump(Duration.zero);

    // Compare against golden file
    await expectLater(
      find.byType(MaterialApp),
      matchesGoldenFile('goldens/home_screen.png'),
    );

    // Explicitly dispose the widget tree to trigger provider cleanup
    // This calls dispose() on all ChangeNotifier providers, including EventModel
    await tester.pumpWidget(const SizedBox.shrink());

    // Give the test framework a moment to process the disposal
    await tester.pump(Duration.zero);
  });
}
