import 'dart:developer' as developer;
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/rust/api/frontend.dart';
import 'package:ui/src/rust/frb_generated.dart';
import 'package:ui/src/segments_widget.dart';
import 'package:window_size/window_size.dart';

Future<void> main() async {
  developer.log("START");
  WidgetsFlutterBinding.ensureInitialized();
  
  if (Platform.isLinux || Platform.isMacOS || Platform.isWindows) {
    setWindowTitle('WPX');
    setWindowMinSize(const Size(700, 400));
    setWindowMaxSize(const Size(700, 400));
  }
  
  await RustLib.init();
  Frontend instance = await Frontend.create();
  developer.log("frontend loaded");
  runApp(MyApp(frontend: instance,));
}

class MyApp extends StatelessWidget {
  final Frontend frontend;
  const MyApp({super.key,required this.frontend});

  @override
  Widget build(BuildContext context) {
    var scaffold=Scaffold(
        appBar: AppBar(title: const Text('WPX')),
        body: SegmentsWidget(key:ValueKey(frontend.epsilon())),
      );
    var home=SegmentsProvider(
      notifier: FrontendNotifier(frontend: frontend),
      child: scaffold,
    );

    return MaterialApp(
      home: home
    );
  }
}
