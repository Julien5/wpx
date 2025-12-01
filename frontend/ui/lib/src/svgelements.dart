import 'dart:io';
import 'package:flutter/material.dart';
import 'package:path_drawing/path_drawing.dart';
import 'package:svg_path_parser/svg_path_parser.dart';
import 'package:xml/xml.dart';

class Transform {
  static List<Transform> readAttribute(String s) {
    List<Transform> transforms = [];
    final transformRegex = RegExp(
      r'(translate\([^)]+\)|scale\([^)]+\)|rotate\([^)]+\))',
    );

    for (final match in transformRegex.allMatches(s)) {
      final transform = match.group(0)!;
      if (transform.startsWith('translate')) {
        transforms.add(Translate(transform));
      } else if (transform.startsWith('scale')) {
        transforms.add(Scale(transform));
      } else if (transform.startsWith('rotate')) {
        transforms.add(Rotate(transform));
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

class Rotate extends Transform {
  double angle = 0.0;
  double cx = 0.0;
  double cy = 0.0;

  // Supports: rotate(angle), rotate(angle cx cy)
  Rotate(String s) {
    final rotateRegex = RegExp(
      r'rotate\(\s*([-\d.]+)(?:[\s,]+([-\d.]+)[\s,]+([-\d.]+))?\s*\)',
    );
    final rotateMatch = rotateRegex.firstMatch(s);
    assert(rotateMatch != null, 'Invalid rotate transform: $s');
    angle = double.parse(rotateMatch!.group(1)!);
    if (rotateMatch.group(2) != null && rotateMatch.group(3) != null) {
      cx = double.parse(rotateMatch.group(2)!);
      cy = double.parse(rotateMatch.group(3)!);
    }
  }
}

class Sheet {
  Canvas canvas;
  Size size;
  double zoom;
  Offset pan;
  Sheet({
    required this.canvas,
    required this.size,
    required this.zoom,
    required this.pan,
  });
}

abstract class SvgElement {
  List<Transform> T = [];
  List<SvgElement> children = [];
  final SvgElement? _parent;

  void paintElement(Sheet sheet);

  void paint(Sheet sheet) {
    _installTransforms(sheet);
    paintElement(sheet);
    _deinstallTransforms(sheet);
  }

  final XmlElement _xmlElement;
  SvgElement(XmlElement xmlElement, SvgElement? parent)
    : _xmlElement = xmlElement,
      _parent = parent {
    if (_xmlElement.attributes.isNotEmpty) {
      for (var attr in _xmlElement.attributes) {
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

  void _installTransforms(Sheet sheet) {
    sheet.canvas.save();
    for (var t in T) {
      if (t is Translate) {
        sheet.canvas.translate(
          t.tx * sheet.zoom + sheet.pan.dx,
          t.ty * sheet.zoom + sheet.pan.dy,
        );
      }
      if (t is Scale) {
        sheet.canvas.scale(t.sx * sheet.zoom, t.sy * sheet.zoom);
      }
      if (t is Rotate) {
        if (t.cx != 0.0 || t.cy != 0.0) {
          sheet.canvas.translate(t.cx, t.cy);
          sheet.canvas.rotate(t.angle * 3.141592653589793 / 180.0);
          sheet.canvas.translate(-t.cx, -t.cy);
        } else {
          sheet.canvas.rotate(t.angle * 3.141592653589793 / 180.0);
        }
      }
    }
  }

  void _deinstallTransforms(Sheet sheet) {
    sheet.canvas.restore();
  }

  String? attribute(String name) {
    var ret = _xmlElement.getAttribute(name);
    if (ret != null) {
      return ret;
    }
    if (_parent != null) {
      return _parent.attribute(name);
    }
    return null;
  }

  static SvgElement fromXml(XmlElement e, SvgElement? parent) {
    if (e.name.local == "path") {
      return PathElement(e, parent);
    } else if (e.name.local == "text") {
      return TextElement(e, parent);
    } else if (e.name.local == "circle") {
      return CircleElement(e, parent);
    } else if (e.name.local == "rect") {
      // Add support for <rect>
      return RectElement(e, parent);
    } else if (e.name.local == "svg") {
      return SvgRootElement(e, parent);
    } else if (e.name.local == "g") {
      return GroupElement(e, parent);
    } else if (e.name.local == "title") {
      // ignore
      return GroupElement(e, parent);
    } else {
      throw Exception("Unknown element type: ${e.name}");
    }
  }
}

class GroupElement extends SvgElement {
  GroupElement(super.xmlElement, super.parent) {
    for (var child in _xmlElement.children) {
      if (child is XmlElement) {
        children.add(SvgElement.fromXml(child, this));
      }
    }
  }

  @override
  void paintElement(Sheet sheet) {
    for (var child in children) {
      child.paint(sheet);
    }
  }
}

class SvgRootElement extends GroupElement {
  late Size size;

  SvgRootElement(super.xmlElement, super.parent) {
    double width = double.parse(_xmlElement.getAttribute("width")!);
    double height = double.parse(_xmlElement.getAttribute("height")!);
    size = Size(width, height);
  }

  @override
  void paintElement(Sheet sheet) {
    for (var child in children) {
      child.paint(sheet);
    }
  }
}

Color parseHexColor(String hexColor) {
  String hex = hexColor.replaceFirst('#', '');
  if (hex.length == 3) {
    // Convert #rgb to #rrggbb
    hex = hex.split('').map((c) => '$c$c').join();
  }
  if (hex.length == 6) {
    hex = 'ff$hex'; // Add alpha if not present
  }
  return Color(int.parse(hex, radix: 16));
}

Color parseColor(String colorName) {
  if (colorName.startsWith("#")) {
    return parseHexColor(colorName);
  }
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
    case 'darkgray':
      return const Color.fromARGB(255, 169, 169, 169);
    case 'transparent':
      return Colors.transparent;
    case 'none':
      return Colors.transparent;
    default:
      throw Exception('Unknown color: $colorName');
  }
}

class PathElement extends SvgElement {
  late String d;
  late Color stroke;
  late Color fill;
  late double strokeWidth;
  late Path path;
  late String strokeDasharray;
  PathElement(super.xmlElement, super.parent) {
    d = _xmlElement.getAttribute("d")!;
    stroke = Colors.black;
    fill = Colors.transparent;
    strokeWidth = 1.0;
    strokeDasharray = "";
    if (attribute("stroke") != null) {
      stroke = parseColor(attribute("stroke")!);
    }
    if (attribute("fill") != null) {
      fill = parseColor(attribute("fill")!);
    }
    if (attribute("stroke-width") != null) {
      strokeWidth = double.parse(attribute("stroke-width")!);
    }
    if (attribute("stroke-dasharray") != null) {
      strokeDasharray = attribute("stroke-dasharray")!;
    }

    path = parseSvgPath(d);
  }

  @override
  void paintElement(Sheet sheet) {
    final paint = Paint()..style = PaintingStyle.stroke;
    paint.isAntiAlias = true;
    if (d.length < 50) {
      //paint.isAntiAlias = false;
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

    final Matrix4 matrix = Matrix4.identity();
    matrix.translateByDouble(sheet.pan.dx, sheet.pan.dy, 0, 1);
    matrix.scaleByDouble(sheet.zoom, sheet.zoom, 1, 1);

    p = p.transform(matrix.storage);

    sheet.canvas.drawPath(p, paint);
  }
}

class CircleElement extends SvgElement {
  late Color stroke;
  late Color fill;
  late double strokeWidth;
  late double cx, cy, r;

  CircleElement(super.xmlElement, super.parent) {
    stroke = Colors.black;
    fill = Colors.white;
    strokeWidth = 1.0;
    if (attribute("stroke") != null) {
      stroke = parseColor(attribute("stroke")!);
    }
    if (attribute("fill") != null) {
      fill = parseColor(attribute("fill")!);
    }
    if (attribute("stroke-width") != null) {
      strokeWidth = double.parse(attribute("stroke-width")!);
    }
    cx = double.parse(attribute("cx")!);
    cy = double.parse(attribute("cy")!);
    r = double.parse(attribute("r")!);
  }

  @override
  void paintElement(Sheet sheet) {
    final center = Offset(cx * sheet.zoom, cy * sheet.zoom) + sheet.pan;

    // Draw fill first, if any
    if (fill != Colors.transparent) {
      final fillPaint =
          Paint()
            ..style = PaintingStyle.fill
            ..isAntiAlias = true
            ..color = fill;
      sheet.canvas.drawCircle(center, r * sheet.zoom, fillPaint);
    }

    // Draw stroke on top, if any
    if (stroke != Colors.transparent && strokeWidth > 0) {
      final strokePaint =
          Paint()
            ..style = PaintingStyle.stroke
            ..isAntiAlias = true
            ..color = stroke
            ..strokeWidth = strokeWidth;
      sheet.canvas.drawCircle(center, r * sheet.zoom, strokePaint);
    }
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
    case "left":
      return TextAlign.left;
    default:
      throw Exception('Unknown text anchor: $textAnchor');
  }
}

class TextElement extends SvgElement {
  late String text;
  late TextAlign textAlign;
  late double fontSize;
  late double x, y;
  TextElement(super.xmlElement, super.parent) {
    text = super._xmlElement.innerText.trim();
    textAlign = TextAlign.center;
    fontSize = 32.0;
    x = y = 0;
    if (attribute("text-anchor") != null) {
      textAlign = readTextAlign(attribute("text-anchor")!);
    }
    if (attribute("font-size") != null) {
      fontSize =
          (double.parse(attribute("font-size").toString())).floorToDouble();
    }
    if (attribute("x") != null) {
      x = double.parse(attribute("x").toString());
    }
    if (attribute("y") != null) {
      y = double.parse(attribute("y").toString());
    }
  }

  @override
  void paintElement(Sheet sheet) {
    final textPainter = TextPainter(
      text: TextSpan(
        text: text,
        style: TextStyle(
          fontFamily: "LibertinusSans",
          color: Colors.black,
          fontSize: fontSize,
        ),
      ),
      textDirection: TextDirection.ltr,
      textAlign: textAlign,
    );

    textPainter.layout();
    // Calculate the position based on textAlign
    double dx = x;

    if (textAlign == TextAlign.center) {
      dx = x - textPainter.width / 2;
    } else if (textAlign == TextAlign.right) {
      dx = x - textPainter.width;
    }
    double dy = y - 0.5 * textPainter.height - 5;
    textPainter.paint(sheet.canvas, Offset(dx, dy));
  }
}

class RectElement extends SvgElement {
  late double x, y, width, height;
  late Color stroke;
  late Color fill;
  late double strokeWidth;

  RectElement(super.xmlElement, super.parent) {
    x = double.parse(attribute("x") ?? "0");
    y = double.parse(attribute("y") ?? "0");
    width = double.parse(attribute("width") ?? "0");
    height = double.parse(attribute("height") ?? "0");
    stroke = Colors.black;
    fill = Colors.white;
    strokeWidth = 1.0;

    if (attribute("stroke") != null) {
      stroke = parseColor(attribute("stroke")!);
    }
    if (attribute("fill") != null) {
      fill = parseColor(attribute("fill")!);
    }
    if (attribute("fill-opacity") != null) {
      double alpha = 255 * double.parse(attribute("fill-opacity")!);
      fill = fill.withAlpha(alpha.round());
    }
    if (attribute("stroke-width") != null) {
      strokeWidth = double.parse(attribute("stroke-width")!);
    }
  }

  @override
  void paintElement(Sheet sheet) {
    final paint = Paint()..style = PaintingStyle.stroke;
    paint.isAntiAlias = true;
    paint.strokeWidth = strokeWidth;
    paint.color = stroke;

    if (fill != Colors.transparent) {
      paint.style = PaintingStyle.fill;
      paint.color = fill;
    }
    Rect rect = Rect.fromLTWH(x, y, width, height);
    sheet.canvas.drawRect(rect, paint);
  }
}

SvgRootElement parseSvg(String s) {
  XmlDocument doc = XmlDocument.parse(s);
  SvgElement root = SvgElement.fromXml(doc.rootElement, null);
  assert(root is SvgRootElement);
  return root as SvgRootElement;
}

SvgRootElement rootElement() {
  /// read xml from file
  String xml = File('track-0.svg').readAsStringSync();
  XmlDocument doc = XmlDocument.parse(xml);
  SvgElement root = SvgElement.fromXml(doc.rootElement, null);
  assert(root is SvgRootElement);
  return root as SvgRootElement;
}
