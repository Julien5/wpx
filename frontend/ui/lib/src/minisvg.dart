import 'dart:io';
import 'package:flutter/material.dart';
import 'package:path_drawing/path_drawing.dart';
import 'package:svg_path_parser/svg_path_parser.dart';
import 'package:xml/xml.dart';

class Transform {
  static List<Transform> readAttribute(String s) {
    List<Transform> transforms = [];
    final transformRegex = RegExp(r'(translate\([^)]+\)|scale\([^)]+\))');

    for (final match in transformRegex.allMatches(s)) {
      final transform = match.group(0)!;
      if (transform.startsWith('translate')) {
        transforms.add(Translate(transform));
      } else if (transform.startsWith('scale')) {
        transforms.add(Scale(transform));
      }
    }

    return transforms;
  }
}

class Scale extends Transform {
  double sx = 1.0;
  double sy = 1.0;
  Scale(String s) {
    final scaleRegex = RegExp(r'scale\(([^,\s]+)[,\s]+([^)]+)\)');
    final scaleMatch = scaleRegex.firstMatch(s);
    assert(scaleMatch != null);
    sx = double.parse(scaleMatch!.group(1)!);
    sy = double.parse(scaleMatch.group(2)!);
  }
}

class Translate extends Transform {
  double tx = 0.0;
  double ty = 0.0;
  Translate(String s) {
    final translateRegex = RegExp(r'translate\(([^,\s]+)[,\s]+([^)]+)\)');
    final translateMatch = translateRegex.firstMatch(s);
    assert(translateMatch != null);
    tx = double.parse(translateMatch!.group(1)!);
    ty = double.parse(translateMatch.group(2)!);
  }
}

abstract class Element {
  List<Transform> T = [];
  List<Element> children = [];

  void paintElement(Canvas canvas, Size size);

  void paint(Canvas canvas, Size size) {
    _installTransforms(canvas);
    paintElement(canvas, size);
    _deinstallTransforms(canvas);
  }

  late XmlElement e;
  Element(XmlElement pe) : e = pe {
    if (e.attributes.isNotEmpty) {
      for (var attr in e.attributes) {
        switch (attr.name.local) {
          case 'transform':
            T = Transform.readAttribute(attr.value);
            break;
          default:
            // Handle other attributes if necessary
            break;
        }
      }
    }
  }

  void _installTransforms(Canvas canvas) {
    canvas.save();
    for (var t in T) {
      if (t is Translate) {
        canvas.translate(t.tx, t.ty);
      }
      if (t is Scale) {
        canvas.scale(t.sx, t.sy);
      }
    }
  }

  void _deinstallTransforms(Canvas canvas) {
    canvas.restore();
  }

  static Element fromXml(XmlElement e) {
    if (e.name.local == "path") {
      return PathElement(e);
    } else if (e.name.local == "text") {
      return TextElement(e);
    } else if (e.name.local == "circle") {
      return CircleElement(e);
    } else if (e.name.local == "svg") {
      return GroupElement(e);
    } else if (e.name.local == "g") {
      return GroupElement(e);
    } else {
      throw Exception("Unknown element type: ${e.name}");
    }
  }
}

class GroupElement extends Element {
  GroupElement(super.e) {
    for (var child in e.children) {
      if (child is XmlElement) {
        children.add(Element.fromXml(child));
      }
    }
  }

  @override
  void paintElement(Canvas canvas, Size size) {
    for (var child in children) {
      child.paint(canvas, size);
    }
  }
}

Color parseColor(String colorName) {
  switch (colorName.toLowerCase()) {
    case 'black':
      return Colors.black;
    case 'white':
      return Colors.white;
    case 'red':
      return Colors.red;
    case 'green':
      return Colors.green;
    case 'blue':
      return Colors.blue;
    case 'yellow':
      return Colors.yellow;
    case 'cyan':
      return Colors.cyan;
    case 'magenta':
      return Colors.purple;
    case 'gray':
      return Colors.grey;
    case 'lightgray':
      return const Color.fromARGB(255, 231, 226, 226);
    case 'transparent':
      return Colors.transparent;
    default:
      throw Exception('Unknown color: $colorName');
  }
}

class PathElement extends Element {
  late String d;
  late Color stroke;
  late Color fill;
  late double strokeWidth;
  late Path path;
  late String strokeDasharray;
  PathElement(super.e) {
    d = e.getAttribute("d")!;
    stroke = Colors.black;
    fill = Colors.transparent;
    strokeWidth = 1.0;
    strokeDasharray = "";
    if (e.getAttribute("stroke") != null) {
      stroke = parseColor(e.getAttribute("stroke")!);
    }
    if (e.getAttribute("fill") != null) {
      fill = parseColor(e.getAttribute("fill")!);
    }
    if (e.getAttribute("stroke-width") != null) {
      strokeWidth = double.parse(e.getAttribute("stroke-width")!);
    }
    if (e.getAttribute("stroke-dasharray") != null) {
      strokeDasharray = e.getAttribute("stroke-dasharray")!;
    }

    path = parseSvgPath(d);
  }

