import 'dart:math';
import 'package:flutter/material.dart';
import 'package:ui/src/svgelements.dart';

class MiniSvgWidget extends StatelessWidget {
  final SvgRootElement svgRootElement;
  final Size? size;
  
  const MiniSvgWidget({super.key, required this.svgRootElement, this.size});

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        Size displaySize=constraints.biggest;
        double scale=gscale(svgRootElement.size,displaySize);
        Size scaledSize=svgRootElement.size*scale;
        //developer.log("svg-size=${svg.size}, constraints-size=$outputSize");
        return CustomPaint(size: scaledSize, painter: SvgPainter(root: svgRootElement));
      },
    );
  }
}

double gscale(Size object, Size drawArea) {
  double sw = drawArea.width / object.width;
  double sh = drawArea.height / object.height;
  return [sw, sh, 1.0].reduce(min);
}

class SvgPainter extends CustomPainter {
  final SvgRootElement root;

  SvgPainter({required this.root});

  @override
  void paint(Canvas canvas, Size drawArea) {
    double scale = gscale(root.size, drawArea);
    //developer.log("input-size=${root.size}, output-size=$drawArea => scale=$s");
    canvas.scale(scale);
    if ((scale < 1)) {
      double tx = 0.5 * (drawArea.width - scale * root.size.width);
      canvas.translate(tx, 0);
    }
    root.paintElement(canvas, drawArea);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) {
    return false; // Return true if the painter should repaint
  }
}
