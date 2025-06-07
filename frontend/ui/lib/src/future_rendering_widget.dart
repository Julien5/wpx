import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:ui/src/backendmodel.dart';

class FutureRenderingWidget extends StatefulWidget {
  final FutureRenderer future;
  const FutureRenderingWidget({super.key, required this.future});

  @override
  State<FutureRenderingWidget> createState() => _FutureRenderingWidgetState();
}

class _FutureRenderingWidgetState extends State<FutureRenderingWidget> {
  Widget? svg;

  Widget grayBackground() {
  return Container(
    width: 1400.0,
    height: 400.0,
    decoration: BoxDecoration(
      color: Colors.grey.withAlpha(150),
    ),
  );
}
  @override
  Widget build(BuildContext context) {
    if (widget.future.done()) {
      svg = SvgPicture.string(widget.future.result(), width: 1400, height: 400);
    }
    if (!widget.future.done() && svg == null) {
      return Text("starting ${widget.future.trackData} ${widget.future.id()}");
    }

    if (!widget.future.done()) {
      return Stack(
        children: <Widget>[grayBackground(),Text("updating ${widget.future.trackData} ${widget.future.id()}"), svg!],
      );
    }
    return svg!;
  }
}
