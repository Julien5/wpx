import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/rust/api/frontend.dart';
import 'package:ui/src/rust/frb_generated.dart';
import 'package:ui/src/segments_widget.dart';

Future<void> main() async {
  developer.log("START");
  WidgetsFlutterBinding.ensureInitialized();
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
        body: SegmentsConsumer(),
      );
    var home=ChangeNotifierProvider(
      create: (ctx) => SegmentsProvider(frontend),
      child: scaffold,
    );

    return MaterialApp(
      home: home
    );
  }
}
