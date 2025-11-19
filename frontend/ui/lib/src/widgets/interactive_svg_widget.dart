import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:ui/src/svgelements.dart';
import 'package:ui/utils.dart';

class InteractiveSvgWidget extends StatelessWidget {
  final SvgRootElement svgRootElement;
  final Size? size;
  
  const InteractiveSvgWidget({super.key, required this.svgRootElement, this.size});

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        Size displaySize=constraints.biggest;
        double scale=scaleDown(svgRootElement.size,displaySize);
        Size scaledSize=svgRootElement.size*scale;
        developer.log("scaledSize=$scaledSize, constraints-size=$displaySize => scale=$scale");
        return CustomPaint(size: scaledSize, painter: InteractiveSvgPainter(root: svgRootElement, scale: scale));
      },
    );
  }
}

class InteractiveSvgPainter extends CustomPainter {
  final SvgRootElement root;
  double scale=1.0;

  InteractiveSvgPainter({required this.root, required this.scale});

  @override
  void paint(Canvas canvas, Size drawArea) {
    canvas.scale(scale);
    root.paintElement(canvas, drawArea);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) {
    return false; // Return true if the painter should repaint
  }
}
