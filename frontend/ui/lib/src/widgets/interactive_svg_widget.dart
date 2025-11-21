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
  double baseScaleFactor = 1.0;
  Offset panOffset = Offset.zero;
  Offset? _lastPointerPosition;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        Size displaySize = constraints.biggest;
        _lastPointerPosition ??= displaySize.center(Offset.zero);
        double scale = scaleDown(widget.svgRootElement.size, displaySize);
        Size scaledSize = widget.svgRootElement.size * scale;
        /*developer.log(
          "scaledSize=$scaledSize, constraints-size=$displaySize => scale=$scale",
        );*/
        return GestureDetector(
          onScaleStart: (details) {
            _lastPointerPosition = details.focalPoint;
            baseScaleFactor = zoomScale;
          },
          onScaleUpdate: (details) {
            setState(() {
              final oldScale = zoomScale;
              final scaleChange = details.scale;
              zoomScale = baseScaleFactor * scaleChange;
              zoomScale = zoomScale.clamp(0.9, 3.5);
              panOffset += details.focalPointDelta / scaleChange;
              final localPos = (context.findRenderObject() as RenderBox?)
                  ?.globalToLocal(details.focalPoint);
              if (localPos != null) {
                final ratio = zoomScale / oldScale;
                panOffset = localPos - (localPos - panOffset) * ratio;
              }
              developer.log(
                "[onScaleUpdate] panOffset=$panOffset baseScaleFactor=$baseScaleFactor scaleChange=$scaleChange zoomScale=$zoomScale ",
              );
            });
          },
          child: Listener(
            onPointerSignal: (pointerSignal) {
              if (pointerSignal is PointerScrollEvent) {
                setState(() {
                  final delta = pointerSignal.scrollDelta.dy;
                  final stepSize = 0.25;
                  final oldScale = zoomScale;
                  zoomScale = (zoomScale + (delta > 0 ? -stepSize : stepSize))
                      .clamp(0.9, 3.5);
                  final localPos = (context.findRenderObject() as RenderBox?)
                      ?.globalToLocal(pointerSignal.position);
                  if (localPos != null) {
                    final ratio = zoomScale / oldScale;
                    panOffset = localPos - (localPos - panOffset) * ratio;
                  }
                  developer.log(
                    "[onPointerSignal] panOffset=$panOffset zoomScale=$zoomScale",
                  );
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
