import 'dart:developer' as developer;
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:ui/src/rust/frb_generated.dart';
import 'package:ui/src/segments_widget.dart';
import 'package:ui/src/globalfrontend.dart';
import 'package:ui/src/rust/api/frontend.dart';
import 'package:window_size/window_size.dart';


Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  if (Platform.isLinux || Platform.isMacOS || Platform.isWindows) {
    setWindowTitle('WPX');
    setWindowMinSize(const Size(700, 700));
    setWindowMaxSize(const Size(700, 700));
  }
  await RustLib.init();
  runApp(const MyApp());
}

class WaitWidget extends StatefulWidget {
  const WaitWidget({super.key});
  @override
  State<WaitWidget> createState() => _WaitWidgetState();
}

class _WaitWidgetState extends State<WaitWidget> {
  bool isLoading = true;
  @override
  void initState() {
    developer.log("init initState"); 
    super.initState();
    _initializeFrontend();
  }
  Future<void> _initializeFrontend() async {
    developer.log("init frontend"); 
    Frontend instance = await Frontend.create();
    developer.log("go frontend");
    GlobalFrontend g = GlobalFrontend();
    g.setFrontend(instance);
    setState(() {
      isLoading=false;
    });
  }

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 500.0,
      height: 600.0,
      child: Builder(
        builder: (context) {
          if (isLoading) {
            return const Center(child: Text("Loading frontend"));
          }
          developer.log("frontend loaded?");
          assert(GlobalFrontend().loaded());
          return SegmentsWidget();
        },
      ),
    );
  }
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('WPX')),
        body: Center(
          child: Column(
            children: [
              WaitWidget()
            ],
          ),
        ),
      ),
    );
  }
}
