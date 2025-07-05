import 'dart:developer' as developer;
import 'package:flutter/material.dart';
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
  developer.log("frontend loaded");
  runApp(Application());
}

class Application extends StatelessWidget {
  const Application({super.key});

  @override
  Widget build(BuildContext context) {
    var scaffold = Scaffold(
      appBar: AppBar(title: const Text('WPX 0.0.7')),
      body: SegmentsConsumer(),
    );
    var home = ChangeNotifierProvider(
      create: (ctx) => SegmentsProvider(),
      child: scaffold,
    );

    return MaterialApp(home: home);
  }
}
