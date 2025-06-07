import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/rust/frb_generated.dart';
import 'package:ui/src/segments_widget.dart';

Future<void> main() async {
  developer.log("START");
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  Bridge instance = await Bridge.create();
  developer.log("frontend loaded");
  runApp(MyApp(bridge: instance,));
}

class MyApp extends StatelessWidget {
  final Bridge bridge;
  const MyApp({super.key,required this.bridge});

  @override
  Widget build(BuildContext context) {
    var scaffold=Scaffold(
        appBar: AppBar(title: const Text('WPX')),
        body: SegmentsConsumer(),
      );
    var home=ChangeNotifierProvider(
      create: (ctx) => SegmentsProvider(bridge),
      child: scaffold,
    );

    return MaterialApp(
      home: home
    );
  }
}
