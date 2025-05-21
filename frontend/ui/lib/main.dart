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
    setWindowMinSize(const Size(700, 700));
    setWindowMaxSize(const Size(700, 700));
  }
  await RustLib.init();
  Frontend instance = await Frontend.create();
  developer.log("frontend loaded");
  BackendNotifier notifier =  BackendNotifier(frontend:instance);
  runApp(BackendModel(notifier:notifier, child: const MyApp()));
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    BackendModel backend = BackendModel.of(context);
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('WPX')),
        body: SegmentsWidget(key:ValueKey(backend.epsilon())),
      ),
    );
  }
}
