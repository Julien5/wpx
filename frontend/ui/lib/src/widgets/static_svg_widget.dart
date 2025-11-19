import 'dart:developer' as developer;
import 'dart:math';
import 'package:flutter/material.dart';
import 'package:ui/src/svgelements.dart';

double gscale(Size object, Size drawArea) {
  double sw = drawArea.width / object.width;
  double sh = drawArea.height / object.height;
  return [sw, sh, 1.0].reduce(min);
}

class StaticSvgWidget extends StatelessWidget {
  final SvgRootElement svgRootElement;
  final Size? size;
  
  const StaticSvgWidget({super.key, required this.svgRootElement, this.size});

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        Size displaySize=constraints.biggest;
        double scale=gscale(svgRootElement.size,displaySize);
        Size scaledSize=svgRootElement.size*scale;
        developer.log("scaledSize=$scaledSize, constraints-size=$displaySize => scale=$scale");
        return CustomPaint(size: scaledSize, painter: StaticSvgPainter(root: svgRootElement, scale: scale));
      },
    );
  }
}

class StaticSvgPainter extends CustomPainter {
  final SvgRootElement root;
  double scale = 1.0;

  StaticSvgPainter({required this.root, required this.scale});

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
