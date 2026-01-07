import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:ui/src/svgelements.dart';
import 'package:ui/utils.dart';

class StaticSvgWidget extends StatelessWidget {
  final SvgRootElement svgRootElement;

  const StaticSvgWidget({super.key, required this.svgRootElement});

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        Size displaySize = constraints.biggest;
        double scale = scaleDown(svgRootElement.size, displaySize);
        Size scaledSize = svgRootElement.size * scale;
        developer.log(
          "scaledSize=$scaledSize, constraints-size=$displaySize => scale=$scale",
        );
        return CustomPaint(
          size: scaledSize,
          painter: StaticSvgPainter(
            root: svgRootElement,
            renderingScale: scale,
          ),
        );
      },
    );
  }
}

class StaticSvgPainter extends CustomPainter {
  final SvgRootElement root;
  final double renderingScale;

  StaticSvgPainter({required this.root, required this.renderingScale});

  @override
  void paint(Canvas canvas, Size drawArea) {
    canvas.scale(renderingScale);
    Sheet sheet = Sheet(
      canvas: canvas,
      size: drawArea,
      zoom: 1.0,
      pan: Offset.zero,
    );
    root.paintElement(sheet);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) {
    return false; // Return true if the painter should repaint
  }
}
