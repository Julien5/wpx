import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:ui/src/backendmodel.dart';

class FutureRenderingWidget extends StatefulWidget {
  final FutureRendering future;
  const FutureRenderingWidget({super.key, required this.future});

  @override
  State<FutureRenderingWidget> createState() => _FutureRenderingWidgetState();
}

class _FutureRenderingWidgetState extends State<FutureRenderingWidget> {
  Widget? svg;

  Widget grayBackground() {
  return Container(
    width: 600.0,
    height: 150.0,
    decoration: BoxDecoration(
      color: Colors.grey.withAlpha(150), // Set the background color to gray
    ),
  );
}
  @override
  Widget build(BuildContext context) {
    developer.log("[FutureRenderingWidget] [build] ${widget.future.id()}");
    if (widget.future.done()) {
      svg = SvgPicture.string(widget.future.result(), width: 600, height: 150);
    }
    if (!widget.future.done() && svg == null) {
      return Text("starting");
    }

    if (!widget.future.done()) {
      return Stack(
        children: <Widget>[grayBackground(),Text("loading ${widget.future.id()}"), svg!],
      );
    }
    return svg!;
  }
}
