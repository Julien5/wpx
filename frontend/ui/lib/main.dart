import 'dart:developer' as developer;
import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/events.dart';
import 'package:wpx/src/models/futurerenderer.dart';
import 'package:wpx/src/models/kindsmodel.dart';
import 'package:wpx/src/models/root.dart';
import 'package:wpx/src/models/screen_configuration.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/models/stackviewscontroller.dart';
import 'package:wpx/src/rust/api/bridge.dart' as bridge;
import 'package:wpx/src/routes.dart';
import 'package:wpx/src/rust/frb_generated.dart';

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
  await bridge.Bridge.initPdfFonts();
  final packageInfo = await PackageInfo.fromPlatform();
  final backend = bridge.Bridge.make();

  developer.log("frontend loaded");
  runApp(
    ApplicationProvider(
      packageInfo: packageInfo,
      backend: backend,
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

  SegmentModel _createSegmentModel(RootModel root) {
    developer.log("create track segment");
    return SegmentModel(
      backend: root.backend,
      segment: root.backend.trackSegment(),
    );
  }

  @override
  Widget build(BuildContext context) {
    RootModel root = context.watch<RootModel>();
    developer.log("build TrackProvider: loaded: ${root.isLoaded()}");
    return ChangeNotifierProxyProvider<RootModel, SegmentModel?>(
      create: (_) => root.isLoaded() ? _createSegmentModel(root) : null,
      update: (context, rootModel, previousSegment) {
        return rootModel.isLoaded() ? _createSegmentModel(root) : null;
      },
      child: child,
    );
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
        ChangeNotifierProvider(create: (_) => RoutesHistory()),
        ChangeNotifierProvider(create: (_) => EventModel(backend: backend)),
        ChangeNotifierProvider(create: (_) => KindsModel()),
        ChangeNotifierProvider(
          create: (_) => PackageModel(packageInfo: packageInfo!),
        ),
        ChangeNotifierProvider(
          create:
              (_) => StackViewsController(
                exposed: StackViewsController.wmp(),
                scales: {
                  BridgeRenderFunction.profile: 1.5,
                  BridgeRenderFunction.map: 1.5,
                },
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
    ScreenConfiguration screen = context.watch<ScreenConfiguration>();
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
