import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:test_async/src/rust/api/simple.dart';

class AsyncTestWidget extends StatefulWidget {
  const AsyncTestWidget({super.key});

  @override
  State<AsyncTestWidget> createState() => _AsyncTestWidgetState();
}

class _AsyncTestWidgetState extends State<AsyncTestWidget> {
  int count = 0;
  bool localProcessing = false;

  String localText() {
    if (localProcessing) {
      return "local processing ...";
    }
    return "local done";
  }

  String rustText() {
    if (rustProcessing) {
      return "rust processing ...";
    }
    return "rust done";
  }

  void onLocalPressed() async {
    setState(() {
      localProcessing = true;
    });
    await localprocess();
    setState(() {
      localProcessing = false;
    });
  }

  Future<int> localprocess() {
    developer.log("[start: $count]");
    var ret = Future<int>.delayed(const Duration(seconds: 2), () {
      count = count + 2;
      return count;
    });
    developer.log("[end:  $count]");
    return ret;
  }

  bool rustProcessing = false;

  void onRustPressed() async {
    setState(() {
      rustProcessing = true;
    });
    count = await rustprocess();
    setState(() {
      rustProcessing = false;
    });
  }

  Future<int> rustprocess() {
    return process(count: count);
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        ElevatedButton(onPressed: onLocalPressed, child: Text("$count")),
        SizedBox(height: 20),
        ElevatedButton(onPressed: onRustPressed, child: Text("$count")),
        SizedBox(height: 20),
        Text(localText()),
        SizedBox(height: 20),
        Text(rustText()),
      ],
    );
  }
}
