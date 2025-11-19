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
  final TransformationController _transformationController =
      TransformationController();

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
          minScale: 1,
          maxScale: 1.5,
          scaleFactor: 1000,
          boundaryMargin: EdgeInsets.all(double.infinity), // <-- Add this line
          onInteractionUpdate: (details) {
            // Extract the current scale from the transformation matrix
            double newZoom =
                _transformationController.value.getMaxScaleOnAxis();
            Offset newPan = Offset(
              _transformationController.value.row0[2],
              _transformationController.value.row1[2],
            );
            setState(() {
              zoomScale = newZoom;
              panOffset = newPan;
            });
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
