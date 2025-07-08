import 'package:flutter/material.dart';
//import 'package:flutter_svg/flutter_svg.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/minisvg.dart';

class FutureRenderingWidget extends StatefulWidget {
  final FutureRenderer future;
  const FutureRenderingWidget({super.key, required this.future});

  @override
  State<FutureRenderingWidget> createState() => _FutureRenderingWidgetState();
}

class _FutureRenderingWidgetState extends State<FutureRenderingWidget> {
  Widget? svg;

  @override
  Widget build(BuildContext context) {
    double width = MediaQuery.sizeOf(context).width - 30;
    Size childSize = Size(width, width / 3);
    widget.future.setSize(childSize);
    if (widget.future.done()) {
      svg = MiniSvgWidget(svg: widget.future.result(), size: childSize);
    }
    if (!widget.future.done() && svg == null) {
      return Text("starting ${widget.future.trackData} ${widget.future.id()}");
    }

    if (!widget.future.done()) {
      return Stack(
        children: <Widget>[
          Text("updating ${widget.future.trackData} ${widget.future.id()}"),
          svg!,
        ],
      );
    }
    return svg!;
  }
}
