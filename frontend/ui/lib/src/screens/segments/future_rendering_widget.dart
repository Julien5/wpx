import 'package:flutter/material.dart';
import 'package:ui/src/log.dart';
//import 'package:flutter_svg/flutter_svg.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/svgelements.dart';
import 'package:ui/src/widgets/minisvg.dart';

class FutureRenderingWidget extends StatefulWidget {
  final FutureRenderer future;
  const FutureRenderingWidget({super.key, required this.future});

  @override
  State<FutureRenderingWidget> createState() => _FutureRenderingWidgetState();
}

class _FutureRenderingWidgetState extends State<FutureRenderingWidget> {
  MiniSvgWidget? svg;

  Widget buildWorker(Size wantedSize) {
    widget.future.setSize(wantedSize);
    if (widget.future.done()) {
      log("[render-parse-start:${widget.future.trackData}]");
      SvgRootElement root = parseSvg(widget.future.result());
      log("[render-parse-end:${widget.future.trackData}]");
      svg = MiniSvgWidget(svg: root);
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

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (BuildContext context, BoxConstraints constraints) {
        Size size = Size(1000, 285);    
        if (widget.future is MapRenderer) {
          size = Size(400,400);
        }
        return buildWorker(size);
      },
    );
  }
}
