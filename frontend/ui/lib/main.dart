import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/screen_configuration.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/models/trackviewswitch.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/frb_generated.dart';
import 'package:ui/utils.dart';

import 'package:window_size/window_size.dart';
import 'dart:io';
import 'package:flutter/foundation.dart'; // Import kIsWeb

Future<void> main() async {
  developer.log("START");
  WidgetsFlutterBinding.ensureInitialized();
  if (!kIsWeb) {
    if (Platform.isWindows || Platform.isLinux || Platform.isMacOS) {
      //setWindowFrame(Rect.fromLTWH(1500, 150, 400, 675));
    }
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
    Provider.of<RootModel>(context);
    developer.log("build TrackProvider: loaded: ${backend.isLoaded()}");
    if (backend.isLoaded()) {
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
        ChangeNotifierProvider(create: (_) => ParameterModel(backend: backend)),
        ChangeNotifierProvider(
          create: (_) => TrackViewsSwitch(exposed: TrackViewsSwitch.wmp()),
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
    return MaterialApp.router(
      routeInformationParser: null,
      routerConfig: router,
      title: "WPX",
      theme: ThemeData(
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
