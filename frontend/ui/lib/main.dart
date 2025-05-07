import 'dart:io';

import 'package:flutter/material.dart';
import 'package:ui/src/rust/frb_generated.dart';
import 'package:ui/src/profile_screen.dart';
import 'package:window_size/window_size.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  if (Platform.isLinux || Platform.isMacOS || Platform.isWindows) {
    setWindowTitle('WPX');
    setWindowMinSize(const Size(700, 500));
    setWindowMaxSize(const Size(700, 500));
  }
  await RustLib.init();
  runApp(const MyApp());
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
              ProfileScreen(),
            ],
          ),
        ),
      ),
    );
  }
}
