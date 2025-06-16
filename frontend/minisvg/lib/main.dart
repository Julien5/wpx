import 'package:flutter/material.dart';
import 'package:minisvg/src/minisvg.dart' as minisvg;

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    minisvg.Element root = minisvg.rootElement();
    return MaterialApp(
      title: 'Flutter Demo',
      home: Scaffold(
        appBar: AppBar(title: const Text('Custom Painter Example')),
        body: CustomPaint(
          painter: SvgPainter(root: root),
        ),
      ),
    );
  }
}

class SvgPainter extends CustomPainter {
  final minisvg.Element root;

  SvgPainter({required this.root});

  @override
  void paint(Canvas canvas, Size size) {
    root.paintElement(canvas, size);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) {
    return false; // Return true if the painter should repaint
  }
}
