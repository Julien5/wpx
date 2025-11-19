import 'dart:developer' as developer;
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
  final TransformationController _transformationController = TransformationController();

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
        return InteractiveViewer(
          transformationController: _transformationController,
          minScale: 0.5,
          maxScale: 5.0,
          onInteractionUpdate: (details) {
            // Extract the current scale from the transformation matrix
            double newZoom = _transformationController.value.getMaxScaleOnAxis();
            setState(() {
              zoomScale = newZoom;
            });
          },
          child: CustomPaint(
            size: scaledSize,
            painter: SvgPainter(
              root: widget.svgRootElement,
              renderingScale: scale,
              zoomScale: zoomScale,
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

  SvgPainter({
    required this.root,
    required this.renderingScale,
    required this.zoomScale,
  });

  @override
  void paint(Canvas canvas, Size drawArea) {
    canvas.scale(renderingScale);
    Sheet sheet = Sheet(canvas: canvas, size: drawArea, zoom: zoomScale);
    root.paintElement(sheet);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) {
    return false; // Return true if the painter should repaint
  }
}
