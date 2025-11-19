import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:ui/src/svgelements.dart';
import 'package:ui/utils.dart';

class SvgWidget extends StatelessWidget {
  final SvgRootElement svgRootElement;
  final Size? size;
  
  const SvgWidget({super.key, required this.svgRootElement, this.size});

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        Size displaySize=constraints.biggest;
        double scale=scaleDown(svgRootElement.size,displaySize);
        Size scaledSize=svgRootElement.size*scale;
        developer.log("scaledSize=$scaledSize, constraints-size=$displaySize => scale=$scale");
        return CustomPaint(size: scaledSize, painter: SvgPainter(root: svgRootElement, renderingScale: scale));
      },
    );
  }
}

class SvgPainter extends CustomPainter {
  final SvgRootElement root;
  double renderingScale=1.0;
  double zoomScale=1.0;

  SvgPainter({required this.root, required this.renderingScale});

  @override
  void paint(Canvas canvas, Size drawArea) {
    canvas.scale(renderingScale);
    Sheet sheet=Sheet(canvas: canvas, size: drawArea, zoom: zoomScale);
    root.paintElement(sheet);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) {
    return false; // Return true if the painter should repaint
  }
}