  @override
  void paintElement(Canvas canvas, Size size) {
    final paint = Paint()..style = PaintingStyle.stroke;
    paint.isAntiAlias = true;
    if (d.length < 50) {
      paint.isAntiAlias = false;
    }
    paint.strokeWidth = strokeWidth;
    paint.color = stroke;

    if (fill != Colors.transparent) {
      paint.style = PaintingStyle.fill;
      paint.color = fill;
    }
    Path p = path;
    if (strokeDasharray.isNotEmpty) {
      p = dashPath(
        path,
        dashArray: CircularIntervalList<double>(<double>[10.0, 5]),
      );
    }
    canvas.drawPath(p, paint);
  }
}

class CircleElement extends Element {
  late Color stroke;
  late Color fill;
  late double strokeWidth;
  late double cx, cy, r;

  CircleElement(super.e) {
    stroke = Colors.black;
    fill = Colors.black;
    strokeWidth = 1.0;
    if (e.getAttribute("stroke") != null) {
      stroke = parseColor(e.getAttribute("stroke")!);
    }
    if (e.getAttribute("fill") != null) {
      fill = parseColor(e.getAttribute("fill")!);
    }
    if (e.getAttribute("stroke-width") != null) {
      strokeWidth = double.parse(e.getAttribute("stroke-width")!);
    }
    cx = double.parse(e.getAttribute("cx")!);
    cy = double.parse(e.getAttribute("cy")!);
    r = double.parse(e.getAttribute("r")!);
  }

  @override
  void paintElement(Canvas canvas, Size size) {
    final paint = Paint()..style = PaintingStyle.stroke;
    paint.isAntiAlias = true;
    paint.strokeWidth = strokeWidth;
    paint.color = stroke;

    if (fill != Colors.transparent) {
      paint.style = PaintingStyle.fill;
      paint.color = fill;
    }
    canvas.drawCircle(Offset(cx, cy), r, paint);
  }
}

TextAlign readTextAlign(String textAnchor) {
  switch (textAnchor) {
    case "middle":
      return TextAlign.center;
    case "end":
      return TextAlign.right;
    case "start":
      return TextAlign.left;
    default:
      throw Exception('Unknown color: $textAnchor');
  }
}

class TextElement extends Element {
  late String text;
  late TextAlign textAlign;
  TextElement(super.e) {
    text = e.innerText.trim();
    textAlign = TextAlign.center;
    if (e.getAttribute("text-anchor") != null) {
      textAlign = readTextAlign(e.getAttribute("text-anchor")!);
    }
  }

  @override
  void paintElement(Canvas canvas, Size size) {
    final textPainter = TextPainter(
      text: TextSpan(
        text: text,
        style: const TextStyle(
          color: Colors.black,
          fontSize: 16.0,
          fontFamily: "Courier",
        ),
      ),
      textDirection: TextDirection.ltr,
      textAlign: textAlign,
    );

    textPainter.layout();
    // Calculate the position based on textAlign
    double dx = 0;

    if (textAlign == TextAlign.center) {
      dx = -textPainter.width / 2;
    } else if (textAlign == TextAlign.right) {
      dx = -textPainter.width;
    }
    double dy = -0.5 * textPainter.height - 4;
    textPainter.paint(canvas, Offset(dx, dy));
  }
}

Element rootElement() {
  /// read xml from file
  String xml = File('track-0.svg').readAsStringSync();
  XmlDocument doc = XmlDocument.parse(xml);
  return Element.fromXml(doc.rootElement);
}

class MiniSvgWidget extends StatelessWidget {
  final String svg;
  final double width;
  final double height;

  static Element parse(String s) {
    XmlDocument doc = XmlDocument.parse(s);
    return Element.fromXml(doc.rootElement);
  }

  const MiniSvgWidget({
    super.key,
    required this.svg,
    required this.width,
    required this.height,
  });

  @override
  Widget build(BuildContext context) {
    return CustomPaint(
      size: Size(width, height),
      painter: SvgPainter(root: MiniSvgWidget.parse(svg)),
    );
  }
}

class SvgPainter extends CustomPainter {
  final Element root;

  SvgPainter({required this.root});

  @override
  void paint(Canvas canvas, Size size) {
    root.paintElement(canvas, size);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) {
    return false; // Return true if the painter should repaint
  }
}
