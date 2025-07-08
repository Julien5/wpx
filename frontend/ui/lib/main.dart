import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/rust/frb_generated.dart';
import 'package:ui/src/segments_widget.dart';

import 'package:window_size/window_size.dart';
import 'dart:io';
import 'package:flutter/foundation.dart'; // Import kIsWeb

Future<void> main() async {
  developer.log("START");
  WidgetsFlutterBinding.ensureInitialized();
  if (!kIsWeb) {
    if (Platform.isWindows || Platform.isLinux || Platform.isMacOS) {
      setWindowFrame(Rect.fromLTWH(150, 150, 1600, 900));
    }
  }
  await RustLib.init();
  PackageInfo packageInfo = await PackageInfo.fromPlatform();
  developer.log("frontend loaded");
  runApp(Application(packageInfo: packageInfo,));
}

class Application extends StatelessWidget {
  final PackageInfo? packageInfo;
  const Application({super.key,required this.packageInfo});

  @override
  Widget build(BuildContext context) {
    var scaffold = Scaffold(
      appBar: AppBar(title: Text('WPX ${packageInfo!.version}')),
      body: SegmentsConsumer(),
    );
    var home = ChangeNotifierProvider(
      create: (ctx) => SegmentsProvider(),
      child: scaffold,
    );

    return MaterialApp(home: home);
  }
}
