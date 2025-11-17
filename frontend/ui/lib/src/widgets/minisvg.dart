import 'dart:math';
import 'package:flutter/material.dart';
import 'package:ui/src/svgelements.dart';

class MiniSvgWidget extends StatelessWidget {
  final SvgRootElement svg;
  final Size? size;
  
  const MiniSvgWidget({super.key, required this.svg, this.size});

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        Size outputSize=Size(constraints.maxWidth,constraints.maxHeight);
        double scale=gscale(svg.size,outputSize);
        Size scaledIntputSize=Size(svg.size.width*scale,svg.size.height*scale);
        //developer.log("svg-size=${svg.size}, constraints-size=$outputSize");
        return CustomPaint(size: scaledIntputSize, painter: SvgPainter(root: svg));
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
    double s = gscale(root.size, drawArea);
    //developer.log("input-size=${root.size}, output-size=$drawArea => scale=$s");
    canvas.scale(s);
    if ((s < 1)) {
      double tx = 0.5 * (drawArea.width - s * root.size.width);
      canvas.translate(tx, 0);
    }
    root.paintElement(canvas, drawArea);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) {
    return false; // Return true if the painter should repaint
  }
}
