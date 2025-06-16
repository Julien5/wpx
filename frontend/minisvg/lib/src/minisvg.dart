import 'dart:developer' as developer;
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:svg_path_parser/svg_path_parser.dart';
import 'package:xml/xml.dart';

class Transform {
  static List<Transform> readAttribute(String s) {
    List<Transform> transforms = [];
    final transformRegex = RegExp(r'(translate\([^)]+\)|scale\([^)]+\))');

    // Match all transformations in order
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
    developer.log(s);
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
    developer.log(s);
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

  void installTransforms(Canvas canvas) {
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

  void deinstallTransforms(Canvas canvas) {
    canvas.restore();
  }

  static Element fromXml(XmlElement e) {
    if (e.name.local == "path") {
      return PathElement(e);
    } else if (e.name.local == "text") {
      return TextElement(e);
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
    developer.log('group paint ${super.e.name.local}');
    installTransforms(canvas);
    for (var child in children) {
      child.paintElement(canvas, size);
    }
    deinstallTransforms(canvas);
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
  PathElement(super.e) {
    d = e.getAttribute("d")!;
    stroke = Colors.black;
    fill = Colors.transparent;
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

    path = parseSvgPath(d);
  }

  @override
  void paintElement(Canvas canvas, Size size) {
    installTransforms(canvas);
    developer.log('path paint ${super.e.name.local} with ${d.length}');
    final paint = Paint()..style = PaintingStyle.stroke;
    paint.isAntiAlias = false;
    developer.log('strokeWidth=$strokeWidth');
    paint.strokeWidth = strokeWidth;

    developer.log('stroke=$stroke');
    paint.color = stroke;

    if (fill != Colors.transparent) {
      developer.log("fill = $fill");
      paint.style = PaintingStyle.fill;
      paint.color = fill;
    }

    canvas.drawPath(path, paint);
    deinstallTransforms(canvas);
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
    text = e.innerText;
    textAlign = TextAlign.right;
    if (e.getAttribute("text-anchor") != null) {
      textAlign = readTextAlign(e.getAttribute("text-anchor")!);
    }
    textAlign = TextAlign.right;
  }

  @override
  void paintElement(Canvas canvas, Size size) {
    installTransforms(canvas);
    developer.log('text paint ${super.e.name.local}');
    final textPainter = TextPainter(
      text: TextSpan(
        text: text,
        style: const TextStyle(
          color: Colors.black, 
          fontSize: 16.0, 
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
    double dy = -textPainter.height/2; 
    textPainter.paint(canvas, Offset(dx, dy));
    deinstallTransforms(canvas);
  }
}

Element rootElement() {
  /// read xml from file
  String xml = File('track-0.svg').readAsStringSync();
  XmlDocument doc = XmlDocument.parse(xml);
  return Element.fromXml(doc.rootElement);
}
