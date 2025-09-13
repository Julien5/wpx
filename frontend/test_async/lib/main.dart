import 'package:flutter/material.dart';
import 'package:test_async/src/rust/api/simple.dart';
import 'package:test_async/src/rust/frb_generated.dart';
import 'package:test_async/testwidget.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('async')),
        body: Center(
          child: Column(
            children: [
              Text(
                'Action: Call Rust `greet("Tom")`\nResult: `${greet(name: "Tom")}`',
              ),
              SizedBox(height:100),
              TestWidget(),
            ],
          ),
        ),
      ),
    );
  }
}
