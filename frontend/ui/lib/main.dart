import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/futurerenderer.dart';
import 'package:wpx/src/models/kindsmodel.dart';
import 'package:wpx/src/models/root.dart';
import 'package:wpx/src/models/screen_configuration.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/models/stackviewscontroller.dart';
import 'package:wpx/src/rust/api/bridge.dart' as bridge;
import 'package:wpx/src/routes.dart';
import 'package:wpx/src/rust/frb_generated.dart';
import 'package:wpx/src/utils/utils.dart';

import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:window_manager/window_manager.dart';
import 'package:window_size/window_size.dart'; // Import kIsWeb

Future<void> main() async {
  developer.log("START");
  WidgetsFlutterBinding.ensureInitialized();
  if (!kIsWeb) {
    if (Platform.isWindows || Platform.isLinux || Platform.isMacOS) {
      // simlulate mobile screen size
      setWindowFrame(Rect.fromLTWH(1500, 150, 400, 675));
      //setWindowFrame(Rect.fromLTWH(150, 150, 1366, 768));
    }
    await windowManager.ensureInitialized();
    await windowManager.setIcon('assets/icons/png/wpx_icon.png');
  }
  await RustLib.init();
  PackageInfo packageInfo = await PackageInfo.fromPlatform();
  developer.log("frontend loaded");
  runApp(
    ApplicationProvider(
      packageInfo: packageInfo,
      backend: bridge.Bridge.make(),
      child: Application(),
    ),
  );
}

class PackageModel extends ChangeNotifier {
  final PackageInfo packageInfo;
  PackageModel({required this.packageInfo});
}

class TrackProvider extends StatelessWidget {
  final Widget child;
  const TrackProvider({super.key, required this.child});

  SegmentModel _create(bridge.Bridge backend) {
    developer.log("create track segment");
    return SegmentModel(backend: backend, segment: backend.trackSegment());
  }

  @override
  Widget build(BuildContext context) {
    bridge.Bridge backend = getBackend(context);
    RootModel root = Provider.of<RootModel>(context);
    developer.log("build TrackProvider: loaded: ${root.isLoaded()}");
    if (root.isLoaded()) {
      /* we cannot simply:
          return MultiProvider(
            providers: [ChangeNotifierProvider(create: (_) => _create(backend))],
            child: child,
          );
        because ChangeNotifierProvider's create callback is only called once when 
        the provider is first created. 
      */
      return ChangeNotifierProxyProvider<RootModel, SegmentModel>(
        create: (_) => _create(backend),
        update: (context, rootModel, previousSegment) {
          // This runs on every rebuild
          return _create(getBackend(context));
        },
        child: child,
      );
    }
    return child;
  }
}

class ApplicationProvider extends StatelessWidget {
  final Widget child;
  final PackageInfo? packageInfo;
  final bridge.Bridge backend;
  const ApplicationProvider({
    super.key,
    required this.child,
    this.packageInfo,
    required this.backend,
  });
  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [
        ChangeNotifierProvider(create: (_) => RootModel(backend: backend)),
        ChangeNotifierProvider(create: (_) => ScreenConfiguration()),
        ChangeNotifierProvider(create: (_) => EventModel(backend: backend)),
        ChangeNotifierProvider(create: (_) => FociModel()),
        ChangeNotifierProvider(create: (_) => KindsModel()),
        ChangeNotifierProvider(create: (_) => ParameterModel(backend: backend)),
        ChangeNotifierProvider(
          create:
              (_) => StackViewsController(
                exposed: StackViewsController.wmp(),
                scales: {TrackData.profile: 1.5, TrackData.map: 1.5},
              ),
        ),
        ChangeNotifierProvider(
          create: (_) => PackageModel(packageInfo: packageInfo!),
        ),
      ],
      child: TrackProvider(child: child),
    );
  }
}

final GoRouter router = getRouter();

class Application extends StatelessWidget {
  const Application({super.key});

  @override
  Widget build(BuildContext context) {
    ScreenConfiguration screen = Provider.of<ScreenConfiguration>(context);
    double textBaseSize = screen.isMobile() ? 12.0 : 14.0;
    return MaterialApp.router(
      routeInformationParser: null,
      routerConfig: router,
      title: "WPX",
      theme: ThemeData(
        textTheme: TextTheme(
          // bodyMedium is the default style for the Text widget
          bodyMedium: TextStyle(fontSize: textBaseSize),
          // You can also map this to other styles like bodyLarge or titleMedium
          bodyLarge: TextStyle(fontSize: textBaseSize),
        ),
        pageTransitionsTheme: PageTransitionsTheme(
          builders: {
            TargetPlatform.android: ZoomPageTransitionsBuilder(),
            TargetPlatform.iOS: CupertinoPageTransitionsBuilder(),
            TargetPlatform.linux: CupertinoPageTransitionsBuilder(),
            //TargetPlatform.linux: ZoomPageTransitionsBuilder(),
            //TargetPlatform.linux:PredictiveBackPageTransitionsBuilder(),
          },
        ),
      ),
    );
  }
}
