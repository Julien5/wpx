import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/screen_configuration.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;
import 'package:ui/src/screens/home/home_screen.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/frb_generated.dart';

import 'package:window_size/window_size.dart';
import 'dart:io';
import 'package:flutter/foundation.dart'; // Import kIsWeb

Future<void> main() async {
  developer.log("START");
  WidgetsFlutterBinding.ensureInitialized();
  if (!kIsWeb) {
    if (Platform.isWindows || Platform.isLinux || Platform.isMacOS) {
      setWindowFrame(Rect.fromLTWH(1500, 150, 400, 675));
    }
  }
  await RustLib.init();
  PackageInfo packageInfo = await PackageInfo.fromPlatform();
  developer.log("frontend loaded");
  runApp(ApplicationProvider(packageInfo: packageInfo, backend:bridge.Bridge.make(),child: Application()));
}

class PackageModel extends ChangeNotifier {
  final PackageInfo packageInfo;
  PackageModel({required this.packageInfo});
}

class ApplicationProvider extends StatelessWidget {
  final Widget child;
  final PackageInfo? packageInfo;
  final bridge.Bridge backend;
  const ApplicationProvider({super.key, required this.child, this.packageInfo, required this.backend});
  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [
        ChangeNotifierProvider(create: (_) => RootModel(backend:backend)),
        ChangeNotifierProvider(create: (_) => ScreenConfiguration()),
        ChangeNotifierProvider(create: (_) => ParameterModel(backend:backend)),
        ChangeNotifierProvider(
          create: (_) => PackageModel(packageInfo: packageInfo!),
        ),
      ],
      child: child,
    );
  }
}

class Application extends StatelessWidget {
  const Application({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: "WPX",
      onGenerateRoute: RouteManager.generateRoute,
      initialRoute: RouteManager.home,
      home: HomeScreen(),
      // 2. The builder wraps the 'home' widget
      builder: (context, child) {
        return LayoutBuilder(
          builder: (context, constraints) {
            Future.microtask(() {
              if (!context.mounted) return;
              context.read<ScreenConfiguration>().updateConstraints(
                constraints,
              );
            });
            return child!;
          },
        );
      },
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
