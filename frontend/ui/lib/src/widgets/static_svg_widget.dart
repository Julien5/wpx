import 'package:flutter/material.dart';
import 'package:ui/src/svgelements.dart';
import 'package:ui/src/utils/utils.dart';

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
        Offset offset = Offset(
          0.5 * (displaySize.width - scaledSize.width),
          0.5 * (displaySize.height - scaledSize.height),
        );
        debugPrint("object=${svgRootElement.size}");
        debugPrint("display=$displaySize");
        debugPrint("scaled=$scaledSize");
        debugPrint("=> scale=$scale, offset=$offset");
        return ClipRect(
          child: CustomPaint(
            size: scaledSize,
            painter: StaticSvgPainter(
              root: svgRootElement,
              renderingScale: scale,
              offset: offset,
            ),
          ),
        );
      },
    );
  }
}

class StaticSvgPainter extends CustomPainter {
  final SvgRootElement root;
  final double renderingScale;
  final Offset offset;

  StaticSvgPainter({
    required this.root,
    required this.renderingScale,
    required this.offset,
  });

  @override
  void paint(Canvas canvas, Size drawArea) {
    /*final Paint framePaint =
        Paint()
          ..color = Colors.red
          ..style = PaintingStyle.stroke
          ..strokeWidth = 10.0;
    canvas.drawRect(Offset.zero & drawArea, framePaint);
    */

    // i wonder if that order is correct, but the content
    // is properly centered...
    canvas.translate(offset.dx, offset.dy);
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
