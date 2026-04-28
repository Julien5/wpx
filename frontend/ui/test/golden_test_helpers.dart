import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:package_info_plus/package_info_plus.dart';
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
import 'package:wpx/main.dart' show TrackProvider;

/// Initialize the Rust library for golden tests
Future<void> initializeGoldenTest() async {
  TestWidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
}

/// Setup the window size for golden tests (1000x800)
void setupGoldenTestWindowSize(WidgetTester tester) {
  tester.view.physicalSize = const Size(1000, 800);
  tester.view.devicePixelRatio = 1.0;
  addTearDown(() => tester.view.resetPhysicalSize());
}

/// Create a mock package info for testing
PackageInfo createMockPackageInfo() {
  return PackageInfo(
    appName: 'WPX Test',
    packageName: 'com.test.wpx',
    version: '1.0.0',
    buildNumber: '1',
  );
}

/// Create a test router with the given routes
GoRouter createTestRouter(List<String> routes) {
  return GoRouter(
    initialLocation: '/',
    routes: routes.map((route) {
      return GoRoute(
        path: route,
        builder: (context, state) => const Placeholder(),
      );
    }).toList(),
  );
}

/// Build the app widget with all necessary providers for golden tests
Widget buildTestApp({
  required bridge.Bridge backend,
  required PackageInfo packageInfo,
  required GoRouter router,
  Size? size,
}) {
  return MultiProvider(
    providers: [
      ChangeNotifierProvider(create: (_) => RootModel(backend: backend)),
      ChangeNotifierProvider(create: (_) => ScreenConfiguration()),
      ChangeNotifierProvider(
        create: (_) => EventModel(backend: backend),
      ),
      ChangeNotifierProvider(create: (_) => FociModel()),
      ChangeNotifierProvider(create: (_) => KindsModel()),
      ChangeNotifierProvider(
        create: (_) => ParameterModel(backend: backend),
      ),
      ChangeNotifierProvider(
        create: (_) => StackViewsController(
          exposed: StackViewsController.wmp(),
          scales: {TrackData.profile: 1.5, TrackData.map: 1.5},
        ),
      ),
      ChangeNotifierProvider(
        create: (_) => PackageModel(packageInfo: packageInfo),
      ),
    ],
    child: TrackProvider(
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
  );
}

/// Cleanup helper for golden tests
Future<void> cleanupGoldenTest(WidgetTester tester) async {
  await tester.pumpWidget(const SizedBox.shrink());
  await tester.pumpAndSettle();
}
