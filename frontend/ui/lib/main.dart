import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/rust/frb_generated.dart';
import 'package:ui/src/segment_stack.dart';
import 'package:ui/src/segments_widget.dart';
import 'package:ui/src/waypoints_widget.dart';
import 'package:window_size/window_size.dart';
import 'dart:io';

Future<void> main() async {
  developer.log("START");
  WidgetsFlutterBinding.ensureInitialized();
  if (Platform.isWindows || Platform.isLinux || Platform.isMacOS) {
    setWindowFrame(Rect.fromLTWH(150, 150, 1600, 900));
  }
  await RustLib.init();
  Bridge instance = await Bridge.create();
  developer.log("frontend loaded");
  runApp(Application(bridge: instance));
}

class Application extends StatelessWidget {
  final Bridge bridge;
  const Application({super.key, required this.bridge});

  @override
  Widget build(BuildContext context) {
    var scaffold = Scaffold(
      appBar: AppBar(title: const Text('WPX')),
      body: SegmentsConsumer(), //WayPointsConsumer(),
    );
    var home = ChangeNotifierProvider(
      create: (ctx) => SegmentsProvider(bridge),
      child: scaffold,
    );

    return MaterialApp(home: home);
  }
}
