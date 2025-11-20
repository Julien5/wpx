import 'dart:developer' as developer;
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:ui/src/svgelements.dart';
import 'package:ui/utils.dart';

class SvgWidget extends StatefulWidget {
  final SvgRootElement svgRootElement;
  const SvgWidget({super.key, required this.svgRootElement});

  @override
  State<SvgWidget> createState() => _SvgWidgetState();
}

class _SvgWidgetState extends State<SvgWidget> {
  double zoomScale = 1.0;
  Offset panOffset = Offset.zero;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        Size displaySize = constraints.biggest;
        double scale = scaleDown(widget.svgRootElement.size, displaySize);
        Size scaledSize = widget.svgRootElement.size * scale;
        developer.log(
          "scaledSize=$scaledSize, constraints-size=$displaySize => scale=$scale",
        );
        return Listener(
          onPointerSignal: (pointerSignal) {
            if (pointerSignal is PointerScrollEvent) {
              setState(() {
                // Zoom in/out based on scroll direction
                final delta = pointerSignal.scrollDelta.dy;
                final stepSize = 0.05;
                zoomScale = (zoomScale + (delta > 0 ? -stepSize : stepSize)).clamp(0.1, 4.0);
              });
            }
          },
          child: CustomPaint(
            size: scaledSize,
            painter: SvgPainter(
              root: widget.svgRootElement,
              renderingScale: scale,
              zoomScale: zoomScale,
              panOffset: panOffset,
            ),
          ),
        );
      },
    );
  }
}

class SvgPainter extends CustomPainter {
  final SvgRootElement root;
  double renderingScale = 1.0;
  double zoomScale = 1.0;
  Offset panOffset = Offset.zero;

  SvgPainter({
    required this.root,
    required this.renderingScale,
    required this.zoomScale,
    required this.panOffset,
  });

  @override
  void paint(Canvas canvas, Size drawArea) {
    canvas.scale(renderingScale);
    Sheet sheet = Sheet(
      canvas: canvas,
      size: drawArea,
      zoom: zoomScale,
      pan: panOffset,
    );
    root.paintElement(sheet);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) {
    return false; // Return true if the painter should repaint
  }
}
